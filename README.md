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

`{{ ... }}` raw JS blocks in `.ree` files are formatted through the same SWC pipeline as `<script>` blocks, so nested object literals get proper indentation.

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

### CLI flags:

| Flag | Description |
|---|---|
| `--check`, `-c`, `--dry-run` | List files that would change without modifying them. Exits with code 1 if any file would change (useful for CI). |
| `--diff` | Show a unified diff of changes without writing files. |
| `--git` | Format only uncommitted (git-changed) files. |
| `--verbose` | Also print files that were already correctly formatted. |
| `--oneline <N>` | Override `oneline` from config for this run. |
| `--wrap-width <N>` | Override `wrapWidth` from config for this run. |
| `--collapse-max-members <N>` | Override `collapseMaxMembers` from config for this run. |
| `--collapse-soft-width <N>` | Override `collapseSoftWidth` from config (`0` disables). |
| `--tab-width <N>` | Override `tabWidth` from config. |
| `--collapse-max-keyvalue-props <N>` | Override `collapseMaxKeyValueProps` from config. |
| `--stdin [.ext]` | Read from stdin, write to stdout. Extension defaults to `.ree`. |
| `--init` | Generate a `reefmt.jsonc` config file in the current directory. |
| `--version` | Print the version. |
| `--help`, `-h` | Print usage. |

## Configuration

reefmt requires a `reefmt.jsonc` config file in your project root.
Run `reefmt --init` to generate one, then edit it to suit your project.
The file supports JSON comments (`//` and `/* */`) for inline documentation.

### Available options:

| Option | Type | Default | Description |
|---|---|---|---|
| `skipDirs` | `string[]` | `["node_modules", "vendor", "vendors", "dist", "templates", "static"]` | Directories to skip when formatting. Any folder matching a name in this list is skipped, regardless of its location in the project. |
| `skipFiles` | `string[]` | `[]` | Glob patterns for files to skip (e.g. `"generator/templates/**/*.ts"`). Matches file paths relative to the project root. |
| `skipExtensions` | `string[]` | `["min.js", "min.css"]` | Compound extensions to always skip, regardless of `extensions`. A file ending in `.min.js` is skipped even if `js` is in `extensions`. |
| `extensions` | `string[]` | `["ree", "ts", "js", "css"]` | File extensions to format. |
| `skipDotDirs` | `boolean` | `true` | Skip directories whose name starts with a dot (e.g. `.git`, `.next`, `.cache`, `.svelte-kit`). |
| `wrapWidth` | `number` | `180` | Maximum line width before elements are broken onto multiple lines. |
| `collapseSingleStatementBlocks` | `boolean` | `true` | When enabled, single-statement blocks (`if`, `for`, `while`, etc.) and object literal function params (`fn({ key: val })`) collapse onto one line when they fit within `wrapWidth`. |
| `collapseMaxMembers` | `number` | `4` | Global fallback maximum member count for all collapse categories below. Any category not explicitly configured falls back to this value. |
| `collapseMaxObjectMembers` | `number` | _(falls back to `collapseMaxMembers`)_ | Maximum members in an object literal before it stays multi-line. |
| `collapseMaxArrayElements` | `number` | _(falls back to `collapseMaxMembers`)_ | Maximum elements in an array literal before it stays multi-line. |
| `collapseMaxFunctionParams` | `number` | _(falls back to `collapseMaxMembers`)_ | Maximum parameters in a function definition before it stays multi-line. |
| `collapseMaxCallArgs` | `number` | _(falls back to `collapseMaxMembers`)_ | Maximum arguments in a function call before it stays multi-line. |
| `collapseMaxImports` | `number` | _(falls back to `collapseMaxMembers`)_ | Maximum named imports before the import stays multi-line. |
| `collapseMaxTypeMembers` | `number` | _(falls back to `collapseMaxMembers`)_ | Maximum members in a type literal before it stays multi-line. |
| `collapseMaxKeyValueProps` | `number` | `1` | Maximum number of `key: value` ("named") properties an object literal may have and still collapse onto one line. Shorthand (`{ a, b }`) and spread (`{ ...x }`) don't count. With the default `1`, `{ x: 1 }` stays inline but `{ x: 1, y: 2 }` always expands. Set high to disable. |
| `collapseSoftWidth` | `number` | `100` | "Soft" collapse width. Any collapsible structure whose inline form fits within this column count collapses onto one line regardless of the count caps above. Above this width the count caps apply. Set to `0` to disable. |
| `oneline` | `number` | `0` | Column-width threshold for collapsing multi-line leaf HTML elements in `.ree` files. A leaf element (no child tags) is collapsed to one line only when its inline form fits within this many columns. Also acts as the hard ceiling for JS/TS collapse decisions (structures won't collapse if the result would exceed this width). Set to `0` to disable. Can be overridden on the command line with `--oneline <N>`. |
| `tabWidth` | `number` | `4` | Display width of a tab character. The formatter indents with hard tabs, so a deeply nested line occupies more screen columns than its raw character count. Width measurements for `wrapWidth`, `collapseSoftWidth`, and `oneline` expand tabs to this many columns. Set it to match your editor's tab size. |
| `removeUnusedImports` | `boolean` | `false` | When enabled, unused import declarations are removed from JS/TS files during formatting. Side-effect imports (`import "./foo"`) are always kept. |

### Example config:

```jsonc
{
	"skipDirs": ["node_modules", "vendor", "dist", ".output"],
	"skipExtensions": ["min.js", "min.css"],
	"extensions": ["ree", "ts", "js", "css"],
	"skipDotDirs": true,
	"wrapWidth": 180,
	"collapseSingleStatementBlocks": true,
	"collapseMaxMembers": 4,
	"collapseMaxObjectMembers": 4,
	"collapseMaxArrayElements": 4,
	"collapseMaxFunctionParams": 4,
	"collapseMaxCallArgs": 4,
	"collapseMaxImports": 4,
	"collapseMaxTypeMembers": 4,
	"collapseMaxKeyValueProps": 1,
	"collapseSoftWidth": 100,
	"oneline": 180,
	"tabWidth": 4,
	"removeUnusedImports": false
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

## onpull

There are two scripts, one for Windows, the other for Linux and MacOS

onpull.sh
onpull.ps1

