use std::fs;
use std::path::Path;

/// Flatten multiline string concatenation into single lines so biome's
/// `useTemplate` rule can detect and convert it. Only joins lines where
/// the continuation line starts with `+` (after trimming), which is the
/// standard pattern for multiline string concatenation in JS.
pub(crate) fn flatten_concat(src: &str) -> String {
    let mut out = String::new();
    let mut prev_line = String::new();
    let mut has_prev = false;

    for line in src.lines() {
        let trimmed = line.trim();
        if has_prev && trimmed.starts_with('+') {
            prev_line.push(' ');
            prev_line.push_str(trimmed);
        } else {
            if has_prev {
                out.push_str(&prev_line);
                out.push('\n');
            }
            prev_line = line.to_string();
            has_prev = true;
        }
    }
    if has_prev {
        out.push_str(&prev_line);
    }
    out
}

/// Format full Ree template content.
///
/// Uses the custom AST-based parser (ree_parser) for zero-dependency formatting,
/// then formats embedded JS/CSS via SWC (native Rust, no subprocess).
pub(crate) fn format_ree_content(content: &str, wrap_width: usize) -> String {
    let ast_output = crate::ree_parser::format_ree(content, wrap_width);
    let result = format_script_blocks(&ast_output);
    result
}

/// Post-process `<script>` blocks to format JS content via SWC.
/// Protects Ree expressions (`{= ... }`, `{~ ... }`) by replacing
/// them with placeholders before formatting, then restores them after.
fn format_script_blocks(content: &str) -> String {
    let mut out = String::with_capacity(content.len());
    let mut remaining = content;
    while let Some(script_start) = remaining.find("<script") {
        // Find the closing > of the opening tag
        if let Some(tag_end) = remaining[script_start..].find('>') {
            let tag_close = script_start + tag_end + 1;
            // Find the closing </script>
            if let Some(script_end) = remaining[tag_close..].find("</script>") {
                let content_start = tag_close;
                let content_end = tag_close + script_end;

                // Detect the indentation level of the <script> tag
                let before = &remaining[..script_start];
                let script_indent: String = before.chars().rev()
                    .take_while(|&c| c == '\t')
                    .collect::<String>()
                    .chars()
                    .rev()
                    .collect();

                // Copy everything before the script tag
                out.push_str(before);
                // Copy the opening <script...> tag
                out.push_str(&remaining[script_start..content_start]);

                let script_content = &remaining[content_start..content_end];
                let formatted = format_script_content(script_content);
                out.push_str(&formatted);

                if formatted.is_empty() {
                    // Empty script block — keep </script> on same line as opening tag
                    // e.g. <script src="..."></script>
                    out.push_str("</script>");
                } else {
                    // Place </script> on its own line at the same indentation as <script>
                    out.push('\n');
                    out.push_str(&script_indent);
                    out.push_str("</script>");
                }
                remaining = &remaining[content_end + 9..]; // 9 = len("</script>")
            } else {
                // No closing tag found, copy as-is
                out.push_str(&remaining[script_start..]);
                remaining = "";
            }
        } else {
            out.push_str(&remaining[script_start..]);
            remaining = "";
        }
    }
    out.push_str(remaining);
    out
}

/// Format the content inside a `<script>` tag.
/// Protects Ree expressions, formats with SWC, then restores them.
/// Detects the base indentation from the original content and preserves it
/// so the output is correctly nested within the AST's tag structure.
fn format_script_content(content: &str) -> String {
    if content.trim().is_empty() {
        return String::new();
    }
    // Detect how many leading newlines there are (separation from <script>>)
    // and trailing newlines (separation from </script>).
    // Crucially, do NOT preserve the AST indentation tabs — the re-indentation
    // logic below adds the correct number of tabs based on detect_min_leading_tabs.
    let leading_nl = count_leading_newlines(content);
    let trailing_nl = count_trailing_newlines(content);
    let trimmed = content.trim();
    
    let base_tabs = detect_min_leading_tabs(trimmed);
    
    // Strip base_tabs from every line to get clean JS for SWC
    let bare_js: String = trimmed.lines()
        .map(|line| {
            if line.is_empty() { return String::new(); }
            let mut tabs = 0;
            let mut chars = line.chars().peekable();
            while tabs < base_tabs {
                match chars.peek() {
                    Some('\t') => { chars.next(); tabs += 1; }
                    _ => break,
                }
            }
            chars.collect()
        })
        .collect::<Vec<_>>()
        .join("\n");
    
    // Protect Ree expressions
    let mut placeholders: Vec<String> = Vec::new();
    let protected = protect_ree_expressions(&bare_js, &mut placeholders);
    
    // Format with SWC
    let formatted = crate::swc_format::format_js_with_indent(&protected, "\t");
    
    // Restore Ree expressions
    let restored = restore_ree_expressions(&formatted, &placeholders);
    
    // Re-indent each line to match AST nesting level
    if restored.trim().is_empty() {
        return String::new();
    }
    let mut out = String::new();
    for line in restored.lines() {
        let ts = line.trim_start();
        if ts.is_empty() {
            out.push('\n');
            continue;
        }
        let leading = line.len() - ts.len();
        let total_tabs = base_tabs + leading;
        for _ in 0..total_tabs { out.push('\t'); }
        out.push_str(ts);
        out.push('\n');
    }
    if out.ends_with('\n') { out.pop(); }
    
    // Prepend leading newlines and append trailing newlines
    // These are just newlines (not tabs) so they don't add extra indentation.
    for _ in 0..leading_nl {
        out.insert(0, '\n');
    }
    for _ in 0..trailing_nl {
        out.push('\n');
    }
    out
}

fn count_leading_newlines(s: &str) -> usize {
    s.chars().take_while(|&c| c == '\n').count()
}

fn count_trailing_newlines(s: &str) -> usize {
    s.chars().rev().take_while(|&c| c == '\n').count()
}

/// Detect the minimum number of leading tab characters in non-empty lines.
/// Ignores lines that have no leading tabs (they're at column 0).
fn detect_min_leading_tabs(content: &str) -> usize {
    let mut min_tabs = usize::MAX;
    for line in content.lines() {
        let trimmed = line.trim_start();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed == line {
            // Line has no leading whitespace — skip it, it's not indented
            continue;
        }
        let leading = line.len() - trimmed.len();
        // Count tabs (each tab is 1 char)
        let tab_count = line[..leading].chars().filter(|&c| c == '\t').count();
        if tab_count > 0 && tab_count < min_tabs {
            min_tabs = tab_count;
        }
    }
    if min_tabs == usize::MAX { 0 } else { min_tabs }
}

/// Replace all Ree syntax with unique placeholders so SWC can parse
/// the surrounding JS without choking on template syntax.
/// Protects: {= expr}, {~ expr}, {#keyword ...}, {:else}, {/keyword}
/// Correctly skips content inside JS single/double-quoted strings.
fn protect_ree_expressions(input: &str, placeholders: &mut Vec<String>) -> String {
    let mut result = String::with_capacity(input.len());
    let bytes = input.as_bytes();
    let len = bytes.len();
    let mut i = 0;
    let mut idx = 0;

    while i < len {
        let b = bytes[i];

        // Skip single/double-quoted JS strings (but NOT backticks — template
        // literals handle {=} via ${} interpolation which SWC parses fine)
        if b == b'\'' || b == b'"' {
            let quote = b;
            result.push(b as char);
            i += 1;
            while i < len {
                let c = bytes[i];
                if c == b'\\' && i + 1 < len {
                    // Escaped character — copy both bytes, skip both
                    result.push(c as char);
                    result.push(bytes[i + 1] as char);
                    i += 2;
                } else if c == quote {
                    result.push(c as char);
                    i += 1;
                    break;
                } else {
                    result.push(c as char);
                    i += 1;
                }
            }
            continue;
        }

        // Check for ANY Ree syntax at '{': {=, {~, {#, {:, {/
        if b == b'{' && i + 1 < len {
            let next = bytes[i + 1];
            let is_ree = next == b'=' || next == b'~' || next == b'#' || next == b':' || next == b'/';
            if is_ree {
                if let Some(end) = input[i + 1..].find('}') {
                    let expr = &input[i..i + 1 + end + 1];
                    let placeholder = format!("__REE_PLACEHOLDER_{}__", idx);
                    idx += 1;
                    placeholders.push(expr.to_string());
                    result.push_str(&placeholder);
                    i += 1 + end + 1;
                    continue;
                }
            }
        }

        result.push(b as char);
        i += 1;
    }
    result
}

/// Restore Ree expression placeholders back to their original expressions.
fn restore_ree_expressions(input: &str, placeholders: &[String]) -> String {
    let mut result = input.to_string();
    for (i, expr) in placeholders.iter().enumerate() {
        let placeholder = format!("__REE_PLACEHOLDER_{}__", i);
        result = result.replace(&placeholder, expr);
    }
    result
}



/// Format a Ree template file. Returns `true` if the file was (or would be) modified.
pub(crate) fn format_ree_file(path: &Path, mode: crate::format::Mode, wrap_width: usize) -> bool {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error reading {}: {}", path.display(), e);
            return false;
        }
    };

    let normalized = content.replace("\r\n", "\n");
    let write_content = format_ree_content(&normalized, wrap_width);

    if write_content == normalized {
        return false;
    }

    match mode {
        crate::format::Mode::Write => {
            match fs::write(path, &write_content) {
                Ok(_) => eprintln!("\r\x1b[KFormatted: {}", path.display()),
                Err(e) => eprintln!("Error writing {}: {}", path.display(), e),
            }
        }
        crate::format::Mode::Check => {
            eprintln!("Would format: {}", path.display());
        }
        crate::format::Mode::Diff => {
            crate::format::print_diff(path, &normalized, &write_content);
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_ree_content_idempotent() {
        let src = "<span>text</span>\n";
        let result = format_ree_content(src, 120);
        assert_eq!(result, src, "format_ree_content should be idempotent for already-formatted content");
    }
}
