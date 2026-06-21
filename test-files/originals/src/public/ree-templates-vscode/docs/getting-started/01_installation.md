---
title: "Installation"
---

# Installation

The Ree Templates extension is published on the Visual Studio Marketplace under
the id `reepolee.ree-templates`. It requires VS Code 1.107 or newer.

## Install the extension

Install from inside VS Code:

1. Open the **Extensions** view (`⇧⌘X` / `Ctrl+Shift+X`).
2. Search for **Ree Templates**.
3. Click **Install**.

Or install from the
[Marketplace page](https://marketplace.visualstudio.com/items?itemName=reepolee.ree-templates),
or from the command line:

```bash
code --install-extension reepolee.ree-templates
```

## Install Reefmt (for formatting)

Syntax highlighting works immediately. **Formatting** is provided by the
[Reefmt](/reefmt/docs) binary, which the extension calls under the hood — so
install Reefmt as well:

```bash
git clone https://github.com/reepolee/reefmt.git
cd reefmt
bash install.sh   # or .\install.ps1 on Windows
```

See the [Reefmt installation guide](/reefmt/docs/getting-started/installation)
for all platforms. If Reefmt is on your `PATH`, the extension finds it
automatically; otherwise point at it with the
[`ree.reefmtPath`](/ree-templates-vscode/docs/configuration/settings) setting.

## Open a Ree file

Open any file with the `.ree` extension. VS Code recognises it as the **Ree
Template** language (`ree`), applies syntax highlighting, and sets this
extension as the default formatter for `.ree` files.

Next, explore [Syntax highlighting](/ree-templates-vscode/docs/features/syntax-highlighting),
[Formatting](/ree-templates-vscode/docs/features/formatting), and
[File icons](/ree-templates-vscode/docs/features/icons).
