## 2026-07-01

- Added `hugCallArgs` config option (default off). When a single-argument call that must wrap holds an object, array, arrow, or function literal, the braces hug the callee (`fn({` … `})` / `fn((x) => {` … `})`) instead of expanding `(` onto its own line.
- Hugging applies even when the argument body contains `//` line comments — the arg re-prints multi-line, so comments stay on their own lines.

## 2026-06-30

- Fixed trailing inline comments being dropped from function call and `new` expression arguments (`// directory type` pattern). Both the inline trial and expanded form of `Expr::Call` / `Expr::New` now emit trailing comments; calls with `//` comments force expansion to multi-line.
