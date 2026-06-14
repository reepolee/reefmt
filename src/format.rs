use std::env;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};
use similar::{ChangeTag, DiffOp};

use crate::ree_tags::{protect, restore, protect_html_comments, restore_html_comments};
use crate::ree_format::{flatten_concat, indent_code};

/// Operating mode: write files, check-only (list files), or diff (show changes).
#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Mode { Write, Check, Diff }

/// Default dprint config, embedded at build time.
pub(crate) const DPRINT_CONFIG: &str = include_str!("../dprint.default.json");
/// Default biome config, embedded at build time.
pub(crate) const BIOME_CONFIG: &str = include_str!("../biome.default.json");

/// Print a unified diff between original and formatted content.
pub(crate) fn print_diff(path: &Path, original: &str, formatted: &str) {
    let path_str = path.to_string_lossy();
    let diff = similar::TextDiff::from_lines(original, formatted);

    println!("--- a/{}", path_str);
    println!("+++ b/{}", path_str);

    for op in diff.ops() {
        match op {
            DiffOp::Equal { .. } => continue,
            _ => {
                let old_range = op.old_range();
                let new_range = op.new_range();
                let old_count = old_range.end - old_range.start;
                let new_count = new_range.end - new_range.start;
                println!(
                    "@@ -{},{} +{},{} @@",
                    old_range.start + 1,
                    if old_count == 0 { 1 } else { old_count },
                    new_range.start + 1,
                    if new_count == 0 { 1 } else { new_count },
                );
                for change in diff.iter_changes(op) {
                    match change.tag() {
                        ChangeTag::Delete => print!("-{}", change.value()),
                        ChangeTag::Insert => print!("+{}", change.value()),
                        ChangeTag::Equal => print!(" {}", change.value()),
                    }
                }
            }
        }
    }
}

/// Resolve the dprint config path. If a `dprint.json` exists in the current
/// directory, use it directly. Otherwise, write the hardcoded defaults to a
/// temp file and return that path.
pub(crate) fn resolve_dprint_config(timestamp: u128) -> Option<String> {
    if let Ok(cwd) = env::current_dir() {
        let external = cwd.join("dprint.json");
        if external.exists() {
            return Some(external.to_string_lossy().into_owned());
        }
        let external_jsonc = cwd.join("dprint.jsonc");
        if external_jsonc.exists() {
            return Some(external_jsonc.to_string_lossy().into_owned());
        }
    }

    let config_path = env::temp_dir()
        .join(format!("reefmt_dprint_config_{}.json", timestamp))
        .to_string_lossy()
        .into_owned();

    if fs::write(&config_path, DPRINT_CONFIG).is_ok() {
        Some(config_path)
    } else {
        None
    }
}

/// Run biome lint --fix on the JS/CSS/TS fragment (via temp file).
pub(crate) fn run_biome_lint(src: &str, ext: &str, timestamp: u128) -> String {
    let dir = env::temp_dir().join(format!("reefmt_biome_{}", timestamp));
    if fs::create_dir_all(&dir).is_err() {
        return src.to_string();
    }

    let tmp_path = dir.join(format!("input.{}", ext));
    let config_path = dir.join("biome.json");

    let biome_config = env::current_dir()
        .ok()
        .and_then(|cwd| {
            let path = cwd.join("biome.json");
            if path.exists() {
                fs::read_to_string(&path).ok()
            } else {
                let path = cwd.join("biome.jsonc");
                if path.exists() {
                    fs::read_to_string(&path).ok()
                } else {
                    None
                }
            }
        })
        .unwrap_or_else(|| BIOME_CONFIG.to_string());

    if fs::write(&config_path, biome_config).is_err()
        || fs::write(&tmp_path, src).is_err()
    {
        let _ = fs::remove_dir_all(&dir);
        return src.to_string();
    }

    let filename = format!("input.{}", ext);
    let result = Command::new("biome")
        .current_dir(&dir)
        .args(["lint", "--write", "--unsafe", &filename])
        .output();

    let output = match result {
        Ok(_) => match fs::read_to_string(&tmp_path) {
            Ok(content) => content,
            Err(_) => src.to_string(),
        },
        Err(_) => src.to_string(),
    };

    let _ = fs::remove_dir_all(&dir);
    output
}

/// Pipe content through dprint for formatting.
pub(crate) fn pipe_dprint(src: &str, ext: &str, config_path: &str) -> String {
    let mut child = match Command::new("dprint")
        .args(["fmt", "--stdin", &format!("file.{}", ext), "--config", config_path])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(c) => c,
        Err(_) => return src.to_string(),
    };

    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(src.as_bytes());
    }

    match child.wait_with_output() {
        Ok(output) if output.status.success() => {
            String::from_utf8_lossy(&output.stdout).to_string()
        }
        _ => src.to_string(),
    }
}

/// Format code (JS/TS/CSS) through biome lint-fix + dprint formatting.
pub(crate) fn dprint_format(src: &str, lang: &str) -> String {
    let ext = if lang == "css" { "css" } else { "js" };
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();

    let (html_protected, html_comments) = protect_html_comments(src.trim());
    let protected = protect(&html_protected);
    let flattened = flatten_concat(&protected);
    let after_lint = run_biome_lint(&flattened, ext, timestamp);

    let config_path = resolve_dprint_config(timestamp);
    let formatted = match config_path {
        Some(ref path) => {
            let result = pipe_dprint(&after_lint, ext, path);
            if path.contains("reefmt_dprint_config_") {
                let _ = fs::remove_file(path);
            }
            result
        }
        None => after_lint,
    };

    let restored = restore(&formatted);
    let restored = restore_html_comments(&restored, &html_comments);
    let indented = indent_code(&restored);

    indented
        .trim()
        .lines()
        .map(|l| format!("\t{}", l))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Format standalone code content (TS/JS/CSS) via biome lint-fix then dprint.
pub(crate) fn format_code_content(content: &str, ext: &str) -> String {
    let normalized = content.replace("\r\n", "\n");

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();

    let flattened = flatten_concat(&normalized);
    let after_lint = run_biome_lint(&flattened, ext, timestamp);

    let formatted = match resolve_dprint_config(timestamp) {
        Some(ref config_path) => {
            let result = pipe_dprint(&after_lint, ext, config_path);
            if config_path.contains("reefmt_dprint_config_") {
                let _ = fs::remove_file(config_path);
            }
            result
        }
        None => after_lint,
    };

    if !formatted.is_empty() && !formatted.ends_with('\n') {
        format!("{}\n", formatted)
    } else {
        formatted
    }
}

/// Format a standalone code file (TS, JS, CSS). Returns `true` if modified.
pub(crate) fn format_code_file(path: &Path, mode: Mode) -> bool {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error reading {}: {}", path.display(), e);
            return false;
        }
    };

    let normalized = content.replace("\r\n", "\n");
    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
    let write_content = format_code_content(&normalized, ext);

    if write_content == normalized {
        return false;
    }

    match mode {
        Mode::Write => {
            match fs::write(path, &write_content) {
                Ok(_) => eprintln!("\r\x1b[KFormatted: {}", path.display()),
                Err(e) => eprintln!("Error writing {}: {}", path.display(), e),
            }
        }
        Mode::Check => {
            eprintln!("Would format: {}", path.display());
        }
        Mode::Diff => {
            print_diff(path, &normalized, &write_content);
        }
    }

    true
}

/// Dispatch to the correct formatter based on file extension.
/// Returns `true` if the file was (or would be) modified.
pub(crate) fn format_file(path: &Path, mode: Mode, config: &crate::ReeConfig) -> bool {
    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
    if !config.extensions.iter().any(|e| e == ext) {
        return false;
    }
    if crate::should_skip_file(path, config) {
        return false;
    }
    match ext {
        "ree" => crate::ree_format::format_ree_file(path, mode),
        "ts" | "js" | "css" => format_code_file(path, mode),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_mode_does_not_modify_ree_file() {
        let dir = env::temp_dir().join("reefmt_test_check_mode");
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test.ree");
        let unformatted = "{#if show}\n<div>\n{=title}\n</div>\n{/if}";
        fs::write(&path, unformatted).unwrap();

        let modified = crate::ree_format::format_ree_file(&path, Mode::Check);
        assert!(modified, "Check mode should return true when file would change");
        let content_after = fs::read_to_string(&path).unwrap();
        assert_eq!(content_after, unformatted, "Check mode should not modify the file");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn check_mode_reports_no_change_for_formatted_ree_file() {
        let dir = env::temp_dir().join(format!("reefmt_test_check_fmt_{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test.ree");
        let content = "<span>text</span>\n";
        fs::write(&path, content).unwrap();

        let modified = crate::ree_format::format_ree_file(&path, Mode::Check);
        assert!(!modified, "Check mode should return false for already-formatted file (modified={})", modified);

        let content_after = fs::read_to_string(&path).unwrap();
        assert_eq!(content_after, content, "Check mode should not modify the file");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn diff_mode_does_not_modify_ree_file() {
        let dir = env::temp_dir().join("reefmt_test_diff_mode");
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test.ree");
        let unformatted = "{#if show}\n<div>\n{=title}\n</div>\n{/if}";
        fs::write(&path, unformatted).unwrap();

        let modified = crate::ree_format::format_ree_file(&path, Mode::Diff);
        assert!(modified, "Diff mode should return true when file would change");
        let content_after = fs::read_to_string(&path).unwrap();
        assert_eq!(content_after, unformatted, "Diff mode should not modify the file");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn write_mode_modifies_ree_file() {
        let dir = env::temp_dir().join("reefmt_test_write_mode");
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test.ree");
        let unformatted = "{#if show}\n<div>\n{=title}\n</div>\n{/if}";
        fs::write(&path, unformatted).unwrap();

        let modified = crate::ree_format::format_ree_file(&path, Mode::Write);
        assert!(modified, "Write mode should return true when file changes");
        let content_after = fs::read_to_string(&path).unwrap();
        assert_ne!(content_after, unformatted, "Write mode should modify the file");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn diff_mode_formatted_ree_file_returns_false() {
        let dir = env::temp_dir().join(format!("reefmt_test_diff_fmt_{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test.ree");
        let content = "<span>text</span>\n";
        fs::write(&path, content).unwrap();

        let modified = crate::ree_format::format_ree_file(&path, Mode::Diff);
        assert!(!modified, "Diff mode should return false for already-formatted file");
        let content_after = fs::read_to_string(&path).unwrap();
        assert_eq!(content_after, content, "Diff mode should not modify the file");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn check_mode_code_file_does_not_modify() {
        let dir = env::temp_dir().join("reefmt_test_code_check");
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test.ts");
        let content = "const x = 1;\n";
        fs::write(&path, content).unwrap();

        let _modified = format_code_file(&path, Mode::Check);
        let content_after = fs::read_to_string(&path).unwrap();
        assert_eq!(content_after, content, "Check mode should not modify the code file");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn diff_mode_code_file_missing_returns_false() {
        let path = Path::new("/tmp/nonexistent_file_reefmt_diff_test.ts");
        let modified = format_code_file(path, Mode::Diff);
        assert!(!modified, "format_code_file Diff should return false for missing file");
    }

    #[test]
    fn check_mode_ree_file_missing_returns_false() {
        let path = Path::new("/tmp/nonexistent_file_reefmt_test.ree");
        let modified = crate::ree_format::format_ree_file(path, Mode::Check);
        assert!(!modified, "format_ree_file should return false for missing file");
    }
}
