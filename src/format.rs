use std::fs;
use std::path::Path;
use similar::{ChangeTag, DiffOp};

use crate::ree_format::flatten_concat;

/// Operating mode: write files, check-only (list files), or diff (show changes).
#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Mode { Write, Check, Diff }

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

/// Format standalone code content (TS/JS/CSS) using native SWC, no subprocess needed.
/// For .ts and .js files, uses the SWC parser/codegen pipeline.
/// For .css files, returns the content unchanged (CSS support is a future improvement).
pub(crate) fn format_code_content(content: &str, ext: &str) -> String {
    let normalized = content.replace("\r\n", "\n");

    let formatted = match ext {
        "ts" | "js" => {
            let flattened = flatten_concat(&normalized);
            crate::swc_format::format_js_with_indent(&flattened, "\t")
        }
        _ => normalized.clone(),
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
        "ree" => crate::ree_format::format_ree_file(path, mode, config.wrap_width),
        "ts" | "js" | "css" => format_code_file(path, mode),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use std::env;
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

    #[test]
    fn format_code_content_js_uses_swc() {
        let src = "const x=1;const y=2;";
        let result = format_code_content(src, "js");
        assert!(result.contains("const x = 1;"), "SWC should format JS: got {:?}", result);
    }

    #[test]
    fn idempotent_format_code_content_js() {
        let src = "const x = 1;\n";
        let pass1 = format_code_content(src, "js");
        let pass2 = format_code_content(&pass1, "js");
        assert_eq!(pass1, pass2, "format_code_content should be idempotent for JS");
    }

    #[test]
    fn idempotent_format_code_content_non_ascii_comment() {
        let src = "// Café naïve — ščüéø\nconst x = 1;\n";
        let pass1 = format_code_content(src, "js");
        let pass2 = format_code_content(&pass1, "js");
        assert_eq!(pass1, pass2,
            "format_code_content should be idempotent with non-ASCII chars");
    }

    #[test]
    fn format_code_content_css_passthrough() {
        let src = "body { color: red; }\n";
        let result = format_code_content(src, "css");
        assert_eq!(result, src, "CSS should pass through unchanged");
    }
}
