## 2026-06-30

- Fixed trailing inline comments being dropped from function call and `new` expression arguments (`// directory type` pattern). Both the inline trial and expanded form of `Expr::Call` / `Expr::New` now emit trailing comments; calls with `//` comments force expansion to multi-line.
