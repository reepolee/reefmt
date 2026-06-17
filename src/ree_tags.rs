/// Ree tag markers and their opaque placeholders.
/// Used by protect()/restore() to hide Ree template syntax
/// from external formatters (dprint, biome).
pub(crate) const REE_TAGS: &[(&str, &str)] = &[
    ("{#layout", "__LY__"),
    ("{#include", "__IN__"),
    ("{#with", "__WI__"),
    ("{#each", "__EA__"),
    ("{#if", "__IF__"),
    ("{/with}", "__EW__"),
    ("{/each}", "__EE__"),
    ("{/if}", "__EI__"),
    ("{:else}", "__EL__"),
    ("{@", "__CP__"),
    ("{{", "__RO__"),
    ("}}", "__RC__"),
    ("{~", "__UN__"),
    ("{=", "__ES__"),
    ("{:", "__CL__"),
    ("{#", "__HA__"),
    ("{/", "__SL__"),
];

/// Replace all Ree tag markers with opaque placeholders
/// so external formatters don't interpret them as code.
pub(crate) fn protect(src: &str) -> String {
    let mut out = src.to_string();
    for (tag, placeholder) in REE_TAGS {
        out = out.replace(tag, placeholder);
    }
    out
}

/// Restore Ree tag placeholders back to their original markers.
pub(crate) fn restore(src: &str) -> String {
    let mut out = src.to_string();
    for (tag, placeholder) in REE_TAGS {
        out = out.replace(placeholder, tag);
    }
    out
}

/// Replace HTML comments (`<!-- ... -->`) with unique placeholders
/// so they are not modified by string transformations.
pub(crate) fn protect_html_comments(src: &str) -> (String, Vec<String>) {
    let mut result = String::new();
    let mut placeholders: Vec<String> = Vec::new();
    let mut rest = src;

    while let Some(start) = rest.find("<!--") {
        // Push everything before the comment
        result.push_str(&rest[..start]);
        // Find the end of the comment
        if let Some(end) = rest[start..].find("-->") {
            let comment_end = start + end + 3;
            placeholders.push(rest[start..comment_end].to_string());
            let placeholder = format!("__CM{}__", placeholders.len() - 1);
            result.push_str(&placeholder);
            rest = &rest[comment_end..];
        } else {
            // Unterminated comment — push the rest as-is
            result.push_str(&rest[start..]);
            rest = "";
            break;
        }
    }

    result.push_str(rest);
    (result, placeholders)
}

/// Restore HTML comments from their placeholders.
pub(crate) fn restore_html_comments(src: &str, placeholders: &[String]) -> String {
    let mut result = src.to_string();
    for (i, comment) in placeholders.iter().enumerate() {
        let placeholder = format!("__CM{}__", i);
        result = result.replace(&placeholder, comment);
    }
    result
}

/// Protect the entire `{{ ... }}` block (including markers and content)
/// with opaque placeholders before HTML formatting.
/// This prevents dprint's markup_fmt from collapsing whitespace
/// inside raw JS blocks.
///
/// Returns (protected_string, placeholders, on_own_line_per_block) where
/// `on_own_line_per_block[i]` is true if block i was preceded by a newline
/// (i.e. on its own line in the source).
pub(crate) fn protect_raw_js_blocks(src: &str) -> (String, Vec<String>, Vec<bool>) {
    let mut result = String::new();
    let mut placeholders: Vec<String> = Vec::new();
    let mut own_line: Vec<bool> = Vec::new();
    let mut rest = src;

    while let Some(start) = rest.find("{{") {
        result.push_str(&rest[..start]);

        // Check if this block starts on its own line (preceded by newline)
        let preceded_by_newline = rest[..start].contains('\n');

        // Find the matching }}
        let after_open = &rest[start + 2..];
        if let Some(end) = after_open.find("}}") {
            let block_end = start + 2 + end + 2;
            placeholders.push(rest[start..block_end].to_string());
            own_line.push(preceded_by_newline);
            let placeholder = format!("__JB{}__", placeholders.len() - 1);
            result.push_str(&placeholder);
            rest = &rest[block_end..];
        } else {
            // Unclosed block — push the rest as-is
            result.push_str(&rest[start..]);
            rest = "";
            break;
        }
    }

    result.push_str(rest);
    (result, placeholders, own_line)
}

/// Restore raw JS blocks from their placeholders.
pub(crate) fn restore_raw_js_blocks(src: &str, placeholders: &[String]) -> String {
    let mut result = src.to_string();
    for (i, content) in placeholders.iter().enumerate() {
        let placeholder = format!("__JB{}__", i);
        result = result.replace(&placeholder, content);
    }
    result
}

/// Find the position just past the balanced closing `}` starting from
/// the opening `{`. Handles nested braces.
fn find_balanced_brace_end(src: &str) -> usize {
    let mut depth = 0usize;
    for (i, c) in src.char_indices() {
        match c {
            '{' => depth += 1,
            '}' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return i + 1;
                }
            }
            _ => {}
        }
    }
    src.len()
}

/// Protect entire Ree inline expressions by replacing the full `{...}` pair
/// with an opaque placeholder. This prevents dprint's markup_fmt from seeing
/// stray `}` characters that it interprets as template syntax errors.
///
/// Must be called AFTER `ree_to_fake_html` and BEFORE `protect()`.
pub(crate) fn protect_ree_expressions(src: &str) -> (String, Vec<String>) {
    let mut result = String::new();
    let mut placeholders: Vec<String> = Vec::new();
    let mut rest = src;

    while let Some(pos) = rest.find('{') {
        let after = &rest[pos..];

        let is_ree = after.starts_with("{=")
            || after.starts_with("{~")
            || after.starts_with("{@")
            || after.starts_with("{#layout")
            || after.starts_with("{#include");

        if is_ree {
            result.push_str(&rest[..pos]);
            let end = find_balanced_brace_end(after);
            let expr = &after[..end];
            placeholders.push(expr.to_string());
            let ph = format!("__RIP{}__", placeholders.len() - 1);
            result.push_str(&ph);
            rest = &after[end..];
        } else {
            result.push_str(&rest[..pos + 1]);
            rest = &rest[pos + 1..];
        }
    }

    result.push_str(rest);
    (result, placeholders)
}

/// Restore Ree inline expressions from their placeholders.
pub(crate) fn restore_ree_expressions(src: &str, placeholders: &[String]) -> String {
    let mut result = src.to_string();
    for (i, expr) in placeholders.iter().enumerate() {
        let placeholder = format!("__RIP{}__", i);
        result = result.replace(&placeholder, expr);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn protect_skips_ree_tags_inside_html_comments() {
        // Simulate the protect pipeline
        let src = "// <!-- {=value} -->\n/* {# if debug} */";

        let (html_protected, html_comments) = protect_html_comments(src);
        let after_protect = protect(&html_protected);
        let after_restore = restore(&after_protect);
        let final_result = restore_html_comments(&after_restore, &html_comments);

        assert!(
            final_result.contains("<!-- {=value} -->"),
            "Ree tag inside HTML comment should be preserved, got: {}",
            final_result
        );
        assert!(
            final_result.contains("{# if debug}"),
            "Ree tag in block comment should be preserved, got: {}",
            final_result
        );
    }

    #[test]
    fn protect_preserves_html_comment_markers() {
        let src = "let x = 1;\n<!--\n  legacy JS guard\n-->\nlet y = 2;";

        let (html_protected, html_comments) = protect_html_comments(src);
        let after_protect = protect(&html_protected);
        let after_restore = restore(&after_protect);
        let final_result = restore_html_comments(&after_restore, &html_comments);

        assert_eq!(final_result, src, "HTML comments should survive protect/restore unchanged");
    }
}
