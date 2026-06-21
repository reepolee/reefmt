---
title: "Configuration"
---

# Configuration

Reefmt reads its settings from a `reefmt.jsonc` config file in your project root.
The `.jsonc` extension means the file supports JSON comments (`//` and
`/* */`), so each option can carry inline documentation.

## Generate a config

Run `--init` to write a fully commented `reefmt.jsonc` into the current
directory, then edit it to suit your project:

```bash
reefmt --init
```

## Options

| Option | Type | Default | Description |
|---|---|---|---|
| `skipDirs` | `string[]` | `["node_modules", "vendor", "vendors", "dist", "static", "templates"]` | Directory names to skip when formatting. Any folder matching one of these names is skipped, regardless of where it sits in the project. |
| `skipFiles` | `string[]` | `[]` | Glob patterns for individual files to skip (e.g. `"generator/templates/**/*.ts"`). Matched against file paths relative to the project root. |
| `skipExtensions` | `string[]` | `["min.js"]` | Compound extensions to always skip, regardless of `extensions`. A file ending in `.min.js` is skipped even when `js` is in `extensions`. |
| `extensions` | `string[]` | `["ree", "ts", "js", "css"]` | File extensions to format. Only files with these extensions are picked up during recursive walks or glob matching. |
| `skipDotDirs` | `boolean` | `true` | Skip directories whose name starts with a dot (e.g. `.git`, `.next`, `.cache`, `.svelte-kit`). |
| `wrapWidth` | `number` | `180` | Maximum line width before elements are broken onto multiple lines. Controls when inline elements are folded and when long tag lines are split. |
| `collapseSingleStatementBlocks` | `boolean` | `true` | When enabled, single-statement blocks (`if`, `for`, `while`, etc.) and object-literal function params (`fn({ key: val })`) collapse onto one line when they fit within `wrapWidth`. |
| `collapseMaxMembers` | `number` | `3` | Maximum number of object-literal or type-literal members before collapsing is prevented. With the default `3`, a 4+ member object literal stays multi-line regardless of `wrapWidth`. |
| `removeUnusedImports` | `boolean` | `false` | When enabled, unused import declarations are removed from JS/TS files during formatting. Side-effect imports (`import "./foo"`) are always kept. |

## Example

```jsonc
{
	// Directory names to skip anywhere in the project.
	"skipDirs": ["node_modules", "vendor", "dist", ".output"],
	// Glob patterns for individual files to skip.
	"skipFiles": [],
	// Compound extensions to always skip.
	"skipExtensions": ["min.js"],
	// File extensions to format.
	"extensions": ["ree", "ts", "js", "css"],
	// Skip dot-directories like .git and .cache.
	"skipDotDirs": true,
	// Maximum line width before wrapping.
	"wrapWidth": 180,
	// Collapse single-statement blocks that fit on one line.
	"collapseSingleStatementBlocks": true,
	// Keep object literals with more than this many members multi-line.
	"collapseMaxMembers": 3,
	// Strip unused imports from JS/TS files.
	"removeUnusedImports": false
}
```
