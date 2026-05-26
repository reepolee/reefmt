# reefmt

A formatter for Ree Templates.

Used by https://marketplace.visualstudio.com/items?itemName=reepolee.ree-templates VSCode extension-

Check out https://www.reepolee.com for more information.

## Development

This is a Rust project, we make cross builds on MacOS and native on Windows.

MacOS:

```bash

bash build.sh all
```

Windows:

```bash

.\build.ps1
```

Install scripts should put them on PATH:

MacOS:

```bash
bash install.sh
```

Outputs:

```
/Users/ales/.local/bin already in PATH
Installed:
  ./reefmt-macos-arm64 → /Users/ales/.local/bin/reefmt

Restart shell or run:
export PATH="/Users/ales/.local/bin:$PATH"
```

Windows:

```bash
.\install.ps1
```

Outputs:

```
C:\Users\ales\bin already in PATH
Installed to C:\Users\ales\bin\reefmt.exe

Restart terminal to use reefmt
```

