use swc_core::ecma::ast::*;
use super::Printer;

impl<'a> Printer<'a> {
    pub(super) fn print_decl(&mut self, decl: &Decl) {
        match decl {
            Decl::Fn(f) => {
                if f.function.is_async { self.w("async "); }
                self.w("function ");
                self.w(&*f.ident.sym);
                self.print_fn_sig(&f.function);
                if let Some(body) = &f.function.body {
                    self.w(" ");
                    self.print_block(body);
                } else {
                    self.w(";");
                    self.nl();
                }
            }
            Decl::Var(v) => {
                match v.kind {
                    VarDeclKind::Var => self.w("var "),
                    VarDeclKind::Let => self.w("let "),
                    VarDeclKind::Const => self.w("const "),
                }
                for (i, d) in v.decls.iter().enumerate() {
                    if i > 0 { self.w(", "); }
                    self.print_var_declarator(d);
                }
                self.w(";");
                self.nl();
            }
            Decl::Class(c) => {
                self.w("class ");
                self.w(&*c.ident.sym);
                self.w(" {}");
                self.nl();
            }
            Decl::TsInterface(i) => {
                self.w("interface ");
                self.w(&*i.id.sym);
                if let Some(ext) = i.extends.first() {
                    self.w(" extends ");
                    self.print_expr(&ext.expr);
                }
                self.w(" {");
                self.nl();
                self.indent();
                for m in &i.body.body {
                    self.wi();
                    self.print_ts_member(m);
                }
                self.dedent();
                self.wi();
                self.w("}");
                self.nl();
            }
            Decl::TsTypeAlias(a) => {
                self.w("type ");
                self.w(&*a.id.sym);
                self.w(" = ");
                self.print_ts_type(&a.type_ann);
                self.w(";");
                self.nl();
            }
            Decl::TsEnum(e) => {
                self.w("enum ");
                self.w(&*e.id.sym);
                self.w(" {");
                self.nl();
                self.indent();
                for (i, m) in e.members.iter().enumerate() {
                    if i > 0 { self.w(","); self.nl(); }
                    self.wi();
                    match &m.id {
                        TsEnumMemberId::Ident(id) => self.w(&*id.sym),
                        TsEnumMemberId::Str(s) => { self.w("\""); self.w(s.value.as_str().unwrap()); self.w("\""); }
                    }
                    if let Some(init) = &m.init {
                        self.w(" = ");
                        self.print_expr(init);
                    }
                }
                self.nl();
                self.dedent();
                self.wi();
                self.w("}");
                self.nl();
            }
            Decl::TsModule(m) => {
                match &m.id {
                    TsModuleName::Ident(id) => { self.w("module "); self.w(&*id.sym); }
                    TsModuleName::Str(s) => { self.w("module \""); self.w(s.value.as_str().unwrap()); self.w("\""); }
                }
                self.w(" {");
                self.nl();
                self.indent();
                if let Some(body) = &m.body {
                    match body {
                        TsNamespaceBody::TsModuleBlock(block) => {
                            for item in &block.body { self.print_module_item(item); }
                        }
                        _ => {}
                    }
                }
                self.dedent();
                self.wi();
                self.w("}");
                self.nl();
            }
            Decl::Using(_) => { self.w("using _;"); self.nl(); }
        }
    }

    pub(super) fn print_var_decl(&mut self, v: &VarDecl, add_semi: bool) {
        match v.kind {
            VarDeclKind::Var => self.w("var "),
            VarDeclKind::Let => self.w("let "),
            VarDeclKind::Const => self.w("const "),
        }
        for (i, d) in v.decls.iter().enumerate() {
            if i > 0 { self.w(", "); }
            self.print_var_declarator(d);
        }
        if add_semi { self.w(";"); self.nl(); }
    }

    pub(super) fn print_var_declarator(&mut self, d: &VarDeclarator) {
        self.print_pat(&d.name);
        if let Some(init) = &d.init {
            self.w(" = ");
            self.print_expr(init);
        }
    }

    pub(super) fn print_fn_sig(&mut self, f: &Function) {
        self.w("(");
        for (i, p) in f.params.iter().enumerate() {
            if i > 0 { self.w(", "); }
            self.print_pat(&p.pat);
        }
        self.w(")");
        if let Some(ret) = &f.return_type {
            self.w(": ");
            self.print_ts_type(&ret.type_ann);
        }
    }
}
