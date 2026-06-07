use glob::glob;
use std::{
    env, fs,
    io::Write,
    path::{Path, PathBuf},
    process::Command,
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

fn biome_format(src: &str, lang: &str) -> String {
    let ext = if lang == "css" { "css" } else { "js" };
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let tmp_path = env::temp_dir()
        .join(format!("reefmt_{}.{}", timestamp, ext))
        .to_string_lossy()
        .into_owned();

    let protected = protect(src.trim());

    let mut tmp = fs::File::create(&tmp_path).expect("create tmp");
    tmp.write_all(protected.as_bytes()).expect("write tmp");
    drop(tmp);

    let output = Command::new("biome")
        .args(["format", "--indent-style=tab", "--write", &tmp_path])
        .output();

    let formatted = match output {
        Ok(_) => match fs::read_to_string(&tmp_path) {
            Ok(content) => indent_code(&restore(&content)),
            Err(_) => indent_code(src),
        },
        Err(_) => indent_code(src),
    };

    let _ = fs::remove_file(&tmp_path);

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
                            let formatted = biome_format(body, lang);
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

fn normalize_ree_spacing(src: &str) -> String {
    let mut out = src.to_string();
    for kw in &["if", "each", "with", "include", "layout"] {
        out = out.replace(&format!("{{# {}", kw), &format!("{{#{}", kw));
        out = out.replace(&format!("{{/ {}", kw), &format!("{{/{}", kw));
    }
    out.replace("{/ if}", "{/if}")
        .replace("{/ each}", "{/each}")
        .replace("{/ with}", "{/with}")
        .replace("{/ if }", "{/if}")
        .replace("{/ each }", "{/each}")
        .replace("{/ with }", "{/with}")
        .replace("{/if }", "{/if}")
        .replace("{/each }", "{/each}")
        .replace("{/with }", "{/with}")
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

fn was_inline_in_original(original: &str, open_tag: &str, body: &str, close_tag: &str) -> bool {
    let open_trimmed = open_tag.trim();
    let close_trimmed = close_tag.trim();
    let body_trimmed = body.trim();

    for line in original.lines() {
        let line = line.trim();
        if let Some(open_pos) = line.find(open_trimmed) {
            let after_open = &line[open_pos + open_trimmed.len()..];
            if let Some(body_pos) = after_open.find(body_trimmed) {
                let after_body = &after_open[body_pos + body_trimmed.len()..];
                if after_body.contains(close_trimmed) {
                    return true;
                }
            }
        }
    }
    false
}

fn is_ree_inline_only(s: &str) -> bool {
    let t = s.trim();
    (t.starts_with("{=") || t.starts_with("{~") || t.starts_with("{{"))
        && t.ends_with('}')
        && !t.contains('\n')
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

        if rest.starts_with("<!--") || rest.starts_with("<!") || rest.starts_with("<?") {
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

    for line in src.lines() {
        let trimmed = line.trim();

        if trimmed.is_empty() {
            out.push('\n');
            continue;
        }

        if trimmed.starts_with("{/if")
            || trimmed.starts_with("{/each")
            || trimmed.starts_with("{/with")
        {
            if depth > 0 {
                depth -= 1;
            }
        }

        let is_else = trimmed.starts_with("{:else") || trimmed.starts_with("{:");
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

        if (trimmed.starts_with("{#if")
            || trimmed.starts_with("{#each")
            || trimmed.starts_with("{#with"))
            && !trimmed.starts_with("{#include")
            && !trimmed.starts_with("{#layout")
        {
            depth += 1;
        }

        if is_else {
            depth += 1;
        }
    }

    collapse_blank_lines(&out)
}

fn collapse_inline_tags(src: &str, original: &str) -> String {
    let lines: Vec<&str> = src.lines().collect();
    let mut out: Vec<String> = Vec::new();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i].trim();

        if is_opening_html_tag(line) && !is_self_closed(line) {
            let tag_name = extract_tag_name(line);
            let close = format!("</{}>", tag_name);

            if !has_inline_close(line, &tag_name) {
                let mut j = i + 1;
                while j < lines.len() && lines[j].trim().is_empty() {
                    j += 1;
                }
                let mut k = j + 1;
                while k < lines.len() && lines[k].trim().is_empty() {
                    k += 1;
                }

                if j < lines.len()
                    && k < lines.len()
                    && is_ree_inline_only(lines[j].trim())
                    && lines[k].trim() == close
                {
                    if was_inline_in_original(original, line, lines[j].trim(), &close) {
                        let indent = leading_tabs(lines[i]);
                        out.push(format!("{}{}{}{}", indent, line, lines[j].trim(), close));
                        i = k + 1;
                        continue;
                    }
                }
            }
        }

        out.push(lines[i].to_string());
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

    let result = normalize_ree_spacing(&content);
    let result = format_html(&result);
    let result = collapse_inline_tags(&result, &content);
    let result = replace_script_style(&result, "script", "js");
    let result = replace_script_style(&result, "style", "css");

    let write_content = if !result.is_empty() && !result.ends_with('\n') {
        format!("{}\n", result)
    } else {
        result
    };

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
