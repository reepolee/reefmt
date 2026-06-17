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
fn collapse_inline_type_literals(code: &str, max_width: usize, max_members: usize) -> String {
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

                    if members.len() > max_members {
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

/// Check if a trimmed line is a block opener like `if (cond) {` or `for (;;) {`
fn is_block_opener(trimmed: &str) -> bool {
    // Must end with `{` and the character before must be `)`
    // e.g. `if (cond) {`, `for (;;) {`, `while (x) {`, `} else if (cond) {`
    if !trimmed.ends_with('{') {
        return false;
    }
    let before_brace = trimmed[..trimmed.len() - 1].trim_end();
    before_brace.ends_with(')')
}

/// Run one pass of the collapse logic. Returns `true` if any changes were made
/// (meaning another pass may find more opportunities).
fn collapse_single_stmt_blocks_pass(code: &str, max_width: usize, max_members: usize, out: &mut String) -> bool {
    out.clear();
    let lines: Vec<&str> = code.lines().collect();
    let mut i = 0;
    let mut modified = false;

    while i < lines.len() {
        let line = lines[i];
        let trimmed = line.trim();

        // ---- Single-statement block: `if (cond) {` stmt `}` ----
        if i + 2 < lines.len() && is_block_opener(trimmed) {
            let stmt_trimmed = lines[i + 1].trim();
            let close_trimmed = lines[i + 2].trim();

            if !stmt_trimmed.is_empty() && close_trimmed == "}" {
                let prefix = &line[..line.len() - trimmed.len()];
                let before_brace = trimmed.trim_end();
                let collapsed = format!("{} {} }}", before_brace, stmt_trimmed);
                let full_line = format!("{}{}", prefix, collapsed);

                if full_line.len() <= max_width {
                    out.push_str(&full_line);
                    out.push('\n');
                    i += 3;
                    modified = true;
                    continue;
                }
            }
        }

        // ---- Object literal function param: `foo({` key:val, `})` ----
        if i + 2 < lines.len() && is_obj_lit_opener(trimmed) {
            let mut depth = 1u32;
            let mut closing = None;
            for j in i + 1..lines.len() {
                for (byte_pos, ch) in lines[j].char_indices() {
                    match ch {
                        '{' => depth += 1,
                        '}' => { depth -= 1; if depth == 0 { closing = Some((j, byte_pos + 1)); break; } }
                        _ => {}
                    }
                }
                if closing.is_some() { break; }
            }

            if let Some((end_idx, after_close)) = closing {
                let body: Vec<&str> = (i + 1..end_idx)
                    .map(|j| lines[j].trim())
                    .filter(|l| !l.is_empty())
                    .collect();

                if !body.is_empty() && body.len() <= max_members && body.iter().all(|l| l.contains(':')) {
                    let prefix = &line[..line.len() - trimmed.len()];
                    let before_paren = trimmed.strip_suffix('{').unwrap_or(trimmed);
                    let after_close_str = &lines[end_idx][after_close..];
                    let members: Vec<&str> = body.iter()
                        .map(|l| l.trim_end_matches(',').trim()).collect();

                    let collapsed = format!(
                        "{}{}{{ {} }}{}",
                        prefix, before_paren, members.join(", "), after_close_str
                    );

                    if collapsed.len() <= max_width {
                        out.push_str(&collapsed);
                        out.push('\n');
                        i = end_idx + 1;
                        modified = true;
                        continue;
                    }
                }
            }
        }

        out.push_str(line);
        out.push('\n');
        i += 1;
    }

    if !code.ends_with('\n') && out.ends_with('\n') {
        out.pop();
    }
    modified
}

/// Collapse single-statement blocks and object literal function params onto
/// one line when they fit within `max_width`. Runs iteratively until no
/// further collapses are possible (handles nested structures like `if (x) {
///   fn({ key: val });
/// }` where the inner object-literal collapses first, then the if-block).
pub(crate) fn collapse_single_stmt_blocks(code: &str, max_width: usize, max_members: usize) -> String {
    let mut result = code.to_string();
    let mut buf = String::with_capacity(code.len());
    loop {
        if !collapse_single_stmt_blocks_pass(&result, max_width, max_members, &mut buf) {
            return result;
        }
        std::mem::swap(&mut result, &mut buf);
    }
}

/// Check if a trimmed line represents a function call with an object literal
/// as an argument, e.g. `foo({` or `foo(arg, {`.
/// Must end with `{` (after trimming), must not be a block opener like
/// `if (cond) {`, and must have a `(` somewhere before the `{`.
fn is_obj_lit_opener(trimmed: &str) -> bool {
    if !trimmed.ends_with('{') {
        return false;
    }
    if is_block_opener(trimmed) {
        return false;
    }
    // Must have a `(` before the `{`, indicating a function call.
    trimmed[..trimmed.len() - 1].contains('(')
}

/// Ensures proper spacing around arrow function `=>` tokens:
/// `()=>{` → `() => {`, `param=>` → `param => `, etc.
/// Avoids modifying `<=` and `>=` comparison operators.
pub(crate) fn fix_arrow_spacing(code: &str) -> String {
    let mut out = String::with_capacity(code.len());
    let bytes = code.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    while i < len {
        // Peek for `=>` — both bytes are ASCII
        if i + 1 < len && bytes[i] == b'=' && bytes[i + 1] == b'>' {
            // Skip if this is `<=` or `>=`
            if i > 0 && (bytes[i - 1] == b'<' || bytes[i - 1] == b'>') {
                out.push(bytes[i] as char);
                i += 1;
                continue;
            }

            // Space before `=>` if not already there
            if i > 0 && bytes[i - 1] != b' ' {
                out.push(' ');
            }

            out.push_str("=>");
            i += 2;

            // Space after `=>` (but not before a newline)
            if i < len && bytes[i] != b' ' && bytes[i] != b'\n' && bytes[i] != b'\r' {
                out.push(' ');
            }

            continue;
        }

        // Properly copy UTF-8 characters (ASCII fast path, multi-byte fallback)
        if bytes[i] & 0x80 == 0 {
            out.push(bytes[i] as char);
            i += 1;
        } else {
            let ch = code[i..].chars().next().unwrap();
            out.push(ch);
            i += ch.len_utf8();
        }
    }

    out
}

/// Format standalone code content (TS/JS/CSS) using native SWC, no subprocess needed.
pub(crate) fn format_code_content(
    content: &str,
    ext: &str,
    wrap_width: usize,
    collapse_blocks: bool,
    max_members: usize,
) -> String {
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
            let spaced = fix_arrow_spacing(&restored);
            let collapsed = collapse_inline_type_literals(&spaced, wrap_width, max_members);
            if collapse_blocks {
                collapse_single_stmt_blocks(&collapsed, wrap_width, max_members)
            } else {
                collapsed
            }
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
pub(crate) fn format_code_file(path: &Path, mode: Mode, wrap_width: usize, collapse_blocks: bool, max_members: usize) -> bool {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error reading {}: {}", path.display(), e);
            return false;
        }
    };

    let normalized = content.replace("\r\n", "\n");
    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
    let write_content = format_code_content(&normalized, ext, wrap_width, collapse_blocks, max_members);

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
        "ree" => crate::ree_format::format_ree_file(path, mode, config.wrap_width, config.collapse_single_stmt_blocks, config.collapse_max_members),
        "ts" | "js" | "css" => format_code_file(path, mode, config.wrap_width, config.collapse_single_stmt_blocks, config.collapse_max_members),
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

        let modified = crate::ree_format::format_ree_file(&path, Mode::Check, 120, true, 3);
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

        let modified = crate::ree_format::format_ree_file(&path, Mode::Check, 120, true, 3);
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

        let modified = crate::ree_format::format_ree_file(&path, Mode::Diff, 120, true, 3);
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

        let modified = crate::ree_format::format_ree_file(&path, Mode::Write, 120, true, 3);
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

        let modified = crate::ree_format::format_ree_file(&path, Mode::Diff, 120, true, 3);
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

        let _modified = format_code_file(&path, Mode::Check, 180, true, 3);
        let content_after = fs::read_to_string(&path).unwrap();
        assert_eq!(content_after, content, "Check mode should not modify the code file");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn diff_mode_code_file_missing_returns_false() {
        let path = Path::new("/tmp/nonexistent_file_reefmt_diff_test.ts");
        let modified = format_code_file(path, Mode::Diff, 180, true, 3);
        assert!(!modified, "format_code_file Diff should return false for missing file");
    }

    #[test]
    fn check_mode_ree_file_missing_returns_false() {
        let path = Path::new("/tmp/nonexistent_file_reefmt_test.ree");
        let modified = crate::ree_format::format_ree_file(path, Mode::Check, 120, true, 3);
        assert!(!modified, "format_ree_file should return false for missing file");
    }

    #[test]
    fn format_code_content_js_uses_swc() {
        let src = "const x=1;const y=2;";
        let result = format_code_content(src, "js", 180, true, 3);
        assert!(result.contains("const x = 1;"), "SWC should format JS: got {:?}", result);
    }

    #[test]
    fn idempotent_format_code_content_js() {
        let src = "const x = 1;\n";
        let pass1 = format_code_content(src, "js", 180, true, 3);
        let pass2 = format_code_content(&pass1, "js", 180, true, 3);
        assert_eq!(pass1, pass2, "format_code_content should be idempotent for JS");
    }

    #[test]
    fn idempotent_format_code_content_non_ascii_comment() {
        let src = "// Café naïve — ščüéø\nconst x = 1;\n";
        let pass1 = format_code_content(src, "js", 180, true, 3);
        let pass2 = format_code_content(&pass1, "js", 180, true, 3);
        assert_eq!(pass1, pass2,
            "format_code_content should be idempotent with non-ASCII chars");
    }

    #[test]
    fn format_code_content_css_passthrough() {
        let src = "body { color: red; }\n";
        let result = format_code_content(src, "css", 180, true, 3);
        assert_eq!(result, src, "CSS should pass through unchanged");
    }

    #[test]
    fn preserves_block_comments() {
        let src = "/**\n * doc\n */\nexport const x = 1;\n";
        let result = format_code_content(src, "ts", 180, true, 3);
        assert!(result.contains("/**"), "block comment /** should be preserved");
        assert!(result.contains(" * doc\n"), "block comment content should be preserved");
        assert!(result.contains(" */\nexport"), "*/ should be on its own line before export");
    }

    #[test]
    fn preserves_blank_lines_between_statements() {
        let src = "export interface A {\n\tx: number;\n}\n\nexport interface B {\n\ty: number;\n}\n";
        let result = format_code_content(src, "ts", 180, true, 3);
        assert!(result.contains("}\n\nexport"), "blank line between interfaces should be preserved");
    }

    #[test]
    fn inline_block_comment_not_extracted() {
        let src = "const x = 1; /* inline */\n";
        let result = format_code_content(src, "ts", 180, true, 3);
        assert!(result.contains("/* inline */"), "inline block comments should stay inline");
    }

    #[test]
    fn block_comment_in_string_not_extracted() {
        let src = "const s = \"/* not a comment */\";\n";
        let result = format_code_content(src, "ts", 180, true, 3);
        assert!(result.contains("/* not a comment */"), "comments inside strings should be preserved");
    }

    #[test]
    fn single_line_block_comment_own_line() {
        let src = "/* standalone */\nexport const x = 1;\n";
        let result = format_code_content(src, "ts", 180, true, 3);
        assert!(result.contains("/* standalone */\nexport"), "standalone block comment should be on its own line");
    }

    #[test]
    fn idempotent_with_block_comments_and_blank_lines() {
        let src = "/**\n * Translation helpers\n */\n\n// ─── Types ─────────────────────────────────────────────────────\n\nexport interface TranslationRow {\n\tid: number;\n\tlang: string;\n}\n\nexport interface GroupInfo {\n\tnamespace: string;\n\tchild_keys: string[];\n}\n";
        let pass1 = format_code_content(src, "ts", 180, true, 3);
        let pass2 = format_code_content(&pass1, "ts", 180, true, 3);
        assert_eq!(pass1, pass2, "output should be idempotent with block comments and blank lines");
    }

    // ─── collapse_single_stmt_blocks tests ────────────────────────

    #[test]
    fn collapse_simple_if_block() {
        let src = "\tif (cond) {\n\t\tdoSomething();\n\t}\n";
        let result = collapse_single_stmt_blocks(src, 180, 3);
        assert_eq!(result, "\tif (cond) { doSomething(); }\n");
    }

    #[test]
    fn collapse_nested_if_with_obj_lit() {
        // The inner obj lit must collapse first, then the outer if-block
        let src = "\tif (items[activeIndex]) {\n\t\titems[activeIndex].scrollIntoView({\n\t\t\tblock: \"nearest\"\n\t\t});\n\t}\n";
        let result = collapse_single_stmt_blocks(src, 180, 3);
        assert_eq!(
            result,
            "\tif (items[activeIndex]) { items[activeIndex].scrollIntoView({ block: \"nearest\" }); }\n"
        );
    }

    #[test]
    fn collapse_else_if_chain() {
        let src = "\t} else if (e.key === \"Escape\") {\n\t\tlist.classList.add(\"hidden\");\n\t}\n";
        let result = collapse_single_stmt_blocks(src, 180, 3);
        assert_eq!(result, "\t} else if (e.key === \"Escape\") { list.classList.add(\"hidden\"); }\n");
    }

    #[test]
    fn collapse_obj_lit_param() {
        let src = "\t\titems[activeIndex].scrollIntoView({\n\t\t\tblock: \"nearest\"\n\t\t});\n";
        let result = collapse_single_stmt_blocks(src, 180, 3);
        assert_eq!(result, "\t\titems[activeIndex].scrollIntoView({ block: \"nearest\" });\n");
    }

    #[test]
    fn collapse_triple_nested_blocks() {
        let src = "if (a) {\n\tif (b) {\n\t\tif (c) {\n\t\t\tstmt;\n\t\t}\n\t}\n}\n";
        let result = collapse_single_stmt_blocks(src, 180, 3);
        assert_eq!(result, "if (a) { if (b) { if (c) { stmt; } } }\n");
    }

    #[test]
    fn collapse_too_wide_stays_multi_line() {
        // The collapsed line would exceed 40 chars, so it stays multi-line
        let src = "if (reallyLongConditionName) {\n\treallyLongFunctionCall(withArgs);\n}\n";
        let result = collapse_single_stmt_blocks(src, 40, 3);
        assert_eq!(result, src);
    }

    #[test]
    fn collapse_already_collapsed_is_idempotent() {
        let src = "if (cond) { doSomething(); }\n";
        let result = collapse_single_stmt_blocks(src, 180, 3);
        assert_eq!(result, src);
    }

    // ─── fix_arrow_spacing tests ────────────────────────────────

    #[test]
    fn arrow_spacing_parens() {
        // `()=>{` → `() => {`
        assert_eq!(fix_arrow_spacing("()=>{"), "() => {");
    }

    #[test]
    fn arrow_spacing_with_param() {
        // `(x)=>` → `(x) => `
        assert_eq!(fix_arrow_spacing("(x)=>x"), "(x) => x");
    }

    #[test]
    fn arrow_spacing_single_param_no_parens() {
        // `x=>` → `x => ` — only arrow spacing is affected, not operators
        assert_eq!(fix_arrow_spacing("x=>x+1"), "x => x+1");
    }

    #[test]
    fn arrow_spacing_async() {
        assert_eq!(fix_arrow_spacing("async ()=>{"), "async () => {");
    }

    #[test]
    fn arrow_spacing_comparison_untouched() {
        // `<=` and `>=` should NOT be modified
        assert_eq!(fix_arrow_spacing("a <= b"), "a <= b");
        assert_eq!(fix_arrow_spacing("a >= b"), "a >= b");
    }

    #[test]
    fn arrow_spacing_no_change_when_already_spaced() {
        assert_eq!(fix_arrow_spacing("() => {"), "() => {");
        assert_eq!(fix_arrow_spacing("(x) => x"), "(x) => x");
    }

    #[test]
    fn arrow_spacing_mixed_code() {
        let input = "const fn = ()=>{\n\treturn a <= b;\n}\n";
        let expected = "const fn = () => {\n\treturn a <= b;\n}\n";
        assert_eq!(fix_arrow_spacing(input), expected);
    }

    #[test]
    fn arrow_spacing_implicit_return() {
        // `(x)=>(` should become `(x) => (`
        assert_eq!(fix_arrow_spacing("(x)=>({x})"), "(x) => ({x})");
    }

    #[test]
    fn collapse_no_false_positive_do_while() {
        // `do {` ends with `{` but before brace is `o`, not `)`
        let src = "do {\n\tstmt;\n} while (cond);\n";
        let result = collapse_single_stmt_blocks(src, 180, 3);
        assert_eq!(result, src);
    }

    #[test]
    fn collapse_consecutive_blocks() {
        let src = "if (a) {\n\tfa();\n}\nif (b) {\n\tfb();\n}\n";
        let result = collapse_single_stmt_blocks(src, 180, 3);
        assert_eq!(result, "if (a) { fa(); }\nif (b) { fb(); }\n");
    }

    #[test]
    fn collapse_empty_body_not_touched() {
        let src = "if (cond) {\n\t\n}\n";
        let result = collapse_single_stmt_blocks(src, 180, 3);
        // Empty body should stay as-is (stmt_trimmed.is_empty() check)
        assert_eq!(result, src);
    }

    #[test]
    fn collapse_for_loop_block() {
        let src = "for (;;) {\n\tstmt();\n}\n";
        let result = collapse_single_stmt_blocks(src, 180, 3);
        assert_eq!(result, "for (;;) { stmt(); }\n");
    }

    #[test]
    fn collapse_while_loop_block() {
        let src = "while (cond) {\n\tstmt();\n}\n";
        let result = collapse_single_stmt_blocks(src, 180, 3);
        assert_eq!(result, "while (cond) { stmt(); }\n");
    }

    #[test]
    fn collapse_no_trailing_newline_in_input() {
        // Input without trailing newline
        let src = "if (cond) {\n\tdoIt();\n}";
        let result = collapse_single_stmt_blocks(src, 180, 3);
        assert_eq!(result, "if (cond) { doIt(); }",
            "Should preserve absence of trailing newline");
    }

    #[test]
    fn collapse_obj_lit_multi_member() {
        let src = "foo({\n\tx: 1,\n\ty: 2,\n\tz: 3\n});\n";
        let result = collapse_single_stmt_blocks(src, 180, 3);
        assert_eq!(result, "foo({ x: 1, y: 2, z: 3 });\n");
    }

    #[test]
    fn collapse_spaced_obj_lit_opener() {
        // Space between function name and `({`
        let src = "foo ({\n\tkey: val\n});\n";
        let result = collapse_single_stmt_blocks(src, 180, 3);
        assert_eq!(result, "foo ({ key: val });\n");
    }

    #[test]
    fn collapse_obj_lit_as_second_arg() {
        // Object literal as non-first argument like dispatchEvent("click", { ... })
        let src = "\t\thidden.dispatchEvent(new Event(\"input\", {\n\t\t\tbubbles: true\n\t\t}));\n";
        let result = collapse_single_stmt_blocks(src, 180, 3);
        assert_eq!(result, "\t\thidden.dispatchEvent(new Event(\"input\", { bubbles: true }));\n");
    }

    #[test]
    fn collapse_obj_lit_as_second_arg_narrow() {
        // Same as above but at a narrow width where it just fits
        let src = "\t\thidden.dispatchEvent(new Event(\"input\", {\n\t\t\tbubbles: true\n\t\t}));\n";
        let result = collapse_single_stmt_blocks(src, 62, 3);
        assert_eq!(result, "\t\thidden.dispatchEvent(new Event(\"input\", { bubbles: true }));\n");
    }

    #[test]
    fn collapse_obj_lit_as_second_arg_too_wide() {
        // Collapsed line is 62 chars, so at width 61 it should NOT collapse
        let src = "\t\thidden.dispatchEvent(new Event(\"input\", {\n\t\t\tbubbles: true\n\t\t}));\n";
        let result = collapse_single_stmt_blocks(src, 61, 3);
        assert_eq!(result, src);
    }

    #[test]
    fn collapse_nested_too_wide_preserves_inner() {
        // Outer if fits, but inner obj lit is too wide for 50 chars
        // Inner should stay expanded since it doesn't fit
        let src = "\tif (cond) {\n\t\tfn(veryLongFunctionName, extremelyLongArgumentThatExceedsFiftyCharacters);\n\t}\n";
        let result = collapse_single_stmt_blocks(src, 50, 3);
        assert_eq!(result, src, "Should stay multi-line if collapsed line exceeds max_width");
    }

    // ─── Narrow wrapWidth boundary tests ──────────────────────────

    #[test]
    fn narrow_width_blocks_that_barely_fit() {
        // Short if-block that just fits in 40 chars
        // "if (cond) { doIt(); }" = 22 chars, with prefix = 23
        let src = "\tif (cond) {\n\t\tdoIt();\n\t}\n";
        let result = collapse_single_stmt_blocks(src, 23, 3);
        assert_eq!(result, "\tif (cond) { doIt(); }\n");
    }

    #[test]
    fn narrow_width_block_one_char_too_wide() {
        // "\tif (cond) { doIt(); }" = 22 chars (tab + 21), so at width=21 it should NOT collapse
        let src = "\tif (cond) {\n\t\tdoIt();\n\t}\n";
        let result = collapse_single_stmt_blocks(src, 21, 3);
        assert_eq!(result, src, "Should stay multi-line when collapsed line is 1 char too wide");
    }

    #[test]
    fn narrow_width_demo_if_block_stays_expanded() {
        // The exact pattern from demo.ree: "if (items[activeIndex]) { items[activeIndex].scrollIntoView({ block: "nearest" }); }"
        // This is ~93 chars + tabs. At width=80, it should NOT collapse.
        let expected_collapsed_len = "\t\t\t\tif (items[activeIndex]) { items[activeIndex].scrollIntoView({ block: \"nearest\" }); }".len();
        assert!(expected_collapsed_len > 80, "collapsed line should be >80 chars for this test to be meaningful");

        let src = "\t\t\t\tif (items[activeIndex]) {\n\t\t\t\t\titems[activeIndex].scrollIntoView({\n\t\t\t\t\t\tblock: \"nearest\"\n\t\t\t\t\t});\n\t\t\t\t}\n";
        let result = collapse_single_stmt_blocks(src, 80, 3);
        // Inner obj lit might still collapse since it's short: "items[activeIndex].scrollIntoView({ block: "nearest" });"
        // That's ~62 chars + 5 tabs = ~67 chars, which fits in 80.
        // But the outer if should NOT collapse because the full line is ~93+ chars
        assert!(
            result.contains("scrollIntoView({ block: \"nearest\" });"),
            "Inner obj lit should collapse even at narrow width"
        );
        assert!(
            !result.contains("if (items[activeIndex]) { items[activeIndex]"),
            "Outer if-block should NOT collapse at width=80"
        );
    }

    #[test]
    fn narrow_width_obj_lit_barely_fits() {
        // "\t\tfoo({ x: 1, y: 2 });" = 22 chars (2 tabs + 20)
        let src = "\t\tfoo({\n\t\t\tx: 1,\n\t\t\ty: 2\n\t\t});\n";
        let result = collapse_single_stmt_blocks(src, 22, 3);
        assert_eq!(result, "\t\tfoo({ x: 1, y: 2 });\n");
    }

    #[test]
    fn narrow_width_obj_lit_too_wide() {
        // "\t\tfoo({ x: 1, y: 2 });" = 22 chars, so at width=21 it should NOT collapse
        let src = "\t\tfoo({\n\t\t\tx: 1,\n\t\t\ty: 2\n\t\t});\n";
        let result = collapse_single_stmt_blocks(src, 21, 3);
        assert_eq!(result, src, "Obj lit should stay multi-line when 1 char too wide");
    }

    #[test]
    fn narrow_width_else_if_barely_fits() {
        // "\t} else if (k) { stmt(); }" = 28 chars
        let src = "\t} else if (k) {\n\t\tstmt();\n\t}\n";
        let result = collapse_single_stmt_blocks(src, 28, 3);
        assert_eq!(result, "\t} else if (k) { stmt(); }\n");
    }

    #[test]
    fn narrow_width_for_loop_barely_fits() {
        // "\tfor (;;) { stmt(); }" = 22 chars
        let src = "\tfor (;;) {\n\t\tstmt();\n\t}\n";
        let result = collapse_single_stmt_blocks(src, 22, 3);
        assert_eq!(result, "\tfor (;;) { stmt(); }\n");
    }

    #[test]
    fn narrow_width_while_barely_fits() {
        // "\twhile (c) { stmt(); }" = 24 chars
        let src = "\twhile (c) {\n\t\tstmt();\n\t}\n";
        let result = collapse_single_stmt_blocks(src, 24, 3);
        assert_eq!(result, "\twhile (c) { stmt(); }\n");
    }
}
