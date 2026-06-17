//! Native JS/TS formatter using SWC (no subprocess needed).
//!
//! Parses JS/TS code into an AST with swc_ecma_parser and prints it back
//! with swc_ecma_codegen for clean, properly-indented output.

use swc_core::common::comments::SingleThreadedComments;
use swc_core::common::input::StringInput;
use swc_core::common::sync::Lrc;
use swc_core::common::{FileName, SourceMap};
use swc_core::ecma::ast::{EsVersion, Module};
use swc_core::ecma::codegen::text_writer::JsWriter;
use swc_core::ecma::codegen::{Config as CodegenConfig, Emitter};
use swc_core::ecma::parser::lexer::Lexer;
use swc_core::ecma::parser::{Parser, Syntax, TsSyntax};

/// Format a JavaScript string with custom indent string.
/// Returns the formatted string, or the original if parsing fails.
pub(crate) fn format_js_with_indent(code: &str, indent: &str) -> String {
    if code.trim().is_empty() {
        return code.to_string();
    }

    let cm: Lrc<SourceMap> = Lrc::new(SourceMap::default());
    let fm = cm.new_source_file(FileName::Anon.into(), code.to_string());

    let comments = SingleThreadedComments::default();

    // Try TypeScript first, fall back to ES
    let module = parse_ts(&fm, &comments).or_else(|| parse_es(&fm, &comments));

    let module = match module {
        Some(m) => m,
        None => return code.to_string(), // Parse failure — return original
    };

    // Print the AST back to code
    let mut buf = vec![];
    {
        let wr = JsWriter::new(cm.clone(), "\n", &mut buf, None);
        let config = CodegenConfig::default();
        let mut emitter = Emitter {
            cfg: config,
            cm: cm.clone(),
            comments: Some(&comments),
            wr,
        };
        if emitter.emit_module(&module).is_err() {
            return code.to_string();
        }
    }

    let result = String::from_utf8_lossy(&buf).to_string();

    // Re-indent to use our preferred indent string
    reindent(&result, indent)
}

fn parse_ts(
    fm: &swc_core::common::SourceFile,
    comments: &SingleThreadedComments,
) -> Option<Module> {
    let syntax = Syntax::Typescript(TsSyntax {
        tsx: false,
        decorators: false,
        ..Default::default()
    });
    let input = StringInput::new(&fm.src, fm.start_pos, fm.end_pos);
    let lexer = Lexer::new(syntax, EsVersion::latest(), input, Some(comments));
    let mut parser = Parser::new_from(lexer);
    parser.parse_module().ok()
}

fn parse_es(
    fm: &swc_core::common::SourceFile,
    comments: &SingleThreadedComments,
) -> Option<Module> {
    let syntax = Syntax::Es(Default::default());
    let input = StringInput::new(&fm.src, fm.start_pos, fm.end_pos);
    let lexer = Lexer::new(syntax, EsVersion::latest(), input, Some(comments));
    let mut parser = Parser::new_from(lexer);
    parser.parse_module().ok()
}

/// Re-indent formatted code to use the target indent string.
/// Detects SWC's indentation width and converts to the target indent.
fn reindent(code: &str, target_indent: &str) -> String {
    let indent_width = detect_indent_width(code);

    let mut out = String::with_capacity(code.len());
    for line in code.lines() {
        let trimmed = line.trim_start();
        if trimmed.is_empty() {
            out.push('\n');
            continue;
        }
        let leading_spaces = line.len() - trimmed.len();
        let indent_levels = if indent_width > 0 {
            leading_spaces / indent_width
        } else {
            0
        };
        for _ in 0..indent_levels {
            out.push_str(target_indent);
        }
        out.push_str(trimmed);
        out.push('\n');
    }
    if !code.ends_with('\n') && out.ends_with('\n') {
        out.pop();
    }
    out
}

/// Detect the indentation width used in code.
/// Uses the minimum non-zero indentation across all indented lines.
fn detect_indent_width(code: &str) -> usize {
    let mut min_indent = usize::MAX;
    for line in code.lines() {
        let trimmed = line.trim_start();
        if trimmed.is_empty() || trimmed == line {
            continue;
        }
        let leading = line.len() - trimmed.len();
        if leading > 0 && leading < min_indent {
            min_indent = leading;
        }
    }
    if min_indent == usize::MAX { 2 } else { min_indent }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_simple_js() {
        let input = "const x=1;const y=2;";
        let result = format_js_with_indent(input, "\t");
        assert!(result.contains("const x = 1;"));
        assert!(result.contains("const y = 2;"));
    }

    #[test]
    fn format_function() {
        let input = "function hello(name){return 'hello '+name;}";
        let result = format_js_with_indent(input, "\t");
        assert!(result.contains("function hello("));
        assert!(result.contains("return"));
    }

    #[test]
    fn format_empty_returns_original() {
        let input = "";
        assert_eq!(format_js_with_indent(input, "\t"), "");
    }

    #[test]
    fn format_invalid_js_returns_original() {
        let input = "this is not valid js {{{{";
        assert_eq!(format_js_with_indent(input, "\t"), input);
    }

    #[test]
    fn detect_indent_width_works() {
        assert_eq!(detect_indent_width("  foo\n    bar"), 2);
        assert_eq!(detect_indent_width("    foo\n      bar"), 4);
        assert_eq!(detect_indent_width("no indent"), 2); // default
    }
}
