---
title: "Installation"
---

# Installation

SQLfmt ships as a single self-contained binary. Prebuilt releases are committed
to the [repository](https://github.com/reepolee/sqlfmt) for macOS (Apple
Silicon, Intel, and a universal build), Linux x64, and Windows x64.

## macOS / Linux

Clone the repository, build for your platform, and install:

```bash
git clone https://github.com/reepolee/sqlfmt.git
cd sqlfmt
chmod +x build.sh install.sh

./build.sh native   # build for your architecture
./install.sh        # copy the binary to ~/.local/bin
source ~/.zshrc     # or restart your terminal
```

The install script copies the platform-specific binary into `~/.local/bin` and
adds it to your `PATH`.

## Windows

Build and install with the PowerShell scripts, then restart your terminal:

```powershell
git clone https://github.com/reepolee/sqlfmt.git
cd sqlfmt
.\build.ps1     # produces sqlfmt-windows-x64.exe
.\install.ps1
```

## Manual install

If you prefer, copy a release binary somewhere on your `PATH` yourself:

```bash
cp ./sqlfmt /usr/local/bin/
```

## Verify

```bash
echo "select 1;" | sqlfmt
```

You should see `SELECT 1;` printed back.

## Building from source

To compile SQLfmt yourself or cross-compile for another platform, see
[Building from source](/sqlfmt/docs/contributing/building).
