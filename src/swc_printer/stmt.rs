use swc_core::ecma::ast::*;
use swc_core::common::{Spanned, BytePos};
use swc_core::common::comments::{Comments, CommentKind};
use super::Printer;

impl<'a> Printer<'a> {
    /// Print a statement in inline/sub-statement position (e.g. `if`/`while`/`for`
    /// body). Emits leading comments but no leading indentation and no trailing
    /// newline — the caller controls separators.
    pub(super) fn print_stmt(&mut self, stmt: &Stmt) {
        self.emit_leading_comments(stmt.span().lo());
        self.print_stmt_body(stmt);
    }

    /// Print a statement's body only: no leading comments, no indentation, no
    /// trailing newline. Block-position callers (`print_block`, module items,
    /// switch cases) handle leading comments + `wi()` + trailing newline via
    /// `print_stmt_in_block`.
    pub(super) fn print_stmt_body(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Block(b) => { self.print_block(b); }
            Stmt::Empty(_) => { self.w(";"); }
            Stmt::Debugger(_) => { self.w("debugger;"); }
            Stmt::With(w) => {
                self.w("with (");
                self.print_expr(&w.obj);
                self.w(") ");
                self.print_stmt(&w.body);
            }
            Stmt::Return(r) => {
                self.w("return");
                if let Some(arg) = &r.arg { self.w(" "); self.print_expr(arg); }
                self.w(";");
            }
            Stmt::If(i) => {
                self.w("if (");
                self.print_expr(&i.test);
                self.w(") ");
                // Force-expand the cons block when there is an else clause so the
                // `else` keyword always lands on its own `} else` line rather than
                // being appended to a collapsed one-liner.
                if i.alt.is_some() {
                    match i.cons.as_ref() {
                        Stmt::Block(b) => self.print_block_expanded(b),
                        _ => self.print_stmt(&i.cons),
                    }
                } else {
                    self.print_stmt(&i.cons);
                }
                if let Some(alt) = &i.alt {
                    self.w(" else ");
                    // Force-expand a block-form else body for consistency with
                    // the expanded if/else-if blocks above it.
                    match alt.as_ref() {
                        Stmt::Block(b) => self.print_block_expanded(b),
                        _ => self.print_stmt(alt),
                    }
                }
            }
            Stmt::While(w) => {
                self.w("while (");
                self.print_expr(&w.test);
                self.w(") ");
                self.print_stmt(&w.body);
            }
            Stmt::DoWhile(d) => {
                self.w("do ");
                self.print_stmt(&d.body);
                self.w(" while (");
                self.print_expr(&d.test);
                self.w(");");
            }
            Stmt::For(f) => {
                self.w("for (");
                if let Some(init) = &f.init {
                    match init {
                        VarDeclOrExpr::VarDecl(v) => self.print_var_decl(v, false),
                        VarDeclOrExpr::Expr(e) => self.print_expr(e),
                    }
                }
                self.w("; ");
                if let Some(test) = &f.test { self.print_expr(test); }
                self.w("; ");
                if let Some(update) = &f.update { self.print_expr(update); }
                self.w(") ");
                self.print_stmt(&f.body);
            }
            Stmt::ForIn(fi) => {
                self.w("for (");
                self.print_for_head(&fi.left);
                self.w(" in ");
                self.print_expr(&fi.right);
                self.w(") ");
                self.print_stmt(&fi.body);
            }
            Stmt::ForOf(fo) => {
                self.w("for ");
                if fo.is_await { self.w("await "); }
                self.w("(");
                self.print_for_head(&fo.left);
                self.w(" of ");
                self.print_expr(&fo.right);
                self.w(") ");
                self.print_stmt(&fo.body);
            }
            Stmt::Try(t) => {
                self.w("try ");
                self.print_block(&t.block);
                if let Some(catch) = &t.handler {
                    self.w(" catch");
                    if let Some(param) = &catch.param {
                        self.w(" (");
                        self.print_pat(param);
                        self.w(")");
                    }
                    self.w(" ");
                    // Always expand catch/finally bodies: collapsing them merges
                    // the block with the following `finally`/`catch` keyword on
                    // the same line, producing unreadable one-liners.
                    self.print_block_expanded(&catch.body);
                }
                if let Some(finalizer) = &t.finalizer {
                    self.w(" finally ");
                    self.print_block_expanded(finalizer);
                }
            }
            Stmt::Switch(s) => {
                self.w("switch (");
                self.print_expr(&s.discriminant);
                self.w(") {");
                self.nl();
                self.indent();
                for case in &s.cases {
                    self.wi();
                    if let Some(test) = &case.test {
                        self.w("case ");
                        self.print_expr(test);
                        self.w(":");
                    } else {
                        self.w("default:");
                    }
                    self.nl();
                    self.indent();
                    for s in &case.cons {
                        self.print_stmt_in_block(s);
                    }
                    self.dedent();
                }
                self.dedent();
                self.wi();
                self.w("}");
            }
            Stmt::Throw(t) => { self.w("throw "); self.print_expr(&t.arg); self.w(";"); }
            Stmt::Decl(d) => self.print_decl(d),
            Stmt::Expr(e) => { self.print_expr(&e.expr); self.w(";"); }
            Stmt::Break(b) => {
                self.w("break");
                if let Some(label) = &b.label { self.w(" "); self.w(&*label.sym); }
                self.w(";");
            }
            Stmt::Continue(c) => {
                self.w("continue");
                if let Some(label) = &c.label { self.w(" "); self.w(&*label.sym); }
                self.w(";");
            }
            Stmt::Labeled(l) => {
                self.w(&*l.label.sym);
                self.w(": ");
                self.print_stmt(&l.body);
            }
        }
    }

    /// Print a statement in block position: leading comments (indented), `wi()`,
    /// body, trailing comments, then a trailing newline. Used by `print_block`
    /// and switch case bodies.
    pub(super) fn print_stmt_in_block(&mut self, stmt: &Stmt) {
        self.emit_leading_comments(stmt.span().lo());
        self.wi();
        self.print_stmt_body(stmt);
        if self.buf.ends_with('\n') {
            self.buf.pop();
        }
        self.emit_trailing_comments(stmt.span().hi());
        self.nl();
    }

    pub(super) fn print_for_head(&mut self, left: &ForHead) {
        match left {
            ForHead::VarDecl(v) => self.print_var_decl(v, false),
            ForHead::Pat(p) => self.print_pat(p),
            ForHead::UsingDecl(_) => self.w("using _"),
        }
    }

    pub(super) fn print_block(&mut self, block: &BlockStmt) {
        // Try single-statement collapse: `if (cond) { stmt; }` on one line
        if self.collapse.enabled && block.stmts.len() == 1 {
            let stmt = &block.stmts[0];
            let has_leading = self
                .comments
                .get_leading(stmt.span().lo())
                .map_or(false, |c| !c.is_empty());
            let has_closing = self
                .comments
                .get_leading(block.span.hi() - BytePos(1))
                .map_or(false, |c| !c.is_empty());
            // Don't collapse blocks containing blocks (would look weird),
            // statements that carry leading comments, blocks with trailing
            // comments before the closing brace, or statements with their own
            // trailing line comment (would be dropped in the collapsed form).
            let has_trailing = self.has_trailing_line_comment(stmt.span().hi());
            let should_collapse = !matches!(stmt, Stmt::Block(_)) && !has_leading && !has_closing && !has_trailing;
            if should_collapse {
                let checkpoint = self.buf.len();
                self.w("{ ");
                self.print_stmt_body(stmt);
                self.w(" }");
                // Accept the collapse only when the trial output is genuinely
                // single-line (no embedded newline) and fits the wrap width.
                let added = &self.buf[checkpoint..];
                if !added.contains('\n') && self.current_line_len() <= self.wrap_width {
                    return;
                }
                // Rollback to expanded form
                self.buf.truncate(checkpoint);
            }
        }
        self.print_block_expanded(block);
    }

    /// Print a block always in expanded form — skips the single-statement
    /// collapse check. Used for catch/finally bodies so they always open on
    /// their own line and are never merged with following `finally`/`catch`.
    pub(super) fn print_block_expanded(&mut self, block: &BlockStmt) {
        if block.stmts.is_empty() {
            // Check for comments before the closing brace (attached as leading
            // comments on the `}` token position, which is hi() - 1).
            let closing_pos = block.span.hi() - BytePos(1);
            let has_inner = self.comments.get_leading(closing_pos)
                .map_or(false, |c| c.iter().any(|c| c.kind == CommentKind::Line));
            if has_inner {
                self.w("{");
                self.nl();
                self.indent();
                self.emit_leading_comments(closing_pos);
                self.dedent();
                self.wi();
                self.w("}");
            } else {
                self.w("{}");
            }
            return;
        }
        self.w("{");
        self.nl();
        self.indent();
        for s in &block.stmts {
            self.print_stmt_in_block(s);
        }
        self.emit_leading_comments(block.span.hi() - BytePos(1));
        self.dedent();
        self.wi();
        self.w("}");
    }
}
