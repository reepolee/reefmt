//! Custom AST printer for TypeScript/JavaScript that outputs directly from SWC's AST.
//! Uses SWC for parsing, then walks the AST with a custom printer that handles
//! spacing, indentation correctly from the start.

mod stmt;
mod expr;
mod decl;
mod lit;
mod prop;
mod pat;
mod types;

use swc_core::ecma::ast::*;
use swc_core::common::comments::{SingleThreadedComments, Comments};
use swc_core::common::input::StringInput;
use swc_core::common::sync::Lrc;
use swc_core::common::{FileName, SourceMap, Spanned, BytePos};
use swc_core::ecma::parser::lexer::Lexer;
use swc_core::ecma::parser::{Parser, Syntax, TsSyntax};
use swc_core::ecma::ast::EsVersion;

pub(crate) struct Printer<'a> {
    buf: String,
    indent_level: usize,
    indent_str: String,
    comments: &'a SingleThreadedComments,
    wrap_width: usize,
    collapse_blocks: bool,
    max_members: usize,
}

impl<'a> Printer<'a> {
    pub fn new(indent_str: &str, wrap_width: usize, collapse_blocks: bool, max_members: usize, comments: &'a SingleThreadedComments) -> Self {
        Self {
            buf: String::with_capacity(4096),
            indent_level: 0,
            indent_str: indent_str.to_string(),
            comments,
            wrap_width,
            collapse_blocks,
            max_members,
        }
    }

    pub fn print_module(mut self, module: &Module) -> String {
        for item in &module.body {
            self.print_module_item(item);
        }
        self.buf
    }

    // ─── helpers ────────────────────────────────────────────

    pub(super) fn w(&mut self, s: &str) { self.buf.push_str(s); }
    pub(super) fn nl(&mut self) { self.buf.push('\n'); }
    pub(super) fn indent(&mut self) { self.indent_level += 1; }
    pub(super) fn dedent(&mut self) { self.indent_level = self.indent_level.saturating_sub(1); }
    pub(super) fn wi(&mut self) {
        for _ in 0..self.indent_level {
            self.buf.push_str(&self.indent_str);
        }
    }

    /// Emit leading comments for a node at the given byte position.
    /// These are `// __REEFMT_*` placeholder lines that the preprocess step
    /// created to preserve block comments and blank lines through SWC formatting.
    pub(super) fn emit_leading_comments(&mut self, pos: BytePos) {
        if let Some(comments) = self.comments.get_leading(pos) {
            for c in &comments {
                if c.kind == swc_core::common::comments::CommentKind::Line {
                    self.wi();
                    self.w("//");
                    self.w(&c.text);
                    self.nl();
                }
            }
        }
    }

    /// Emit trailing comments (inline) for a node at the given byte position.
    /// Used for block comments that appear inline after statements, like
    /// `const x = 1; /* inline */`.
    /// Measure the current line length (chars emitted since last \n).
    pub(super) fn current_line_len(&self) -> usize {
        self.buf.lines().last().map(|l| l.len()).unwrap_or(0)
    }

    pub(super) fn emit_trailing_comments(&mut self, pos: BytePos) {
        if let Some(comments) = self.comments.get_trailing(pos) {
            for c in &comments {
                if c.kind == swc_core::common::comments::CommentKind::Block {
                    self.w(" /*");
                    self.w(&c.text);
                    self.w("*/");
                }
            }
        }
    }

    // ─── module items ───────────────────────────────────────

    fn print_module_item(&mut self, item: &ModuleItem) {
        let pos = item.span().lo;
        self.emit_leading_comments(pos);
        match item {
            ModuleItem::ModuleDecl(d) => self.print_module_decl(d),
            ModuleItem::Stmt(s) => self.print_stmt(s),
        }
        // Emit trailing comments inline (e.g. `const x = 1; /* inline */`)
        // Strip the trailing \n added by the print_* function, emit comment, then re-add \n
        if self.buf.ends_with('\n') {
            self.buf.pop();
        }
        self.emit_trailing_comments(item.span().hi);
        self.nl();
    }

    fn print_module_decl(&mut self, decl: &ModuleDecl) {
        match decl {
            ModuleDecl::Import(d) => {
                self.w("import ");
                if d.type_only { self.w("type "); }

                let has_default = d.specifiers.iter().any(|s| matches!(s, ImportSpecifier::Default(_)));
                let named: Vec<_> = d.specifiers.iter().filter(|s| matches!(s, ImportSpecifier::Named(_))).collect();
                let has_ns = d.specifiers.iter().any(|s| matches!(s, ImportSpecifier::Namespace(_)));

                // Default import
                if has_default {
                    for s in &d.specifiers {
                        if let ImportSpecifier::Default(def) = s {
                            self.w(&*def.local.sym);
                        }
                    }
                    if !named.is_empty() || has_ns {
                        self.w(", ");
                    }
                }

                // Namespace import
                if has_ns {
                    for s in &d.specifiers {
                        if let ImportSpecifier::Namespace(ns) = s {
                            self.w("* as ");
                            self.w(&*ns.local.sym);
                        }
                    }
                    if !named.is_empty() {
                        self.w(", ");
                    }
                }

                // Named imports
                if !named.is_empty() {
                    self.w("{ ");
                    for (i, n) in named.iter().enumerate() {
                        if i > 0 { self.w(", "); }
                        if let ImportSpecifier::Named(named_spec) = n {
                            self.w(&*named_spec.local.sym);
                        }
                    }
                    self.w(" }");
                }

                self.w(" from \"");
                self.w(d.src.value.as_str().unwrap());
                self.w("\";");
                self.nl();
            }
            ModuleDecl::ExportDecl(d) => {
                self.w("export ");
                self.print_decl(&d.decl);
            }
            ModuleDecl::ExportNamed(n) => {
                self.w("export ");
                if n.type_only { self.w("type "); }
                self.w("{ ");
                for (i, s) in n.specifiers.iter().enumerate() {
                    if i > 0 { self.w(", "); }
                    match s {
                        ExportSpecifier::Named(ns) => {
                            match &ns.orig {
                                ModuleExportName::Ident(id) => self.w(&*id.sym),
                                ModuleExportName::Str(ss) => self.w(ss.value.as_str().unwrap()),
                            }
                        }
                        ExportSpecifier::Default(ds) => self.w(&*ds.exported.sym),
                        ExportSpecifier::Namespace(ns) => {
                            match &ns.name {
                                ModuleExportName::Ident(id) => self.w(&*id.sym),
                                ModuleExportName::Str(ss) => self.w(ss.value.as_str().unwrap()),
                            }
                        }
                    }
                }
                self.w(" }");
                if let Some(src) = &n.src {
                    self.w(" from \"");
                    self.w(src.value.as_str().unwrap());
                    self.w("\"");
                }
                self.w(";");
                self.nl();
            }
            ModuleDecl::ExportDefaultExpr(d) => {
                self.w("export default ");
                self.print_expr(&d.expr);
                self.w(";");
                self.nl();
            }
            ModuleDecl::ExportDefaultDecl(d) => {
                self.w("export default ");
                match &d.decl {
                    DefaultDecl::Fn(f) => {
                        if f.function.is_async { self.w("async "); }
                        if let Some(id) = &f.ident {
                            self.w("function ");
                            self.w(&*id.sym);
                        } else {
                            self.w("function");
                        }
                        self.print_fn_sig(&f.function);
                        if let Some(body) = &f.function.body {
                            self.w(" ");
                            self.print_block(body);
                        } else {
                            self.w(";");
                            self.nl();
                        }
                    }
                    DefaultDecl::Class(_c) => { self.w("class {}"); self.nl(); }
                    DefaultDecl::TsInterfaceDecl(_i) => { self.w("interface {}"); self.nl(); }
                }
            }
            ModuleDecl::ExportAll(e) => {
                self.w("export * from \"");
                self.w(e.src.value.as_str().unwrap());
                self.w("\";");
                self.nl();
            }
            _ => { self.w("// unhandled module decl\n"); }
        }
    }
}

// ─── public entry point ──────────────────────────────────────

pub(crate) fn format_js_with_printer(
    code: &str,
    indent: &str,
    wrap_width: usize,
    collapse_blocks: bool,
    max_members: usize,
    _remove_unused: bool,
) -> String {
    if code.trim().is_empty() {
        return code.to_string();
    }

    let cm: Lrc<SourceMap> = Lrc::new(SourceMap::default());
    let fm = cm.new_source_file(FileName::Anon.into(), code.to_string());
    let comments = SingleThreadedComments::default();

    let module = parse_ts(&fm, &comments).or_else(|| parse_es(&fm, &comments));
    let module = match module {
        Some(m) => m,
        None => return code.to_string(),
    };

    let printer = Printer::new(indent, wrap_width, collapse_blocks, max_members, &comments);
    printer.print_module(&module)
}

fn parse_ts(fm: &swc_core::common::SourceFile, comments: &SingleThreadedComments) -> Option<Module> {
    let syntax = Syntax::Typescript(TsSyntax { tsx: false, decorators: false, ..Default::default() });
    let input = StringInput::new(&fm.src, fm.start_pos, fm.end_pos);
    let lexer = Lexer::new(syntax, EsVersion::latest(), input, Some(comments));
    let mut parser = Parser::new_from(lexer);
    parser.parse_module().ok()
}

fn parse_es(fm: &swc_core::common::SourceFile, comments: &SingleThreadedComments) -> Option<Module> {
    let syntax = Syntax::Es(Default::default());
    let input = StringInput::new(&fm.src, fm.start_pos, fm.end_pos);
    let lexer = Lexer::new(syntax, EsVersion::latest(), input, Some(comments));
    let mut parser = Parser::new_from(lexer);
    parser.parse_module().ok()
}
