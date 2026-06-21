---
title: "Building from source"
---

# Building from source

Reemerge is a Rust project (edition 2024). You'll need a Rust toolchain —
install one with [rustup](https://rustup.rs/).

## Plain cargo build

```bash
cargo build --release
# Binary at ./target/release/reemerge
```

## Build scripts

The repository ships convenience scripts that produce a platform-named binary
and install it locally.

### macOS / Linux

```bash
./build.sh
# Produces reemerge-macos-arm64 (or -x64 / -linux-x64 / -linux-arm64)
# and installs to ~/.local/bin/
```

### Windows

```powershell
.\build.ps1
# Produces reemerge-windows-x64.exe and installs to ~\bin\
```

Pass `--no-install` (or `-NoInstall` on Windows) to skip the local install step
— useful in CI.

## Releasing

Maintainers cut releases with the `release.sh` / `release.ps1` scripts, run once
per platform after pushing code:

1. **macOS (first):** `bash release.sh` — bumps the version, creates the tag and
   GitHub Release, and uploads the macOS binary.
2. **Linux:** `bash release.sh` — uploads the Linux binary to the existing
   release.
3. **Windows:** `.\release.ps1` — uploads the Windows binary to the existing
   release.

Add `--draft` (or `-Draft`) to create the release as a draft.
