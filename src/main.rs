use glob::glob;
use serde::Deserialize;
use similar::{ChangeTag, DiffOp};
use std::{
    env, fs,
    io::{Read, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

const REE_TAGS: &[(&str, &str)] = &[
    ("{#layout", "__REE_LAYOUT__"),
    ("{#include", "__REE_INCLUDE__"),
    ("{#with", "__REE_WITH__"),
    ("{#each", "__REE_EACH__"),
    ("{#if", "__REE_IF__"),
    ("{/with}", "__REE_END_WITH__"),
    ("{/each}", "__REE_END_EACH__"),
    ("{/if}", "__REE_END_IF__"),
    ("{:else}", "__REE_ELSE__"),
    ("{@", "__REE_COMPONENT__"),
    ("{{", "__REE_RAW_JS_OPEN__"),
    ("}}", "__REE_RAW_JS_CLOSE__"),
    ("{~", "__REE_UNESCAPED__"),
    ("{=", "__REE_ESCAPED__"),
    ("{:", "__REE_COLON__"),
    ("{#", "__REE_HASH__"),
    ("{/", "__REE_SLASH__"),
];

// Default configs are embedded from standalone files at build time.
const DPRINT_CONFIG: &str = include_str!("../dprint.default.json");
const BIOME_CONFIG: &str = include_str!("../biome.default.json");
/// Reefmt configuration — loaded from `reefmt.jsonc` in the project root.
#[derive(Deserialize)]
#[serde(default)]
struct ReeConfig {
    /// Directories to skip when formatting.
    #[serde(rename = "skipDirs")]
    skip_dirs: Vec<String>,
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
            ],
            extensions: vec![
                "ree".to_string(),
                "ts".to_string(),
                "js".to_string(),
                "css".to_string(),
            ],
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

fn protect(src: &str) -> String {
    let mut out = src.to_string();
    for (tag, placeholder) in REE_TAGS {
        out = out.replace(tag, placeholder);
    }
    out
}

fn restore(src: &str) -> String {
    let mut out = src.to_string();
    for (tag, placeholder) in REE_TAGS {
        out = out.replace(placeholder, tag);
    }
    out
}

/// Resolve the dprint config path. If a `dprint.json` exists in the current
/// directory, use it directly. Otherwise, write the hardcoded defaults to a
/// temp file and return that path.
fn resolve_dprint_config(timestamp: u128) -> Option<String> {
    // Check for external config in CWD
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

    // Fall back to hardcoded defaults
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

/// Run biome lint --fix on the JS/CSS/TS fragment (via temp file)
/// to apply refactoring rules. Uses an external `biome.json`/`biome.jsonc`
/// from the current directory if available, otherwise falls back to the
/// hardcoded defaults. Returns the source unchanged if biome is not installed.
fn run_biome_lint(src: &str, ext: &str, timestamp: u128) -> String {
    let dir = env::temp_dir().join(format!("reefmt_biome_{}", timestamp));
    if fs::create_dir_all(&dir).is_err() {
        return src.to_string();
    }

    let tmp_path = dir.join(format!("input.{}", ext));
    let config_path = dir.join("biome.json");

    // Use external biome.json from CWD if it exists, otherwise use hardcoded defaults
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

    // Set current_dir to the temp dir so biome finds the config we wrote there
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

/// Pipe content through dprint for formatting. Returns the formatted output
/// unchanged (no Ree-specific processing, no extra indentation).
/// Falls back to returning the source unchanged if dprint is not installed.
fn pipe_dprint(src: &str, ext: &str, config_path: &str) -> String {
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

fn dprint_format(src: &str, lang: &str) -> String {
    let ext = if lang == "css" { "css" } else { "js" };
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();

    // Protect HTML comments so content inside them is not modified
    let (html_protected, html_comments) = protect_html_comments(src.trim());
    let protected = protect(&html_protected);

    // Step 1: Join multiline concatenation so biome's useTemplate can detect it
    let flattened = flatten_concat(&protected);

    // Step 2: Run biome lint --fix on the fragment (refactoring like prefer-template)
    let after_lint = run_biome_lint(&flattened, ext, timestamp);

    // Step 3: Pipe through dprint for formatting (with external config if available)
    let config_path = resolve_dprint_config(timestamp);

    let formatted = match config_path {
        Some(ref path) => {
            let result = pipe_dprint(&after_lint, ext, path);
            // Only clean up temp configs — never delete an external dprint.json!
            if path.contains("reefmt_dprint_config_") {
                let _ = fs::remove_file(path);
            }
            result
        }
        None => indent_code(src),
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

/// Strip the `<ree-root>` wrapper added by `dprint_format_html`.
/// Removes the first and last lines, then un-indents each remaining line
/// by removing one leading tab.
fn strip_ree_root_wrapper(src: &str) -> String {
    let mut lines: Vec<&str> = src.lines().collect();
    if lines.len() >= 2 {
        lines.remove(0); // <ree-root>
        lines.pop();     // </ree-root>
    }
    lines
        .iter()
        .map(|line| {
            if line.is_empty() {
                String::new()
            } else {
                line.strip_prefix('\t').unwrap_or(line).to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Insert guard `<span></span>` lines between consecutive lines where
/// both lines contain protected Ree placeholders but no HTML tags.
/// This prevents dprint from collapsing newlines between text-only
/// Ree tag lines.
fn insert_ree_guards(src: &str) -> String {
    let lines: Vec<&str> = src.lines().collect();
    let mut out = String::new();
    for i in 0..lines.len() {
        if i > 0 {
            let prev = lines[i - 1].trim();
            let curr = lines[i].trim();
            let prev_is_ree = prev.contains("__REE_") && !prev.starts_with('<');
            let curr_is_ree = curr.contains("__REE_") && !curr.starts_with('<');
            if prev_is_ree && curr_is_ree {
                out.push_str("<!--R-->\n");
            }
        }
        out.push_str(lines[i]);
        out.push('\n');
    }
    out
}

/// Pipe Ree template content through dprint's markup_fmt plugin for HTML formatting.
/// Protects Ree tags and HTML comments before formatting, then restores them.
/// Falls back to returning the source unchanged if dprint is not installed.
///
/// Wraps content in `<ree-root>` to give dprint a proper HTML context, and
/// inserts `<span></span>` guards between consecutive Ree-only lines to
/// prevent dprint from collapsing newlines between text nodes.
fn dprint_format_html(src: &str) -> String {
    if src.trim().is_empty() {
        return String::new();
    }

    // Protect HTML comments so content inside them is not modified
    let (html_protected, html_comments) = protect_html_comments(src.trim());
    // Protect Ree tags so they are not modified by dprint
    let protected = protect(&html_protected);

    // Wrap in a root element to give dprint a proper HTML context.
    let wrapped = format!("<ree-root>\n{}\n</ree-root>", protected);

    // Insert <span></span> guards between consecutive Ree-only lines
    // to prevent dprint from collapsing newlines between them.
    let guarded = insert_ree_guards(&wrapped);

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();

    // Get dprint config (external or fallback default with markup_fmt)
    let config_path = resolve_dprint_config(timestamp);

    let formatted = match config_path {
        Some(ref path) => {
            // Pass as .html to trigger markup_fmt plugin
            let result = pipe_dprint(&guarded, "html", path);
            if path.contains("reefmt_dprint_config_") {
                let _ = fs::remove_file(path);
            }
            result
        }
        None => guarded,
    };

    // Remove guard <!--R--> lines (HTML comments that dprint preserves verbatim)
    let cleaned: String = formatted
        .lines()
        .filter(|l| l.trim() != "<!--R-->")
        .collect::<Vec<_>>()
        .join("\n");

    // Strip the <ree-root> wrapper and un-indent by one level
    let unwrapped = strip_ree_root_wrapper(&cleaned);

    let restored = restore(&unwrapped);
    restore_html_comments(&restored, &html_comments)
}

fn indent_code(src: &str) -> String {
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
fn flatten_concat(src: &str) -> String {
    let mut out = String::new();
    let mut prev_line = String::new();
    let mut has_prev = false;

    for line in src.lines() {
        let trimmed = line.trim();
        if has_prev && trimmed.starts_with('+') {
            // Join continuation line to previous: remove the newline and leading whitespace
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

/// Format raw JS blocks ({{ ... }}) in a Ree template.
/// Extracts JS code between `{{` and `}}` markers,
/// formats it through biome lint-fix + dprint (same as `<script>` content),
/// and re-indents properly.
fn replace_raw_js_blocks(content: &str) -> String {
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

                // Find closing }}
                match remaining[2..].find("}}") {
                    None => {
                        result.push_str(remaining);
                        break;
                    }
                    Some(end) => {
                        let body = &remaining[2..end + 2]; // between {{ and }}
                        let close_end = end + 4;

                        if body.trim().is_empty() {
                            result.push_str("{{");
                            result.push_str("}}");
                        } else {
                            let is_multiline = body.contains('\n');
                            let formatted = dprint_format(body, "js");

                            if is_multiline {
                                // Multiline block — expand like <script>...</script>
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
                                // Single line — keep inline
                                let trimmed = formatted.trim();
                                result.push_str(&format!("{{ {} }}", trimmed));
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

fn replace_script_style(content: &str, tag_name: &str, lang: &str) -> String {
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
                            let formatted = dprint_format(body, lang);
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

/// Replace HTML comments (`<!-- ... -->`) with unique placeholders
/// so they are not modified by string transformations.
fn protect_html_comments(src: &str) -> (String, Vec<String>) {
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
            let placeholder = format!("__REE_HTML_COMMENT_{}__", placeholders.len() - 1);
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
fn restore_html_comments(src: &str, placeholders: &[String]) -> String {
    let mut result = src.to_string();
    for (i, comment) in placeholders.iter().enumerate() {
        let placeholder = format!("__REE_HTML_COMMENT_{}__", i);
        result = result.replace(&placeholder, comment);
    }
    result
}

fn normalize_ree_spacing(src: &str) -> String {
    // Protect HTML comments so Ree tag-like text inside them is not normalized
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

fn collapse_blank_lines(src: &str) -> String {
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

/// Adjust indentation for HTML tag depth + Ree block-level tags after dprint.
/// dprint indents child HTML elements correctly but NOT text/Ree content.
/// This function tracks HTML tag depth for text content and Ree block depth
/// for {#if}/{#each}/{#with} blocks, combining both with dprint's element indentation.
fn adjust_ree_block_depth(src: &str) -> String {
    let void_tags = [
        "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param",
        "source", "track", "wbr",
    ];

    let mut out = String::new();
    let mut ree_depth: usize = 0;
    // Track HTML depth ourselves (dprint only indents HTML elements, not text content)
    let mut html_depth: usize = 0;
    let mut in_comment: bool = false;

    for line in src.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            out.push('\n');
            continue;
        }

        // Track multiline HTML comment state
        if trimmed.contains("<!--") && !trimmed.contains("-->") {
            in_comment = true;
        }
        if trimmed.contains("-->") {
            in_comment = false;
        }

        // dprint's leading tab indentation (reliable for HTML element lines)
        let dprint_depth = line.chars().take_while(|&c| c == '\t').count();
        let is_html_line = trimmed.starts_with('<') && !trimmed.starts_with("<!--") && !trimmed.starts_with("<!");

        // Count Ree opens/closes on this line (skip if inside multiline comment)
        let count_source = if in_comment {
            String::new()
        } else {
            strip_html_comment_content(trimmed)
        };
        let (ree_opens, ree_closes) = if count_source.is_empty() {
            (0, 0)
        } else {
            count_ree_block_tags(&count_source)
        };
        let is_else = !in_comment && (trimmed.starts_with("{:else") || trimmed.starts_with("{:"));

        // Apply net Ree closes before writing
        let net_ree_close = if is_else { 1 } else { (ree_closes as usize).saturating_sub(ree_opens as usize) };
        ree_depth = ree_depth.saturating_sub(net_ree_close);

        // For HTML lines, use dprint's indentation. For text/Ree lines, use tracked depth.
        let effective_html_depth = if is_html_line { dprint_depth } else { html_depth };

        // Write line at combined depth
        out.push_str(&"\t".repeat(effective_html_depth + ree_depth));
        out.push_str(trimmed);
        out.push('\n');

        // Apply net Ree opens after writing
        let net_ree_open = if is_else { 1 } else { (ree_opens as usize).saturating_sub(ree_closes as usize) };
        ree_depth += net_ree_open;

        // Update tracked HTML depth for subsequent text/Ree lines
        if is_html_line {
            let delta = html_depth_delta(trimmed, &void_tags);
            if delta >= 0 {
                html_depth = effective_html_depth + delta as usize;
            } else {
                html_depth = effective_html_depth.saturating_sub((-delta) as usize);
            }
        }
    }

    collapse_blank_lines(&out)
}

/// Count Ree block tags on a trimmed line. Returns (opens, closes).
fn count_ree_block_tags(trimmed: &str) -> (u64, u64) {
    let mut opens = 0u64;
    let mut closes = 0u64;
    let mut pos;

    for pat in &["{#if", "{#each", "{#with"] {
        pos = 0;
        while let Some(idx) = trimmed[pos..].find(pat) {
            opens += 1;
            pos += idx + pat.len();
        }
    }
    for pat in &["{/if", "{/each", "{/with"] {
        pos = 0;
        while let Some(idx) = trimmed[pos..].find(pat) {
            closes += 1;
            pos += idx + pat.len();
        }
    }
    (opens, closes)
}

/// Compute net HTML tag depth change for a line.
/// Returns -1, 0, or 1 (capped).
fn html_depth_delta(trimmed: &str, void_tags: &[&str]) -> isize {
    let mut pos = 0;
    let mut delta: isize = 0;
    // Track closing tags that are paired with inline opens on this line
    let mut skip_close: isize = 0;

    while let Some(idx) = trimmed[pos..].find('<') {
        let abs = pos + idx;
        let rest = &trimmed[abs..];

        if rest.starts_with("</") {
            if skip_close > 0 {
                skip_close -= 1;
            } else {
                delta -= 1;
            }
        } else if !rest.starts_with("<!--") && !rest.starts_with("<!") {
            let tag_start = abs + 1;
            let tag_name_end = trimmed[tag_start..]
                .find(|c: char| c.is_whitespace() || c == '>' || c == '/')
                .map(|p| tag_start + p)
                .unwrap_or(trimmed.len());
            let tag_name = &trimmed[tag_start..tag_name_end];

            if !void_tags.contains(&tag_name) {
                if let Some(close) = rest.find('>') {
                    let is_self_closed = rest[..close].ends_with('/');
                    let after_tag = &trimmed[abs + close + 1..];
                    let has_inline = after_tag.contains(&format!("</{}>", tag_name));
                    if !is_self_closed && !has_inline {
                        delta += 1;
                    }
                    if has_inline {
                        skip_close += 1;
                    }
                }
            }
        }

        if let Some(close) = rest.find('>') {
            pos = abs + close + 1;
        } else {
            break;
        }
    }

    if delta > 0 { 1 } else if delta < 0 { -1 } else { 0 }
}

/// Strip content between HTML comment markers (`<!-- ... -->`) from a line,
/// replacing it with a space so tokens don't merge. This prevents Ree tags
/// like `{#if}` inside comments from affecting depth tracking.
fn strip_html_comment_content(s: &str) -> String {
    let mut result = String::new();
    let mut rest = s;
    loop {
        match rest.find("<!--") {
            None => {
                result.push_str(rest);
                break;
            }
            Some(start) => {
                result.push_str(&rest[..start]);
                match rest[start..].find("-->") {
                    None => {
                        // Unterminated comment — push the rest as-is
                        result.push_str(&rest[start..]);
                        break;
                    }
                    Some(end) => {
                        rest = &rest[start + end + 3..];
                        result.push(' '); // keep token separation
                    }
                }
            }
        }
    }
    result
}

fn format_html(src: &str) -> String {
    // Step 1: Pipe through dprint's markup_fmt plugin for HTML formatting
    let formatted = dprint_format_html(src);
    // Step 2: Adjust indentation for Ree block-level tags
    adjust_ree_block_depth(&formatted)
}

/// Operating mode: write files, check-only (list files), or diff (show changes).
#[derive(Clone, Copy, PartialEq)]
enum Mode { Write, Check, Diff }

/// Print a unified diff between original and formatted content.
/// Uses the `similar` crate (Myers diff algorithm) for proper hunk detection.
fn print_diff(path: &Path, original: &str, formatted: &str) {
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

/// Format Ree template content, returning the formatted string.
fn format_ree_content(content: &str) -> String {
    let normalized = content.replace("\r\n", "\n");
    let result = normalize_ree_spacing(&normalized);
    let result = format_html(&result);
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
fn format_ree_file(path: &Path, mode: Mode) -> bool {
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

/// Format standalone code content (TS/JS/CSS) by piping through
/// biome lint-fix then dprint formatting. No Ree-specific processing.
/// Falls back to lint-only if dprint is not installed.
fn format_code_content(content: &str, ext: &str) -> String {
    let normalized = content.replace("\r\n", "\n");

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();

    // Step 1: Flatten multiline string concatenation so biome's useTemplate
    // rule can detect and convert it to template literals.
    let flattened = flatten_concat(&normalized);

    // Step 2: biome lint --fix (uses external biome.json if available)
    let after_lint = run_biome_lint(&flattened, ext, timestamp);

    // Step 2: dprint format (uses external dprint.json if available)
    let formatted = match resolve_dprint_config(timestamp) {
        Some(ref config_path) => {
            let result = pipe_dprint(&after_lint, ext, config_path);
            // Only clean up if it was a temp config (not an external one in CWD)
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

/// Format a standalone code file (TS, JS, CSS) by piping through
/// biome lint-fix then dprint formatting. No Ree-specific processing.
/// Falls back to lint-only if dprint is not installed.
/// Returns `true` if the file was (or would be) modified.
fn format_code_file(path: &Path, mode: Mode) -> bool {
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
fn format_file(path: &Path, mode: Mode, config: &ReeConfig) -> bool {
    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
    if !config.extensions.iter().any(|e| e == ext) {
        return false;
    }
    match ext {
        "ree" => format_ree_file(path, mode),
        "ts" | "js" | "css" => format_code_file(path, mode),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_with_directive_as_block() {
        let src = "{#with props.headers}\n<div>\n{=title}\n</div>\n{/with}";

        assert_eq!(
            format_html(&normalize_ree_spacing(src)),
            "{#with props.headers}\n\t<div>\n\t\t{=title}\n\t</div>\n{/with}\n"
        );
    }

    #[test]
    fn normalizes_with_directive_spacing() {
        let src = "{# with props.headers}\n{/ with }";

        assert_eq!(normalize_ree_spacing(src), "{#with props.headers}\n{/with}");
    }

    #[test]
    fn indents_single_line_html_comment() {
        let src = "<div>\n<!-- single line comment -->\n</div>";

        assert_eq!(
            format_html(src),
            "<div>\n\t<!-- single line comment -->\n</div>\n"
        );
    }

    #[test]
    fn indents_multiline_html_comment() {
        let src = "<div>\n<!--\n  comment\n-->\n</div>";

        // Comment content lines get trimmed and indented at same depth
        // as the comment markers, not deeper.
        assert_eq!(
            format_html(src),
            "<div>\n\t<!--\n\tcomment\n\t-->\n</div>\n"
        );
    }

    #[test]
    fn indents_comment_with_html_like_tags_inside() {
        // HTML comment that contains something that looks like an HTML tag
        let src = "<div>\n<!-- this is <b>bold</b> text -->\n<span>text</span>\n</div>";

        assert_eq!(
            format_html(src),
            "<div>\n\t<!-- this is <b>bold</b> text -->\n\t<span>text</span>\n</div>\n"
        );
    }

    #[test]
    fn indents_comment_inside_ree_block() {
        let src = "{#if show}\n<div>\n<!-- comment -->\n<span>text</span>\n</div>\n{/if}";

        assert_eq!(
            format_html(src),
            "{#if show}\n\t<div>\n\t\t<!-- comment -->\n\t\t<span>text</span>\n\t</div>\n{/if}\n"
        );
    }

    #[test]
    fn ree_tag_inside_multiline_comment_does_not_affect_depth() {
        // Ree template directives inside multiline HTML comments
        // should not affect depth tracking.
        let src =
            "{#if show}\n<div>\n<!--\n{#if debug}\n-->\n<p>visible</p>\n</div>\n{/if}";

        let result = format_html(src);
        let lines: Vec<&str> = result.lines().collect();

        assert_eq!(lines[0], "{#if show}", "opening if");
        assert_eq!(lines[1], "\t<div>", "div at depth 1");
        assert_eq!(lines[2], "\t\t<!--", "comment start at depth 2");
        assert_eq!(
            lines[3], "\t\t{#if debug}",
            "ree tag inside comment at depth 2"
        );
        assert_eq!(lines[4], "\t\t-->", "comment end at depth 2");
        assert_eq!(lines[5], "\t\t<p>visible</p>", "p at depth 2");
        assert_eq!(lines[6], "\t</div>", "closing div at depth 1");
        assert_eq!(lines[7], "{/if}", "closing if at depth 0");
    }

    #[test]
    fn ree_if_inside_multiline_comment_doesnt_open_block() {
        // {#if} inside a comment should NOT increase depth,
        // so content after the comment stays at correct depth.
        let src = "<div>\n<!--\n{#if debug}\n-->\n<p>hi</p>\n</div>";

        let result = format_html(src);
        let lines: Vec<&str> = result.lines().collect();

        assert_eq!(lines[0], "<div>", "opening div at depth 0");
        assert_eq!(lines[1], "\t<!--", "comment at depth 1");
        assert_eq!(
            lines[3], "\t-->",
            "comment end at depth 1, not deeper"
        );
        assert_eq!(lines[4], "\t<p>hi</p>", "p at depth 1");
        assert_eq!(lines[5], "</div>", "closing div at depth 0");
    }

    #[test]
    fn ree_slash_if_inside_comment_doesnt_close_block() {
        // {/if} inside a comment should NOT decrease depth.
        let src = "<div>\n<!--\n{/if}\n-->\n<p>hi</p>\n</div>";

        let result = format_html(src);
        let lines: Vec<&str> = result.lines().collect();

        assert_eq!(lines[0], "<div>", "opening div at depth 0");
        assert_eq!(lines[2], "\t{/if}", "/if inside comment at depth 1");
        assert_eq!(lines[4], "\t<p>hi</p>", "p still at depth 1, not unindented");
        assert_eq!(lines[5], "</div>", "closing div at depth 0");
    }

    #[test]
    fn normalize_skips_ree_directives_in_html_comments() {
        // Ree-style syntax inside HTML comments should not be normalized
        let src = "<!-- {# if show} -->\n<div>\n{# if show}\n<span>text</span>\n{/ if }";

        // The comment content should be preserved exactly;
        // real Ree directives outside comments should still be normalized.
        let result = normalize_ree_spacing(src);
        assert_eq!(
            result,
            "<!-- {# if show} -->\n<div>\n{#if show}\n<span>text</span>\n{/if}"
        );
    }

    #[test]
    fn normalize_skips_ree_directives_in_multiline_html_comments() {
        // Ree-style syntax inside multiline HTML comments should not be normalized
        let src = "<div>\n<!--\n{# if debug}\n{/ if }\n-->\n{# each items}\n<p>item</p>\n{/each}";

        let result = normalize_ree_spacing(src);
        let lines: Vec<&str> = result.lines().collect();

        // Comment content preserved exactly
        assert_eq!(lines[1], "<!--");
        assert_eq!(lines[2], "{# if debug}", "inside comment - preserved");
        assert_eq!(lines[3], "{/ if }", "inside comment - preserved");
        assert_eq!(lines[4], "-->");
        // Real directives outside comment are normalized
        assert_eq!(lines[5], "{#each items}", "outside comment - normalized");
        assert_eq!(lines[7], "{/each}", "outside comment - normalized");
    }

    #[test]
    fn normalize_preserves_comment_near_real_directives() {
        // A comment adjacent to a real Ree directive should not interfere
        let src = "{#if show}<!-- {# if comment} -->\n<p>text</p>\n{/if}";

        // The comment's {# if comment} should stay as-is; the real {#if and {/if} should be fine
        let result = normalize_ree_spacing(src);
        assert_eq!(
            result,
            "{#if show}<!-- {# if comment} -->\n<p>text</p>\n{/if}"
        );
    }

    #[test]
    fn normalize_with_no_comments_is_unchanged() {
        // Normal behavior without HTML comments should be unaffected
        let src = "{# if show}\n{/ if }";

        assert_eq!(
            normalize_ree_spacing(src),
            "{#if show}\n{/if}"
        );
    }

    #[test]
    fn single_line_comment_with_ree_tag_still_correct() {
        // Single-line HTML comments (opening and closing on same line)
        // should not enter comment state at all.
        let src = "<div>\n<!-- {#if debug} -->\n<span>text</span>\n</div>";

        let result = format_html(src);
        let lines: Vec<&str> = result.lines().collect();

        assert_eq!(lines[0], "<div>");
        assert_eq!(lines[1], "\t<!-- {#if debug} -->", "single line comment at depth 1");
        assert_eq!(lines[2], "\t<span>text</span>", "span at depth 1");
        assert_eq!(lines[3], "</div>");
    }

    #[test]
    fn protect_skips_ree_tags_inside_html_comments() {
        // Simulate the protect pipeline from biome_format:
        // protect_html_comments -> protect -> restore -> restore_html_comments
        let src = "// <!-- {=value} -->\n/* {# if debug} */";

        let (html_protected, html_comments) = protect_html_comments(src);
        let after_protect = protect(&html_protected);
        let after_restore = restore(&after_protect);
        let final_result = restore_html_comments(&after_restore, &html_comments);

        // The Reese tags inside HTML comments should be preserved exactly
        assert!(
            final_result.contains("<!-- {=value} -->"),
            "Ree tag inside HTML comment should be preserved, got: {}",
            final_result
        );
        // The Ree tag outside HTML comment (in /* */) should be restored too
        assert!(
            final_result.contains("{# if debug}"),
            "Ree tag in block comment should be preserved, got: {}",
            final_result
        );
    }

    #[test]
    fn protect_preserves_html_comment_markers() {
        // Even without Ree tags inside, HTML comment markers should survive
        // the protect/restore pipeline.
        let src = "let x = 1;\n<!--\n  legacy JS guard\n-->\nlet y = 2;";

        let (html_protected, html_comments) = protect_html_comments(src);
        let after_protect = protect(&html_protected);
        let after_restore = restore(&after_protect);
        let final_result = restore_html_comments(&after_restore, &html_comments);

        assert_eq!(final_result, src, "HTML comments should survive protect/restore unchanged");
    }

    #[test]
    fn check_mode_does_not_modify_ree_file() {
        let dir = env::temp_dir().join("reefmt_test_check_mode");
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test.ree");
        let unformatted = "{#if show}\n<div>\n{=title}\n</div>\n{/if}";
        fs::write(&path, unformatted).unwrap();

        // Check mode should report change without modifying the file
        let modified = format_ree_file(&path, Mode::Check);
        assert!(modified, "Check mode should return true when file would change");
        let content_after = fs::read_to_string(&path).unwrap();
        assert_eq!(content_after, unformatted, "Check mode should not modify the file");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn check_mode_reports_no_change_for_formatted_ree_file() {
        // Use simple content that is idempotent through the full pipeline
        let dir = env::temp_dir().join(format!("reefmt_test_check_fmt_{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test.ree");
        let content = "<span>text</span>\n";
        fs::write(&path, content).unwrap();

        // Check mode should report no change for already-formatted file
        let modified = format_ree_file(&path, Mode::Check);
        assert!(!modified, "Check mode should return false for already-formatted file (modified={})", modified);

        // Also verify the file was not modified
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

        // Diff mode should report change without modifying the file
        let modified = format_ree_file(&path, Mode::Diff);
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

        // Write mode should format the file
        let modified = format_ree_file(&path, Mode::Write);
        assert!(modified, "Write mode should return true when file changes");
        let content_after = fs::read_to_string(&path).unwrap();
        assert_ne!(content_after, unformatted, "Write mode should modify the file");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn format_file_unsupported_extension_returns_false() {
        let dir = env::temp_dir().join("reefmt_test_unsupported");
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test.txt");
        fs::write(&path, "hello world").unwrap();

        let config = ReeConfig::default();
        let modified = format_file(&path, Mode::Write, &config);
        assert!(!modified, "format_file should return false for unsupported extension");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn check_mode_ree_file_missing_returns_false() {
        let path = Path::new("/tmp/nonexistent_file_reefmt_test.ree");
        let modified = format_ree_file(path, Mode::Check);
        assert!(!modified, "format_ree_file should return false for missing file");
    }

    #[test]
    fn comment_does_not_affect_sibling_indentation() {
        // A comment with nested tag-looking content should not affect
        // indentation of subsequent sibling elements.
        let src = "<ul>\n<li>first</li>\n<!-- comment with <b>nested</b> -->\n<li>second</li>\n</ul>";

        let result = format_html(src);
        let lines: Vec<&str> = result.lines().collect();

        // <li>first</li> should be at depth 1
        assert_eq!(lines[1], "\t<li>first</li>", "first li indented correctly");
        // comment should be at depth 1
        assert_eq!(
            lines[2], "\t<!-- comment with <b>nested</b> -->",
            "comment indented correctly"
        );
        // <li>second</li> should be at depth 1
        assert_eq!(
            lines[3], "\t<li>second</li>",
            "second li indented correctly"
        );
        // </ul> should be at depth 0
        assert_eq!(lines[4], "</ul>", "closing ul indented correctly");
    }

    #[test]
    fn check_mode_via_format_file_dispatcher() {
        // format_file should delegate to format_ree_file for .ree files
        let dir = env::temp_dir().join("reefmt_test_file_check");
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test.ree");
        let unformatted = "{#if show}\n<div>\n{=title}\n</div>\n{/if}";
        fs::write(&path, unformatted).unwrap();

        let config = ReeConfig::default();
        let modified = format_file(&path, Mode::Check, &config);
        assert!(modified, "format_file Check should detect unformatted .ree file");
        let content_after = fs::read_to_string(&path).unwrap();
        assert_eq!(content_after, unformatted, "format_file Check should not modify the file");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn diff_mode_via_format_file_dispatcher() {
        // format_file should delegate to format_ree_file for .ree files
        let dir = env::temp_dir().join("reefmt_test_file_diff");
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test.ree");
        let unformatted = "{#if show}\n<div>\n{=title}\n</div>\n{/if}";
        fs::write(&path, unformatted).unwrap();

        let config = ReeConfig::default();
        let modified = format_file(&path, Mode::Diff, &config);
        assert!(modified, "format_file Diff should detect unformatted .ree file");
        let content_after = fs::read_to_string(&path).unwrap();
        assert_eq!(content_after, unformatted, "format_file Diff should not modify the file");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn diff_mode_formatted_ree_file_returns_false() {
        // Diff mode on already-formatted content should return false
        let dir = env::temp_dir().join(format!("reefmt_test_diff_fmt_{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test.ree");
        let content = "<span>text</span>\n";
        fs::write(&path, content).unwrap();

        let modified = format_ree_file(&path, Mode::Diff);
        assert!(!modified, "Diff mode should return false for already-formatted file (modified={})", modified);
        let content_after = fs::read_to_string(&path).unwrap();
        assert_eq!(content_after, content, "Diff mode should not modify the file");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn check_mode_code_file_does_not_modify() {
        // Check mode on a code file should not modify the file,
        // regardless of whether external tools are installed.
        // The return value depends on whether tools are available to format.
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
    fn format_file_check_mode_unsupported_extension() {
        // format_file with Check mode on unsupported extension
        let dir = env::temp_dir().join("reefmt_test_unsupported_check");
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test.txt");
        fs::write(&path, "hello world").unwrap();

        let config = ReeConfig::default();
        let modified = format_file(&path, Mode::Check, &config);
        assert!(!modified, "format_file Check should return false for unsupported extension");
        let content_after = fs::read_to_string(&path).unwrap();
        assert_eq!(content_after, "hello world", "Check mode should not modify the file");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn check_mode_empty_ree_file_returns_false() {
        // An empty file is already "formatted" as far as the pipeline is concerned
        let dir = env::temp_dir().join("reefmt_test_empty_check");
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("empty.ree");
        fs::write(&path, "").unwrap();

        let modified = format_ree_file(&path, Mode::Check);
        assert!(!modified, "Check mode should return false for empty file");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn format_ree_content_idempotent() {
        // The Ree formatting pipeline should be idempotent:
        // formatting already-formatted content should produce the same output.
        // Use simple idempotent content that the full pipeline doesn't transform.
        let src = "<span>text</span>\n";
        let result = format_ree_content(src);
        assert_eq!(result, src, "format_ree_content should be idempotent for already-formatted content");
    }

    #[test]
    fn diff_mode_code_file_missing_returns_false() {
        let path = Path::new("/tmp/nonexistent_file_reefmt_diff_test.ts");
        let modified = format_code_file(path, Mode::Diff);
        assert!(!modified, "format_code_file Diff should return false for missing file");
    }

    #[test]
    fn init_template_parses_as_valid_config() {
        // The exact template produced by `reefmt --init` (must match main())
        let template = r#"{
	// Directories to skip when formatting (glob patterns not needed,
	// just directory names — any folder with this name is skipped).
	"skipDirs": ["node_modules", "vendor", "vendors", "dist"],

	// File extensions to format.
	"extensions": ["ree", "ts", "js", "css"],

	// Whether to skip dot-directories (folders starting with '.',
	// like .git, .next, .cache, .svelte-kit, etc.).
	"skipDotDirs": true
}"#;

        let config: ReeConfig =
            json5::from_str(template).expect("--init template should be valid JSONC");

        assert_eq!(config.skip_dirs.len(), 4);
        assert!(config.skip_dirs.contains(&"node_modules".to_string()));
        assert!(config.skip_dirs.contains(&"vendor".to_string()));
        assert!(config.skip_dirs.contains(&"vendors".to_string()));
        assert!(config.skip_dirs.contains(&"dist".to_string()));

        assert_eq!(config.extensions.len(), 4);
        assert!(config.extensions.contains(&"ree".to_string()));
        assert!(config.extensions.contains(&"ts".to_string()));
        assert!(config.extensions.contains(&"js".to_string()));
        assert!(config.extensions.contains(&"css".to_string()));

        assert!(config.skip_dot_dirs);
    }

    #[test]
    fn load_config_parses_reefmt_jsonc_from_directory() {
        let dir = env::temp_dir().join(format!(
            "reefmt_test_load_config_{}",
            std::process::id()
        ));
        fs::create_dir_all(&dir).unwrap();

        // Write a reefmt.jsonc with custom values
        let config_content = r#"{
	"skipDirs": ["node_modules", "dist"],
	"extensions": ["ree", "ts"],
	"skipDotDirs": false
}"#;
        fs::write(dir.join("reefmt.jsonc"), config_content).unwrap();

        // Save current dir, change to temp dir, load config, restore
        let original_cwd = env::current_dir().unwrap();
        env::set_current_dir(&dir).unwrap();

        let config = load_config();

        env::set_current_dir(&original_cwd).unwrap();

        // Verify custom values were loaded
        assert_eq!(config.skip_dirs.len(), 2);
        assert!(!config.skip_dirs.contains(&"vendor".to_string()));

        assert_eq!(config.extensions.len(), 2);
        assert!(!config.extensions.contains(&"js".to_string()));

        assert!(!config.skip_dot_dirs);

        let _ = fs::remove_dir_all(&dir);
    }
}

/// Check whether a path is inside a directory that should be skipped
/// (e.g. `node_modules`, `vendor`, `vendors`, or any dot-folder like `.git`).
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
                // Skip vendor / dependency directories
                if should_skip_path(&path, config) {
                    continue;
                }
                collect_source_files(&path, files, config)?;
            } else if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                if config.extensions.iter().any(|e| e == ext) {
                    files.push(path);
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

    let mode = if diff_mode {
        Mode::Diff
    } else if check_mode {
        Mode::Check
    } else {
        Mode::Write
    };

    // Parse --stdin flag (consumes an optional extension argument)
    let stdin_mode = args.iter().position(|a| a == "--stdin");
    let stdin_ext: Option<String> = stdin_mode.and_then(|pos| {
        args.remove(pos);
        // If the next argument is an extension (starts with '.'), consume it
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
            "ree" => format_ree_content(&input),
            "ts" | "js" | "css" => format_code_content(&input, ext),
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
	"skipDirs": ["node_modules", "vendor", "vendors", "dist"],

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

    let mut any_modified = false;
    let show_progress = mode == Mode::Write;

    for target in targets {
        let path = Path::new(&target);

        if path.is_dir() {
            let mut files = Vec::new();
            if let Err(e) = collect_source_files(path, &mut files, &config) {
                eprintln!("Error reading directory {}: {}", target, e);
                continue;
            }
            for file in files {
                if show_progress {
                    eprint!("\rChecking: {}", file.display());
                    let _ = std::io::stderr().flush();
                }
                if format_file(&file, mode, &config) {
                    any_modified = true;
                }
            }
        } else if path.exists() {
            if show_progress {
                eprint!("\rChecking: {}", path.display());
                let _ = std::io::stderr().flush();
            }
            if format_file(path, mode, &config) {
                any_modified = true;
            }
        } else {
            match glob(&target) {
                Ok(paths) => {
                    for entry in paths.flatten() {
                        if should_skip_path(&entry, &config) {
                            continue;
                        }
                        if show_progress {
                            eprint!("\rChecking: {}", entry.display());
                            let _ = std::io::stderr().flush();
                        }
                        if format_file(&entry, mode, &config) {
                            any_modified = true;
                        }
                    }
                }
                Err(e) => eprintln!("Invalid glob {}: {}", target, e),
            }
        }
    }

    // Clear the progress line
    if show_progress {
        eprint!("\r\x1b[K");
        let _ = std::io::stderr().flush();
    }

    if mode != Mode::Write && any_modified {
        std::process::exit(1);
}
}
