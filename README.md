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

reefmt also supports formatting from stdin with the `--stdin` flag, which is useful for editor integrations and pipe workflows.

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

### Diff mode:

Show a unified diff of the changes that would be made without modifying any files:

```bash
reefmt --diff
```

### Stdin mode:

Format input from stdin and write the result to stdout:

```bash
cat file.ree | reefmt --stdin        # default: format as Ree
cat file.ts  | reefmt --stdin .ts    # format as TypeScript
cat file.js  | reefmt --stdin .js    # format as JavaScript
cat file.css | reefmt --stdin .css   # format as CSS
```

If no extension argument is given, reefmt defaults to `.ree`.

### Init mode:

Generate a `reefmt.jsonc` config file with comments explaining each option:

```bash
reefmt --init
```

This creates a commented config file in the current directory that you can
edit to customize skip directories, file extensions, and dot-folder behavior.

## Configuration

reefmt can be configured with a `reefmt.jsonc` file in your project root.
Create one with `reefmt --init`, then edit it. The file supports JSON
comments (`//` and `/* */`) for inline documentation.

### Available options:

| Option | Type | Default | Description |
|---|---|---|---|
| `skipDirs` | `string[]` | `["node_modules", "vendor", "vendors", "dist"]` | Directories to skip when formatting. Any folder matching a name in this list is skipped, regardless of its location in the project. |
| `extensions` | `string[]` | `["ree", "ts", "js", "css"]` | File extensions to format. |
| `skipDotDirs` | `boolean` | `true` | Skip directories whose name starts with a dot (e.g. `.git`, `.next`, `.cache`, `.svelte-kit`). |

### Example config:

```jsonc
{
	"skipDirs": ["node_modules", "vendor", "dist", ".output"],
	"extensions": ["ree", "ts", "js", "css", "jsx"],
	"skipDotDirs": true
}
```

If no `reefmt.jsonc` is found, reefmt uses sensible defaults (same values
shown above). If the file is present but invalid, a warning is printed and
defaults are used.

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

