---
title: "File icons"
---

# File icons

<img src="/ree-file.svg" alt="The branded .ree file icon" width="64" height="64" />

The extension gives `.ree` files a branded red "R" icon. There are two ways to
get it, depending on which icon theme you use.

## Bundled icon theme

The extension ships a file-icon theme called **ree Icons**. Select it from the
Command Palette:

1. Run **Preferences: File Icon Theme**.
2. Choose **ree Icons**.

`.ree` files and `ree-templates` folders then show the branded icons.

## vscode-icons integration

If you prefer the popular
[vscode-icons](https://github.com/vscode-icons/vscode-icons) theme, the
extension also ships SVGs you can register as custom icons so `.ree` files keep
the red "R".

### 1. Create the custom-icons folder

vscode-icons reads custom icons from a folder named exactly
`vsicons-custom-icons` inside your VS Code user-data directory:

```bash
# macOS
mkdir -p "$HOME/Library/Application Support/Code/User/vsicons-custom-icons"

# Linux
mkdir -p "$HOME/.config/Code/User/vsicons-custom-icons"
```

```powershell
# Windows (PowerShell)
mkdir "$env:APPDATA\Code\User\vsicons-custom-icons"
```

Replace `Code` with `Code - Insiders`, `Code - OSS`, or `code-oss-dev` for those
variants.

### 2. Copy the SVGs

vscode-icons resolves custom icons by filename. Copy the extension's icon into
the three required names:

```bash
cp icons/ree-file.svg "<dir>/file_type_ree.svg"
cp icons/ree-file.svg "<dir>/folder_type_ree.svg"
cp icons/ree-file.svg "<dir>/folder_type_ree_opened.svg"
```

### 3. Add the associations

In your `settings.json`:

```json
{
	"vsicons.associations.files": [{ "icon": "ree", "extensions": ["ree"], "format": "svg" }],
	"vsicons.associations.folders": [{ "icon": "ree", "extensions": ["ree-templates"], "format": "svg" }]
}
```

### 4. Apply the customization

Run **Icons: Apply Icons Customization** from the Command Palette. VS Code
reloads and the custom icons appear.

## Troubleshooting

- Make sure the folder is named exactly `vsicons-custom-icons` and the filenames
  match the convention above.
- Confirm `workbench.iconTheme` is set to `vscode-icons`.
- Re-run **Icons: Apply Icons Customization** after any change.
- Ensure `settings.json` is valid JSON (no trailing commas) and reload the
  window.
