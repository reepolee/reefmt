---
title: "Settings"
---

# Settings

The extension contributes a small set of settings and defaults. Configure them in
your VS Code `settings.json` (User or Workspace).

## `ree.reefmtPath`

- **Type:** `string`
- **Default:** `""`

Absolute path to the `reefmt` executable used for formatting. Leave it empty to
use `reefmt` from your `PATH`. Set it when Reefmt is installed somewhere that
isn't on your `PATH`:

```json
{
	"ree.reefmtPath": "/Users/you/.local/bin/reefmt"
}
```

## Language defaults

The extension applies these defaults for the `ree` language automatically — you
can override them, but you rarely need to set them yourself:

```json
{
	"[ree]": {
		"editor.defaultFormatter": "reepolee.ree-templates"
	},
	"emmet.includeLanguages": {
		"ree": "html"
	}
}
```

- **`editor.defaultFormatter`** makes the extension the formatter for `.ree`
  files, so **Format Document** and format-on-save use reefmt.
- **`emmet.includeLanguages`** enables Emmet HTML abbreviations inside Ree files.

## Command

The extension also contributes a command:

| Command      | Palette title           |
| ------------ | ----------------------- |
| `ree.format` | **Format ree Template** |

## Related

- [Formatting](/ree-templates-vscode/docs/features/formatting) — how formatting works
- [Reefmt](/reefmt/docs) — the formatter binary behind the extension
