---
title: "Build Pipeline"
layout: "reeweb/docs/docs.layout"
---

# Build Pipeline

<a name="introduction"></a>

## Introduction

Running `bun run build` executes `scripts/build.ts` — the static site generator. The build script runs in a series of phases, each with a specific responsibility. Understanding this pipeline helps you debug build issues and know when each piece of configuration is consumed.

<a name="quick-reference"></a>

## Quick Reference

```bash
bun scripts/build.ts [--public ./src/public] [--dist ./dist] [--base-url /] [--site-url https://example.com] [--verbose]
```

| Option             | Default              | Description                                                        |
| ------------------ | -------------------- | ------------------------------------------------------------------ |
| `--public <dir>`   | `./src/public`       | Source directory with `.ree` templates and `.md` files             |
| `--dist <dir>`     | `./dist`             | Output directory for static HTML                                   |
| `--base-url <url>` | `/`                  | Base URL for the site                                              |
| `--site-url <url>` | `$SITE_URL` or empty | Full site URL for hreflang links (required for multi-language SEO) |
| `--verbose`        | off                  | Log each rendered file                                             |
| `--help`           | —                    | Print usage and exit                                               |

If `--site-url` is not provided, hreflang alternate links are skipped. Google requires absolute URLs for hreflang, so the build emits a warning when it's missing.

<a name="phase-1-validate-redirects"></a>

## Phase 1: Validate Redirects

**Input:** `config/redirects.ts`

The build first schema-validates the redirects array. This is done **before any rendering work** so config errors fail fast rather than after a long build.

Validation checks:

- `redirects` is an array
- Each entry has a `from` (string, starts with `/`, no file extensions in last segment) and `to` (non-empty string)
- `status` (optional) is `301` or `302`
- No duplicate `from` paths

If validation fails, the build exits immediately with a clear error message pointing to the specific entry.

<a name="phase-2-clear-output"></a>

## Phase 2: Clear Output Directory

The `dist/` directory is deleted and recreated. This ensures stale files from previous builds don't linger and accidentally get deployed.

<a name="phase-3-load-translations"></a>

## Phase 3: Load Translations

**Input:** `src/public/` directory — walks all subdirectories for `{lang}.json` files

The `load_all_translations()` function:

1. Walks the entire `src/public/` directory tree
2. Discovers every `{lang}.json` file
3. Groups by language code, keyed by directory namespace
4. Runs cross-language fallback — missing keys in one language inherit from any language that has them

The loader also derives:

- `language_self_names` — each language's own name from its translation file
- `language_urls` — URL prefixes per language (empty string for default, `/{lang}` for others)

<a name="phase-4-collect-files"></a>

## Phase 4: Collect Files

The builder scans `src/public/` and separates files into three categories:

1. **`.ree` templates** — files that the template engine compiles and renders
2. **`.md` markdown files** — files rendered via `Bun.markdown.html()` with `process_docs_markdown` post-processing
3. **Static assets** — everything else (images, fonts, CSS, JS, PDFs) — copied verbatim

The `collect_page_files()` helper:

- Skips top-level `layout.ree` and `*.layout.ree` files (used only via `{#layout()}`)
- Collapses language-variant siblings into one canonical entry (e.g., `about.en.ree` + `about.sl.ree` → `about.ree`)

<a name="phase-5-load-data"></a>

## Phase 5: Load Dynamic Data

**Input:** `.ts` files with the same base name as `.ree` templates

For every `.ree` template, the builder checks for a sibling `.ts` file with the same name (e.g., `index.ree` → `index.ts`). If found and it exports `load_template_data()`, the function is called and its return value is stored for that template.

The convention is documented in [Project Structure](/reeweb/docs/getting-started/project-structure#data-loading). On Windows, the import uses `pathToFileURL()` to ensure compatibility with Bun's module resolution.

<a name="phase-6-build-route-map"></a>

## Phase 6: Build Route Map

**Input:** Translations + collected template files

`build_static_route_map()` walks each canonical path segment-by-segment and substitutes `route_name` from translations where present. The result is a map of:

```
canonical_path → { language → localized_path }
```

This is used:

- To generate correct per-language URLs for rendered pages
- To build hreflang alternate links
- By the `localized_path()` helper in templates
- By the `localized_path_for_lang()` helper for language switchers

<a name="phase-7-render-templates"></a>

## Phase 7: Render Templates

For each `.ree` file, the build renders it for **every configured language**:

1. Merge global translations with route-specific translations
2. Resolve the localized path for this language
3. Build hreflang links (only when `--site-url` is set)
4. Merge data from the data loader (`load_template_data()`)
5. Call the template engine to compile and render
6. Write the output to `dist/` as `{localized_path}/index.html`

Special cases:

- **Default language** at root: `/index.html`, `/about/index.html`
- **Other languages** with prefix: `/en/index.html`, `/en/about/index.html`
- **Home page** (`/`): always `index.html` or `{lang}/index.html`

<a name="phase-8-build-sidebar"></a>

## Phase 8: Build Sidebar Navigation

The builder scans markdown `index.md` files for the `has_sidebar: true` frontmatter flag. When found:

1. Lists all `.md` files in that folder
2. Sorts them alphabetically (use ordering prefixes like `01_`, `02_` to control order)
3. Resolves language-specific titles
4. Excludes pages with `skip-navigation: true`
5. Builds a per-language sidebar link list

The sidebar data is passed to the markdown layout template as `props.sidebar`.

<a name="phase-9-render-markdown"></a>

## Phase 9: Render Markdown

For each collected `.md` file, the build renders it for every language:

1. Resolves language-specific variant (`file.{lang}.md` → `file.{default}.md` → `file.md`)
2. Parses frontmatter for layout selection and metadata
3. Renders markdown body via `Bun.markdown.html()` with options for tables, strikethrough, tasklists, autolinks, and heading IDs
4. Post-processes with `process_docs_markdown(raw_html, markdown_styles)` — syntax highlighting, external link handling, and injection of the project's Tailwind classes from `src/lib/markdown_styles.ts`
5. Resolves the layout template (from frontmatter or default `"layout"`)
6. Renders the layout with the processed HTML as `props.body`
7. Writes to `dist/` following the same path rules as templates

<a name="phase-10-copy-static-files"></a>

## Phase 10: Copy Static Files

Every non-template, non-translation file from `src/public/` is copied verbatim to `dist/` preserving the directory structure. This includes:

- CSS files (`src/public/css/style.css` → `dist/css/style.css`)
- Images, fonts, PDFs
- Client-side JavaScript files
- Root-level files (`favicon.ico`, `robots.txt`, `sitemap.xml`)

<a name="phase-11-emit-redirects"></a>

## Phase 11: Emit Redirects

**Output:** `dist/_redirects` + HTML stub files

This is the second phase of redirect processing (Phase 1 validated the schema; Phase 2 now checks for collisions):

1. **Collision checks**: Verifies no `from` path collides with a generated page route or a static asset
2. **Target validation**: For internal `to` targets, verifies the file exists in `dist/`
3. **`_redirects` file**: Writes Cloudflare-compatible redirect rules — two lines per entry (with and without trailing slash)
4. **HTML stubs**: For each redirect, creates `dist/{from}/index.html` with a meta-refresh tag. This provides redirect behavior for non-Cloudflare hosts and local preview

External URL targets are not fetched or validated at build time.

<a name="error-handling"></a>

## Error Handling

The build script fails fast — if any phase encounters an error, the build stops immediately and exits with a non-zero status code. Common error scenarios and what they look like:

### Template compilation errors

When a `.ree` template has a syntax error (unclosed `{#each}`, invalid expression, malformed layout tag), the template engine throws with a clear error message at compile time. The build log shows the template name, the line number if possible, and the specific syntax error:

```
✗ Build failed
  Template compilation failed in "about/index.ree":
  Unclosed block(s): each
```

The full generated JavaScript that failed to compile is also available in the error output — you can inspect it to see exactly what the engine produced and find the mismatch. This is primarily useful when debugging complex nested blocks.

### Redirect validation errors

If `config/redirects.ts` has a malformed entry — a `from` path that doesn't start with `/`, a duplicate redirect, or a `to` target that's empty — Phase 1 catches it before any rendering starts:

```
✗ Redirect validation failed:
  Entry #3: "from" path must start with "/" (got "old-page")
```

Fix the `config/redirects.ts` entry and re-run the build.

### Missing data loader errors

A `.ts` data loader that throws at build time (a missing import, a network error in `load_template_data()`) causes the build to fail when it tries to import and execute the module:

```
✗ Failed to load data for "index.ree":
  Cannot find module '$config/some_file' imported from 'src/public/index.ts'
```

Check the import path in the `.ts` file — relative imports should work from the `src/public/` directory.

### Missing translation errors

Missing translations don't cause build failures — the fallback merge fills missing keys from any language that has them. A warning is logged during Phase 3 for each missing file, but the build continues:

```
⚠ Translation file not found: src/public/de.json (will fall back to other languages)
```

### Exit codes

| Code | Meaning                                      |
| ---- | -------------------------------------------- |
| `0`  | Build succeeded                              |
| `1`  | Build failed — check the error message above |

In CI/CD pipelines, a non-zero exit code stops the deployment automatically.

<a name="utility-functions"></a>

## Utility Functions

The build scripts and the shared `lib/static_site.ts` module export several helper functions that are useful for custom build extensions, custom data loaders, or understanding how the pipeline works:

### `walk_dir(root)`

Recursively walks a directory and returns all file paths relative to the root:

```ts
import { walk_dir } from "$lib/static_site";

const files = walk_dir("./src/public");
// ["index.ree", "about/index.ree", "blog/post.md", "css/style.css", ...]
```

### `template_to_canonical(rel_path)`

Converts a template file path to its canonical route, stripping ordering prefixes (`\d+_`) from each segment:

```ts
import { template_to_canonical } from "$lib/static_site";

template_to_canonical("index.ree"); // "/"
template_to_canonical("about/index.ree"); // "/about"
template_to_canonical("docs/05_auth.md"); // "/docs/auth"
template_to_canonical("blog/029_post-title.md"); // "/blog/post-title"
```

This is how the build pipeline derives the output URL path from a file's position in the source tree. The ordering prefixes (`01_`, `05_`, `029_`) are stripped so they don't appear in the final URL.

### `path_to_namespace(rel_path)`

Converts a template path to a translation namespace (dot-separated, with `index` stripped):

```ts
import { path_to_namespace } from "$lib/static_site";

path_to_namespace("blog/post.ree"); // "blog.post"
path_to_namespace("blog/index.ree"); // "blog"
path_to_namespace("index.ree"); // ""
```

This is used to find the right translation keys for a page. The namespace `blog.post` maps to `translations[lang]["blog"]["post"]` in the translation tree.

### `file_mtime_iso_date(file_path)`

Returns the file's modification time as an ISO date string (`YYYY-MM-DD`). Used by the sitemap generator for `<lastmod>` when no frontmatter override is supplied:

```ts
import { file_mtime_iso_date } from "$lib/static_site";

file_mtime_iso_date("./src/public/blog/post.md");
// → "2026-05-15"
```

### `read_frontmatter(file_path)`

Reads a file from disk and returns its parsed frontmatter. Returns an empty object if the file doesn't exist or has no frontmatter block:

```ts
import { read_frontmatter } from "$lib/static_site";

const fm = read_frontmatter("./src/public/blog/post.md");
// { title: "Post Title", date: "2026-05-15", author: "Alice" }
```

This is useful in custom build scripts or data loaders that need to inspect page metadata without re-parsing the file yourself.

### `collect_page_files(public_dir, languages, extensions?)`

Collects all renderable page files, skipping layout files and collapsing language variants into canonical entries:

```ts
import { collect_page_files } from "$lib/static_site";

const pages = collect_page_files("./src/public", ["en", "sl"], ["ree", "md"]);
// ["index.ree", "about/index.ree", "about/index.en.ree" collapses to "about/index.ree", ...]
```

This is what Phase 4 of the build pipeline uses to discover every page that needs rendering. The `extensions` parameter defaults to `["ree", "md"]`.

### `build_static_route_map(translations, page_files, languages)`

Builds a map of canonical paths → (language → localized path), walking the translation tree segment-by-segment and substituting `route_name` where present:

```ts
import { build_static_route_map } from "$lib/static_site";

const route_map = build_static_route_map(translations, page_files, languages);
// Map { "/about" => Map { "en" => "/about", "sl" => "/o-nas" } }
```

This is the data structure that drives localized URL generation and hreflang links in Phase 6.

<a name="output-structure"></a>

## Output Structure

The final `dist/` directory:

```
dist/
├── index.html              ← Default language homepage
├── about/
│   └── index.html          ← Default language /about/
├── en/
│   ├── index.html          ← English homepage
│   └── about/
│       └── index.html      ← English /about/
├── css/
│   └── style.css           ← Compiled Tailwind CSS
├── _redirects              ← Redirect rules (Cloudflare format)
├── favicon.ico
├── sitemap.xml             ← Generated sitemap
└── ...                     ← Other static assets
```

Each page is rendered as `index.html` inside a directory named after the URL path — the standard pattern for static hosting.
