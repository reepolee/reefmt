mod ree_format;
mod format;
mod ree_parser;
mod swc_format;
mod swc_printer;
mod remove_unused_imports;
mod ast_check;

use serde::Deserialize;
use std::{
    env, fs,
    io::Read,
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
};
use glob::glob;
use rayon::prelude::*;

fn default_true() -> bool { true }
fn default_four() -> usize { 4 }
fn default_soft_width() -> usize { 100 }
fn default_tab_width() -> usize { 4 }
fn default_keyvalue_props() -> usize { 1 }
fn default_skip_extensions() -> Vec<String> { vec!["min.js".to_string()] }

/// Reefmt configuration — loaded from `reefmt.jsonc` in the project root.
/// Run `reefmt --init` to create one.
#[derive(Deserialize)]
pub(crate) struct ReeConfig {
    /// Directories to skip when formatting.
    #[serde(rename = "skipDirs")]
    skip_dirs: Vec<String>,
    /// Glob patterns for files to skip (e.g. "generator/templates/**/*.ts").
    #[serde(rename = "skipFiles")]
    skip_files: Vec<String>,
    /// Compound extensions to skip (e.g. "min.js" skips any file ending in ".min.js").
    /// Checked before `extensions` — matched files are always skipped.
    #[serde(rename = "skipExtensions", default = "default_skip_extensions")]
    skip_extensions: Vec<String>,
    /// File extensions to format.
    extensions: Vec<String>,
    /// Whether to skip dot directories (folders starting with '.').
    #[serde(rename = "skipDotDirs")]
    skip_dot_dirs: bool,
    /// Maximum line width before elements are broken onto multiple lines.
    #[serde(rename = "wrapWidth")]
    wrap_width: usize,
    /// When true, single-statement blocks and object literal function params
    /// are collapsed onto one line when they fit within wrapWidth.
    #[serde(rename = "collapseSingleStatementBlocks", default = "default_true")]
    collapse_single_stmt_blocks: bool,
    /// Global fallback limit for all categories below. Any category not
    /// explicitly configured falls back to this value.
    #[serde(rename = "collapseMaxMembers", default = "default_four")]
    collapse_max_members: usize,
    /// Per-category overrides (fall back to collapseMaxMembers when absent).
    #[serde(rename = "collapseMaxObjectMembers", default)]
    collapse_max_object_members: Option<usize>,
    #[serde(rename = "collapseMaxArrayElements", default)]
    collapse_max_array_elements: Option<usize>,
    #[serde(rename = "collapseMaxFunctionParams", default)]
    collapse_max_function_params: Option<usize>,
    #[serde(rename = "collapseMaxCallArgs", default)]
    collapse_max_call_args: Option<usize>,
    #[serde(rename = "collapseMaxImports", default)]
    collapse_max_imports: Option<usize>,
    #[serde(rename = "collapseMaxTypeMembers", default)]
    collapse_max_type_members: Option<usize>,
    /// "Soft" wrap width. Any collapsible structure (call args, array/object/
    /// type members, imports) whose inline form fits within this width is kept
    /// on one line regardless of the per-category count caps above. Above this
    /// width the count caps apply and wrapWidth is the hard ceiling. Set to 0
    /// to disable (count caps always apply).
    #[serde(rename = "collapseSoftWidth", default = "default_soft_width")]
    collapse_soft_width: usize,
    /// Display width of a tab, used to measure line widths for wrap/collapse
    /// decisions. The formatter indents with hard tabs, so this controls how
    /// many columns each indent level counts as — set it to match your editor's
    /// tab size so wrapWidth/collapseSoftWidth reflect on-screen width.
    #[serde(rename = "tabWidth", default = "default_tab_width")]
    tab_width: usize,
    /// Maximum number of `key: value` ("named") properties an object literal may
    /// have and still collapse onto one line. Shorthand (`{ a, b }`) and spread
    /// (`{ ...x }`) don't count. With the default `1`, `{ x: 1 }` stays inline
    /// but `{ x: 1, y: 2 }` always expands one-per-line. Set high to disable.
    #[serde(rename = "collapseMaxKeyValueProps", default = "default_keyvalue_props")]
    collapse_max_keyvalue_props: usize,
    /// When true, a call with a single object or array literal argument that
    /// doesn't fit inline is printed with `({` on the same line as the callee
    /// and `})` on its own line, instead of expanding the `{` onto a separate
    /// indented line. Mirrors Prettier's "argument hugging" style.
    #[serde(rename = "hugCallArgs", default)]
    hug_call_args: bool,
    /// When true, unused import declarations are removed from JS/TS files
    /// during formatting. Side-effect imports (`import "./foo"`) are always kept.
    #[serde(rename = "removeUnusedImports", default)]
    remove_unused_imports: bool,
    /// Width threshold for collapsing multi-line HTML leaf elements in .ree
    /// files onto one line. An element is collapsed only when its inline form
    /// fits within this many columns. Set to 0 to disable.
    #[serde(default)]
    oneline: usize,
}

impl ReeConfig {
    pub(crate) fn collapse_config(&self) -> crate::format::CollapseConfig {
        let def = self.collapse_max_members;
        crate::format::CollapseConfig {
            enabled: self.collapse_single_stmt_blocks,
            max_object_members: self.collapse_max_object_members.unwrap_or(def),
            max_array_elements: self.collapse_max_array_elements.unwrap_or(def),
            max_function_params: self.collapse_max_function_params.unwrap_or(def),
            max_call_args: self.collapse_max_call_args.unwrap_or(def),
            max_imports: self.collapse_max_imports.unwrap_or(def),
            max_type_members: self.collapse_max_type_members.unwrap_or(def),
            soft_wrap_width: self.collapse_soft_width,
            tab_width: self.tab_width,
            max_keyvalue_props: self.collapse_max_keyvalue_props,
            collapse_width: self.oneline,
            hug_call_args: self.hug_call_args,
        }
    }
}



/// Load reefmt config from `reefmt.jsonc` in the current directory.
/// Exits with an error if the file is missing or invalid.
fn load_config() -> ReeConfig {
    let cwd = env::current_dir().unwrap_or_else(|e| {
        eprintln!("Error: could not determine current directory: {}", e);
        std::process::exit(1);
    });
    let config_path = cwd.join("reefmt.jsonc");
    if !config_path.exists() {
        eprintln!("Error: reefmt.jsonc not found in {}", cwd.display());
        eprintln!("Run 'reefmt --init' to create a config file.");
        std::process::exit(1);
    }
    let content = fs::read_to_string(&config_path).unwrap_or_else(|e| {
        eprintln!("Error: could not read {}: {}", config_path.display(), e);
        std::process::exit(1);
    });
    json5::from_str(&content).unwrap_or_else(|e| {
        eprintln!("Error: invalid reefmt.jsonc: {}", e);
        eprintln!("Fix the error or run 'reefmt --init' to regenerate the config.");
        std::process::exit(1);
    })
}

/// Check whether a file path matches any `skipFiles` glob pattern.
pub(crate) fn should_skip_file(file_path: &Path, config: &ReeConfig) -> bool {
    if config.skip_files.is_empty() {
        return false;
    }
    let rel_path = env::current_dir()
        .ok()
        .and_then(|cwd| file_path.strip_prefix(&cwd).ok())
        .unwrap_or(file_path);
    let path_str = rel_path.to_string_lossy().replace('\\', "/");
    config.skip_files.iter().any(|pattern| {
        glob::Pattern::new(pattern)
            .map(|p| p.matches(&path_str))
            .unwrap_or(false)
    })
}

/// Check whether a file's name ends with any of the `skipExtensions` compound extensions.
fn should_skip_extension(path: &Path, config: &ReeConfig) -> bool {
    if config.skip_extensions.is_empty() {
        return false;
    }
    let name = match path.file_name().and_then(|s| s.to_str()) {
        Some(n) => n,
        None => return false,
    };
    config.skip_extensions.iter().any(|ext| name.ends_with(&format!(".{}", ext)))
}

/// Check whether a path is inside a directory that should be skipped.
fn should_skip_path(path: &Path, config: &ReeConfig) -> bool {
    path.components().any(|c| {
        if let std::path::Component::Normal(s) = c {
            if let Some(name) = s.to_str() {
                if config.skip_dirs.iter().any(|d| d == name) {
                    return true;
                }
                if config.skip_dot_dirs && name.starts_with('.') && name != "." {
                    return true;
                }
            }
        }
        false
    })
}

fn collect_source_files(
    dir: &Path,
    files: &mut Vec<PathBuf>,
    config: &ReeConfig,
) -> std::io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                if should_skip_path(&path, config) {
                    continue;
                }
                collect_source_files(&path, files, config)?;
            } else if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                if config.extensions.iter().any(|e| e == ext)
                    && !should_skip_extension(&path, config)
                    && !should_skip_file(&path, config)
                {
                    files.push(path);
                }
            }
        }
    }
    Ok(())
}

/// Get files changed since last commit using git.
/// Runs `git diff --name-only HEAD` to get staged + unstaged changes,
/// then `git diff --name-only --cached` for staged-only, and
/// `git ls-files --others --exclude-standard` for untracked files.
/// Returns paths that match the configured extensions.
fn get_git_changed_files(config: &ReeConfig) -> Vec<PathBuf> {
    let cwd = match env::current_dir() {
        Ok(c) => c,
        Err(_) => return vec![],
    };

    let mut files = Vec::new();

    // Get all modified files (staged + unstaged) and untracked files
    let output = Command::new("git")
        .args(["status", "--porcelain"])
        .output();

    match output {
        Ok(out) if out.status.success() => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            for line in stdout.lines() {
                // porcelain format: XY filename
                // X = index status, Y = worktree status
                // We want files that are modified (M), added (A), or untracked (?)
                // Skip deleted (D) and renamed (R) without content changes
                if line.len() < 4 {
                    continue;
                }
                let status_chars: Vec<char> = line[..2].chars().collect();
                let x = status_chars[0];
                let y = status_chars[1];

                // Skip deleted files
                if x == 'D' || y == 'D' {
                    continue;
                }

                let raw_path = &line[3..];
                // Handle renamed files (format: "old -> new")
                let filepath = if let Some(arrow_pos) = raw_path.find(" -> ") {
                    raw_path[arrow_pos + 4..].trim()
                } else {
                    raw_path.trim()
                };
                // Strip quotes (git quotes paths with special characters)
                let filepath = filepath.trim_matches('"');

                let path = cwd.join(filepath);
                if !path.exists() {
                    continue;
                }

                if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                    if config.extensions.iter().any(|e| e == ext)
                        && !should_skip_path(&path, config)
                        && !should_skip_extension(&path, config)
                        && !should_skip_file(&path, config)
                    {
                        files.push(path);
                    }
                }
            }
        }
        _ => {
            // Not a git repo or git not available — fall back to empty
            eprintln!("Warning: not a git repository, --git has no effect");
        }
    }

    files
}

fn main() {
    let mut args: Vec<String> = env::args().skip(1).collect();

    // Parse mode flags
    let diff_mode = args.iter().position(|a| a == "--diff").is_some();
    if diff_mode {
        args.retain(|a| a != "--diff");
    }

    let check_mode = args.iter().position(|a| a == "--check" || a == "--dry-run" || a == "-c").is_some();
    if check_mode {
        args.retain(|a| a != "--check" && a != "--dry-run" && a != "-c");
    }

    let verbose = args.iter().position(|a| a == "--verbose").is_some();
    if verbose {
        args.retain(|a| a != "--verbose");
    }

    let git_mode = args.iter().position(|a| a == "--git").is_some();
    if git_mode {
        args.retain(|a| a != "--git");
    }

    // Parse --collapse-max-members CLI override
    let cli_max_members: Option<usize> = args.iter().position(|a| a == "--collapse-max-members").map(|pos| {
        args.remove(pos); // remove the flag
        if pos >= args.len() {
            eprintln!("Error: --collapse-max-members requires a number argument");
            std::process::exit(1);
        }
        let val = args.remove(pos);
        val.parse().unwrap_or_else(|_| {
            eprintln!("Error: --collapse-max-members must be a positive integer");
            std::process::exit(1);
        })
    });

    // Parse --wrap-width CLI override
    let cli_wrap_width: Option<usize> = args.iter().position(|a| a == "--wrap-width").map(|pos| {
        args.remove(pos);
        if pos >= args.len() {
            eprintln!("Error: --wrap-width requires a number argument");
            std::process::exit(1);
        }
        let val = args.remove(pos);
        val.parse().unwrap_or_else(|_| {
            eprintln!("Error: --wrap-width must be a positive integer");
            std::process::exit(1);
        })
    });

    // Parse --collapse-soft-width CLI override
    let cli_soft_width: Option<usize> = args.iter().position(|a| a == "--collapse-soft-width").map(|pos| {
        args.remove(pos);
        if pos >= args.len() {
            eprintln!("Error: --collapse-soft-width requires a number argument");
            std::process::exit(1);
        }
        let val = args.remove(pos);
        val.parse().unwrap_or_else(|_| {
            eprintln!("Error: --collapse-soft-width must be a non-negative integer");
            std::process::exit(1);
        })
    });

    // Parse --tab-width CLI override
    let cli_tab_width: Option<usize> = args.iter().position(|a| a == "--tab-width").map(|pos| {
        args.remove(pos);
        if pos >= args.len() {
            eprintln!("Error: --tab-width requires a number argument");
            std::process::exit(1);
        }
        let val = args.remove(pos);
        val.parse().unwrap_or_else(|_| {
            eprintln!("Error: --tab-width must be a positive integer");
            std::process::exit(1);
        })
    });

    // Parse --collapse-max-keyvalue-props CLI override
    let cli_max_keyvalue_props: Option<usize> = args.iter().position(|a| a == "--collapse-max-keyvalue-props").map(|pos| {
        args.remove(pos);
        if pos >= args.len() {
            eprintln!("Error: --collapse-max-keyvalue-props requires a number argument");
            std::process::exit(1);
        }
        let val = args.remove(pos);
        val.parse().unwrap_or_else(|_| {
            eprintln!("Error: --collapse-max-keyvalue-props must be a non-negative integer");
            std::process::exit(1);
        })
    });

    // Parse --oneline <N> flag (collapse multi-line HTML leaf elements to one line when they fit within N columns)
    let cli_oneline: Option<usize> = if let Some(pos) = args.iter().position(|a| a == "--oneline") {
        args.remove(pos);
        let val = args.remove(pos).parse::<usize>().unwrap_or_else(|_| {
            eprintln!("Error: --oneline requires a numeric width argument");
            std::process::exit(1);
        });
        Some(val)
    } else {
        None
    };

    let mode = if diff_mode {
        format::Mode::Diff
    } else if check_mode {
        format::Mode::Check
    } else {
        format::Mode::Write
    };

    // Parse --stdin flag (parse args before version/init so --version --stdin works)
    let stdin_mode = args.iter().position(|a| a == "--stdin");
    let stdin_ext: Option<String> = stdin_mode.and_then(|pos| {
        args.remove(pos);
        if let Some(first) = args.first() {
            if first.starts_with('.') {
                // Explicit bare extension: `.ts`, `.js`, `.ree`
                Some(args.remove(0))
            } else if std::path::Path::new(first).extension().is_some() {
                // Full filename hint: `edit_handlers.ts` — extract the extension
                let ext = format!(".{}", std::path::Path::new(first).extension().unwrap().to_str().unwrap_or(""));
                args.remove(0);
                Some(ext)
            } else {
                None
            }
        } else {
            None
        }
    });

    // Check for --help
    if args.iter().any(|a| a == "--help" || a == "-h") {
        println!("reefmt v{} — formatter for .ree, .ts, .js, .css files", env!("CARGO_PKG_VERSION"));
        println!();
        println!("USAGE:");
        println!("  reefmt [OPTIONS] [PATH...]");
        println!();
        println!("OPTIONS:");
        println!("  --diff                   Show diff without writing files");
        println!("  --check, -c              List files that would change (exit 1 if any)");
        println!("  --git                    Format only uncommitted (git-changed) files");
        println!("  --verbose                Print files that were already formatted");
        println!("  --oneline <N>            Collapse multi-line leaf HTML elements to one line when they fit within N columns");
        println!("  --wrap-width <N>         Override wrapWidth from config");
        println!("  --collapse-max-members <N>  Override collapseMaxMembers from config");
        println!("  --collapse-soft-width <N>   Override collapseSoftWidth from config (0 disables)");
        println!("  --tab-width <N>              Override tabWidth from config (tab display columns)");
        println!("  --collapse-max-keyvalue-props <N>  Override collapseMaxKeyValueProps (0 never collapses)");
        println!("  --stdin <.ext>           Read from stdin, write to stdout (.ree, .ts, .js, .css)");
        println!("  --init                   Create or upgrade reefmt.jsonc in the current directory");
        println!("  --version, -v            Print version");
        println!("  --help, -h               Print this help");
        println!();
        println!("CONFIG: reefmt.jsonc in the current directory (or nearest parent)");
        return;
    }

    // Check for --version (no config needed)
    if args.len() == 1 && (args[0] == "-v" || args[0] == "--version") {
        println!("reefmt v{}", env!("CARGO_PKG_VERSION"));
        return;
    }

    // Check for --init (no config needed — creates or upgrades it)
    if args.iter().any(|a| a == "--init") {
        let cwd = env::current_dir().unwrap_or_else(|_| {
            eprintln!("Error: could not determine current directory");
            std::process::exit(1);
        });
        let config_path = cwd.join("reefmt.jsonc");
        let template = include_str!("../reefmt.jsonc");
        if config_path.exists() {
            let existing = fs::read_to_string(&config_path).unwrap_or_else(|e| {
                eprintln!("Error reading {}: {}", config_path.display(), e);
                std::process::exit(1);
            });
            let (upgraded, added_keys) = upgrade_init_config(&existing, template);
            if added_keys.is_empty() {
                println!("{} is already up to date.", config_path.display());
            } else {
                match fs::write(&config_path, &upgraded) {
                    Ok(_) => {
                        println!("Upgraded: {}", config_path.display());
                        for key in &added_keys {
                            println!("  + {}", key);
                        }
                    }
                    Err(e) => {
                        eprintln!("Error writing {}: {}", config_path.display(), e);
                        std::process::exit(1);
                    }
                }
            }
        } else {
            match fs::write(&config_path, template.trim_start()) {
                Ok(_) => {
                    println!("Created: {}", config_path.display());
                    println!("Edit this file to configure reefmt formatting behavior.");
                }
                Err(e) => {
                    eprintln!("Error writing {}: {}", config_path.display(), e);
                    std::process::exit(1);
                }
            }
        }
        return;
    }

    // Load config (required for all remaining operations)
    let mut config = load_config();

    // CLI --collapse-max-members overrides config
    if let Some(max_members) = cli_max_members {
        config.collapse_max_members = max_members;
    }

    // CLI --wrap-width overrides config
    if let Some(w) = cli_wrap_width {
        config.wrap_width = w;
    }

    // CLI --collapse-soft-width overrides config
    if let Some(w) = cli_soft_width {
        config.collapse_soft_width = w;
    }

    // CLI --tab-width overrides config
    if let Some(w) = cli_tab_width {
        config.tab_width = w;
    }

    // CLI --collapse-max-keyvalue-props overrides config
    if let Some(max) = cli_max_keyvalue_props {
        config.collapse_max_keyvalue_props = max;
    }

    // CLI --oneline overrides config
    if let Some(w) = cli_oneline {
        config.oneline = w;
    }

    // Handle --stdin (uses config)
    if stdin_mode.is_some() {
        let mut input = String::new();
        if let Err(e) = std::io::stdin().read_to_string(&mut input) {
            eprintln!("Error reading stdin: {}", e);
            std::process::exit(1);
        }

        let ext = stdin_ext.as_deref().unwrap_or(".ts");
        let ext = ext.trim_start_matches('.');

        // If the extension matches a skipExtension entry, pass content through unchanged.
        if config.skip_extensions.iter().any(|skip| ext == skip.as_str()) {
            print!("{}", input);
            return;
        }

        let collapse = config.collapse_config();
        let formatted = match ext {
            "ree" => ree_format::format_ree_content(&input, config.wrap_width, config.oneline, collapse, config.remove_unused_imports),
            "ts" | "js" | "css" => format::format_code_content(&input, ext, config.wrap_width, collapse, config.remove_unused_imports),
            _ => {
                eprintln!("Unsupported extension for --stdin: .{}", ext);
                std::process::exit(1);
            }
        };

        print!("{}", formatted);
        return;
    }

    let targets: Vec<String> = if args.is_empty() {
        vec![".".to_string()]
    } else {
        args
    };

    let modified_count = AtomicU64::new(0);

    let start = std::time::Instant::now();

    // Collect files to format
    let all_files: Vec<PathBuf> = if git_mode {
        get_git_changed_files(&config)
    } else {
        let mut files = Vec::new();
        for target in &targets {
            let path = Path::new(target);
            if path.is_dir() {
                if let Err(e) = collect_source_files(path, &mut files, &config) {
                    eprintln!("Error reading directory {}: {}", target, e);
                    continue;
                }
            } else if path.exists() {
                files.push(path.to_path_buf());
            } else {
                match glob(target) {
                    Ok(paths) => {
                        for entry in paths.flatten() {
                            if should_skip_path(&entry, &config) {
                                continue;
                            }
                            files.push(entry);
                        }
                    }
                    Err(e) => eprintln!("Invalid glob {}: {}", target, e),
                }
            }
        }
        files
    };

    if git_mode && all_files.is_empty() {
        return; // No uncommitted changes — nothing to do
    }

    all_files.par_iter().for_each(|file| {
        if format::format_file(file, mode, &config) {
            modified_count.fetch_add(1, Ordering::SeqCst);
        } else if verbose {
            eprintln!("Already formatted: {}", file.display());
        }
    });

    let elapsed = start.elapsed();
    let file_count = all_files.len();
    let modified = modified_count.load(Ordering::SeqCst);
    if file_count > 0 {
        eprintln!("Formatted {} of {} file{} in {:.2}s",
            modified,
            file_count,
            if file_count == 1 { "" } else { "s" },
            elapsed.as_secs_f64());
    }

    if mode != format::Mode::Write && modified > 0 {
        std::process::exit(1);
    }
}

/// Merge template keys into an existing reefmt.jsonc.
/// Returns the merged content and the list of keys that were added.
fn upgrade_init_config(existing: &str, template: &str) -> (String, Vec<String>) {
    // Parse template into groups: each group is (comment_lines, key_name, full_key_line).
    // Comment lines are the `// ...` lines immediately preceding a key.
    struct Group {
        comments: Vec<String>,
        key: String,
        line: String,
    }
    let mut groups: Vec<Group> = vec![];
    let mut pending_comments: Vec<String> = vec![];
    for raw in template.lines() {
        let trimmed = raw.trim();
        if trimmed.starts_with("//") {
            pending_comments.push(raw.to_string());
        } else if trimmed.starts_with('"') {
            // Extract key name (the part between the first pair of quotes)
            let key = trimmed
                .trim_start_matches('"')
                .split('"')
                .next()
                .unwrap_or("")
                .to_string();
            if !key.is_empty() {
                groups.push(Group {
                    comments: std::mem::take(&mut pending_comments),
                    key,
                    line: raw.to_string(),
                });
            } else {
                pending_comments.clear();
            }
        } else if !trimmed.is_empty() && trimmed != "{" && trimmed != "}" {
            pending_comments.clear();
        }
    }

    // Find which keys are missing from the existing file.
    let mut additions = String::new();
    let mut added_keys: Vec<String> = vec![];
    for g in &groups {
        let needle = format!("\"{}\"", g.key);
        if !existing.contains(&needle) {
            for c in &g.comments {
                additions.push_str(c);
                additions.push('\n');
            }
            // Ensure key line has a trailing comma (valid in JSON5/JSONC).
            let key_line = if g.line.trim_end().ends_with(',') {
                g.line.clone()
            } else {
                format!("{},", g.line.trim_end())
            };
            additions.push_str(&key_line);
            additions.push('\n');
            added_keys.push(g.key.clone());
        }
    }

    if added_keys.is_empty() {
        return (existing.to_string(), added_keys);
    }

    // Insert additions just before the final `}`.
    // Find the last `}` line.
    let last_brace = existing.rfind('}');
    let (before, after) = if let Some(pos) = last_brace {
        (&existing[..pos], &existing[pos..])
    } else {
        return (existing.to_string(), vec![]);
    };

    // Ensure the content before the new entries ends with a comma.
    let before_trimmed = before.trim_end();
    let needs_comma = !before_trimmed.ends_with(',')
        && !before_trimmed.ends_with('{')
        && !before_trimmed.is_empty();

    let mut result = before_trimmed.to_string();
    if needs_comma {
        result.push(',');
    }
    result.push('\n');
    result.push_str(&additions);
    result.push_str(after);
    if !result.ends_with('\n') {
        result.push('\n');
    }
    (result, added_keys)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_file_unsupported_extension_returns_false() {
        let dir = env::temp_dir().join("reefmt_test_unsupported");
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test.txt");
        fs::write(&path, "hello world").unwrap();

        let config = ReeConfig {
            skip_dirs: vec![],
            skip_files: vec![],
            skip_extensions: vec![],
            extensions: vec!["ree".to_string()],
            skip_dot_dirs: false,
            wrap_width: 120,
            collapse_single_stmt_blocks: true,
            collapse_max_members: 3,
            collapse_max_object_members: None,
            collapse_max_array_elements: None,
            collapse_max_function_params: None,
            collapse_max_call_args: None,
            collapse_max_imports: None,
            collapse_max_type_members: None,
            collapse_soft_width: 0,
            tab_width: 4,
            collapse_max_keyvalue_props: 1,
            hug_call_args: false,
            remove_unused_imports: false,
            oneline: 0,
        };
        let modified = format::format_file(&path, format::Mode::Write, &config);
        assert!(!modified, "format_file should return false for unsupported extension");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn check_mode_does_not_modify_ree_file() {
        let dir = env::temp_dir().join("reefmt_test_check_mode");
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test.ree");
        let unformatted = "{#if show}\n<div>\n{=title}\n</div>\n{/if}";
        fs::write(&path, unformatted).unwrap();

        let modified = crate::ree_format::format_ree_file(&path, format::Mode::Check, 120, 0, crate::format::CollapseConfig::uniform(true, 3), false);
        assert!(modified, "Check mode should return true when file would change");
        let content_after = fs::read_to_string(&path).unwrap();
        assert_eq!(content_after, unformatted, "Check mode should not modify the file");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn config_file_deserializes_correctly() {
        // This test parses the actual reefmt.jsonc shipped with the project.
        // It would have caught the missing #[serde(rename = "skipFiles")] bug
        // on the skip_files field — none of the other tests exercise
        // json5 deserialization, so that bug slipped through.
        let content = include_str!("../reefmt.jsonc");
        let config: ReeConfig = json5::from_str(content).expect(
            "Failed to parse reefmt.jsonc — serde field names or types may be out of sync with the config file"
        );
        // Verify a few known values from the template
        assert!(config.skip_dirs.contains(&"node_modules".to_string()));
        assert!(config.skip_files.is_empty());
        assert!(config.extensions.contains(&"ree".to_string()));
        assert!(config.skip_dot_dirs);
        assert_eq!(config.wrap_width, 180);
        assert_eq!(config.collapse_max_members, 4);
        assert_eq!(config.collapse_max_object_members, Some(4));
        assert_eq!(config.collapse_max_array_elements, Some(4));
        assert_eq!(config.collapse_max_function_params, Some(4));
        assert_eq!(config.collapse_max_call_args, Some(4));
        assert_eq!(config.collapse_max_imports, Some(4));
        assert_eq!(config.collapse_max_type_members, Some(4));
        assert_eq!(config.collapse_soft_width, 100);
        assert_eq!(config.tab_width, 4);
        assert_eq!(config.collapse_max_keyvalue_props, 1);
    }

    #[test]
    fn skip_files_glob_matches_relative_path() {
        let config = ReeConfig {
            skip_dirs: vec![],
            skip_files: vec!["generator/templates/**/*.ts".to_string()],
            skip_extensions: vec![],
            extensions: vec!["ts".to_string()],
            skip_dot_dirs: false,
            wrap_width: 120,
            collapse_single_stmt_blocks: true,
            collapse_max_members: 3,
            collapse_max_object_members: None,
            collapse_max_array_elements: None,
            collapse_max_function_params: None,
            collapse_max_call_args: None,
            collapse_max_imports: None,
            collapse_max_type_members: None,
            collapse_soft_width: 0,
            tab_width: 4,
            collapse_max_keyvalue_props: 1,
            hug_call_args: false,
            remove_unused_imports: false,
            oneline: 0,
        };

        let matched = Path::new("generator/templates/ui/button.ts");
        assert!(should_skip_file(matched, &config));
        let not_matched = Path::new("src/ui/button.ts");
        assert!(!should_skip_file(not_matched, &config));
    }
}
