---
title: "Installation"
layout: "reeweb/docs/docs.layout"
---

# Installation

<a name="introduction"></a>

## Introduction

Reeweb has one prerequisite — Bun. The CLI is the runtime, the package manager, the bundler, and the test runner all in one. Once Bun is installed, you're ready to build.

<a name="installing-bun"></a>

## Installing Bun

The official installer covers macOS and Linux:

```bash
curl -fsSL https://bun.sh/install | bash
```

For Windows, install through WSL2 or use the experimental native build (documented at [bun.sh/docs/installation](https://bun.sh/docs/installation)).

Verify the install:

```bash
bun --version
```

The output should be a version number. If `bun: command not found`, the installer's `~/.bun/bin` directory isn't on your `$PATH` — open a new terminal or run `source ~/.bashrc` (or `~/.zshrc`) to pick up the change.

<a name="starting-a-project"></a>

## Starting a Project

This project is the Reepolee.com website itself, built with the Reeweb static site generator. To work with it locally, clone the repository (if you have access) or copy the project structure as a starting point for your own site:

```bash
git clone <repository-url> my-site
cd my-site
bun install
```

This installs a small set of dev dependencies (TypeScript types, Tailwind CSS v4). That's all — zero runtime dependencies.

<a name="building-the-site"></a>

## Building the Site

```bash
bun run build
```

This runs the static build script (`scripts/build.ts`), which:

- Renders all `.ree` templates and `.md` files to static HTML
- Copies static assets (CSS, JS, images, fonts) to the output directory
- Generates localized routes for every configured language
- Emits redirects for paths declared in `config/redirects.ts`

The output goes to `dist/`. Serve it with any static file server:

```bash
bun run preview
```

This starts a lightweight HTTP server on `localhost:3000` serving the built files.

<a name="development-mode"></a>

## Development Mode

```bash
bun dev
```

This uses `conc` (concurrently) to start two processes in parallel: the Tailwind CSS watcher (`bun css:watch`) and the development server (`bun development`). Changes to templates, markdown, translations, or CSS are detected automatically and the browser reloads. No manual refresh needed.

The CSS watcher recompiles `src/css/style.css` to `src/public/css/style.min.css` on every change.

<a name="other-commands"></a>

## Other Commands

| Purpose                    | Command                                                                                   |
| -------------------------- | ----------------------------------------------------------------------------------------- |
| Build with verbose logging | `bun scripts/build.ts --public ./src/public --dist ./dist --verbose`                      |
| Build with hreflang links  | `bun scripts/build.ts --public ./src/public --dist ./dist --site-url https://example.com` |
| CSS once (minified)        | `bun run css:build`                                                                       |
| CSS watch                  | `bun run css:watch`                                                                       |
| Generate sitemap           | `bun run sitemap`                                                                         |
| Generate RSS feed          | `bun run rss`                                                                             |
| Format source              | `bun run format` (uses oxfmt)                                                             |

<a name="verifying-the-setup"></a>

## Verifying the Setup

```bash
cd my-site
bun install
bun run build
bun run preview
```

If you see "✓ Build complete" in the terminal and the preview loads at `http://localhost:3000`, everything is wired up correctly.

<a name="editor-setup"></a>

## Editor Setup

For VSCode, [.vscode/settings.json](/.vscode/settings.json) in this project maps `.ree` files to a custom language. For other editors, treating `.ree` as HTML gets you most of the way — the tag syntax (`{= }`, `{#if}`, `{#each}`) is distinct enough that the HTML highlighter ignores it cleanly.
