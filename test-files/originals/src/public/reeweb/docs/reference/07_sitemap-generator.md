---
title: "Sitemap Generator"
layout: "reeweb/docs/docs.layout"
---

# Sitemap Generator

<a name="introduction"></a>

## Introduction

The sitemap generator (`scripts/generate_sitemap.ts`) produces a `sitemap.xml` file for search engine crawlers, along with a `robots.txt` that references it. It generates one `<url>` entry per (canonical page × active language), each carrying `<xhtml:link rel="alternate">` entries for hreflang — mirroring the same hreflang block that the static build inlines into HTML pages.

```bash
bun run sitemap
```

This should be run **after** `bun run build` — it reads from the built `dist/` directory.

<a name="cli-options"></a>

## CLI Options

```bash
bun scripts/generate_sitemap.ts [options]
```

| Option             | Default                | Description                                        |
| ------------------ | ---------------------- | -------------------------------------------------- |
| `--public <dir>`   | `./src/public`         | Source directory with page templates               |
| `--dist <dir>`     | `./dist`               | Output directory; `sitemap.xml` is written here    |
| `--site-url <url>` | `$SITE_URL` (required) | Absolute origin used for `<loc>` and hreflang URLs |
| `--help`           | —                      | Print usage and exit                               |

If `--site-url` is not provided and `SITE_URL` is not in the environment, the script exits with an error.

<a name="output"></a>

## Output

The generator produces two files:

| File               | Description                          |
| ------------------ | ------------------------------------ |
| `dist/sitemap.xml` | XML Sitemap with hreflang alternates |
| `dist/robots.txt`  | Robots file pointing to the sitemap  |

### sitemap.xml Structure

```xml
<?xml version="1.0" encoding="UTF-8"?>
<?xml-stylesheet type="text/xsl" href="/sitemap.xsl"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9"
        xmlns:xhtml="http://www.w3.org/1999/xhtml">
  <url>
    <loc>https://example.com/</loc>
    <lastmod>2026-05-12</lastmod>
    <xhtml:link rel="alternate" hreflang="sl" href="https://example.com/"/>
    <xhtml:link rel="alternate" hreflang="en" href="https://example.com/en/"/>
    <xhtml:link rel="alternate" hreflang="x-default" href="https://example.com/"/>
  </url>
  <url>
    <loc>https://example.com/en/</loc>
    <lastmod>2026-05-12</lastmod>
    <xhtml:link rel="alternate" hreflang="sl" href="https://example.com/"/>
    <xhtml:link rel="alternate" hreflang="en" href="https://example.com/en/"/>
    <xhtml:link rel="alternate" hreflang="x-default" href="https://example.com/"/>
  </url>
</urlset>
```

Each canonical page generates one `<url>` per active language. All entries carry `xhtml:link` references to every language variant plus `x-default` (pointing to the default language).

### robots.txt

```
User-agent: *
Allow: /

Sitemap: https://example.com/sitemap.xml
```

<a name="per-page-frontmatter"></a>

## Per-Page Frontmatter

Pages can opt out of the sitemap or override metadata through frontmatter in either the base file or any language variant:

| Frontmatter           | Effect                                                                                                                            |
| --------------------- | --------------------------------------------------------------------------------------------------------------------------------- |
| `sitemap: false`      | Exclude this canonical page from the sitemap                                                                                      |
| `noindex: true`       | Same effect — exclude from sitemap                                                                                                |
| `localize: false`     | Identical content in every language — emit only the default-language URL (no per-language `<url>` entries or hreflang alternates) |
| `lastmod: 2026-06-01` | Override the `<lastmod>` date (format: `YYYY-MM-DD`)                                                                              |

The `<lastmod>` date is resolved in this order:

1. `lastmod` frontmatter, if set (highest priority)
2. `published_at` frontmatter — or its alias `date` — for dated content like blog posts
3. The file's modification time (`mtime`) — the most recent mtime across all language variants

This means a blog post dated with `published_at` reports that publication date as its `<lastmod>` even after the file is touched, without needing an explicit `lastmod`.

<a name="how-it-works"></a>

## How It Works

1. **Loads translations** via `load_all_translations()` to build the route map
2. **Collects page files** via `collect_page_files()` — includes both `.ree` and `.md` files
3. **Builds route map** via `build_static_route_map()` for localized URL resolution
4. **Reads frontmatter** across all language variants for each canonical page
5. **Generates `<url>` entries** — one per (canonical × active language) with hreflang alternates
6. **Writes `sitemap.xml`** referencing the `/sitemap.xsl` stylesheet
7. **Writes `robots.txt`** with a `Sitemap:` directive

<a name="excluding-pages"></a>

## Excluding Pages

To exclude a page from the sitemap, add to its frontmatter:

```markdown
---
title: "Draft Page"
sitemap: false
---
```

Or using the alias:

```markdown
---
title: "Private Notes"
noindex: true
---
```

Both have the same effect. The check is performed across all language variants — if any variant has `sitemap: false` or `noindex: true`, the canonical page is excluded.
