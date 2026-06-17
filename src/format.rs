use std::env;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::OnceLock;
use similar::{ChangeTag, DiffOp};

use crate::ree_format::flatten_concat;

/// Atomic counter for generating unique temp file names across parallel threads.
static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Operating mode: write files, check-only (list files), or diff (show changes).
#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Mode { Write, Check, Diff }

/// Default oxfmt config with tabs, embedded at build time.
const OXFMT_CONFIG: &str = "{\"useTabs\": true, \"tabWidth\": 1}";

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

/// Generate a unique identifier for temp files (process ID + atomic counter).
/// Guarantees uniqueness across parallel threads.
fn temp_uid() -> String {
    let pid = std::process::id();
    let count = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("{}_{}", pid, count)
}

/// Resolve the oxfmt config path. Write the default config to a temp file
/// once and reuse it for all subsequent calls.
fn resolve_oxfmt_config() -> &'static str {
    static CONFIG_PATH: OnceLock<String> = OnceLock::new();
    CONFIG_PATH.get_or_init(|| {
        // Check for project-level oxfmt config first
        if let Ok(cwd) = env::current_dir() {
            for name in &[".oxfmtrc.json", "oxfmt.json", "oxfmt.jsonc"] {
                let path = cwd.join(name);
                if path.exists() {
                    return path.to_string_lossy().into_owned();
                }
            }
        }

        let uid = temp_uid();
        let config_path = env::temp_dir()
            .join(format!("reefmt_oxfmt_config_{}.json", uid))
            .to_string_lossy()
            .into_owned();

        if fs::write(&config_path, OXFMT_CONFIG).is_ok() {
            config_path
        } else {
            String::new()
        }
    })
}

/// Pipe content through oxfmt for formatting.
pub(crate) fn pipe_oxfmt(src: &str, ext: &str) -> String {
    let filepath = format!("file.{}", ext);
    let mut args = vec!["--stdin-filepath", &filepath];
    let config_path = resolve_oxfmt_config();
    if !config_path.is_empty() {
        args.push("-c");
        args.push(config_path);
    }

    let mut child = match Command::new("oxfmt")
        .args(&args)
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

/// Format standalone code content (TS/JS/CSS) via oxfmt.
pub(crate) fn format_code_content(content: &str, ext: &str) -> String {
    let normalized = content.replace("\r\n", "\n");

    let flattened = flatten_concat(&normalized);
    let formatted = pipe_oxfmt(&flattened, ext);

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
        "ree" => crate::ree_format::format_ree_file(path, mode, config.wrap_width),
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

        let modified = crate::ree_format::format_ree_file(&path, Mode::Check, 120);
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

        let modified = crate::ree_format::format_ree_file(&path, Mode::Check, 120);
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

        let modified = crate::ree_format::format_ree_file(&path, Mode::Diff, 120);
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

        let modified = crate::ree_format::format_ree_file(&path, Mode::Write, 120);
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

        let modified = crate::ree_format::format_ree_file(&path, Mode::Diff, 120);
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
        let modified = crate::ree_format::format_ree_file(path, Mode::Check, 120);
        assert!(!modified, "format_ree_file should return false for missing file");
    }
}
