---
title: "Scripts Reference"
layout: "reeweb/docs/docs.layout"
---

# Scripts Reference

<a name="introduction"></a>

## Introduction

Reeweb's `package.json` defines a set of scripts for building, developing, and maintaining your site. This page lists every script with its exact command and purpose.

<a name="development"></a>

## Development Scripts

| Script             | Command                                                                                                | Description                                                                                                         |
| ------------------ | ------------------------------------------------------------------------------------------------------ | ------------------------------------------------------------------------------------------------------------------- |
| `bun dev`          | `conc -n img,tw,dev -c magenta,yellow,blue "bun run prepare:images" "bun css:watch" "bun development"` | Generate responsive images, then run the CSS watcher and dev server concurrently, with colour-coded output          |
| `bun run launcher` | `bun scripts/dev_watcher.ts`                                                                           | Alternative launcher that coordinates processes via `dev_watcher.ts` (auto-restart on `.env`/`bunfig.toml` changes) |
| `bun run preview`  | `bun scripts/preview.ts`                                                                               | Serve the built `dist/` output locally to verify the production build                                               |

### How `bun dev` Works

`bun dev` uses `conc` (the `concurrently` package) to start three processes in parallel:

1. **`bun run prepare:images`** — generates responsive image variants; runs non-blocking so the server starts instantly (near-instant on warm runs)
2. **`bun css:watch`** — the Tailwind CSS watcher, recompiling on every file change
3. **`bun development`** — the dev server (`scripts/dev.ts`), serving templates with live reload

Both processes run in the same terminal with colour-coded prefixes so their output is easy to distinguish. Open `http://localhost:3000` once both are running.

If you need auto-restart behaviour when `scripts/dev.ts`, `.env`, or `bunfig.toml` change, use `bun run launcher` instead — it runs `scripts/dev_watcher.ts`, which watches those files and restarts the server automatically.

The dev server is documented in detail on the [Dev Server](/reeweb/docs/reference/dev-server) page.

<a name="build-scripts"></a>

## Build Scripts

| Script                   | Command                                                                               | Description                                                                      |
| ------------------------ | ------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------- |
| `bun run prepare:images` | `bun scripts/prepare_images.ts`                                                       | Generate responsive image variants from `assets/images/` (see below)             |
| `bun run build`          | `bun run prepare:images && bun scripts/build.ts --public ./src/public --dist ./dist`  | Static site build — renders templates + markdown, copies assets, emits redirects |
| `bun run build:dist`     | `bun run prepare:images && bun build:css && bun scripts/build.ts … && sitemap && rss` | Full production build with fresh images, CSS, sitemap, and RSS feed              |

### Responsive Images (`prepare:images`)

`scripts/prepare_images.ts` turns full-size originals in `assets/images/` into
width-stepped WebP + JPEG variants in `src/public/images/responsive/`
(git-ignored), using Bun's native `Bun.Image` — no external dependency. It runs
**before** `build`/`build:dist` (blocking) and **concurrently** inside `dev`
(non-blocking, so the server starts instantly), and is incremental (mtime-based):
warm runs are near-instant, and editing one original rebuilds only that image.
Widths and quality live in `config/responsive_images.ts`. See the
[Responsive Images](/reeweb/docs/recipes/responsive-images) recipe for detail.

| Flag                                        | Default                         | Description                       |
| ------------------------------------------- | ------------------------------- | --------------------------------- |
| `--widths <list>`                           | from `config/responsive_images` | Comma-separated width breakpoints |
| `--quality <n>`                             | from `config/responsive_images` | Quality for both formats (1-100)  |
| `--quality-webp <n>` / `--quality-jpeg <n>` | `80` / `80`                     | Per-format quality                |
| `--force`                                   | off                             | Re-encode even if outputs exist   |

### Build Options

All build scripts accept these options via command-line flags:

```bash
bun scripts/build.ts [options]
```

| Flag               | Default        | Description                            |
| ------------------ | -------------- | -------------------------------------- |
| `--public <dir>`   | `./src/public` | Source directory with `.ree` templates |
| `--dist <dir>`     | `./dist`       | Output directory for static HTML       |
| `--base-url <url>` | `/`            | Base URL for the site                  |
| `--site-url <url>` | empty          | Full site URL for hreflang links       |
| `--verbose`        | off            | Log each rendered file                 |

Pass `--verbose` directly for verbose output:

```bash
bun scripts/build.ts --public ./src/public --dist ./dist --verbose
```

The build pipeline is documented on the [Build Pipeline](/reeweb/docs/reference/build-pipeline) page.

<a name="css-scripts"></a>

## CSS Scripts

| Script              | Command                                                                                  | Description                                                             |
| ------------------- | ---------------------------------------------------------------------------------------- | ----------------------------------------------------------------------- |
| `bun run css:build` | `tailwindcss -i ./src/css/style.css -o ./src/public/css/style.min.css --minify`          | One-time CSS build (minified), output to `src/public/css/style.min.css` |
| `bun run css:watch` | `tailwindcss -i ./src/css/style.css -o ./src/public/css/style.min.css --watch --verbose` | Watch mode — recompiles on every file change, same output path          |
| `bun run build:css` | `tailwindcss -i ./src/css/style.css -o ./src/public/css/style.css --minify`              | Production CSS variant used by `build:dist`                             |

Note: `bun run build` does **not** automatically run a CSS build — it renders templates only. If you want a complete production build with fresh CSS, use `bun run build:dist`. Alternatively, run `bun run css:build` manually before deploying.

The Tailwind setup is documented on the [Tailwind CSS Setup](/reeweb/docs/styling/tailwind-setup) page.

<a name="seo-scripts"></a>

## SEO Scripts

| Script            | Command                                                               | Description                                |
| ----------------- | --------------------------------------------------------------------- | ------------------------------------------ |
| `bun run sitemap` | `bun scripts/generate_sitemap.ts --public ./src/public --dist ./dist` | Generate `sitemap.xml`                     |
| `bun run rss`     | `bun scripts/generate_rss.ts --public ./src/public --dist ./dist`     | Generate RSS/JSON feeds from blog markdown |

Both should be run **after** `bun run build` — they read from the built `dist/` directory. Neither command hardcodes a site URL; they read the absolute origin from the `SITE_URL` environment variable (see [Configuration](/reeweb/docs/getting-started/configuration)), so set `SITE_URL` in `.env` or the build environment before running them.

### Sitemap Options

```bash
bun scripts/generate_sitemap.ts [options]
```

| Option             | Default                | Description                                    |
| ------------------ | ---------------------- | ---------------------------------------------- |
| `--public <dir>`   | `./src/public`         | Source directory with page templates           |
| `--dist <dir>`     | `./dist`               | Output directory (sitemap.xml is written here) |
| `--site-url <url>` | `$SITE_URL` (required) | Absolute origin for `<loc>` and hreflang       |
| `--help`           | —                      | Print usage and exit                           |

See the [Sitemap Generator](/reeweb/docs/reference/sitemap-generator) page for per-page frontmatter options.

### RSS Options

```bash
bun scripts/generate_rss.ts [options]
```

| Option                      | Default                | Description                            |
| --------------------------- | ---------------------- | -------------------------------------- |
| `--public <dir>`            | `./src/public`         | Source directory                       |
| `--dist <dir>`              | `./dist`               | Output directory                       |
| `--site-url <url>`          | `$SITE_URL` (required) | Absolute origin                        |
| `--blog-dir <name>`         | `blog`                 | Sub-directory under `--public` to scan |
| `--formats <list>`          | `xml,json`             | Comma list: `xml`, `json`, or both     |
| `--max-items <n>`           | `50`                   | Limit items per feed                   |
| `--feed-title <text>`       | —                      | Override the feed title                |
| `--feed-description <text>` | —                      | Override the feed description          |
| `--help`                    | —                      | Print usage and exit                   |

See the [RSS Generator](/reeweb/docs/reference/rss-generator) page for per-post frontmatter options.

<a name="formatting"></a>

## Formatting

| Script           | Command | Description                        |
| ---------------- | ------- | ---------------------------------- |
| `bun run format` | `oxfmt` | Format all source files with oxfmt |

<a name="package-json-reference"></a>

## package.json Reference

The scripts above are defined in `package.json` under the `"scripts"` key. You can add your own scripts at any time — there's no Reeweb-specific structure to follow:

```json
{
	"scripts": {
		"dev": "conc -n tw,dev -c yellow,blue \"bun css:watch\" \"bun development\"",
		"launcher": "bun scripts/dev_watcher.ts",
		"development": "bun scripts/dev.ts",
		"css:build": "tailwindcss -i ./src/css/style.css -o ./src/public/css/style.min.css --minify",
		"css:watch": "tailwindcss -i ./src/css/style.css -o ./src/public/css/style.min.css --watch --verbose",
		"build:dist": "bun build:css && bun scripts/build.ts --public ./src/public --dist ./dist && bun run sitemap && bun run rss",
		"build:css": "tailwindcss -i ./src/css/style.css -o ./src/public/css/style.css --minify",
		"build": "bun scripts/build.ts --public ./src/public --dist ./dist",
		"rss": "bun scripts/generate_rss.ts --public ./src/public --dist ./dist",
		"sitemap": "bun scripts/generate_sitemap.ts --public ./src/public --dist ./dist",
		"format": "oxfmt",
		"preview": "bun scripts/preview.ts"
	}
}
```
