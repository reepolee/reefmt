# reefmt

A formatter for Ree Templates.

Used by https://marketplace.visualstudio.com/items?itemName=reepolee.ree-templates VSCode extension-

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

