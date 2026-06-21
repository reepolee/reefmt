---
title: "Formatting"
---

# Formatting

The extension formats `.ree` files using the [Reefmt](/reefmt/docs) binary. It
registers as the default formatter for the `ree` language, so the standard VS
Code formatting commands and settings just work.

## Format a document

With a `.ree` file open, run **Format Document** (`⇧⌥F` / `Shift+Alt+F`), or open
the Command Palette (`F1`) and run **Format ree Template**.

## Format on save

Turn on format-on-save for Ree files in your `settings.json`:

```json
{
	"[ree]": {
		"editor.defaultFormatter": "reepolee.ree-templates",
		"editor.formatOnSave": true
	}
}
```

The extension already sets `reepolee.ree-templates` as the default formatter for
`[ree]`, so in most cases enabling `editor.formatOnSave` is all you need.

## Requirements

Formatting requires the `reefmt` binary. The extension looks for `reefmt` on your
`PATH` by default; if it lives somewhere else, set its absolute path with
[`ree.reefmtPath`](/ree-templates-vscode/docs/configuration/settings). See the
[Reefmt docs](/reefmt/docs) for installation and details on what it formats.

## What you get

Reefmt re-indents HTML and Ree blocks, normalises tag and directive spacing,
keeps short inline elements on one line, and formats embedded `<script>` and
`<style>` bodies (via Biome, when available). See
[Reefmt → Command line](/reefmt/docs/usage/command-line) for the complete list.
