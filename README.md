# reefmt

A formatter for Ree Templates.

Used by https://marketplace.visualstudio.com/items?itemName=reepolee.ree-templates VSCode extension-

Check out https://www.reepolee.com for more information.

## Development

This is a Rust project, we make cross builds on MacOS and native on Windows.

MacOS:

```bash
bash macos.sh all
```

Linux:

```bash
bash linux.sh
```

Windows:

```powershell
.\windows.ps1
```

Each combined script builds **and** installs the binary to your PATH:

- **macOS** builds for arm64 (default), x64, or all targets via subcommands:
  `bash macos.sh native` | `intel` | `universal` | `windows` | `linux` | `all`

- **Linux** builds a native x64 binary and installs it.

- **Windows** builds a native x64 binary and installs it.

Install output (macOS):

```
/Users/ales/.local/bin already in PATH
Installed:
  ./reefmt-macos-arm64 → /Users/ales/.local/bin/reefmt

Restart shell or run:
export PATH="/Users/ales/.local/bin:$PATH"
```

Install output (Windows):

```
C:\Users\ales\bin already in PATH
Installed to C:\Users\ales\bin\reefmt.exe

Restart terminal to use reefmt
```

