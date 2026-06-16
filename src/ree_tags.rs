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

/// Protect the content inside `<script>` and `<style>` tags with opaque placeholders
/// before Ree→fake-HTML conversion. This prevents Ree block tags (`{#if}`, `{#each}`,
/// `{#with}`) that appear inside `<script>` or `<style>` content from being converted
/// to fake HTML elements (`<ree-N>`), which would cause dprint's markup_fmt to fail
/// since HTML elements are not valid inside `<script>` or `<style>` content.
///
/// Returns (protected_string, placeholders) where placeholders[i] is the original
/// content that was inside the script/style tag.
pub(crate) fn protect_script_style_content(src: &str) -> (String, Vec<String>) {
    let mut result = String::new();
    let mut placeholders: Vec<String> = Vec::new();
    let mut rest = src;

    while let Some(start) = rest.find('<') {
        // Check if this is a <script or <style opening tag
        let after_open = &rest[start..];
        let is_script = after_open.starts_with("<script");
        let is_style = after_open.starts_with("<style");

        if !is_script && !is_style {
            result.push_str(&rest[..start + 1]);
            rest = &rest[start + 1..];
            continue;
        }

        // Find the end of the opening tag (the closing >)
        let tag_start = start;
        let _tag_prefix = if is_script { "<script" } else { "<style" };
        let close_tag = if is_script { "</script>" } else { "</style>" };

        let Some(open_tag_end) = rest[tag_start..].find('>') else {
            result.push_str(&rest[..start + 1]);
            rest = &rest[start + 1..];
            continue;
        };
        let open_tag_close = tag_start + open_tag_end + 1;

        // Find the closing </script> or </style>
        let content_start = open_tag_close;
        let after_content = &rest[content_start..];
        let Some(close_pos) = after_content.find(close_tag) else {
            result.push_str(&rest[..start + 1]);
            rest = &rest[start + 1..];
            continue;
        };
        let content_end = content_start + close_pos;
        let close_end = content_end + close_tag.len();

        // Push the open tag
        result.push_str(&rest[..open_tag_close]);

        // Extract the content and replace with placeholder
        let content = rest[content_start..content_end].to_string();
        placeholders.push(content);
        let ph = format!("__SS{}__", placeholders.len() - 1);
        result.push_str(&ph);
        result.push_str(close_tag);

        rest = &rest[close_end..];
    }

    result.push_str(rest);
    (result, placeholders)
}

/// Restore script/style tag content from their placeholders.
pub(crate) fn restore_script_style_content(src: &str, placeholders: &[String]) -> String {
    let mut result = src.to_string();
    for (i, content) in placeholders.iter().enumerate() {
        let placeholder = format!("__SS{}__", i);
        result = result.replace(&placeholder, content);
    }
    result
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
