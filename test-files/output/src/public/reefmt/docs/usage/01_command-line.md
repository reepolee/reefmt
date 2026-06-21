---
title: "Command line"
---

# Command line

Reefmt formats files **in place**. Point it at a file, a directory, or a glob,
and it rewrites each matching file with the formatted result, printing a line
for every file it touches. By default it formats `.ree`, `.ts`, `.js`, and
`.css` files.

> 📘 Reefmt reads its settings from a `reefmt.jsonc` config file in your project
> root. Run `reefmt --init` to generate one — see
> [Configuration](/reefmt/docs/configuration/options) for the full list of options.

## Format a single file

```bash
reefmt src/public/index.ree
reefmt src/lib/component.ts
reefmt src/styles/app.css
```

## Format a directory

Pass a directory and Reefmt walks it recursively, formatting every supported
file it finds (skipping directories listed in your config):

```bash
reefmt src/
```

## Format your whole project

With no arguments, Reefmt formats every supported file under the current
directory:

```bash
reefmt
```

## Use a glob

Quote the pattern so your shell passes it through to Reefmt rather than
expanding it first:

```bash
reefmt "**/*.ree"
reefmt "src/**/*.ts"
reefmt "src/**/*.{ts,js,css}"
```

## Check mode

Report which files would be reformatted without modifying them. Reefmt exits
with code `1` if any file would change, which makes it ideal for CI:

```bash
reefmt --check
reefmt --dry-run
reefmt -c
```

## Diff mode

Show a unified diff of the changes that would be made, without writing anything
to disk:

```bash
reefmt --diff
```

## Stdin mode

Format input piped on stdin and write the result to stdout. This is what editor
integrations use under the hood. Pass an extension argument to choose the
language; if omitted, Reefmt defaults to `.ree`:

```bash
cat file.ree | reefmt --stdin        # default: format as Ree
cat file.ts  | reefmt --stdin .ts    # format as TypeScript
cat file.js  | reefmt --stdin .js    # format as JavaScript
cat file.css | reefmt --stdin .css   # format as CSS
```

## Generate a config

Create a commented `reefmt.jsonc` in the current directory that you can edit to
customize skip rules, file extensions, and formatting behavior:

```bash
reefmt --init
```

## Check the version

```bash
reefmt --version
# or
reefmt -v
```

## What gets formatted

- **Indentation** — HTML tags and Ree blocks (`{#if}`, `{#each}`, `{#with}`) are
  re-indented with tabs; `{#layout}` and `{#include}` do not open a block.
- **Tag spacing** — spacing between tags and Ree expressions is normalised, and
  short inline elements are kept on a single line.
- **Directive spacing** — loose directives such as `{# if }` are tightened to
  `{#if}`.
- **HTML comments** — `<!-- ... -->` content is preserved exactly, even when it
  contains tag- or directive-like text.
- **Embedded code** — `<script>` and `<style>` bodies, along with standalone
  `.ts`, `.js`, and `.css` files, are formatted with the built-in
  [SWC](https://swc.rs) formatter. No external tools are required.

## Continuous integration

Use `--check` to fail a CI job when any file is not already formatted, without
touching the working tree:

```bash
reefmt --check
```

The command exits non-zero if any file would change, so the step fails until the
code is reformatted and committed.
