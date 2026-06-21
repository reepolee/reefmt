---
title: "Preview Server"
layout: "reeweb/docs/docs.layout"
---

# Preview Server

<a name="introduction"></a>

## Introduction

After building your site with `bun run build`, the preview server serves the static output from `dist/` so you can verify everything looks correct before deploying. It handles directory-style URLs (`/about/` → `about/index.html`), detects language subdirectories, and serves assets with proper MIME types.

```bash
bun run preview
```

By default this opens `http://localhost:3000` serving `./dist`.

<a name="cli-options"></a>

## CLI Options

```bash
bun scripts/preview.ts [--port 3000] [--dist ./dist]
```

| Flag           | Alias   | Default  | Description                |
| -------------- | ------- | -------- | -------------------------- |
| `--port <n>`   | `-p`    | `3000`   | Port the server listens on |
| `--dist <dir>` | `--dir` | `./dist` | Output directory to serve  |
| `--help`       | `-h`    | —        | Print usage and exit       |

<a name="how-it-works"></a>

## How It Works

The preview server (`scripts/preview.ts`) is a minimal Bun HTTP server that:

1. **Serves `dist/index.html`** at the root (`/`) — the default language homepage
2. **Resolves directory paths** — `/about/` looks for `dist/about/index.html`
3. **Falls back to `.html` extensions** — `/en/about` tries `dist/en/about.html`
4. **Serves any other file directly** — CSS, JS, images, fonts with correct MIME types
5. **Returns 404** for anything that doesn't exist

<a name="language-detection"></a>

## Language Detection

At startup, the preview server scans the `dist/` directory for subdirectories whose names match a configured language code in `config/supported_languages.ts` (e.g., `en/`, `sl/`, `de/`). Matching against the configured list — rather than any two-character directory — means content directories that happen to be two characters long (such as `js/`) are never mistaken for a language. It lists the detected languages in the startup log:

```
🖥️  Preview server ready at http://localhost:3000/
📂 Serving: ./dist
🌐 Languages: sl (default), en
```

The server doesn't apply language-specific logic — it simply serves files from the path the browser requests. If you navigate to `/en/about/`, it serves `dist/en/about/index.html` directly.

<a name="production-vs-development"></a>

## Production vs Development

The preview server is meant for verifying the **built** output — it's not a development server. The key differences:

|                   | Dev Server (`bun dev`)    | Preview Server (`bun run preview`) |
| ----------------- | ------------------------- | ---------------------------------- |
| Renders templates | ✅ Live, on-demand        | ❌ Static files only               |
| Live reload       | ✅ WebSocket              | ❌                                 |
| File watching     | ✅ Auto-refresh           | ❌                                 |
| Requires build    | ❌                        | ✅ Must run `bun run build` first  |
| Serves from       | `src/public/` + `static/` | `dist/`                            |
| Language routing  | ✅ Full resolution        | ✅ File-based                      |

<a name="mime-types"></a>

## MIME Types

The preview server maps file extensions to MIME types for correct `Content-Type` headers:

| Extension                 | Content-Type                            |
| ------------------------- | --------------------------------------- |
| `.html`                   | `text/html; charset=utf-8`              |
| `.css`                    | `text/css; charset=utf-8`               |
| `.js`                     | `application/javascript; charset=utf-8` |
| `.json`                   | `application/json; charset=utf-8`       |
| `.svg`                    | `image/svg+xml`                         |
| `.png`                    | `image/png`                             |
| `.jpg`, `.jpeg`           | `image/jpeg`                            |
| `.gif`                    | `image/gif`                             |
| `.webp`                   | `image/webp`                            |
| `.ico`                    | `image/x-icon`                          |
| `.woff2`, `.woff`, `.ttf` | `font/*`                                |
| `.txt`                    | `text/plain; charset=utf-8`             |
| `.xml`                    | `application/xml; charset=utf-8`        |
| (other)                   | `application/octet-stream`              |
