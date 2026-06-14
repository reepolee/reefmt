mod ree_tags;
mod ree_format;
mod format;
mod cache;

use serde::Deserialize;
use std::{
    env, fs,
    io::Read,
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, Ordering},
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
    #[serde(rename = "skipFiles")]
    skip_files: Vec<String>,
    /// File extensions to format.
    extensions: Vec<String>,
    /// Whether to skip dot directories (folders starting with '.').
    #[serde(rename = "skipDotDirs")]
    skip_dot_dirs: bool,
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
        }
    }
}

/// Load reefmt config from `reefmt.jsonc` in the current directory.
/// Falls back to hardcoded defaults if the file doesn't exist or is invalid.
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

/// Check whether a path is inside a directory that should be skipped
/// (e.g. `node_modules`, `vendor`, or any dot-folder like `.git`).
fn should_skip_path(path: &Path, config: &ReeConfig) -> bool {
    path.components().any(|c| {
        if let std::path::Component::Normal(s) = c {
            if let Some(name) = s.to_str() {
                if config.skip_dirs.iter().any(|d| d == name) {
                    return true;
                }
                if config.skip_dot_dirs && name.starts_with(".") && name != "." {
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

    let no_cache = args.iter().position(|a| a == "--no-cache").is_some();
    if no_cache {
        args.retain(|a| a != "--no-cache");
    }

    let mode = if diff_mode {
        format::Mode::Diff
    } else if check_mode {
        format::Mode::Check
    } else {
        format::Mode::Write
    };

    // Parse --stdin flag (consumes an optional extension argument)
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
            "ree" => ree_format::format_ree_content(&input),
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

    // Check for --init flag (generate config template)
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
	// Directories to skip when formatting (glob patterns not needed,
	// just directory names — any folder with this name is skipped).
	"skipDirs": ["node_modules", "vendor", "vendors", "dist", "templates", "static"],

	// Glob patterns for files to skip (e.g. "generator/templates/**/*.ts").
	// Matches file paths relative to the project root.
	"skipFiles": [".reefmt-cache"],

	// File extensions to format.
	"extensions": ["ree", "ts", "js", "css"],

	// Whether to skip dot-directories (folders starting with '.',
	// like .git, .next, .cache, .svelte-kit, etc.).
	"skipDotDirs": true
}
"##;
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

    let config = load_config();
    let any_modified = AtomicBool::new(false);

    // Phase 1: collect all file paths from all targets
    let mut all_files: Vec<PathBuf> = Vec::new();

    for target in targets {
        let path = Path::new(&target);

        if path.is_dir() {
            let mut files = Vec::new();
            if let Err(e) = collect_source_files(path, &mut files, &config) {
                eprintln!("Error reading directory {}: {}", target, e);
                continue;
            }
            all_files.extend(files);
        } else if path.exists() {
            all_files.push(path.to_path_buf());
        } else {
            match glob(&target) {
                Ok(paths) => {
                    for entry in paths.flatten() {
                        if should_skip_path(&entry, &config) {
                            continue;
                        }
                        all_files.push(entry);
                    }
                }
                Err(e) => eprintln!("Invalid glob {}: {}", target, e),
            }
        }
    }

    if no_cache {
        // Phase 2 (no-cache): process all files in parallel, skip cache entirely
        all_files.par_iter().for_each(|file| {
            if format::format_file(file, mode, &config) {
                any_modified.store(true, Ordering::SeqCst);
            }
        });
    } else {
        // Phase 2: load cache and filter out files that are already up to date
        let mut cache = cache::FormatCache::load();
        let uncached: Vec<PathBuf> = all_files
            .into_iter()
            .filter(|f| !cache.is_fresh(f))
            .collect();

        // Phase 3: format remaining files in parallel
        if !uncached.is_empty() {
            uncached.par_iter().for_each(|file| {
                if format::format_file(file, mode, &config) {
                    any_modified.store(true, Ordering::SeqCst);
                }
            });

            // Phase 4: update cache for all processed files
            for file in &uncached {
                cache.mark_fresh(file);
            }
            cache.save();
        }
    }

    if mode != format::Mode::Write && any_modified.load(Ordering::SeqCst) {
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
    fn format_file_check_mode_unsupported_extension() {
        let dir = env::temp_dir().join("reefmt_test_unsupported_check");
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test.txt");
        fs::write(&path, "hello world").unwrap();

        let config = ReeConfig::default();
        let modified = format::format_file(&path, format::Mode::Check, &config);
        assert!(!modified, "format_file Check should return false for unsupported extension");
        let content_after = fs::read_to_string(&path).unwrap();
        assert_eq!(content_after, "hello world", "Check mode should not modify the file");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn check_mode_empty_ree_file_returns_false() {
        let dir = env::temp_dir().join("reefmt_test_empty_check");
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("empty.ree");
        fs::write(&path, "").unwrap();

        let modified = ree_format::format_ree_file(&path, format::Mode::Check);
        assert!(!modified, "Check mode should return false for empty file");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn check_mode_via_format_file_dispatcher() {
        let dir = env::temp_dir().join("reefmt_test_file_check");
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test.ree");
        let unformatted = "{#if show}\n<div>\n{=title}\n</div>\n{/if}";
        fs::write(&path, unformatted).unwrap();

        let config = ReeConfig::default();
        let modified = format::format_file(&path, format::Mode::Check, &config);
        assert!(modified, "format_file Check should detect unformatted .ree file");
        let content_after = fs::read_to_string(&path).unwrap();
        assert_eq!(content_after, unformatted, "format_file Check should not modify the file");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn diff_mode_via_format_file_dispatcher() {
        let dir = env::temp_dir().join("reefmt_test_file_diff");
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test.ree");
        let unformatted = "{#if show}\n<div>\n{=title}\n</div>\n{/if}";
        fs::write(&path, unformatted).unwrap();

        let config = ReeConfig::default();
        let modified = format::format_file(&path, format::Mode::Diff, &config);
        assert!(modified, "format_file Diff should detect unformatted .ree file");
        let content_after = fs::read_to_string(&path).unwrap();
        assert_eq!(content_after, unformatted, "format_file Diff should not modify the file");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn skip_files_glob_matches_relative_path() {
        let mut config = ReeConfig::default();
        config.skip_files = vec!["generator/templates/**/*.ts".to_string()];

        let matched = Path::new("generator/templates/ui/button.ts");
        assert!(should_skip_file(&matched, &config));

        let not_matched_ree = Path::new("generator/templates/ui/button.ree");
        assert!(!should_skip_file(&not_matched_ree, &config));

        let not_matched_outside = Path::new("src/ui/button.ts");
        assert!(!should_skip_file(&not_matched_outside, &config));

        config.skip_files = vec![];
        let empty_skip = Path::new("generator/templates/ui/button.ts");
        assert!(!should_skip_file(&empty_skip, &config));
    }

    #[test]
    fn skip_files_handles_absolute_paths() {
        let mut config = ReeConfig::default();
        config.skip_files = vec!["templates/**/*.ts".to_string()];

        let cwd = env::current_dir().unwrap();
        let abs_path = cwd.join("templates/ui/button.ts");
        assert!(should_skip_file(&abs_path, &config));

        let outside = Path::new("/nonexistent/templates/ui/button.ts");
        assert!(!should_skip_file(&outside, &config));
    }

    #[test]
    fn init_template_parses_as_valid_config() {
        let template = r#"{
	// Directories to skip when formatting (glob patterns not needed,
	// just directory names — any folder with this name is skipped).
	"skipDirs": ["node_modules", "vendor", "vendors", "dist", "templates", "static"],

	// Glob patterns for files to skip (e.g. "generator/templates/**/*.ts").
	// Matches file paths relative to the project root.
	"skipFiles": [".reefmt-cache"],

	// File extensions to format.
	"extensions": ["ree", "ts", "js", "css"],

	// Whether to skip dot-directories (folders starting with '.',
	// like .git, .next, .cache, .svelte-kit, etc.).
	"skipDotDirs": true
}"#;

        let config: ReeConfig =
            json5::from_str(template).expect("--init template should be valid JSONC");

        assert_eq!(config.skip_dirs.len(), 6);
        assert!(config.skip_dirs.contains(&"node_modules".to_string()));
        assert!(config.skip_dirs.contains(&"vendor".to_string()));
        assert!(config.skip_dirs.contains(&"vendors".to_string()));
        assert!(config.skip_dirs.contains(&"dist".to_string()));
        assert!(config.skip_dirs.contains(&"templates".to_string()));
        assert!(config.skip_dirs.contains(&"static".to_string()));

        assert_eq!(config.extensions.len(), 4);
        assert!(config.extensions.contains(&"ree".to_string()));
        assert!(config.extensions.contains(&"ts".to_string()));
        assert!(config.extensions.contains(&"js".to_string()));
        assert!(config.extensions.contains(&"css".to_string()));

        assert!(config.skip_dot_dirs);
        assert!(config.skip_files.contains(&".reefmt-cache".to_string()));
    }

    #[test]
    fn load_config_parses_reefmt_jsonc_from_directory() {
        let dir = env::temp_dir().join(format!(
            "reefmt_test_load_config_{}",
            std::process::id()
        ));
        fs::create_dir_all(&dir).unwrap();

        let config_content = r#"{
	"skipDirs": ["node_modules", "dist"],
	"extensions": ["ree", "ts"],
	"skipDotDirs": false
}"#;
        fs::write(dir.join("reefmt.jsonc"), config_content).unwrap();

        let original_cwd = env::current_dir().unwrap();
        env::set_current_dir(&dir).unwrap();

        let config = load_config();

        env::set_current_dir(&original_cwd).unwrap();

        assert_eq!(config.skip_dirs.len(), 2);
        assert!(!config.skip_dirs.contains(&"vendor".to_string()));

        assert_eq!(config.extensions.len(), 2);
        assert!(!config.extensions.contains(&"js".to_string()));

        assert!(!config.skip_dot_dirs);

        let _ = fs::remove_dir_all(&dir);
    }
}
