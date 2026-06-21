---
title: "Configuration"
layout: "reeweb/docs/docs.layout"
---

# Configuration

<a name="introduction"></a>

## Introduction

Reeweb has very little to configure. Everything that varies between environments lives in `.env` and is read directly from `Bun.env`. Everything that varies between projects lives in TypeScript files under `config/` that you edit like any other source file.

<a name="the-env-file"></a>

## The .env File

Copy the example file:

```bash
cp .env.example .env
```

The full set of variables:

```env
SITE_URL=https://example.com
```

| Variable   | Required             | Purpose                                         |
| ---------- | -------------------- | ----------------------------------------------- |
| `SITE_URL` | No (but recommended) | Full site URL used for hreflang alternate links |

If `SITE_URL` is set, the build generates `<link rel="alternate" hreflang="...">` tags pointing to absolute URLs for each language variant — required by Google for multi-language SEO. Without it, hreflang links are skipped.

The `build:dist`, `sitemap`, and `rss` scripts no longer hardcode a site URL on the command line — they read `SITE_URL` from the environment automatically. The `--site-url` flag still works when passed explicitly and takes precedence over the environment variable. Note that `generate_sitemap.ts` and `generate_rss.ts` **require** a site URL (from `--site-url` or `SITE_URL`) and exit with an error if neither is provided.

<a name="build-options"></a>

## Build Options

The `bun run build` script accepts several flags:

```bash
bun scripts/build.ts --public ./src/public --dist ./dist --base-url / --site-url https://example.com --verbose
```

| Option       | Default        | Description                          |
| ------------ | -------------- | ------------------------------------ |
| `--public`   | `./src/public` | Source directory with .ree templates |
| `--dist`     | `./dist`       | Output directory for static HTML     |
| `--base-url` | `/`            | Base URL for the site                |
| `--site-url` | (empty)        | Full site URL for hreflang links     |
| `--verbose`  | false          | Log each rendered file               |

<a name="config-supported-languages"></a>

## config/supported_languages.ts

Declares the languages your site supports:

```ts
export const languages = ["en", "sl"] as const;
export const active_languages = ["sl", "en"] as const;
export const default_language = "sl";

export const language_names: Record<string, string> = {
	en: "English",
	sl: "Slovenian",
};

export const language_locales: Record<string, string> = {
	en: "en-US",
	sl: "sl-SI",
};
```

Adding a new language is a one-file change here plus a matching `{lang}.json` translation file.

<a name="config-redirects"></a>

## config/redirects.ts

Declares URL redirects (301 by default, 302 for temporary):

```ts
export const redirects: { from: string; to: string; status?: 301 | 302 }[] = [
	{ from: "/old-page", to: "/new-page" }, // 301
	{ from: "/promo", to: "https://example.com/landing", status: 302 }, // 302
];
```

Redirects are validated in two phases by `lib/redirects.ts`:

1. **Schema validation** — checks `from` starts with `/`, has no file extension, `to` is non-empty
2. **Collision check** — after rendering, verifies `from` doesn't collide with a page or asset

Emitted as:

- `dist/_redirects` — Cloudflare Pages format
- `dist/{from}/index.html` — HTML meta-refresh stubs (fallback for other hosts)

<a name="package-json-scripts"></a>

## package.json Scripts

The scripts you'll use most:

| Script              | Purpose                                                |
| ------------------- | ------------------------------------------------------ |
| `bun dev`           | Development server + Tailwind CSS watcher (concurrent) |
| `bun run build`     | Build the static site to `dist/`                       |
| `bun run preview`   | Serve `dist/` locally                                  |
| `bun run css:build` | Build minified Tailwind CSS from `src/css/style.css`   |
| `bun run css:watch` | Watch and rebuild CSS during development               |
| `bun run format`    | Format all source files (oxfmt)                        |
| `bun run sitemap`   | Generate `sitemap.xml`                                 |
| `bun run rss`       | Generate RSS feed                                      |

**Note:** There is no `build:full` or `build:verbose` script in package.json. For hreflang links, pass `--site-url` directly:

```bash
bun scripts/build.ts --public ./src/public --dist ./dist --site-url https://example.com --verbose
```

<a name="how-bun-loads-env"></a>

## How Bun Loads .env

Bun reads `.env` files automatically at startup — there's no `dotenv` dependency to install. Variables become available on `Bun.env.NAME`.
