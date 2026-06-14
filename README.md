# reefmt

A formatter for Ree Templates.

Used by https://marketplace.visualstudio.com/items?itemName=reepolee.ree-templates VSCode extension  

Check out https://www.reepolee.com for more information.

## Install

**macOS / Linux:**

```bash
curl -fsSL https://raw.githubusercontent.com/reepolee/reefmt/main/install.sh | bash
```

**Windows:**

```powershell
irm https://raw.githubusercontent.com/reepolee/reefmt/main/install.ps1 | iex
```

The script detects your OS and architecture, downloads the correct binary from the latest GitHub Release, and adds it to your PATH.

Or download a binary directly from the [latest release](https://github.com/reepolee/reefmt/releases/latest).

### Optional: install formatter dependencies

For best results (JS/CSS formatting via dprint and lint fixes via biome):

```bash
brew install dprint biome
# or
bun install -g dprint biome
```

reefmt works without these — it falls back to basic indentation if they're not found.

## Usage

reefmt formats `.ree` template files and can also format `.ts`, `.js`, and `.css` files by piping them through dprint (formatting) and biome (lint fixes) in a single pass — a convenient unified formatter for your projects.

### Format all supported files in the current directory (recursive):

```bash
reefmt
```

### Format a specific file:

```bash
reefmt path/to/template.ree
reefmt path/to/component.ts
reefmt path/to/styles.css
```

### Format a specific directory:

```bash
reefmt src/
```

### Format files matching a glob pattern:

```bash
reefmt "**/*.ree"
reefmt "src/**/*.ts"
reefmt "src/**/*.{ts,js,css}"
```

### Print the version:

```bash
reefmt --version
```

### Check mode (dry-run for CI):

Report which files would be reformatted without modifying them. Exits with code 1 if any file would change (useful for CI pipelines):

```bash
reefmt --check
reefmt --dry-run
reefmt -c
```

> 📘 For detailed documentation, examples, and configuration reference, visit [reepolee.com/reefmt/docs](https://www.reepolee.com/reefmt/docs/).

## Development

This is a Rust project. Build and install the latest local source:

**macOS / Linux:**

```bash
bash release.sh
```

**Windows:**

```powershell
.\release.ps1
```

This builds from source, tags the release, publishes to GitHub, and installs the binary to your PATH.

To just test locally without releasing:

```bash
cargo build --release
cp target/release/reefmt ~/.local/bin/   # macOS/Linux
# or
Copy-Item .\target\release\reefmt.exe ~\bin\   # Windows
```

### Release workflow

Run on each machine after pushing code:

1. **macOS (first):** `bash release.sh` — bumps version, creates tag and GitHub Release, uploads macOS binary
2. **Linux:** `bash release.sh` — uploads Linux binary to existing release
3. **Windows:** `.\release.ps1` — uploads Windows binary to existing release

Add `--draft` / `-Draft` to create the release as a draft.

