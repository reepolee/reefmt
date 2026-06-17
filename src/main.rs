mod ree_format;
mod format;
mod ree_parser;
mod swc_format;

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

/// Reefmt configuration — loaded from `reefmt.jsonc` in the project root.
#[derive(Deserialize)]
#[serde(default)]
pub(crate) struct ReeConfig {
    /// Directories to skip when formatting.
    #[serde(rename = "skipDirs")]
    skip_dirs: Vec<String>,
    /// Glob patterns for files to skip (e.g. "generator/templates/**/*.ts").
    skip_files: Vec<String>,
    /// File extensions to format.
    extensions: Vec<String>,
    /// Whether to skip dot directories (folders starting with '.').
    #[serde(rename = "skipDotDirs")]
    skip_dot_dirs: bool,
    /// Maximum line width before elements are broken onto multiple lines.
    #[serde(rename = "wrapWidth")]
    wrap_width: usize,
}

impl Default for ReeConfig {
    fn default() -> Self {
        Self {
            skip_dirs: vec![
                "node_modules".to_string(),
                "vendor".to_string(),
                "vendors".to_string(),
                "dist".to_string(),
                "templates".to_string(),
                "static".to_string(),
            ],
            extensions: vec![
                "ree".to_string(),
                "ts".to_string(),
                "js".to_string(),
                "css".to_string(),
            ],
            skip_files: vec![],
            skip_dot_dirs: true,
            wrap_width: 120,
        }
    }
}

/// Load reefmt config from `reefmt.jsonc` in the current directory.
fn load_config() -> ReeConfig {
    if let Ok(cwd) = env::current_dir() {
        let config_path = cwd.join("reefmt.jsonc");
        if config_path.exists() {
            match fs::read_to_string(&config_path) {
                Ok(content) => match json5::from_str(&content) {
                    Ok(config) => return config,
                    Err(e) => eprintln!(
                        "Warning: invalid reefmt.jsonc: {}, using defaults",
                        e
                    ),
                },
                Err(e) => eprintln!(
                    "Warning: could not read reefmt.jsonc: {}, using defaults",
                    e
                ),
            }
        }
    }
    ReeConfig::default()
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
                if config.extensions.iter().any(|e| e == ext) {
                    if !should_skip_file(&path, config) {
                        files.push(path);
                    }
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
                    if config.extensions.iter().any(|e| e == ext) {
                        if !should_skip_file(&path, config) {
                            files.push(path);
                        }
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

    let mode = if diff_mode {
        format::Mode::Diff
    } else if check_mode {
        format::Mode::Check
    } else {
        format::Mode::Write
    };

    let config = load_config();

    // Parse --stdin flag
    let stdin_mode = args.iter().position(|a| a == "--stdin");
    let stdin_ext: Option<String> = stdin_mode.and_then(|pos| {
        args.remove(pos);
        if args.first().is_some_and(|a| a.starts_with('.')) {
            Some(args.remove(0))
        } else {
            None
        }
    });

    if stdin_mode.is_some() {
        let mut input = String::new();
        if let Err(e) = std::io::stdin().read_to_string(&mut input) {
            eprintln!("Error reading stdin: {}", e);
            std::process::exit(1);
        }

        let ext = stdin_ext.as_deref().unwrap_or(".ree");
        let ext = ext.trim_start_matches('.');

        let formatted = match ext {
            "ree" => ree_format::format_ree_content(&input, config.wrap_width),
            "ts" | "js" | "css" => format::format_code_content(&input, ext),
            _ => {
                eprintln!("Unsupported extension for --stdin: .{}", ext);
                std::process::exit(1);
            }
        };

        print!("{}", formatted);
        return;
    }

    if args.len() == 1 && (args[0] == "-v" || args[0] == "--version") {
        println!("reefmt v{}", env!("CARGO_PKG_VERSION"));
        return;
    }

    // Check for --init flag
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
        let template = r##"{
	// Directories to skip when formatting.
	"skipDirs": ["node_modules", "vendor", "vendors", "dist", "templates", "static"],

	// Glob patterns for files to skip.
	"skipFiles": [],

	// File extensions to format.
	"extensions": ["ree", "ts", "js", "css"],

	// Whether to skip dot-directories.
	"skipDotDirs": true,

	// Maximum line width before elements are broken onto multiple lines.
	"wrapWidth": 120
}"##;
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

        let config = ReeConfig::default();
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

        let modified = crate::ree_format::format_ree_file(&path, format::Mode::Check, 120);
        assert!(modified, "Check mode should return true when file would change");
        let content_after = fs::read_to_string(&path).unwrap();
        assert_eq!(content_after, unformatted, "Check mode should not modify the file");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn skip_files_glob_matches_relative_path() {
        let mut config = ReeConfig::default();
        config.skip_files = vec!["generator/templates/**/*.ts".to_string()];

        let matched = Path::new("generator/templates/ui/button.ts");
        assert!(should_skip_file(&matched, &config));

        let not_matched = Path::new("src/ui/button.ts");
        assert!(!should_skip_file(&not_matched, &config));
    }
}
