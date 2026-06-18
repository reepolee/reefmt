# Changelog














## [0.7.19] - 2026-06-18

## [0.7.18] - 2026-06-18

## [0.7.17] - 2026-06-18

## [0.7.16] - 2026-06-18

## [0.7.15] - 2026-06-18

## [0.7.14] - 2026-06-17

## [0.7.13] - 2026-06-17

## [0.7.12] - 2026-06-17

## [0.7.11] - 2026-06-17

## [0.7.10] - 2026-06-17

## [0.7.9] - 2026-06-17

## [0.7.8] - 2026-06-17

## [0.7.7] - 2026-06-17

## [0.7.6] - 2025-06-17

### Removed
- **oxfmt subprocess dependency** — reefmt is now fully native Rust with no external tool
  requirements. JS/TS formatting uses the built-in SWC parser (`swc_core`) instead of
  spawning an external `oxfmt` process.
  - Removed `pipe_oxfmt()`, `resolve_oxfmt_config()`, `temp_uid()`, `OXFMT_CONFIG`,
    and associated code from `src/format.rs`
  - Removed orphaned `src/ree_tags.rs` stub (leftover from the oxfmt pipeline era)
  - CSS files now pass through unchanged (native CSS formatting is a future improvement)

### Changed
- **Configurable wrap width** — The hardcoded 160/140 character wrap limits are now
  configurable via the `wrapWidth` option in `reefmt.jsonc` (default: 120).
- **UTF-8 idempotency fix** — Fixed a bug in `protect_ree_expressions()` that corrupted
  multi-byte UTF-8 characters (e.g., `š`, `č`, `ü`) by converting each byte to a char.
  This caused non-idempotent formatting — files with non-ASCII characters in comments
  would be rewritten on every formatting pass.
- **DOCTYPE handling** — `<!DOCTYPE html>` declarations are now preserved as-is instead
  of being parsed as HTML elements, which previously caused spurious `</!DOCTYPE>`
  closing tags and incorrect `<html>` indentation.

### Added
- Idempotency regression tests covering UTF-8, DOCTYPE, blank lines, and script blocks.
- This changelog.

