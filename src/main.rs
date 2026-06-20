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
fn default_three() -> usize { 3 }
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
    /// Maximum number of object literal members or type literal members
    /// allowed before collapsing is prevented. Members beyond this count
    /// stay multi-line regardless of wrapWidth.
    #[serde(rename = "collapseMaxMembers", default = "default_three")]
    collapse_max_members: usize,
    /// When true, unused import declarations are removed from JS/TS files
    /// during formatting. Side-effect imports (`import "./foo"`) are always kept.
    #[serde(rename = "removeUnusedImports", default)]
    remove_unused_imports: bool,
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

    // Check for --version (no config needed)
    if args.len() == 1 && (args[0] == "-v" || args[0] == "--version") {
        println!("reefmt v{}", env!("CARGO_PKG_VERSION"));
        return;
    }

    // Check for --init (no config needed — creates it)
    if args.iter().any(|a| a == "--init") {
        let cwd = env::current_dir().unwrap_or_else(|_| {
            eprintln!("Error: could not determine current directory");
            std::process::exit(1);
        });
        let config_path = cwd.join("reefmt.jsonc");
        if config_path.exists() {
            eprintln!(
                "Error: {} already exists in this directory",
                config_path.display()
            );
            std::process::exit(1);
        }
        let template = include_str!("../reefmt.jsonc");
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
        return;
    }

    // Load config (required for all remaining operations)
    let mut config = load_config();

    // CLI --collapse-max-members overrides config
    if let Some(max_members) = cli_max_members {
        config.collapse_max_members = max_members;
    }

    // Handle --stdin (uses config)
    if stdin_mode.is_some() {
        let mut input = String::new();
        if let Err(e) = std::io::stdin().read_to_string(&mut input) {
            eprintln!("Error reading stdin: {}", e);
            std::process::exit(1);
        }

        let ext = stdin_ext.as_deref().unwrap_or(".ree");
        let ext = ext.trim_start_matches('.');

        let formatted = match ext {
            "ree" => ree_format::format_ree_content(&input, config.wrap_width, config.collapse_single_stmt_blocks, config.collapse_max_members, config.remove_unused_imports),
            "ts" | "js" | "css" => format::format_code_content(&input, ext, config.wrap_width, config.collapse_single_stmt_blocks, config.collapse_max_members, config.remove_unused_imports),
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
            remove_unused_imports: false,
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

        let modified = crate::ree_format::format_ree_file(&path, format::Mode::Check, 120, true, 3, false);
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
        assert_eq!(config.collapse_max_members, 3);
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
            remove_unused_imports: false,
        };

        let matched = Path::new("generator/templates/ui/button.ts");
        assert!(should_skip_file(matched, &config));
        let not_matched = Path::new("src/ui/button.ts");
        assert!(!should_skip_file(not_matched, &config));
    }
}
