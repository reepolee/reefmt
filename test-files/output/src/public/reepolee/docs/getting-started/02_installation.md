---
title: "Installation"
---

# Installation

<a name="introduction"></a>

## Introduction

Reepolee has one hard prerequisite — Bun. The CLI is the runtime, the package manager, the bundler, and the test runner all in one. Three more global tools (Tailwind, the formatter, the linter) round out the development environment, but they install once and apply across every Reepolee project on your machine.

This page is the first-time setup. Once you've worked through it, [Quick Start](/getting-started/quick-start) gets you from a fresh clone to a running application in a few minutes.

<a name="installing-bun"></a>

## Installing Bun

The official installer covers macOS and Linux:

```bash
curl -fsSL https://bun.sh/install | bash
```

For Windows, install through WSL2 or use the experimental native build (the macOS/Linux instructions cover the WSL path; the native one is documented at [bun.sh/docs/installation](https://bun.sh/docs/installation)).

Verify the install:

```bash
bun --version
```

The output should be the Bun version (`1.3.13` at time of writing). If `bun: command not found`, the installer's `~/.bun/bin` directory isn't on your `$PATH` — open a new terminal or run `source ~/.bashrc` (or `~/.zshrc`) to pick up the change.

<a name="pinning-bun"></a>

### Pinning a Specific Version

For a project that needs a specific Bun version (matching a teammate's environment, matching production), install that version explicitly:

```bash
curl -fsSL https://bun.sh/install | bash -s "bun-v1.3.13"
```

Then leave `bun upgrade` alone until you've tested a newer version against your project. Reepolee's README documents the Bun version it tracks; running with the same version (or newer) is safe.

<a name="installing-global-tools"></a>

## Installing Global Tools

Four tools install globally and stay there. The recommended one-shot:

```bash
bun add -g concurrently @tailwindcss/cli oxfmt oxlint
```

What each tool does:

| Tool                            | Purpose                                                                                                                                       |
| ------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------- |
| **concurrently** (alias `conc`) | Runs the Tailwind watcher and the dev server side-by-side from a single `bun dev` command. Used until Bun's `bun --parallel` ships in stable. |
| **@tailwindcss/cli**            | The Tailwind v4 CLI. Reads `css/app.css`, scans your sources for class names, writes the compiled stylesheet to `static/`.                    |
| **oxfmt**                       | Fast JavaScript/TypeScript formatter. Used by `bun run format`.                                                                               |
| **oxlint**                      | Fast JavaScript/TypeScript linter. Used by `bun run format` alongside oxfmt.                                                                  |

Verify each one:

```bash
bun -v
tailwindcss -h | grep v
conc -v
oxfmt --version
oxlint --version
```

Each should print a version. If any are missing, re-run the `bun add -g` command — sometimes the global PATH update needs a new terminal session before the binary is found.

> **One-shot bootstrap.** Instead of installing the global tools by hand, run `bun get:pre` from a cloned project. It installs the global tools **and** downloads the vendored browser libraries the project expects — Zod, highlight.js, alien-deepsignals, libvips ([below](#libvips)), and the [DPU streaming](/database/generators#streaming) polyfill (`static/dpu.min.js`, fetched by `bun get:dpu`). Each piece also has its own `bun get:<name>` script if you need to refresh just one.

<a name="why-global"></a>

### Why Global Tools?

Reepolee has zero runtime dependencies and only the four global dev tools. Keeping them global means:

- **Project clones don't bring 200 MB of `node_modules`** for tooling that's identical across projects.
- **`bun install --frozen-lockfile`** in production stays minimal — it's installing a small handful of dev dependencies, not the entire toolchain.
- **Updates apply once.** Upgrading the Tailwind CLI on your machine upgrades it for every Reepolee project at once.

The trade-off: the tooling versions aren't pinned per-project. For a team, sync versions occasionally to stay aligned. For a solo developer, the tools rarely break across versions.

<a name="editor-setup"></a>

## Editor Setup

For VSCode, the [Ree Templates extension](https://marketplace.visualstudio.com/items?itemName=reepolee.ree-templates) adds syntax highlighting and formatting for `.ree` files. Install it from the marketplace or with the command-line:

```bash
code --install-extension reepolee.ree-templates
```

For other editors:

- **Treat `.ree` as HTML.** The HTML syntax highlighter handles most of Ree fine — the tag prefixes (`{=`, `{~`, `{#`, `{:`, `{/`, `{@`, `{{`) are visually distinct enough that the HTML grammar ignores them cleanly.
- **Tailwind IntelliSense** (the official VSCode extension) works inside `.ree` files if you tell it to — add `"tailwindCSS.includeLanguages": { "ree": "html" }` to your VSCode settings.

The TypeScript language server picks up `.ts` files in Reepolee projects natively — no extra configuration needed.

<a name="optional-tooling"></a>

## Optional Tooling

A few tools are useful but not required:

- **`ncu`** ([npm-check-updates](https://github.com/raineorshine/npm-check-updates)) — checks `package.json` for newer dev-dependency versions. The `bun ncu` script in `package.json` runs `ncu -u -t minor && ncu` to update minor versions and report majors.
- **`gh`** ([GitHub CLI](https://cli.github.com/)) — creates pull requests, manages issues, and runs the deploy workflow from the command line.
- **`jq`** — for filtering NDJSON SQL logs (covered in [Logs](/deployment/logs)).
- **`certbot`** — only needed on the production server, for TLS certificates ([Reverse Proxy](/deployment/reverse-proxy)).

Install whichever fits your workflow. None are needed to run Reepolee.

<a name="libvips"></a>

### libvips (for image processing)

The image-editor / avatar pipeline ([Image Processing](/forms/image-processing)) relies on the native **libvips** library. Reepolee ships a small installer that fetches a prebuilt libvips for your platform so you don't have to install it through a system package manager:

```bash
bun get:vips                             # install the latest libvips (alias for the command below)
bun scripts/cli.ts vips                  # same thing, explicit
bun scripts/cli.ts vips --version=8.15.3 # pin a specific version
```

The installer supports Windows (prebuilt from GitHub releases), macOS (Homebrew), and Linux (apt/dnf/pacman), and adds libvips to your `PATH` automatically.

If you don't use the image editor or avatar uploads, you can skip this — the rest of the app runs without libvips.

<a name="verifying-the-setup"></a>

## Verifying the Setup

Before moving on to [Quick Start](/getting-started/quick-start), confirm the four tools work together by cloning a sample project and running it:

```bash
git clone https://github.com/reepolee/reepolee.git test-reepolee
cd test-reepolee
bun install
cp .env.example .env   # already sets CONNECTION_STRING="sqlite:app.db"
bun dev
```

If you get a "Listening on http://localhost:2338" line and the page loads in a browser, every tool is wired up correctly. Delete the `test-reepolee` directory — for a real project, [Quick Start](/getting-started/quick-start) walks through using Reepolee as a [GitHub template repository](https://docs.github.com/en/repositories/creating-and-managing-repositories/creating-a-repository-from-a-template) to spin up your own repo, then wiring `reepolee/reepolee` back in as `upstream` so future releases flow in via `bun git:sync`.

If anything fails — missing tool, broken `bun install`, the dev server not starting — fix it now. The rest of the documentation assumes the install works.
