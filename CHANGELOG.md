## 2026-07-01

- Fixed `//` inside string/template/regex literals (e.g. `import(`file://…`)`, `fetch(`https://…`)`, `["a//b"]`) being misread as a line comment, which forced call/array/object/`new` arguments to wrap. Line-comment detection now skips string content; real `//` comments and masked block-comment placeholders still force expansion.
- Fixed `<script>`/`<style>`/`<pre>` raw blocks splitting JS/CSS at every `<`. Now only `<!--` is a split point, so `<` in comparisons, generics, `<pre>` inside `//` comments, and `<svg>` inside template literals is preserved instead of fragmenting the JS.
- Fixed block comments being dropped when the closing `*/` shares a line with code (e.g. `*/ export function f()`). Such comments are now masked and restored; the trailing code moves to its own line.
- A lone arrow/function call argument now always hugs (`fn(() => {` … `})`) even when the body must break (e.g. it contains a block comment). Object/array literal hugging stays behind `hugCallArgs`.
- `release.sh` and `release.ps1`: after local install, remove stale `~/.cargo/bin/reefmt` (macOS/Linux) / `~\.cargo\bin\reefmt.exe` (Windows) if present, to prevent wrong-version conflicts.
- Added `hugCallArgs` config option (default off). When a single-argument call that must wrap holds an object, array, arrow, or function literal, the braces hug the callee (`fn({` … `})` / `fn((x) => {` … `})`) instead of expanding `(` onto its own line.
- Hugging applies even when the argument body contains `//` line comments — the arg re-prints multi-line, so comments stay on their own lines.

## 2026-06-30

- Fixed trailing inline comments being dropped from function call and `new` expression arguments (`// directory type` pattern). Both the inline trial and expanded form of `Expr::Call` / `Expr::New` now emit trailing comments; calls with `//` comments force expansion to multi-line.
