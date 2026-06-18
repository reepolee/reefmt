# AST Printer Refactor — Wrap-up & Backlog

**Branch:** `ast-printer`
**Phase:** Analysis complete. Implementation paused — resume in next phase.
**Date:** 2026-06-18

## What was done this session

Ran the reefmt test loop (per `reefmt-test-loop` skill) against `test-files/originals/`
to validate the new custom AST printer in `src/swc_printer/` that replaces the old
SWC-codegen + `reindent` path (`src/swc_format.rs`).

- Built `target/release/reefmt.exe` (v0.7.28).
- Formatted all 9 originals (6 root + 3 in `schema/`) via `--stdin` to
  `test-files/output/` (gitignored, regenerable). All exit 0.
- Diffed originals vs formatted; verified idempotency (pass 1 vs pass 2);
  ran `cargo test`.
- **No source code was modified.** This was analysis only.

## Current state

- **Formatting:** all files parse/exit 0, but 6 of 7 `.ts` files are corrupted.
  Only `schema/table.generated.ts` is clean.
- **Idempotency:** 3 non-idempotent — `edit_handlers.ts`, `index.ts`, `schema/table.ts`
  (all files with leading comments).
- **cargo test:** 170 passed, **7 failed**.
- The `.ree` path (still uses old codegen for `<script>` blocks) is **unaffected** —
  regression is isolated to standalone `.ts`/`.js` via the new printer.

## Architecture (for context)

Wired at `src/format.rs:1934` (`format_code_content` → `format_js_with_printer`).
Pipeline: `flatten_concat` → `preprocess_for_swc` (block-comment + blank-line
placeholders) → custom printer (`src/swc_printer/`) → `postprocess_from_swc`
(restore placeholders). Printer files: `mod.rs`, `stmt.rs`, `expr.rs`, `decl.rs`,
`lit.rs`, `prop.rs`, `pat.rs`, `types.rs`.

## Assumptions made

- Target indent is `\t` (tabs), `wrapWidth=180`, `collapseMaxMembers=3`,
  `collapseSingleStatementBlocks=true`, `removeUnusedImports=false` (from `reefmt.jsonc`).
- The old `swc_format.rs` path is intended to be fully replaced by `swc_printer/`
  for standalone TS/JS (the `.ree` script-block path at `ree_format.rs:139` still
  calls the old `format_js_with_indent` — left as-is, not in scope).
- The now-unused `wrap_long_method_chains` / `wrap_long_function_params` /
  `classify_declaration` etc. in `format.rs` (4 build warnings) are expected to be
  re-wired into the new printer, not deleted.

---

# Backlog — bugs to fix (next phase)

Ordered by severity. Each lists file + root cause.

## P0 — produces invalid JS or silent data loss

### B1. String escapes not re-encoded
- **File:** `src/swc_printer/lit.rs` — `print_lit` `Lit::Str` uses decoded `s.value`
  without re-escaping.
- **Effect:** `"a\"b\nc"` → `"a"b` + raw newline + `c"` — **invalid JS**. `\t` → literal
  tab. Also affects `PropName::Str`, `TsLit::Str`, import source strings.
- **Not triggered by current test files** but will break any code with escaped strings.
- **Fix:** re-escape `"`, `\`, newlines, tabs, etc. when emitting. (Consider using
  SWC's `JsWriter`/`EmitExpr` just for string literals, or a manual escaper.)

### B2. Class bodies dropped
- **File:** `src/swc_printer/decl.rs` — `Decl::Class` ignores `c.body`; also
  `expr.rs` `Expr::Class` and `mod.rs` `ExportDefaultDecl` `DefaultDecl::Class`.
- **Effect:** `class Foo { ...members... }` → `class Foo {}`. All members lost.
- **Fix:** implement class member printing (fields, methods, ctor, getters/setters,
  static, modifiers, decorators, `implements`/`extends`).

### B3. Spread `...` dropped
- **File:** `src/swc_printer/expr.rs` — `Expr::Call` ignores `a.spread`;
  `Expr::Array` ignores `e.spread`. Also check `Expr::Object` via `print_prop_or_spread`
  (that one may be OK since it matches `PropOrSpread::Spread`).
- **Effect:** `[...params, limit, offset]` → `[params, limit, offset]`;
  `push(...x)` → `push(x)`. **Semantic change.** Confirmed in `sql.ts`.
- **Fix:** emit `...` when `spread.is_some()` before the arg/element expr.

### B4. Optional call mangled
- **File:** `src/swc_printer/expr.rs` — `print_opt_chain_base` `OptChainBase::Call`.
- **Effect:** `x?.foo()` → `x?.foo?.()` (different runtime semantics).
  Confirmed in `index.ts`, `sql.ts`.
- **Fix:** the `?.` belongs on the callee/member access, not forced on the call.
  `a?.b.c()` should stay `a?.b.c()`. Only emit `?.(` when the call itself is optional
  in the AST (short-circuit). Re-check against SWC `OptCall` semantics.

## P1 — broken formatting / structural

### B5. No statement indentation
- **File:** `src/swc_printer/stmt.rs` — `print_block` never calls `wi()` before
  `print_stmt`; `print_stmt` never calls `wi()` itself.
- **Effect:** every statement inside a block prints at column 0. Why `table.ts`
  (top-level only) looked OK but `sql.ts`/`index.ts` collapsed to col 0.
- **Drives 3 test failures** (`ts_with_template_literal_*`, etc.).
- **Fix:** call `self.wi()` at the start of each statement in `print_block`'s loop
  (or at the top of `print_stmt`). Decide one canonical place to avoid double-indent.

### B6. `print_block` trailing newline breaks continuations
- **File:** `src/swc_printer/stmt.rs` — `print_block` always ends with `nl()`.
- **Effect:** in expression/continuation contexts the next token lands on a new line:
  stray `;` after arrow bodies (`validation_server.ts`), and `} catch` / `} else` /
  `} while` split across lines (`index.ts`, `sql.ts`).
- **Fix:** `print_block` should not emit a trailing `nl()` when used as a sub-statement
  (e.g. `if`/`while`/`for` body, `try` block, arrow body). Add a flag or split into
  `print_block_inline` (no trailing nl) vs `print_block_stmt` (with nl). The caller
  in `if`/`while`/`for`/`try`/`arrow` should use the no-trailing-nl variant and let
  the *parent* add the separator.

### B7. Over-aggressive collapse (width check only measures last line)
- **File:** `src/swc_printer/stmt.rs` `print_block`; `expr.rs` Object/Array;
  `types.rs` `TsTypeLit`.
- **Effect:** "fits" check uses `current_line_len()` = last line only, so multi-line
  nested content passes. `function f() { try { ... } }` and
  `render("form", { data: { ...multiline... }, ctx })` get half-collapsed.
- **Drives** `ts_complex_template_literal_preserved` failure.
- **Fix:** record `buf.len()` at the checkpoint and require the trial output to
  contain **no `\n`** (i.e. genuinely single-line) before accepting the collapse.
  Roll back otherwise.

### B8. Comments + blank lines duplicated
- **File:** `src/swc_printer/mod.rs` `print_module_item` AND `stmt.rs` `print_stmt`
  both call `emit_leading_comments` at the same span.
- **Effect:** every leading comment / blank-line placeholder emitted twice.
  Causes non-idempotency (compounds each pass) and
  `idempotent_format_code_content_non_ascii_comment` failure.
- **Fix:** emit leading comments in exactly one place. Either remove from
  `print_module_item` and rely on `print_stmt`/`print_module_decl`, or vice-versa.
  Ensure nested statements (inside blocks) still get their leading comments.

### B9. Optional param `?` dropped
- **File:** `src/swc_printer/pat.rs` `print_pat` `Pat::Ident`; `types.rs`
  `print_ts_fn_param` `TsFnParam::Ident`.
- **Effect:** `messages?: T` → `messages: T` (`validation_server.ts`).
- **Fix:** emit `?` when `i.optional` / `i.id.optional` is true.

### B10. Type annotations on destructuring patterns dropped
- **File:** `src/swc_printer/pat.rs` — `Pat::Array` and `Pat::Object` don't print
  `type_ann`.
- **Effect:** `([_, v]: [string, any]) =>` → `([_, v]) =>` (`index.ts`).
- **Fix:** `Pat::Array`/`Pat::Object` carry `type_ann: Option<TsTypeAnn>` — print
  `: <type>` when present.

### B11. Line wrapping no longer wired in
- **File:** `src/format.rs` `format_code_content` — new printer doesn't call
  `wrap_long_method_chains` / `wrap_long_function_params` (now unused → 4 build warnings).
- **Effect:** long lines no longer wrap. Drives `wrap_full_pipeline_multi_param_function`
  and `wrap_method_chain_full_pipeline` failures.
- **Fix:** re-wire the wrapping passes after `format_js_with_printer` (and after
  placeholder restore, or before — verify ordering against template literals).
  Note: these wrappers operate line-by-line on text, so they should work on printer
  output, but confirm tab-indent handling.

## P2 — minor / cosmetic

### B12. Empty object literal double-space
- **File:** `src/swc_printer/expr.rs` Object inline path.
- **Effect:** `{}` → `{  }`.
- **Fix:** special-case `o.props.is_empty()` → emit `{}`.

### B13. Single-ident arrow param parens dropped
- **File:** `src/swc_printer/expr.rs` `Expr::Arrow`.
- **Effect:** `(s) =>` → `s =>`. Valid JS, but a style change vs original.
- **Decision needed:** is this intended? If preserving parens is desired, always emit `(...)`.

### B14. Number literal forms lost
- **File:** `src/swc_printer/lit.rs` `Lit::Num` (and `PropName::Num`, `TsLit::Number`).
- **Effect:** `0x1F`→`31`, `1_000`→`1000`, `1e3`→`1000`, `0o17`→`15`.
- **Fix:** preserve the raw source text for numeric literals (SWC stores `span` —
  extract from source, or store raw on parse). Same risk for BigInt.

## Edge cases to verify later (not yet triggered by test files)

- **Regex literals** (`lit.rs` `Lit::Regex`): flags ordering, escaped `/`. Untested.
- **Unicode/identifier escapes** in identifiers (e.g. `\u0041`).
- **Decorators** on classes/methods/params (parser has `decorators: false` — may be fine).
- **`export =` / `import =`** (TS namespace) — `mod.rs` `_ => unhandled module decl`.
- **`TsModule`** body variants beyond `TsModuleBlock` (`decl.rs` skips `_`).
- **`UsingDecl`** stubbed as `using _;` (`decl.rs`).
- **`Stmt::With`** — emitted but strict-mode questionable; verify intent.
- **Labeled statements**, `for-await-of`, `do-while` — implemented, untested.
- **Computed member with optional chain** `a?.[b]` — check `print_opt_chain_base` Member.
- **Template literal escapes** inside quasis (`lit.rs` `print_tpl` uses `q.raw` —
  should be OK, but verify `${}` nesting and tagged templates).
- **JSX** — parser is `tsx: false`; out of scope but confirm no `.tsx` files expected.
- **Comments**: trailing block comments (`emit_trailing_comments`) and line comments
  inside blocks — only lightly exercised. Verify blank-line placeholder count matches
  when a block comment spans N lines (preprocess emits N placeholder lines).

## Suggested fix order (next phase)

1. **B5** (indentation) + **B6** (block trailing newline) — root of most visible
   corruption; fixes 3 tests.
2. **B8** (comment duplication) — fixes non-idempotency + 1 test.
3. **B1** (string escapes) + **B2** (classes) + **B3** (spread) + **B4** (opt call) —
   data-loss / invalid-output P0s.
4. **B7** (collapse width check) — fixes 1 test + object/arrow corruption.
5. **B9** + **B10** (optional `?`, destructuring types).
6. **B11** (re-wire wrapping) — fixes 2 tests.
7. **B12–B14** cosmetic.

After each fix: `cargo build --release` → re-seed `test-files/output` → re-format →
diff → idempotency → `cargo test`. Add a unit test per fixed bug.

## Verification checklist (resume point)

- [ ] `cargo test` passes completely (currently 7 failing)
- [ ] `cargo build --release` succeeds with no warnings (currently 4 unused-fn warnings)
- [ ] All 9 originals format without corruption
- [ ] Every formatted output is idempotent (currently 3 non-idempotent)
- [ ] No semantic changes (spread, optional-call, `?`, types preserved)
- [ ] String/class/regex edge cases produce valid JS
