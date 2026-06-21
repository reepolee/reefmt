---
title: "Syntax highlighting"
---

# Syntax highlighting

The extension registers the **Ree Template** language for the `.ree` extension
and ships a TextMate grammar (scope `text.html.ree`) that highlights the full
template surface.

## What is highlighted

- **HTML** — tags, attributes, and text, the same way VS Code highlights regular
  HTML.
- **Embedded JavaScript** — code inside raw JS blocks and `<script>` elements.
- **Embedded CSS** — styles inside `<style>` elements.
- **Ree directives** — block and inline directives such as layouts, includes,
  conditionals, loops, and components.
- **Ree expressions** — escaped and unescaped output expressions and raw
  JavaScript interpolations.

Because the grammar nests the HTML, JavaScript, and CSS grammars, your existing
theme colours apply automatically — there is nothing to configure.

## Language association

VS Code maps the `.ree` extension to the `ree` language id automatically.
Emmet is also enabled for Ree files (the extension maps `ree` to `html` for
Emmet), so HTML abbreviations expand as you type.

## Themes

Highlighting works with any colour theme. The extension also bundles its own
**ree Icons** file-icon theme — see [File icons](/ree-templates-vscode/docs/features/icons).
