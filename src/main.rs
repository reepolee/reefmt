use glob::glob;
use std::{
    env, fs,
    io::Write,
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

/// Run biome lint --fix on the protected JS/CSS fragment (via temp file)
/// to apply refactoring rules (e.g. prefer-template). Falls back to
/// returning the source unchanged if biome is not installed.
fn run_biome_lint(src: &str, ext: &str, timestamp: u128) -> String {
    let dir = env::temp_dir().join(format!("reefmt_biome_{}", timestamp));
    if fs::create_dir_all(&dir).is_err() {
        return src.to_string();
    }

    let tmp_path = dir.join(format!("input.{}", ext));
    let config_path = dir.join("biome.json");

    let biome_config = r#"{
  "formatter": {
    "enabled": false
  },
  "linter": {
    "enabled": true,
    "rules": {
      "preset": "recommended",
      "correctness": {
        "noUnusedImports": "error"
      },
      "style": {
        "useTemplate": "on"
      }
    }
  },
  "assist": {
    "actions": {
      "source": {
        "organizeImports": "on"
      }
    }
  }
}"#;

    if fs::write(&config_path, biome_config).is_err()
        || fs::write(&tmp_path, src).is_err()
    {
        let _ = fs::remove_dir_all(&dir);
        return src.to_string();
    }

    // Biome discovers config from the CWD, not via --config.
    // Set current_dir to the temp dir so it finds the biome.json we wrote there.
    let filename = format!("input.{}", ext);
    let result = Command::new("biome")
        .current_dir(&dir)
        .args([
            "lint",
            "--write",
            "--unsafe",
            &filename,
        ])
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

    // Step 2: Pipe through dprint for formatting
    let config_path = env::temp_dir()
        .join(format!("reefmt_dprint_config_{}.json", timestamp))
        .to_string_lossy()
        .into_owned();

    let dprint_config = r#"{
  "incremental": true,
  "lineWidth": 120,
  "indentWidth": 4,
  "useTabs": true,
  "typescript": {
    "quoteStyle": "preferDouble",
    "semiColons": "always",
    "trailingCommas": "onlyMultiLine"
  },
  "json": {
    "indentWidth": 4,
    "useTabs": true
  },
  "markdown": {
    "lineWidth": 120
  },
  "plugins": [
    "https://plugins.dprint.dev/typescript-0.90.3.wasm",
    "https://plugins.dprint.dev/json-0.19.0.wasm",
    "https://plugins.dprint.dev/markdown-0.17.8.wasm",
    "https://plugins.dprint.dev/g-plane/malva-v0.16.0.wasm"
  ]
}"#;

    if fs::write(&config_path, dprint_config).is_err() {
        return indent_code(src);
    }

    let mut child = match Command::new("dprint")
        .args(["fmt", "--stdin", &format!("file.{}", ext), "--config", &config_path])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(c) => c,
        Err(_) => {
            let _ = fs::remove_file(&config_path);
            return indent_code(src);
        }
    };

    // Write lint-fixed content to stdin and close the pipe so dprint can start
    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(after_lint.as_bytes());
    }

    let formatted = match child.wait_with_output() {
        Ok(output) if output.status.success() => {
            let content = String::from_utf8_lossy(&output.stdout);
            let restored = restore(&content);
            let restored = restore_html_comments(&restored, &html_comments);
            indent_code(&restored)
        }
        _ => indent_code(src),
    };

    let _ = fs::remove_file(&config_path);

    formatted
        .trim()
        .lines()
        .map(|l| format!("\t{}", l))
        .collect::<Vec<_>>()
        .join("\n")
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

fn is_opening_html_tag(line: &str) -> bool {
    line.starts_with('<')
        && !line.starts_with("</")
        && !line.starts_with("<!--")
        && !line.starts_with("<!")
        && line.contains('>')
}

fn is_self_closed(line: &str) -> bool {
    line.trim_end().ends_with("/>")
}

fn extract_tag_name(line: &str) -> String {
    let start = if line.starts_with('<') { 1 } else { 0 };
    let rest = &line[start..];
    rest.split(|c: char| c.is_whitespace() || c == '>' || c == '/')
        .next()
        .unwrap_or("")
        .to_lowercase()
}

fn has_inline_close(line: &str, tag_name: &str) -> bool {
    line.contains(&format!("</{}>", tag_name))
}

fn leading_tabs(s: &str) -> String {
    s.chars().take_while(|&c| c == '\t').collect()
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

fn is_ree_line(line: &str) -> bool {
    line.starts_with('{')
}

fn compute_html_tag_delta(line: &str, void_tags: &[&str]) -> (isize, isize) {
    let mut close_before = 0isize;
    let mut open_after = 0isize;
    let mut pos = 0;

    while let Some(open_pos) = line[pos..].find('<') {
        let abs_pos = pos + open_pos;
        let rest = &line[abs_pos..];

        if rest.starts_with("</") {
            let tag_start = abs_pos + 2;
            let tag_name_end = line[tag_start..]
                .find(|c: char| c.is_whitespace() || c == '>')
                .map(|p| tag_start + p)
                .unwrap_or(line.len());
            let tag_name = &line[tag_start..tag_name_end];
            if !void_tags.contains(&tag_name) {
                close_before += 1;
            }
            if let Some(close) = rest.find('>') {
                pos = abs_pos + close + 1;
            } else {
                break;
            }
            continue;
        }

        if rest.starts_with("<!--") {
            // HTML comment - find the closing --> marker
            if let Some(close) = rest.find("-->") {
                pos = abs_pos + close + 3;
            } else {
                break;
            }
            continue;
        }

        if rest.starts_with("<!") || rest.starts_with("<?") {
            // Doctype, CDATA, processing instructions - find the first >
            if let Some(close) = rest.find('>') {
                pos = abs_pos + close + 1;
            } else {
                break;
            }
            continue;
        }

        let tag_start = abs_pos + 1;
        let tag_name_end = line[tag_start..]
            .find(|c: char| c.is_whitespace() || c == '>' || c == '/')
            .map(|p| tag_start + p)
            .unwrap_or(line.len());
        let tag_name = &line[tag_start..tag_name_end];

        if let Some(close) = rest.find('>') {
            let tag_len = close + 1;
            let is_self_closed = rest[..close].ends_with('/');

            let close_tag = format!("</{}>", tag_name);
            let after_tag = &line[abs_pos + tag_len..];
            let has_inline = after_tag.contains(&close_tag);

            if !void_tags.contains(&tag_name) && !is_self_closed && !has_inline {
                open_after += 1;
            }

            if has_inline {
                close_before -= 1;
            }

            pos = abs_pos + tag_len;
        } else {
            break;
        }
    }

    (close_before, open_after)
}

fn normalize_inline_spacing(line: &str) -> String {
    let mut out = String::new();
    let chars: Vec<char> = line.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        if chars[i] == '>' {
            out.push('>');
            i += 1;
            if let Some(n) = chars[i..].iter().position(|&c| !c.is_whitespace()) {
                if chars[i + n] == '{' {
                    i += n;
                }
            }
        } else if chars[i] == '}' {
            out.push('}');
            i += 1;
            if let Some(n) = chars[i..].iter().position(|&c| !c.is_whitespace()) {
                if chars[i + n] == '<' {
                    i += n;
                }
            }
        } else {
            out.push(chars[i]);
            i += 1;
        }
    }

    out
}

fn format_html(src: &str) -> String {
    let void_tags = [
        "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param",
        "source", "track", "wbr",
    ];

    let mut out = String::new();
    let mut depth: usize = 0;
    let mut in_comment: bool = false;

    for line in src.lines() {
        let trimmed = line.trim();

        if trimmed.is_empty() {
            out.push('\n');
            continue;
        }

        // Track whether we're inside a multiline HTML comment.
        // Ree template directives inside comments should not affect depth.
        if trimmed.contains("<!--") && !trimmed.contains("-->") {
            in_comment = true;
        }
        if trimmed.contains("-->") {
            in_comment = false;
        }

        // Count Ree open and close occurrences on this full line, so inline
        // usage like {#if cond}content{/if} doesn't mis-track depth.
        // Strip HTML comment content so Ree tags inside <!-- ... --> (both
        // single-line and multiline) are not counted. {#include} and
        // {#layout} don't match {#if}/{#each}/{#with} so they're safe.
        let count_source = if in_comment {
            String::new()
        } else {
            strip_html_comment_content(trimmed)
        };
        let (ree_opens, ree_closes) = if !count_source.is_empty() {
            let mut opens = 0usize;
            let mut closes = 0usize;
            let mut pos = 0;
            while let Some(idx) = count_source[pos..].find("{#if") {
                opens += 1;
                pos += idx + 4;
            }
            pos = 0;
            while let Some(idx) = count_source[pos..].find("{#each") {
                opens += 1;
                pos += idx + 6;
            }
            pos = 0;
            while let Some(idx) = count_source[pos..].find("{#with") {
                opens += 1;
                pos += idx + 6;
            }
            pos = 0;
            while let Some(idx) = count_source[pos..].find("{/if") {
                closes += 1;
                pos += idx + 4;
            }
            pos = 0;
            while let Some(idx) = count_source[pos..].find("{/each") {
                closes += 1;
                pos += idx + 6;
            }
            pos = 0;
            while let Some(idx) = count_source[pos..].find("{/with") {
                closes += 1;
                pos += idx + 6;
            }
            (opens, closes)
        } else {
            (0, 0)
        };

        // Adjust depth for net Ree open/close balance
        let ree_net_close = ree_closes.saturating_sub(ree_opens);
        let ree_net_open = ree_opens.saturating_sub(ree_closes);

        // Apply closes before writing
        for _ in 0..ree_net_close {
            if depth > 0 {
                depth -= 1;
            }
        }

        let is_else = !in_comment
            && (trimmed.starts_with("{:else") || trimmed.starts_with("{:"));
        if is_else && depth > 0 {
            depth -= 1;
        }

        if !is_ree_line(trimmed) {
            let (close_before, open_after) = compute_html_tag_delta(trimmed, &void_tags);

            let close_before = close_before.min(1);
            let open_after = open_after.min(1);

            if close_before > 0 {
                for _ in 0..close_before as usize {
                    if depth > 0 {
                        depth -= 1;
                    }
                }
            }

            let normalized = normalize_inline_spacing(trimmed);
            out.push_str(&"\t".repeat(depth));
            out.push_str(&normalized);
            out.push('\n');

            depth += open_after as usize;
        } else {
            let normalized = normalize_inline_spacing(trimmed);
            out.push_str(&"\t".repeat(depth));
            out.push_str(&normalized);
            out.push('\n');
        }

        // Apply net opens after writing
        depth += ree_net_open;

        if is_else {
            depth += 1;
        }
    }

    collapse_blank_lines(&out)
}

/// After format_html, collapse HTML tags that fit on one line.
/// Handles:
///   1. Multiline opening tags (attributes on separate lines) — merge to one line
///   2. <tag>\n  content\n</tag> — collapse to <tag>content</tag>
///   3. <tag>\n</tag> — collapse to <tag></tag>
fn collapse_fitting_tags(src: &str) -> String {
    const LINE_WIDTH: usize = 120;
    let lines: Vec<&str> = src.lines().collect();
    let len = lines.len();
    let mut out: Vec<String> = Vec::new();
    let mut i = 0;

    while i < len {
        let line_i = lines[i];
        let trimmed_i = line_i.trim();
        let indent_i = leading_tabs(line_i);

        // --- 1. Multiline opening/self-closing tag ---
        // If a line starts with '<' (but not '<!--' or '</') and doesn't
        // contain '>' or '/>', the tag continues on the next line.
        // Also handle: <input ... attr="..."
        //                     {#if cond}checked{/if} />
        if trimmed_i.starts_with('<') && !trimmed_i.starts_with("<!--")
            && !trimmed_i.starts_with("</") && !trimmed_i.contains('>')
        {
            let mut merged = trimmed_i.to_string();
            let mut j = i + 1;
            while j < len {
                let tj = lines[j].trim();
                merged.push(' ');
                merged.push_str(tj);
                j += 1;
                if tj.contains('>') || tj.ends_with("/>") {
                    break;
                }
            }
            // Only collapse if the merged line fits
            if merged.len() + indent_i.len() <= LINE_WIDTH {
                out.push(format!("{}{}", indent_i, merged));
                i = j;
                continue;
            }
            // Keep original lines
            for k in i..j.min(len) {
                out.push(lines[k].to_string());
            }
            i = j;
            continue;
        }

        // --- 2. Opening tag followed by content and closing tag ---
        // Pattern: <tag ...>\n  content\n</tag>
        // Only for Ree/non-HTML content that fits on one line.
        if is_opening_html_tag(trimmed_i) && !is_self_closed(trimmed_i) {
            let tag_name = extract_tag_name(trimmed_i);
            let close_str = format!("</{}>", tag_name);

            if !has_inline_close(trimmed_i, &tag_name) {
                // Skip blank lines after opening tag
                let mut j = i + 1;
                while j < len && lines[j].trim().is_empty() {
                    j += 1;
                }

                if j < len {
                    let body = lines[j].trim();
                    // Only collapse if body is a Ree expression or short text (not an HTML tag)
                    if !body.starts_with('<') {
                        // Skip blank lines after content
                        let mut k = j + 1;
                        while k < len && lines[k].trim().is_empty() {
                            k += 1;
                        }

                        if k < len && lines[k].trim() == close_str {
                            // Try collapsing: <tag>content</tag> (with space)
                            let collapsed = format!("{}{} {}{}", indent_i, trimmed_i, body, close_str);
                            if collapsed.len() <= LINE_WIDTH {
                                out.push(collapsed);
                                i = k + 1;
                                continue;
                            }
                            // Try without space: <tag>content</tag>
                            let collapsed_tight = format!("{}{}{}{}", indent_i, trimmed_i, body, close_str);
                            if collapsed_tight.len() <= LINE_WIDTH {
                                out.push(collapsed_tight);
                                i = k + 1;
                                continue;
                            }
                        }
                    }
                }

                // --- 3. Empty element: <tag>\n</tag> ---
                let mut j = i + 1;
                while j < len && lines[j].trim().is_empty() {
                    j += 1;
                }
                if j < len && lines[j].trim() == close_str {
                    let collapsed = format!("{}{}{}", indent_i, trimmed_i, close_str);
                    if collapsed.len() <= LINE_WIDTH {
                        out.push(collapsed);
                        i = j + 1;
                        continue;
                    }
                }
            }
        }

        out.push(line_i.to_string());
        i += 1;
    }

    out.join("\n")
}

fn format_file(path: &Path) {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error reading {}: {}", path.display(), e);
            return;
        }
    };

    // Normalize line endings so comparison with formatted output is fair
    let normalized = content.replace("\r\n", "\n");

    let result = normalize_ree_spacing(&normalized);
    let result = format_html(&result);
    let result = collapse_fitting_tags(&result);
    let result = replace_script_style(&result, "script", "js");
    let result = replace_script_style(&result, "style", "css");

    let write_content = if !result.is_empty() && !result.ends_with('\n') {
        format!("{}\n", result)
    } else {
        result
    };

    if write_content == normalized {
        return;
    }

    match fs::write(path, &write_content) {
        Ok(_) => println!("Formatted: {}", path.display()),
        Err(e) => eprintln!("Error writing {}: {}", path.display(), e),
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
}

fn collect_ree_files(dir: &Path, files: &mut Vec<PathBuf>) -> std::io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                collect_ree_files(&path, files)?;
            } else if path.extension().and_then(|s| s.to_str()) == Some("ree") {
                files.push(path);
            }
        }
    }
    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.len() == 1 && (args[0] == "-v" || args[0] == "--version") {
        println!("reefmt v{}", env!("CARGO_PKG_VERSION"));
        return;
    }

    let targets: Vec<String> = if args.is_empty() {
        vec![".".to_string()]
    } else {
        args
    };

    for target in targets {
        let path = Path::new(&target);

        if path.is_dir() {
            let mut files = Vec::new();
            if let Err(e) = collect_ree_files(path, &mut files) {
                eprintln!("Error reading directory {}: {}", target, e);
                continue;
            }
            for file in files {
                format_file(&file);
            }
        } else if path.exists() {
            format_file(path);
        } else {
            match glob(&target) {
                Ok(paths) => {
                    for entry in paths.flatten() {
                        format_file(&entry);
                    }
                }
                Err(e) => eprintln!("Invalid glob {}: {}", target, e),
            }
        }
    }
}
