# Changelog


















































































## [0.7.104] - 2026-06-29

## [0.7.103] - 2026-06-29

## [0.7.102] - 2026-06-29

## [0.7.101] - 2026-06-25

## [0.7.100] - 2026-06-23

## [0.7.99] - 2026-06-23

## [0.7.98] - 2026-06-23

## [0.7.97] - 2026-06-23

## [0.7.96] - 2026-06-23

## [0.7.95] - 2026-06-23

## [0.7.94] - 2026-06-23

## [0.7.93] - 2026-06-23

## [0.7.92] - 2026-06-23

## [0.7.91] - 2026-06-23

## [0.7.90] - 2026-06-23

## [0.7.89] - 2026-06-23

## [0.7.88] - 2026-06-23

## [0.7.87] - 2026-06-23

## [0.7.86] - 2026-06-23

## [0.7.85] - 2026-06-22

## [0.7.84] - 2026-06-22

## [0.7.83] - 2026-06-22

## [0.7.82] - 2026-06-22

## [0.7.81] - 2026-06-22

## [0.7.80] - 2026-06-21

## [0.7.79] - 2026-06-21

## [0.7.78] - 2026-06-21

## [0.7.77] - 2026-06-21

## [0.7.76] - 2026-06-21

## [0.7.75] - 2026-06-21

## [0.7.74] - 2026-06-21

## [0.7.73] - 2026-06-21

## [0.7.72] - 2026-06-21

## [0.7.71] - 2026-06-21

## [0.7.70] - 2026-06-21

## [0.7.69] - 2026-06-21

## [0.7.68] - 2026-06-21

## [0.7.67] - 2026-06-21

## [0.7.66] - 2026-06-21

## [0.7.65] - 2026-06-21

## [0.7.64] - 2026-06-21

## [0.7.63] - 2026-06-21

## [0.7.62] - 2026-06-21

## [0.7.61] - 2026-06-21

## [0.7.60] - 2026-06-21

## [0.7.59] - 2026-06-20

## [0.7.58] - 2026-06-20

## [0.7.57] - 2026-06-20

## [0.7.56] - 2026-06-20

## [0.7.55] - 2026-06-20

## [0.7.54] - 2026-06-20

## [0.7.53] - 2026-06-19

## [0.7.52] - 2026-06-19

## [0.7.51] - 2026-06-19

## [0.7.33] - 2026-06-19

## [0.7.32] - 2026-06-19

## [0.7.31] - 2026-06-19

## [0.7.30] - 2026-06-19

## [0.7.29] - 2026-06-19

## [0.7.28] - 2026-06-18

## [0.7.27] - 2026-06-18

## [0.7.26] - 2026-06-18

## [0.7.25] - 2026-06-18

## [0.7.24] - 2026-06-18

## [0.7.23] - 2026-06-18

## [0.7.22] - 2026-06-18

## [0.7.21] - 2026-06-18

## [0.7.20] - 2026-06-18

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

