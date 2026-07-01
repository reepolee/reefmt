# Outstanding edge cases

Known gaps found while fixing the 2026-07-01 session bugs. Not yet implemented — captured here for later.

## 1. `new X(...)` does not hug a lone function/arrow arg

`Expr::Call` now always hugs a single arrow/function argument, but `Expr::New`
(`src/swc_printer/expr.rs`) has no hug branch, so a body that must break expands:

```js
// current
const x = new Foo(
	() => {
		/* only comment */
	},
);
// wanted
const x = new Foo(() => {
	/* only comment */
});
```

Fix: mirror the `can_hug` logic from the `Expr::Call` branch into `Expr::New`.

## 2. `</script>` / `</style>` inside a string/template/regex breaks `.ree` script blocks

Raw-block close scanning (`parse_raw_block_content`, `src/ree_parser.rs`) matches
the literal close marker without respecting JS string, template-literal, comment,
or regex context, so it terminates early and corrupts the file:

```
<script>
	const s = "a</script>b";   // block ends at the inner </script>, output mangled
</script>
```

Fix: make the raw-block scanner skip over string/template/regex/comment spans when
looking for the close marker. Pre-existing; not caused by the `<!--`-only change.

## 3. Object/array call args with comments expand while arrows hug

A lone arrow/function arg always hugs now, but object/array literal args still
only hug when `hugCallArgs` is enabled. With a `//` or block comment inside, the
object arg expands, which now looks inconsistent next to hugged callbacks:

```js
foo(
	{
		/* c */
		x: 1,
	},
);
```

Decide whether object/array hugging should also become unconditional (or at least
match the arrow behaviour when the arg contains only comments).

## 4. `<!--` still a split point inside `<script>`

The raw-block scanner intentionally still stops at `<!--` inside `<script>`/`<style>`
to isolate HTML comments. Whether legacy `<!-- ... -->` inside a script should be
isolated (vs left fully verbatim) is untested.

## 5. Test suite does not compile

`cargo test` fails with ~34 errors: test call sites pass `format_ree(input, 120, false)`
but `oneline` is now `usize` (bool → usize mismatch). This blocks adding regression
tests for the three 2026-07-01 fixes until the call sites are updated (`false` → `0`).
