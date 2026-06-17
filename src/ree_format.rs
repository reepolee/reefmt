use std::fs;
use std::path::Path;

use crate::ree_tags::{protect, restore, protect_html_comments, restore_html_comments, protect_raw_js_blocks, restore_raw_js_blocks, protect_ree_expressions, restore_ree_expressions};

/// Re-indent code based on brace depth.
pub(crate) fn indent_code(src: &str) -> String {
    let mut out = String::new();
    let mut depth: usize = 0;

    for line in src.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            out.push('\n');
            continue;
        }

        let mut opens = 0usize;
        let mut closes = 0usize;
        for ch in trimmed.chars() {
            match ch {
                '{' => opens += 1,
                '}' if opens > 0 => opens -= 1,
                '}' => closes += 1,
                _ => {}
            }
        }

        if closes > 0 {
            depth = depth.saturating_sub(closes);
        }

        out.push_str(&"\t".repeat(depth));
        out.push_str(trimmed);
        out.push('\n');

        depth += opens;
    }

    out
}

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

/// Collapse consecutive blank lines into at most one.
pub(crate) fn collapse_blank_lines(src: &str) -> String {
    let mut out = String::new();
    let mut blank_count = 0usize;
    for line in src.lines() {
        if line.trim().is_empty() {
            blank_count += 1;
            if blank_count <= 1 {
                out.push('\n');
            }
        } else {
            blank_count = 0;
            out.push_str(line);
            out.push('\n');
        }
    }
    out
}

/// Normalize spacing in Ree template directives.
/// Converts `{# if` to `{#if`, `{/ if}` to `{/if}`, etc.
pub(crate) fn normalize_ree_spacing(src: &str) -> String {
    let (protected, comment_placeholders) = protect_html_comments(src);
    let mut out = protected;

    for kw in &["if", "each", "with", "include", "layout"] {
        out = out.replace(&format!("{{# {}", kw), &format!("{{#{}", kw));
        out = out.replace(&format!("{{/ {}", kw), &format!("{{/{}", kw));
    }
    out = out
        .replace("{/ if}", "{/if}")
        .replace("{/ each}", "{/each}")
        .replace("{/ with}", "{/with}")
        .replace("{/ if }", "{/if}")
        .replace("{/ each }", "{/each}")
        .replace("{/ with }", "{/with}")
        .replace("{/if }", "{/if}")
        .replace("{/each }", "{/each}")
        .replace("{/with }", "{/with}");

    restore_html_comments(&out, &comment_placeholders)
}

// ── Fake HTML elements (for dprint markup_fmt integration) ────────────────

/// One entry in the fake-element map. Each `<ree-N>` / `</ree-N>` pair in the
/// fake HTML corresponds to one FakeEntry at index N-1.
pub(crate) struct FakeEntry {
    /// What `<ree-N>` (or `<ree-N />`) restores to.
    pub(crate) open_text: String,
    /// What `</ree-N>` restores to. Empty string = synthetic close emitted for
    /// an {:else} boundary; the tag is simply deleted on restore.
    pub(crate) close_text: String,
}

/// Find the position just past the balanced closing `}` starting from the
/// opening `{`. Handles nested braces (e.g. `{#if obj.items.find(x => x)}`).
fn find_ree_block_end(src: &str) -> usize {
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

/// Convert Ree block-control tags to paired fake HTML elements so dprint's
/// HTML formatter understands the nesting structure.
///
/// Only handles structural tags: {#if}, {#each}, {#with}, their closers, and
/// {:else}/{:else if}. All other Ree syntax is passed through unchanged and
/// will be handled by the existing protect()/restore() calls that wrap this.
pub(crate) fn ree_to_fake_html(src: &str) -> (String, Vec<FakeEntry>) {
    let mut out = String::new();
    let mut entries: Vec<FakeEntry> = Vec::new();
    let mut stack: Vec<usize> = Vec::new();
    let mut rest = src;

    while !rest.is_empty() {
        let Some(pos) = rest.find('{') else {
            out.push_str(rest);
            break;
        };

        let after = &rest[pos..];

        let is_if_open   = after.starts_with("{#if}")
                        || matches!(after.as_bytes().get(4), Some(b' ' | b'\t' | b'\r' | b'\n'))
                           && after.starts_with("{#if");
        let is_each_open = after.starts_with("{#each}")
                        || matches!(after.as_bytes().get(6), Some(b' ' | b'\t' | b'\r' | b'\n'))
                           && after.starts_with("{#each");
        let is_with_open = after.starts_with("{#with}")
                        || matches!(after.as_bytes().get(6), Some(b' ' | b'\t' | b'\r' | b'\n'))
                           && after.starts_with("{#with");

        let is_if_close   = after.starts_with("{/if}");
        let is_each_close = after.starts_with("{/each}");
        let is_with_close = after.starts_with("{/with}");

        let is_else = after.starts_with("{:else}") || after.starts_with("{:else ");

        if is_if_open || is_each_open || is_with_open {
            out.push_str(&rest[..pos]);
            let end = find_ree_block_end(after);
            let tag = after[..end].to_string();
            let idx = entries.len();
            entries.push(FakeEntry { open_text: tag, close_text: String::new() });
            stack.push(idx);
            out.push_str(&format!("<ree-{}>", idx + 1));
            rest = &after[end..];
        } else if is_if_close || is_each_close || is_with_close {
            out.push_str(&rest[..pos]);
            let end = find_ree_block_end(after);
            let tag = after[..end].to_string();
            match stack.pop() {
                Some(idx) => {
                    entries[idx].close_text = tag;
                    out.push_str(&format!("</ree-{}>", idx + 1));
                }
                None => out.push_str(&tag),
            }
            rest = &after[end..];
        } else if is_else {
            out.push_str(&rest[..pos]);
            let end = find_ree_block_end(after);
            let tag = after[..end].to_string();

            if let Some(prev_idx) = stack.pop() {
                out.push_str(&format!("</ree-{}>", prev_idx + 1));
            }

            let idx = entries.len();
            entries.push(FakeEntry { open_text: tag, close_text: String::new() });
            stack.push(idx);
            out.push_str(&format!("<ree-{}>", idx + 1));
            rest = &after[end..];
        } else {
            out.push_str(&rest[..pos + 1]);
            rest = &rest[pos + 1..];
        }
    }

    (out, entries)
}

/// Restore fake HTML elements back to original Ree tags.
pub(crate) fn fake_html_to_ree(src: &str, entries: &[FakeEntry]) -> String {
    let mut result = src.to_string();
    for (i, entry) in entries.iter().enumerate().rev() {
        let n = i + 1;
        result = result.replace(&format!("<ree-{}>", n), &entry.open_text);
        result = result.replace(&format!("</ree-{}>", n), &entry.close_text);
    }
    collapse_blank_lines(&result)
}

/// After dprint formatting, ensure raw JS block placeholders that were
/// originally on their own line stay on their own line. dprint's markup_fmt
/// collapses adjacent text-like placeholders into a single line, losing the
/// structural line break.
fn fix_raw_js_line_breaks(formatted: &str, own_line: &[bool]) -> String {
    let mut result = formatted.to_string();
    for (i, &should_be_own_line) in own_line.iter().enumerate().rev() {
        if should_be_own_line {
            let placeholder = format!("__JB{}__", i);
            if let Some(pos) = result.find(&placeholder) {
                // Check if the placeholder is already at the start of a line
                let before = &result[..pos];
                let at_line_start = before.ends_with('\n') || before.is_empty();
                if !at_line_start {
                    // Match preceding line's indentation for the new line
                    let indent = before.rfind('\n').map_or("", |nl| {
                        let rest = &before[nl + 1..];
                        let ws_len = rest.len() - rest.trim_start().len();
                        &rest[..ws_len]
                    });
                    result.insert_str(pos, &format!("\n{}", indent));
                }
            }
        }
    }
    result
}

/// Format Ree template HTML via dprint's markup_fmt plugin.
///
/// Pipeline:
///   protect_html_comments
///   → protect_raw_js_blocks   ({{ ... }} → __REE_RAW_JS_BLOCK_N__)
///   → ree_to_fake_html        (block tags → <ree-N> pairs, {:else} → split)
///   → protect                 (remaining inline Ree syntax → __REE_*__ strings)
///   → pipe_dprint html        (markup_fmt sees valid HTML with real structure)
///   → restore                 (inline syntax back)
///   → fake_html_to_ree        (block tags back)
///   → restore_raw_js_blocks   (raw JS blocks back)
///   → restore_html_comments
///
/// Returns None if dprint produced no change.
pub(crate) fn format_ree_html_via_dprint(src: &str, config_path: &str) -> Option<String> {
    let (after_comments, html_comments) = protect_html_comments(src.trim());
    let (after_raw_js, raw_js_blocks, raw_js_own_line) = protect_raw_js_blocks(&after_comments);
    let (after_block_tags, entries) = ree_to_fake_html(&after_raw_js);
    // Protect entire inline Ree expressions so dprint doesn't see stray `}` chars
    let (after_ree_exprs, ree_expr_placeholders) = protect_ree_expressions(&after_block_tags);
    let protected = protect(&after_ree_exprs);

    // Strip existing indentation so dprint can apply fresh indentation
    let stripped: String = protected
        .lines()
        .map(|line| line.trim_start())
        .collect::<Vec<_>>()
        .join("\n");

    let formatted = crate::format::pipe_dprint(&stripped, "html", config_path);

    if formatted.trim() == stripped.trim() {
        return None;
    }

    // Ensure raw JS blocks that were on their own line remain on their own line.
    // dprint's markup_fmt may collapse adjacent text-like placeholders into
    // a single line, losing the original line break between a comment and a
    // {{ }} expression that follow each other on separate lines.
    let formatted = fix_raw_js_line_breaks(&formatted, &raw_js_own_line);

    let restored = restore(&formatted);
    let restored = restore_ree_expressions(&restored, &ree_expr_placeholders);
    let restored = fake_html_to_ree(&restored, &entries);
    let restored = restore_raw_js_blocks(&restored, &raw_js_blocks);
    Some(restore_html_comments(&restored, &html_comments))
}

/// Format raw JS blocks ({{ ... }}) in a Ree template.
pub(crate) fn replace_raw_js_blocks(content: &str) -> String {
    let mut result = String::new();
    let mut remaining = content;

    loop {
        match remaining.find("{{") {
            None => {
                result.push_str(remaining);
                break;
            }
            Some(start) => {
                result.push_str(&remaining[..start]);
                let before_block = &remaining[..start];
                let line_indent = match before_block.rfind('\n') {
                    Some(pos) => &before_block[pos + 1..],
                    None => before_block,
                };
                remaining = &remaining[start..];

                match remaining[2..].find("}}") {
                    None => {
                        result.push_str(remaining);
                        break;
                    }
                    Some(end) => {
                        let body = &remaining[2..end + 2];
                        let close_end = end + 4;

                        if body.trim().is_empty() {
                            result.push_str("{{");
                            result.push_str("}}");
                        } else {
                            let is_multiline = body.contains('\n');
                            let formatted = crate::format::dprint_format(body, "js");

                            if is_multiline {
                                let indent_content = format!("{}\t", line_indent);
                                let indented = formatted
                                    .lines()
                                    .map(|l| {
                                        let stripped = l.strip_prefix('\t').unwrap_or(l);
                                        format!("{}{}", indent_content, stripped)
                                    })
                                    .collect::<Vec<_>>()
                                    .join("\n");
                                result.push_str("{{");
                                result.push('\n');
                                result.push_str(&indented);
                                result.push('\n');
                                result.push_str(line_indent);
                                result.push_str("}}");
                            } else {
                                let trimmed = formatted.trim();
                                result.push_str(&format!("{{{{ {} }}}}", trimmed));
                            }
                        }
                        remaining = &remaining[close_end..];
                    }
                }
            }
        }
    }

    result
}

/// Format `<script>` or `<style>` tag content in a Ree template.
pub(crate) fn replace_script_style(content: &str, tag_name: &str, lang: &str) -> String {
    let open_marker = format!("<{}", tag_name);
    let close_marker = format!("</{}>", tag_name);

    let mut result = String::new();
    let mut remaining = content;

    loop {
        match remaining.find(&open_marker) {
            None => {
                result.push_str(remaining);
                break;
            }
            Some(start) => {
                let base_indent = &remaining[..start];
                result.push_str(base_indent);
                let line_indent = match base_indent.rfind('\n') {
                    Some(pos) => &base_indent[pos + 1..],
                    None => base_indent,
                };
                remaining = &remaining[start..];

                let tag_end = match remaining.find('>') {
                    Some(i) => i + 1,
                    None => {
                        result.push_str(remaining);
                        break;
                    }
                };
                let open_tag = &remaining[..tag_end];
                remaining = &remaining[tag_end..];

                match remaining.find(&close_marker) {
                    None => {
                        result.push_str(open_tag);
                        result.push_str(remaining);
                        break;
                    }
                    Some(body_end) => {
                        let body = &remaining[..body_end];
                        let close_end = body_end + close_marker.len();
                        let close_tag = &remaining[body_end..close_end];
                        remaining = &remaining[close_end..];

                        if body.trim().is_empty() {
                            result.push_str(open_tag);
                            result.push_str(close_tag);
                        } else {
                            let formatted = crate::format::dprint_format(body, lang);
                            let indent_content = format!("{}\t", line_indent);
                            let indented = formatted
                                .lines()
                                .map(|l| {
                                    let stripped = l.strip_prefix('\t').unwrap_or(l);
                                    format!("{}{}", indent_content, stripped)
                                })
                                .collect::<Vec<_>>()
                                .join("\n");
                            result.push_str(open_tag);
                            result.push('\n');
                            result.push_str(&indented);
                            result.push('\n');
                            result.push_str(line_indent);
                            result.push_str(close_tag);
                        }
                    }
                }
            }
        }
    }

    result
}

/// Format full Ree template content.
pub(crate) fn format_ree_content(content: &str) -> String {
    let normalized = content.replace("\r\n", "\n");
    let result = normalize_ree_spacing(&normalized);

    // Format the HTML skeleton via dprint's markup_fmt plugin.
    let result = {
        match crate::format::resolve_dprint_config() {
            Some(ref path) => {
                let dprint_result = format_ree_html_via_dprint(&result, path);
                if path.contains("reefmt_dprint_config_") {
                    let _ = fs::remove_file(path);
                }
                dprint_result.unwrap_or(result)
            }
            None => result,
        }
    };

    let result = replace_raw_js_blocks(&result);
    let result = replace_script_style(&result, "script", "js");
    let result = replace_script_style(&result, "style", "css");
    if !result.is_empty() && !result.ends_with('\n') {
        format!("{}\n", result)
    } else {
        result
    }
}

/// Format a Ree template file. Returns `true` if the file was (or would be) modified.
pub(crate) fn format_ree_file(path: &Path, mode: crate::format::Mode) -> bool {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error reading {}: {}", path.display(), e);
            return false;
        }
    };

    let normalized = content.replace("\r\n", "\n");
    let write_content = format_ree_content(&normalized);

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
    fn normalizes_with_directive_spacing() {
        let src = "{# with props.headers}\n{/ with }";
        assert_eq!(normalize_ree_spacing(src), "{#with props.headers}\n{/with}");
    }

    #[test]
    fn normalize_skips_ree_directives_in_html_comments() {
        let src = "<!-- {# if show} -->\n<div>\n{# if show}\n<span>text</span>\n{/ if }";
        let result = normalize_ree_spacing(src);
        assert_eq!(
            result,
            "<!-- {# if show} -->\n<div>\n{#if show}\n<span>text</span>\n{/if}"
        );
    }

    #[test]
    fn normalize_skips_ree_directives_in_multiline_html_comments() {
        let src = "<div>\n<!--\n{# if debug}\n{/ if }\n-->\n{# each items}\n<p>item</p>\n{/each}";
        let result = normalize_ree_spacing(src);
        let lines: Vec<&str> = result.lines().collect();
        assert_eq!(lines[1], "<!--");
        assert_eq!(lines[2], "{# if debug}", "inside comment - preserved");
        assert_eq!(lines[3], "{/ if }", "inside comment - preserved");
        assert_eq!(lines[4], "-->");
        assert_eq!(lines[5], "{#each items}", "outside comment - normalized");
        assert_eq!(lines[7], "{/each}", "outside comment - normalized");
    }

    #[test]
    fn normalize_preserves_comment_near_real_directives() {
        let src = "{#if show}<!-- {# if comment} -->\n<p>text</p>\n{/if}";
        let result = normalize_ree_spacing(src);
        assert_eq!(
            result,
            "{#if show}<!-- {# if comment} -->\n<p>text</p>\n{/if}"
        );
    }

    #[test]
    fn normalize_with_no_comments_is_unchanged() {
        let src = "{# if show}\n{/ if }";
        assert_eq!(normalize_ree_spacing(src), "{#if show}\n{/if}");
    }

    #[test]
    fn format_ree_content_idempotent() {
        let src = "<span>text</span>\n";
        let result = format_ree_content(src);
        assert_eq!(result, src, "format_ree_content should be idempotent for already-formatted content");
    }
}
