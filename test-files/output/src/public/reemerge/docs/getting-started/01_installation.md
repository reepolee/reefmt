---
title: "Installation"
---

# Installation

Reemerge ships as a single self-contained binary. Prebuilt releases are
published on the [repository](https://github.com/reepolee/reemerge) for macOS
(Apple Silicon and Intel), Linux (x64 and arm64), and Windows x64 — so most
people never need a Rust toolchain.

## Quick install

### macOS / Linux

The install script detects your OS and architecture, downloads the correct
binary from the latest GitHub Release, and adds it to your `PATH`:

```bash
curl -fsSL https://raw.githubusercontent.com/reepolee/reemerge/main/install.sh | bash
```

### Windows

```powershell
irm https://raw.githubusercontent.com/reepolee/reemerge/main/install.ps1 | iex
```

You can also download a binary directly from the
[latest release](https://github.com/reepolee/reemerge/releases/latest).

## Verify

Once installed, confirm Reemerge is on your `PATH`:

```bash
reemerge --version
```

You should see the installed version printed, for example `reemerge 0.1.9`.

## Building from source

Prefer to compile it yourself? Reemerge is a Rust project (edition 2024). See
[Building from source](/reemerge/docs/contributing/building) for the full
toolchain and build-script details.
