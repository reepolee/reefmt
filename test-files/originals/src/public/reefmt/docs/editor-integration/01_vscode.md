---
title: "VSCode"
---

# Editor integration

Reefmt is the formatter behind the **Ree Templates** extension for Visual Studio
Code. Installing the extension gives you format-on-save and the _Format Document_
command for `.ree` files, backed by the same binary you run on the command line.

## Install the extension

Install **Ree Templates** from the
[Visual Studio Marketplace](https://marketplace.visualstudio.com/items?itemName=reepolee.ree-templates),
then read the [extension documentation](/ree-templates-vscode/docs) for syntax
highlighting, icons, and settings.

## Point the extension at Reefmt

The extension calls the `reefmt` binary. By default it looks for `reefmt` on your
`PATH`, so once you have [installed Reefmt](/reefmt/docs/getting-started/installation)
formatting works out of the box.

If Reefmt is not on your `PATH`, set its absolute location in your VSCode
settings:

```json
{
	"ree.reefmtPath": "/Users/you/.local/bin/reefmt"
}
```

## Format on save

Enable format-on-save for Ree files in your settings:

```json
{
	"[ree]": {
		"editor.defaultFormatter": "reepolee.ree-templates",
		"editor.formatOnSave": true
	}
}
```

The extension sets `reepolee.ree-templates` as the default formatter for the
`ree` language automatically, so you usually only need to turn on
`editor.formatOnSave`.

See the extension's [Configuration](/ree-templates-vscode/docs/configuration/settings)
page for the full list of settings.
