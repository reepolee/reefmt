use std::fs;
use std::path::Path;
use std::collections::HashMap;
use similar::{ChangeTag, DiffOp};

use crate::ree_format::flatten_concat;

/// Operating mode: write files, check-only (list files), or diff (show changes).
#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Mode { Write, Check, Diff }

/// A placeholder entry for content extracted before SWC formatting
/// and restored afterward.
struct Placeholder {
    tag: String,
    original: String,
}

/// Pre-process source code before SWC formatting to preserve content that SWC
/// would otherwise reformat in undesirable ways.
///
/// Extracts two types of content:
/// - **Block comments** (`/* ... */` and `/** ... */`) that appear on their own line
///   (only whitespace before the `/*` and after the `*/`). These get `*/` merged
///   with the next line by SWC's codegen.
/// - **Blank lines** (empty or whitespace-only lines). These are removed by SWC's
///   codegen since it doesn't preserve blank lines between statements.
///
/// Each extracted piece is replaced with a `// __REEFMT_{type}_{id}__` placeholder
/// comment that SWC preserves. After SWC formatting, the placeholders are restored
/// to their original text.
fn preprocess_for_swc(code: &str) -> (String, Vec<Placeholder>) {
    let mut placeholders = Vec::new();
    let mut id_counter = 0usize;

    // ---- Pass 1: Extract block comments via character scan ----
    let pass1 = extract_block_comments(code, &mut placeholders, &mut id_counter);

    // ---- Pass 2: Extract blank lines from the result of pass 1 ----
    let pass2 = extract_blank_lines(&pass1, &mut placeholders, &mut id_counter);

    (pass2, placeholders)
}

/// Copy a single UTF-8 character from position `i` in `code` to `out`.
/// Advances `i` past the consumed bytes.
fn copy_utf8_char(code: &str, bytes: &[u8], out: &mut String, i: &mut usize) {
    if *i < bytes.len() && bytes[*i] & 0x80 == 0 {
        out.push(bytes[*i] as char);
        *i += 1;
    } else {
        let ch = code[*i..].chars().next().unwrap();
        out.push(ch);
        *i += ch.len_utf8();
    }
}

/// Scan character-by-character to find block comments on their own line
/// and replace them with `// __REEFMT_BLOCK_N__` placeholders.
/// Properly skips string literals, template literals, and single-line comments.
fn extract_block_comments(code: &str, placeholders: &mut Vec<Placeholder>, id: &mut usize) -> String {
    let mut out = String::with_capacity(code.len());
    let bytes = code.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    while i < len {

        // Skip double-quoted strings
        if bytes[i] == b'"' {
            out.push('"');
            i += 1;
            while i < len {
                let b = bytes[i];
                if b == b'\\' && i + 1 < len {
                    out.push('\\');
                    i += 1;
                    copy_utf8_char(code, bytes, &mut out, &mut i);
                } else if b == b'"' {
                    out.push('"');
                    i += 1;
                    break;
                } else {
                    copy_utf8_char(code, bytes, &mut out, &mut i);
                }
            }
            continue;
        }

        // Skip single-quoted strings
        if bytes[i] == b'\'' {
            out.push('\'');
            i += 1;
            while i < len {
                let b = bytes[i];
                if b == b'\\' && i + 1 < len {
                    out.push('\\');
                    i += 1;
                    copy_utf8_char(code, bytes, &mut out, &mut i);
                } else if b == b'\'' {
                    out.push('\'');
                    i += 1;
                    break;
                } else {
                    copy_utf8_char(code, bytes, &mut out, &mut i);
                }
            }
            continue;
        }

        // Skip template literals (handling nested `${}`)
        if bytes[i] == b'`' {
            out.push('`');
            i += 1;
            let mut depth = 0u32;
            while i < len {
                let b = bytes[i];
                if b == b'\\' && i + 1 < len {
                    out.push('\\');
                    i += 1;
                    out.push(bytes[i] as char); // escaped char is always ASCII
                    i += 1;
                } else if b == b'$' && i + 1 < len && bytes[i + 1] == b'{' {
                    out.push_str("${");
                    i += 2;
                    depth += 1;
                } else if b == b'}' && depth > 0 {
                    out.push('}');
                    i += 1;
                    depth -= 1;
                } else if b == b'`' && depth == 0 {
                    out.push('`');
                    i += 1;
                    break;
                } else {
                    copy_utf8_char(code, bytes, &mut out, &mut i);
                }
            }
            continue;
        }

        // Skip single-line `//` comments
        if bytes[i] == b'/' && i + 1 < len && bytes[i + 1] == b'/' {
            while i < len && bytes[i] != b'\n' {
                copy_utf8_char(code, bytes, &mut out, &mut i);
            }
            if i < len {
                out.push('\n');
                i += 1;
            }
            continue;
        }

        // Handle block comments `/* ... */`
        if bytes[i] == b'/' && i + 1 < len && bytes[i + 1] == b'*' {
            let start = i;
            i += 2;
            while i + 1 < len {
                if bytes[i] == b'*' && bytes[i + 1] == b'/' {
                    i += 2;
                    break;
                }
                i += 1;
            }
            let comment_text = &code[start..i];

            if is_block_comment_own_line(code, start, i) {
                let tag = format!("__REEFMT_BLOCK_{}__", *id);
                *id += 1;
                let line_count = comment_text.lines().count();
                for _ in 0..line_count {
                    out.push_str(&format!("// {}\n", tag));
                }
                placeholders.push(Placeholder {
                    tag,
                    original: comment_text.to_string(),
                });
                // Placeholder lines already end with \n, so skip
                // the trailing newline after */ to avoid double newlines.
                if i < len && bytes[i] == b'\n' {
                    i += 1;
                }
            } else {
                out.push_str(comment_text);
            }
            continue;
        }

        // Regular character (or start of multi-byte UTF-8)
        copy_utf8_char(code, bytes, &mut out, &mut i);
    }

    out
}

/// Check whether a block comment (between `start` and `end` byte offsets)
/// is on its own line: only whitespace before `/*` and after `*/` on their
/// respective lines.
fn is_block_comment_own_line(code: &str, start: usize, end: usize) -> bool {
    // Check before `/*` on the same line
    let line_start = code[..start].rfind('\n').map(|p| p + 1).unwrap_or(0);
    let before = &code[line_start..start];
    if !before.trim().is_empty() {
        return false;
    }

    // Check after `*/` on the same line
    let line_end = code[end..].find('\n').map(|p| end + p).unwrap_or(code.len());
    let after = &code[end..line_end];
    if !after.trim().is_empty() {
        return false;
    }

    true
}

/// Extract blank lines (empty or whitespace-only lines) and replace each
/// with a `// __REEFMT_BLANK_N__` placeholder comment.
///
/// Operates on code that has already had block comments extracted, so
/// block-comment placeholder lines (`// __REEFMT_BLOCK_*`) are treated
/// as non-blank lines to avoid interfering with block comment spacing.
fn extract_blank_lines(code: &str, placeholders: &mut Vec<Placeholder>, id: &mut usize) -> String {
    let mut out = String::with_capacity(code.len());

    for line in code.lines() {
        let trimmed = line.trim();

        // A blank line is one that is empty or whitespace-only
        // AND is not a block-comment placeholder line
        if trimmed.is_empty() {
            let tag = format!("__REEFMT_BLANK_{}__", *id);
            *id += 1;
            out.push_str(&format!("// {}\n", tag));
            placeholders.push(Placeholder {
                tag,
                original: String::new(), // blank lines restore to empty
            });
        } else {
            out.push_str(line);
            out.push('\n');
        }
    }

    out
}

/// Restore original content from placeholders in the SWC-formatted output.
///
/// Works in two passes:
/// 1. First restore blank lines (they become empty lines, which won't interfere
///    with block comment pattern matching).
/// 2. Then restore block comments (multi-line replacement).
fn postprocess_from_swc(formatted: &str, placeholders: &[Placeholder]) -> String {
    // Build a map for fast lookup
    let mut by_tag: HashMap<&str, &Placeholder> = HashMap::new();
    for p in placeholders {
        by_tag.insert(&p.tag, p);
    }

    // ---- Pass 1: Restore blank lines ----
    // Blank lines are single `// __REEFMT_BLANK_N__` lines → replace with `\n`
    let mut result = String::with_capacity(formatted.len());

    for line in formatted.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("// __REEFMT_BLANK_") {
            // Extract the tag
            if let Some(tag) = extract_tag(trimmed) {
                if let Some(ph) = by_tag.get(tag.as_str()) {
                    if ph.original.is_empty() {
                        // Blank line — emit an empty line
                        // Preserve the indentation context: use the same leading
                        // whitespace as the SWC-formatted placeholder line had
                        let indent = &line[..line.len() - trimmed.len()];
                        result.push_str(indent);
                        result.push('\n');
                        continue;
                    }
                }
            }
        }
        result.push_str(line);
        result.push('\n');
    }

    // ---- Pass 2: Restore block comments ----
    // Block comments span multiple placeholder lines. We need to find contiguous
    // groups of `// __REEFMT_BLOCK_N__` lines and replace each group.
    let mut final_result = String::with_capacity(result.len());
    let lines: Vec<&str> = result.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let trimmed = lines[i].trim();
        if trimmed.starts_with("// __REEFMT_BLOCK_") {
            if let Some(tag) = extract_tag(trimmed) {
                if let Some(ph) = by_tag.get(tag.as_str()) {
                    let line_count = ph.original.lines().count();

                    // Verify we have enough consecutive lines with the same tag
                    let all_match = (0..line_count).all(|offset| {
                        i + offset < lines.len()
                            && lines[i + offset].trim() == format!("// {}", tag)
                    });

                    if all_match && !ph.original.is_empty() {
                        // Capture the indentation from the first placeholder line
                        let indent = &lines[i][..lines[i].len() - lines[i].trim().len()];

                        // Re-indent the original block comment to match
                        let reindented = reindent_block_comment(&ph.original, indent);
                        final_result.push_str(&reindented);
                        final_result.push('\n');
                        i += line_count;
                        continue;
                    }
                }
            }
        }
        final_result.push_str(lines[i]);
        final_result.push('\n');
        i += 1;
    }

    // Remove trailing newline if formatted didn't have one
    if !formatted.ends_with('\n') && final_result.ends_with('\n') {
        final_result.pop();
    }

    final_result
}

/// Extract the tag (`__REEFMT_BLOCK_N__` or `__REEFMT_BLANK_N__`) from a
/// `// __REEFMT_...` comment line.
fn extract_tag(trimmed: &str) -> Option<String> {
    // trimmed looks like "// __REEFMT_BLOCK_0__" or "// __REEFMT_BLANK_0__"
    let trimmed = trimmed.trim();
    let after = trimmed.strip_prefix("// ")?;
    if after.starts_with("__REEFMT_") {
        Some(after.to_string())
    } else {
        None
    }
}

/// Re-indent a block comment to use the given indent string, preserving its
/// internal structure.
///
/// The original block comment has some indentation (e.g. `\t/**`). We need to
/// strip its original base indentation and apply the new one.
fn reindent_block_comment(comment: &str, new_indent: &str) -> String {
    let lines: Vec<&str> = comment.lines().collect();
    if lines.is_empty() {
        return comment.to_string();
    }

    // Detect the base indentation of the first line
    let first_trimmed = lines[0].trim_start();
    let base_indent = lines[0].len() - first_trimmed.len();

    let mut out = String::with_capacity(comment.len());
    for (idx, line) in lines.iter().enumerate() {
        let trimmed = line.trim_start();
        if trimmed.is_empty() {
            out.push('\n');
            continue;
        }
        // Only indent lines that had the base indentation
        let leading = line.len() - trimmed.len();
        if leading >= base_indent {
            out.push_str(new_indent);
            // Preserve remaining indentation beyond the base level
            let extra = leading - base_indent;
            for _ in 0..extra {
                out.push(' ');
            }
        }
        out.push_str(trimmed);
        if idx < lines.len() - 1 {
            out.push('\n');
        }
    }

    out
}

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

/// Collapse expanded inline type literals back to a single line when they
/// fit within `max_width`.
///
/// SWC's codegen expands TS inline type literals like `{ a: string; b: number; }`
/// across multiple lines. This function detects the multi-line form and collapses
/// it back when the result fits within the max width.
///
/// Detection heuristic: a line ending with `{` where the body consists of lines
/// ending with `;` (TS type members), followed by a properly brace-matched `}`.
fn collapse_inline_type_literals(code: &str, max_width: usize) -> String {
    let mut result = String::with_capacity(code.len());
    let lines: Vec<&str> = code.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];
        let trimmed = line.trim();
        let is_comment = trimmed.starts_with("//") || trimmed.starts_with("/*");

        // Look for a line ending with `{` preceded by `:` (e.g. `	keys: {`)
        if trimmed.ends_with('{') && trimmed.contains(':') && !is_comment {
            // Track brace depth to handle nested type literals correctly
            let mut depth: u32 = 1;
            let mut closing_line = None;
            for j in i + 1..lines.len() {
                for (byte_pos, ch) in lines[j].char_indices() {
                    match ch {
                        '{' => depth += 1,
                        '}' => {
                            depth -= 1;
                            if depth == 0 {
                                // byte_pos is position of `}`, +1 for byte after it
                                closing_line = Some((j, byte_pos + 1));
                                break;
                            }
                        }
                        _ => {}
                    }
                }
                if closing_line.is_some() {
                    break;
                }
            }

            if let Some((end_idx, after_close_start)) = closing_line {
                let body_lines: Vec<&str> = (i + 1..end_idx)
                    .map(|j| lines[j].trim())
                    .collect();

                // Type literal check: all non-empty body lines end with `;`
                // and contain `:` before `;` (type member pattern like `key: type;`)
                let is_type_literal = !body_lines.is_empty()
                    && body_lines.iter().all(|l| {
                        l.is_empty() || (l.ends_with(';') && l.contains(':'))
                    });

                if is_type_literal {
                    // Build members list (strip trailing `;`)
                    let members: Vec<&str> = body_lines.iter()
                        .filter(|l| !l.is_empty())
                        .map(|l| l.trim_end_matches(';').trim())
                        .collect();

                    if members.len() > 6 {
                        // Too many members — keep multi-line
                        result.push_str(line);
                        result.push('\n');
                        i += 1;
                        continue;
                    }

                    // Get the prefix before `{` on the opening line
                    let line_prefix = &line[..line.len() - trimmed.len()];
                    let before_brace = trimmed.strip_suffix('{').unwrap_or(trimmed);

                    // Get everything after the matching `}` on the closing line
                    let rest_of_closing_line = &lines[end_idx][after_close_start..];

                    // Build inline form: `{ member; member; member; }`
                    let inner = if members.is_empty() {
                        String::new()
                    } else {
                        format!(" {}; ", members.join("; "))
                    };

                    let collapsed = format!(
                        "{}{}{{{}}}{}",
                        line_prefix,
                        before_brace,
                        inner,
                        rest_of_closing_line
                    );

                    if collapsed.len() <= max_width {
                        result.push_str(&collapsed);
                        result.push('\n');
                        i = end_idx + 1;
                        continue;
                    }
                }
            }
        }

        result.push_str(line);
        result.push('\n');
        i += 1;
    }

    // Remove trailing newline if original didn't have one
    if !code.ends_with('\n') && result.ends_with('\n') {
        result.pop();
    }

    result
}

/// Format standalone code content (TS/JS/CSS) using native SWC, no subprocess needed.
/// For .ts and .js files, uses the SWC parser/codegen pipeline.
/// For .css files, returns the content unchanged (CSS support is a future improvement).
/// `wrap_width` controls the max line width for collapsing inline type literals.
pub(crate) fn format_code_content(content: &str, ext: &str, wrap_width: usize) -> String {
    let normalized = content.replace("\r\n", "\n");

    let formatted = match ext {
        "ts" | "js" => {
            let flattened = flatten_concat(&normalized);
            let (preprocessed, placeholders) = preprocess_for_swc(&flattened);
            let swc_formatted = crate::swc_format::format_js_with_indent(&preprocessed, "\t");
            let restored = if placeholders.is_empty() {
                swc_formatted
            } else {
                postprocess_from_swc(&swc_formatted, &placeholders)
            };
            collapse_inline_type_literals(&restored, wrap_width)
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
/// `wrap_width` controls the max line width for collapsing inline type literals.
pub(crate) fn format_code_file(path: &Path, mode: Mode, wrap_width: usize) -> bool {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error reading {}: {}", path.display(), e);
            return false;
        }
    };

    let normalized = content.replace("\r\n", "\n");
    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
    let write_content = format_code_content(&normalized, ext, wrap_width);

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
        "ts" | "js" | "css" => format_code_file(path, mode, config.wrap_width),
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

        let _modified = format_code_file(&path, Mode::Check, 180);
        let content_after = fs::read_to_string(&path).unwrap();
        assert_eq!(content_after, content, "Check mode should not modify the code file");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn diff_mode_code_file_missing_returns_false() {
        let path = Path::new("/tmp/nonexistent_file_reefmt_diff_test.ts");
        let modified = format_code_file(path, Mode::Diff, 180);
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
        let result = format_code_content(src, "js", 180);
        assert!(result.contains("const x = 1;"), "SWC should format JS: got {:?}", result);
    }

    #[test]
    fn idempotent_format_code_content_js() {
        let src = "const x = 1;\n";
        let pass1 = format_code_content(src, "js", 180);
        let pass2 = format_code_content(&pass1, "js", 180);
        assert_eq!(pass1, pass2, "format_code_content should be idempotent for JS");
    }

    #[test]
    fn idempotent_format_code_content_non_ascii_comment() {
        let src = "// Café naïve — ščüéø\nconst x = 1;\n";
        let pass1 = format_code_content(src, "js", 180);
        let pass2 = format_code_content(&pass1, "js", 180);
        assert_eq!(pass1, pass2,
            "format_code_content should be idempotent with non-ASCII chars");
    }

    #[test]
    fn format_code_content_css_passthrough() {
        let src = "body { color: red; }\n";
        let result = format_code_content(src, "css", 180);
        assert_eq!(result, src, "CSS should pass through unchanged");
    }

    #[test]
    fn preserves_block_comments() {
        let src = "/**\n * doc\n */\nexport const x = 1;\n";
        let result = format_code_content(src, "ts", 180);
        assert!(result.contains("/**"), "block comment /** should be preserved");
        assert!(result.contains(" * doc\n"), "block comment content should be preserved");
        assert!(result.contains(" */\nexport"), "*/ should be on its own line before export");
    }

    #[test]
    fn preserves_blank_lines_between_statements() {
        let src = "export interface A {\n\tx: number;\n}\n\nexport interface B {\n\ty: number;\n}\n";
        let result = format_code_content(src, "ts", 180);
        assert!(result.contains("}\n\nexport"), "blank line between interfaces should be preserved");
    }

    #[test]
    fn inline_block_comment_not_extracted() {
        let src = "const x = 1; /* inline */\n";
        let result = format_code_content(src, "ts", 180);
        assert!(result.contains("/* inline */"), "inline block comments should stay inline");
    }

    #[test]
    fn block_comment_in_string_not_extracted() {
        let src = "const s = \"/* not a comment */\";\n";
        let result = format_code_content(src, "ts", 180);
        assert!(result.contains("/* not a comment */"), "comments inside strings should be preserved");
    }

    #[test]
    fn single_line_block_comment_own_line() {
        let src = "/* standalone */\nexport const x = 1;\n";
        let result = format_code_content(src, "ts", 180);
        assert!(result.contains("/* standalone */\nexport"), "standalone block comment should be on its own line");
    }

    #[test]
    fn idempotent_with_block_comments_and_blank_lines() {
        let src = "/**\n * Translation helpers\n */\n\n// ─── Types ─────────────────────────────────────────────────────\n\nexport interface TranslationRow {\n\tid: number;\n\tlang: string;\n}\n\nexport interface GroupInfo {\n\tnamespace: string;\n\tchild_keys: string[];\n}\n";
        let pass1 = format_code_content(src, "ts", 180);
        let pass2 = format_code_content(&pass1, "ts", 180);
        assert_eq!(pass1, pass2, "output should be idempotent with block comments and blank lines");
    }
}
