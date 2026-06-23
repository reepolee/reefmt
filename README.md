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

## Usage

reefmt formats `.ree` template files using a custom Rust-based AST parser. It also formats `.ts`, `.js`, and `.css` files via the built-in SWC formatter (no external tools needed).

reefmt is a pure Rust binary with zero runtime dependencies — no Node.js, no dprint, no biome required.

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

reefmt requires a `reefmt.jsonc` config file in your project root.
Run `reefmt --init` to generate one, then edit it to suit your project.
The file supports JSON comments (`//` and `/* */`) for inline documentation.

### Available options:

| Option | Type | Default (`--init`) | Description |
|---|---|---|---|
| `skipDirs` | `string[]` | `["node_modules", "vendor", "vendors", "dist", "templates", "static"]` | Directories to skip when formatting. Any folder matching a name in this list is skipped, regardless of its location in the project. |
| `skipFiles` | `string[]` | `[]` | Glob patterns for files to skip (e.g. `"generator/templates/**/*.ts"`). Matches file paths relative to the project root. |
| `extensions` | `string[]` | `["ree", "ts", "js", "css"]` | File extensions to format. |
| `skipDotDirs` | `boolean` | `true` | Skip directories whose name starts with a dot (e.g. `.git`, `.next`, `.cache`, `.svelte-kit`). |
| `wrapWidth` | `number` | `180` | Maximum line width before elements are broken onto multiple lines. |
| `collapseSingleStatementBlocks` | `boolean` | `true` | When enabled, single-statement blocks (`if`, `for`, `while`, etc.) and object literal function params (`fn({ key: val })`) collapse onto one line when they fit within `wrapWidth`. |
| `collapseMaxMembers` | `number` | `4` | Maximum number of object literal or type literal members before collapsing is prevented. With the default `4`, a 5+ member object literal stays multi-line regardless of `wrapWidth`. |
| `collapseSoftWidth` | `number` | `100` | "Soft" wrap width. Any collapsible structure (call args, array/object/type members, imports) whose single-line form fits within this width stays on one line **regardless** of the count caps above — so a short call with many short args collapses instead of exploding one-per-line. Above this width the count caps apply and `wrapWidth` is the hard ceiling. Set to `0` to disable (count caps always apply). |
| `tabWidth` | `number` | `4` | Display width of a tab character. The formatter indents with hard tabs, so a deeply nested line occupies more screen columns than its raw character count. Width measurements for `wrapWidth` and `collapseSoftWidth` expand tabs to this many columns, so those limits reflect where the last character actually lands on screen. Set it to match your editor's tab size. |
| `collapseMaxKeyValueProps` | `number` | `1` | Maximum number of `key: value` ("named") properties an object literal may have and still collapse onto one line. Shorthand (`{ a, b }`) and spread (`{ ...x }`) don't count. With the default `1`, `{ x: 1 }` stays inline but `{ x: 1, y: 2 }` always expands — inline lists of assignments are hard to scan. Set high to disable. |

### Example config:

```jsonc
{
	"skipDirs": ["node_modules", "vendor", "dist", ".output"],
	"extensions": ["ree", "ts", "js", "css", "jsx"],
	"skipDotDirs": true,
	"wrapWidth": 180,
	"collapseSingleStatementBlocks": true,
	"collapseMaxMembers": 4,
	"collapseSoftWidth": 100,
	"tabWidth": 4,
	"collapseMaxKeyValueProps": 1
}
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

