---
title: "Installation"
---

# Installation

Reefmt ships as a single self-contained binary — a pure Rust executable with
zero runtime dependencies. There is no Node.js, no `node_modules`, and no
external formatter to install. Prebuilt releases are published for macOS (Apple
Silicon and Intel), Linux (x64 and arm64), and Windows x64.

## Install script (recommended)

The install script detects your OS and architecture, downloads the matching
binary from the [latest GitHub Release](https://github.com/reepolee/reefmt/releases/latest),
and adds it to your `PATH`.

**macOS / Linux:**

```bash
curl -fsSL https://raw.githubusercontent.com/reepolee/reefmt/main/install.sh | bash
```

**Windows:**

```powershell
irm https://raw.githubusercontent.com/reepolee/reefmt/main/install.ps1 | iex
```

On macOS and Linux the binary is installed to `~/.local/bin`. If that directory
is not already on your `PATH`, restart your shell or run:

```bash
export PATH="$HOME/.local/bin:$PATH"
```

On Windows, restart your terminal so the new `PATH` entry takes effect.

## Manual download

Prefer to grab the binary yourself? Download the asset for your platform from
the [latest release](https://github.com/reepolee/reefmt/releases/latest), make
it executable, and move it somewhere on your `PATH`.

## Verify

Once installed, confirm Reefmt is on your `PATH`:

```bash
reefmt --version
```

You should see the installed version printed, for example `reefmt 0.7.53`.

## What Reefmt formats

Reefmt formats `.ree` template files using its custom Rust-based AST parser. It
also formats `.ts`, `.js`, and `.css` files via the built-in
[SWC](https://swc.rs) formatter — no external tools, no `dprint`, no Biome
required. Embedded `<script>` and `<style>` blocks inside your templates are
formatted with the same built-in formatter.

## Building from source

Prefer to compile it yourself? See [Building from
source](/reefmt/docs/contributing/building).
