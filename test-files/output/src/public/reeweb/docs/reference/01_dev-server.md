---
title: "Dev Server"
layout: "reeweb/docs/docs.layout"
---

# Dev Server

<a name="introduction"></a>

## Introduction

Reeweb's development server serves templates and markdown files directly — no build step required. It renders `.ree` and `.md` files on demand, provides live reload via WebSocket when you edit sources, and serves static assets from both `src/public/` and the project root `static/` directory.

The dev server is started via the `bun dev` command, which uses `conc` (concurrently) to run two processes in parallel:

1. **`bun css:watch`** — the Tailwind CSS watcher, recompiling on every file change
2. **`bun development`** (`scripts/dev.ts`) — handles HTTP requests with live reload

If you need the auto-restart behaviour that watches `scripts/dev.ts`, `.env`, and `bunfig.toml` for changes, use `bun run launcher` instead — that runs `scripts/dev_watcher.ts` directly.

<a name="starting-the-dev-server"></a>

## Starting the Dev Server

```bash
bun dev
```

This single command runs the Tailwind watcher and the dev server side-by-side. Open `http://localhost:3000` in a browser.

The default port is `3000`. The default source directory is `./src/public`.

<a name="cli-options"></a>

## CLI Options

The dev server accepts these flags when invoked directly:

```bash
bun scripts/dev.ts [--port 3000] [--public ./src/public]
```

| Flag             | Alias   | Default        | Description                                |
| ---------------- | ------- | -------------- | ------------------------------------------ |
| `--port <n>`     | `-p`    | `3000`         | Port the server listens on                 |
| `--public <dir>` | `--dir` | `./src/public` | Source directory with templates and assets |
| `--help`         | `-h`    | —              | Print usage and exit                       |

All flags are optional. Running `bun dev` passes no flags and uses the defaults.

<a name="dev-watcher"></a>

## Dev Watcher (`scripts/dev_watcher.ts`)

The dev watcher is an alternative launcher available as `bun run launcher`. It:

1. **Kills any previous server** by reading a `.server.pid` file in the project root
2. **Starts the Tailwind watcher**: `tailwindcss -i ./src/css/style.css -o ./src/public/css/style.min.css --watch --verbose`
3. **Starts the dev server**: `bun scripts/dev.ts`
4. **Writes the PID** of the dev server to `.server.pid` so subsequent restarts can kill it
5. **Watches key files** for changes and auto-restarts:
    - `scripts/dev.ts` — server code changes trigger a restart
    - `.env` — environment variable changes trigger a restart
    - `bunfig.toml` — Bun configuration changes trigger a restart
6. **Enforces a 2-second cooldown** between restarts to prevent rapid cycling

### Keyboard Shortcuts

When running in a terminal (TTY), the dev watcher supports:

| Key       | Action                                                |
| --------- | ----------------------------------------------------- |
| `o` / `O` | Opens `http://localhost:3000` in the default browser  |
| `Ctrl+C`  | Kills the Tailwind watcher, the dev server, and exits |

<a name="live-reload"></a>

## Live Reload

The dev server injects a WebSocket client script into every rendered HTML page. When a source file changes, the server broadcasts a reload message to all connected WebSocket clients, and the browser refreshes automatically.

### How It Works

1. The dev server runs a WebSocket endpoint at `/__reload`
2. Every rendered page receives an injected `<script>` that connects to this WebSocket
3. The file watcher monitors `src/public/` for changes to `.ree`, `.md`, `.json`, and `.ts` files
4. Changes are debounced at **100ms** — rapid edits don't trigger multiple reloads
5. When a `.json` file changes, translations are **hot-reloaded** without needing a full restart — the route map and language names are rebuilt in-memory

### What Triggers a Reload

| File Change           | Behavior                                                              |
| --------------------- | --------------------------------------------------------------------- |
| `.json` (translation) | Hot-reloads translations + route map, then triggers browser refresh   |
| `.ree` (template)     | Triggers browser refresh (template re-read from disk on next request) |
| `.md` (markdown)      | Triggers browser refresh                                              |
| `.ts` (data loader)   | Triggers browser refresh                                              |

### What Triggers a Full Restart (via dev watcher)

| File Change      | Behavior                                                 |
| ---------------- | -------------------------------------------------------- |
| `scripts/dev.ts` | Full restart of both the Tailwind watcher and dev server |
| `.env`           | Full restart (environment variables are read at startup) |
| `bunfig.toml`    | Full restart                                             |

<a name="language-resolution"></a>

## Language Resolution

The dev server resolves language from the URL path following the same rules as the static build:

```
/                  → Slovenian (default) at root
/about/            → Slovenian /about/
/en/               → English homepage
/en/about/         → English /about/
```

The resolution algorithm:

1. Check if the first URL segment matches a language code in `config/supported_languages.ts`
2. If it does, set the active language to that code and strip the prefix
3. If not, use the `default_language` (configured in `supported_languages.ts`)

The resolved language + canonical path is then looked up in the route map (which includes localized `route_name` substitutions) to find the correct template or markdown file.

<a name="built-in-data-variables"></a>

## Built-in Data Variables

Every template rendered by the dev server receives these variables automatically in its data context. You access them as `props.xxx` in templates:

| Variable                    | Source                      | Description                                                         |
| --------------------------- | --------------------------- | ------------------------------------------------------------------- |
| `props.lang`                | URL resolution              | Active language code (`"en"`, `"sl"`)                               |
| `props.lang_url_prefix`     | Derived                     | `""` for default language, `"/sl"` for others                       |
| `props.locale`              | `language_locales`          | BCP-47 locale string (`"en-US"`, `"sl-SI"`)                         |
| `props.active_languages`    | Config                      | List of languages shown in the language switcher                    |
| `props.language_names`      | Config                      | Map of code → display name                                          |
| `props.language_self_names` | Derived                     | Map of code → native name from each language's own translation file |
| `props.default_language`    | Config                      | Default language code                                               |
| `props.base_url`            | Hard-coded                  | Always `"/"`                                                        |
| `props.site_url`            | Hard-coded                  | Always `""` (set in production build)                               |
| `props.hreflang_links`      | Hard-coded                  | Always `[]` (set in production build)                               |
| `props.site_name`           | Hard-coded                  | `"Dev"`                                                             |
| `props.year`                | Runtime                     | Current year for copyright                                          |
| `props.is_dev`              | Hard-coded                  | `true`                                                              |
| `props.rendered_at`         | Runtime                     | ISO string of when the render happened                              |
| `props.request_url`         | Resolved                    | Full URL of the current page, including language prefix             |
| `props.canonical_path`      | Resolved                    | Canonical path (without language prefix)                            |
| `props.language_urls`       | Derived                     | Map of code → URL prefix (`""` for default, `"/{lang}"` for others) |
| `props.helpers`             | `create_template_helpers()` | Object of template helper functions                                 |

<a name="template-resolution"></a>

## Template Resolution

The dev server resolves templates in this order:

1. **Hash map lookup** — checks the pre-built canonical → template map
2. **Reverse route map** — if the path is a localized route, resolves back to canonical
3. **Direct file check** — tries `{path}.ree`, `{path}.md`
4. **Index file check** — tries `{path}/index.ree`, `{path}/index.md`
5. **404** — if nothing matches, returns a `404 Not Found` response

### Language-Variant Templates

Templates can have language-specific variants. The resolution chain:

```
{name}.{requestedLang}.ree → {name}.{default_language}.ree → {name}.ree
```

For example, with `lang=de` and `default_language=sl`:

1. Try `about/index.de.ree` — not found
2. Try `about/index.sl.ree` — found (used)
3. Try `about/index.ree` — fallback

This applies to every template load — pages, layouts, includes, and components.

<a name="static-file-serving"></a>

## Static File Serving

The dev server serves static files from two locations:

1. **`src/public/`** — templates and their sibling assets (CSS, images)
2. **`static/`** — project-level static assets (compiled CSS, JS, favicon)

Static files are served with `Cache-Control: no-cache` headers to ensure changes appear immediately during development. The server checks `src/public/` first — if a file exists there and isn't a `.ree`, `.md`, `.json`, or `.ts` file, it's served directly. If not found, it falls back to `static/`.

<a name="generated-build-artifacts"></a>

## Generated Build Artifacts (sitemap & feeds)

`sitemap.xml` and the RSS/JSON feeds (`feed.xml`, `feed.json`) are **build artifacts** — they're emitted to `dist/` by `bun run sitemap` / `bun run rss`, not served from `src/public/`. To avoid 404s on links like `/sitemap.xml` during development, the dev server serves the last-built copy straight from `dist/` as a convenience:

| Request path                | Served from                     |
| --------------------------- | ------------------------------- |
| `/sitemap.xml`              | `dist/sitemap.xml`              |
| `*/feed.xml`, `*/feed.json` | the matching file under `dist/` |

These copies are **stale** until you regenerate them — run `bun run build:dist` (or `bun run sitemap` / `bun run rss`) to refresh `dist/`. If the artifact hasn't been built yet, the dev server returns a `404` with a hint telling you which command produces it.

`robots.txt` is intentionally **not** served from `dist/`: in dev it's served from the source `src/public/robots.txt` (which contains `Disallow: /`) so the dev server stays unindexable.

<a name="error-handling"></a>

## Error Handling

- **500 errors** — Template compilation errors (syntax errors, missing includes, unclosed blocks) return a `500 Error` page with the error message details
- **404 errors** — Unknown paths return a simple `404 Not Found` page
- **Missing source directory** — If `--public` points to a non-existent directory, the server exits with an error immediately
