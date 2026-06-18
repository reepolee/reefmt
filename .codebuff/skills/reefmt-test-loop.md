---
name: reefmt-test-loop
description: |
  Runs reefmt on test files using glob patterns, captures formatted output,
  compares against originals, and iteratively fixes formatter bugs. Supports
  all file types (.ree, .ts, .js, .css) discovered via glob. Use when refining
  the reefmt formatter with test-driven regression detection — format files
  from test-files/originals/, analyze diffs and idempotency, fix source bugs
  in Rust, rebuild, and loop until all tests pass cleanly.
---

# reefmt Test Loop

## Overview

This skill drives a regression-test loop for the `reefmt` Rust formatter.
It discovers all supported files (`.ree`, `.ts`, `.js`, `.css`) in
`test-files/originals/` via glob, formats each via stdin to `test-files/output/`,
diffs originals vs formatted output, checks idempotency, runs `cargo test`,
and then fixes any discovered formatting bugs in the Rust source. The outer
loop repeats until all checks pass.

```
┌─────────────────────────────────────────────────────┐
│                                                     │
│   Build ──→ Glob ──→ Format ──→ Diff ──→ Idempot ─┐│
│       ▲                                            ││
│       │    ┌────────────────────────────────────────┘│
│       │    │                                         │
│       │    ▼                                         │
│       └─── Fix bug ←── Analyze diff ←── cargo test   │
│                                                      │
└──────────────────────────────────────────────────────┘
```

## When to Use

- After making changes to any `src/` file in the reefmt project
- Before releasing a new version, to verify existing templates still format correctly
- When a bug report mentions a specific Ree template pattern
- When adding a new formatter feature that could affect existing templates
- To validate that formatter output is idempotent
- To test JS/TS/CSS formatting (SWC) alongside Ree formatting

**When NOT to use:** Single-file, trivial changes where existing unit tests
already cover the affected code paths.

## Phases

### Phase 1 — Preparation

1. **Build reefmt:**
   ```bash
   cargo build --release
   ```
   Verify the binary exists at `target/release/reefmt`.

2. **Seed output with a snapshot of originals:**
   ```bash
   rm -rf test-files/output
   cp -r test-files/originals test-files/output
   ```
   This copies all originals into the output directory. After formatting, formatted
   versions overwrite the copies, so `git diff` shows exactly what the formatter
   changed per file — the originals (before) vs the formatted versions (after).

3. **Commit the seed state so we can examine a clean diff later:**
   ```bash
   git add -A
   git commit -m "checkpoint before reefmt test loop"
   ```
   This creates a reference point with the output directory tracked in version
   control. After formatting, `git diff HEAD -- test-files/output/` will show
   only the formatting changes — the snapshot before vs the formatted result.

### Phase 2 — Format Originals

4. **Discover all source files via glob and format each one:**

   Use a glob to find all supported files (`.ree`, `.ts`, `.js`, `.css`) in
   `test-files/originals/`, then format each via stdin, preserving the file
   extension for the `--stdin` flag. This ensures JS/TS files go through
   SWC formatting and Ree files go through the AST formatter.

   ```bash
   find test-files/originals -type f \( -name "*.ree" -o -name "*.ts" -o -name "*.js" -o -name "*.css" \) | while read -r f; do
     # Compute output path preserving subdirectory structure
     rel="${f#test-files/originals/}"
     outdir="test-files/output/$(dirname "$rel")"
     mkdir -p "$outdir"
     ext="${f##*.}"
     cat "$f" | target/release/reefmt --stdin ".$ext" > "test-files/output/$rel"
     echo "Formatted: $rel"
   done
   ```

   Alternatively, use `reefmt`'s built-in glob matching to check all files
   at once (diff mode only, since it doesn't write to a separate directory):
   ```bash
   target/release/reefmt --diff "test-files/originals/**/*.{ree,ts,js,css}"
   ```

5. **Capture originals and formatted output** for comparison:
   - **Original snapshot:** `test-files/output/<relpath>` (before formatting)
   - **Formatted pass 1:** `test-files/output/<relpath>` (after formatting, overwrites snapshot)
   - **Original pristine:** `test-files/originals/<relpath>` (never modified)
   
   Since output was seeded from originals in step 3, `git diff HEAD -- test-files/output/`
   shows exactly what each formatting pass changed.

### Phase 3 — Compare & Analyze

6. **Diff original vs formatted for all file types using git:**
   ```bash
   git diff HEAD -- test-files/output/
   ```
   This shows a clean diff of every formatting change. Because output was seeded from
   originals and committed, the diff contains only what the formatter changed — no noise.
   
   Alternatively, use a regular diff for quick inline viewing:
   ```bash
   find test-files/originals -type f \( -name "*.ree" -o -name "*.ts" -o -name "*.js" -o -name "*.css" \) | while read -r f; do
     rel="${f#test-files/originals/}"
     echo "=== $rel ==="
     diff -u "$f" "test-files/output/$rel" || true
     echo ""
   done
   ```

   Focus on **structural bugs** (not trivial whitespace):

   | Category | What to look for |
   |----------|-----------------|
   | **Nesting drift** | Blocks gain or lose indentation on every pass |
   | **Missing/added content** | Tags or expressions that disappear or appear after formatting |
   | **Malformed output** | Syntax errors introduced by formatting |
   | **Non-idempotency** | Running the formatter a second time changes output further |
   | **Collapsing errors** | Single-statement blocks or object literals collapse/expand incorrectly |
   | **Script corruption** | JS inside `<script>` tags gets mangled |
   | **Ree expression corruption** | `{= expr}`, `{~ expr}`, `{#block}` etc. get modified |
   | **Blank line handling** | Blank lines removed or added where they shouldn't be |
   | **Encoding issues** | Non-ASCII characters corrupted |
   | **SWC JS/TS issues** | Arrow spacing, type literals, import/export formatting errors |
   | **CSS passthrough** | CSS files should pass through unchanged (reefmt formats .ree/ts/js only) |

7. **Verify idempotency (pass 1 vs pass 2) for all file types:**
   ```bash
   find test-files/originals -type f \( -name "*.ree" -o -name "*.ts" -o -name "*.js" -o -name "*.css" \) | while read -r f; do
     rel="${f#test-files/originals/}"
     ext="${f##*.}"
     out="test-files/output/$rel"
     cat "$out" | target/release/reefmt --stdin ".$ext" > "${out%.*}_pass2.${ext}"
     if ! diff -q "$out" "${out%.*}_pass2.${ext}" > /dev/null 2>&1; then
       echo "NON-IDEMPOTENT: $rel"
       diff -u "$out" "${out%.*}_pass2.${ext}"
     else
       echo "IDEMPOTENT: $rel"
     fi
   done
   ```
   Every file must produce identical output on a second pass. If pass 1 ≠ pass 2,
   the formatter does not stabilize — this is a priority bug.

8. **Run the test suite:**
   ```bash
   cargo test 2>&1
   ```
   - Examine each failing test's output — it often pinpoints the exact pattern
     that broke.
   - A new failure may be a regression from an incomplete fix.

### Phase 4 — Fix Bugs (Inner Loop)

9. **For each discovered bug:**

   a. **Read the relevant source file:**
      - `src/ree_parser.rs` — Ree template AST parser
      - `src/ree_format.rs` — Ree template formatting + script block pipeline
      - `src/format.rs` — JS/TS/CSS formatting, block collapsing, SWC pre/post processing
      - `src/swc_format.rs` — SWC integration for JS/TS
      - `src/remove_unused_imports.rs` — Unused import removal

   b. **Understand the pipeline stage** where the bug manifests:
      - Parser output → AST node ordering/indentation
      - Script block extraction → JS formatting → re-insertion
      - SWC pre-processing (placeholder extraction)
      - Post-processing (arrow spacing, block collapsing, type literal collapsing)

   c. **Apply the fix** in the Rust source.

   d. **Rebuild:**
      ```bash
      cargo build --release
      ```

### Phase 5 — Iterate

10. **Re-seed output from originals** (to get a clean snapshot for the next diff):
    ```bash
    rm -rf test-files/output
    cp -r test-files/originals test-files/output
    ```

11. **Re-run Phases 2–3** with the rebuilt binary.

12. **Check result:**
    - **Bug fixed?** → Move to the next bug or proceed to validation.
    - **Bug not fixed?** → Refine the fix and re-run.
    - **New regression introduced?** → Fix that too — don't leave the tree broken.

13. **Exit loop** when all of these are true:
    - All files in `test-files/originals/` format without errors
    - Every formatted output is idempotent (pass 1 == pass 2)
    - `cargo test` passes with zero failures

### Phase 6 — Validation

14. **Final end-of-loop git diff to review all changes at once:**
    ```bash
    git diff HEAD -- test-files/output/
    ```
    This highlights exactly what the formatter changed, grouped by file.

15. **Final checklist:**
    ```
    [ ] cargo test passes completely
    [ ] cargo build --release succeeds
    [ ] Every original file formats without errors
    [ ] Every formatted output is idempotent
    [ ] No structural changes beyond formatting
    [ ] All previously found regressions are fixed
    ```

## Debugging Tips

| Tip | Detail |
|-----|--------|
| **Check `reefmt.jsonc`** | Bugs are sometimes config-related. `wrapWidth`, `collapseSingleStatementBlocks`, and `collapseMaxMembers` all affect output. |
| **Use `--diff` mode with glob** | `target/release/reefmt --diff "test-files/originals/**/*.{ree,ts,js,css}"` shows all changes at once without modifying files. |
| **Use `--check` mode with glob** | `target/release/reefmt --check "test-files/originals/**/*.{ree,ts,js,css}"` lists all files that would change. |
| **Narrow `wrapWidth`** | Collapsing bugs only trigger at certain widths. Try passing `--collapse-max-members 1` or reducing `wrapWidth` in `reefmt.jsonc` to expose edge cases. |
| **Understand the pipeline** | Ree templates flow through: parser → AST → formatter → script extraction → SWC → script re-insertion → post-processing. Any stage can introduce bugs. TS/JS files go through SWC only. |
| **Isolate the pattern** | Strip the test file down to the minimum pattern that reproduces the bug, then test just that pattern in a small file with the right extension. |
| **Add a unit test** | For every fixed bug, add a unit test that would fail with the old code. This prevents regressions. |
| **Check SWC version** | `swc_core` versions change behavior. If idempotency broke after a version bump, check the SWC changelog. |
| **Test `.ts` and `.js` separately** | SWC formatting has its own pipeline. Add separate `.ts`/`.js` test files to `test-files/originals/` to test JS-specific bugs. |

## Common Rationalizations

| Rationalization | Reality |
|----------------|---------|
| "I'll analyze all bugs at once before fixing any" | Differences compound. Fix one bug, re-format, then check what's left — many apparent "bugs" are cascading effects of one root cause. |
| "The diff looks fine, no need to check idempotency" | Non-idempotent formatters are the most frustrating user experience. Always verify. |
| "I'll skip `cargo test` until the loop is done" | A bug in one phase can break unrelated tests. Run tests every iteration. |
| "This fix is so small I don't need to re-run the full loop" | Small fixes have big cascades in a multi-stage pipeline. Always re-run. |

## Red Flags

- Skipping the idempotency check
- Multiple differences in one file that look unrelated — they likely share a root cause
- Fixing the same bug twice (the fix wasn't complete the first time)
- `cargo test` failures that grow with each iteration instead of shrinking
- Making changes to source files without re-running the loop
- Binary fails to build — don't format with a stale binary
- Ignoring a known bug to "fix it later" in the same session
- Not adding a regression test for a bug you fixed

## Verification

After completing the full loop:

- [ ] `cargo test` passes completely with zero failures
- [ ] All original `.ree` files format without runtime errors
- [ ] Every formatted output is idempotent (pass 1 == pass 2)
- [ ] No structural content is lost or added during formatting
- [ ] All discovered regressions from previous iterations are fixed
- [ ] Each fixed bug has a corresponding unit test (if one was missing)
