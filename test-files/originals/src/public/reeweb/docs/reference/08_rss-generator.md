---
title: "RSS Generator"
layout: "reeweb/docs/docs.layout"
---

# RSS Generator

<a name="introduction"></a>

## Introduction

The RSS generator (`scripts/generate_rss.ts`) produces per-language RSS 2.0 (`feed.xml`) and JSON Feed 1.1 (`feed.json`) files for the blog directory. It reads `.md` files only — no database or runtime dependency — so it works on statically generated sites. Per-post metadata is read from frontmatter, and the full post content is rendered to HTML for the feed.

```bash
bun run rss
```

This should be run **after** `bun run build` — it reads from the built `dist/` directory.

<a name="cli-options"></a>

## CLI Options

```bash
bun scripts/generate_rss.ts [options]
```

| Option                      | Default                    | Description                                                 |
| --------------------------- | -------------------------- | ----------------------------------------------------------- |
| `--public <dir>`            | `./src/public`             | Source directory                                            |
| `--dist <dir>`              | `./dist`                   | Output directory                                            |
| `--site-url <url>`          | `$SITE_URL` (**required**) | Absolute origin for feed URLs                               |
| `--blog-dir <name>`         | `blog`                     | Sub-directory under `--public` to scan for markdown posts   |
| `--formats <list>`          | `xml,json`                 | Comma-separated list: `xml`, `json`, or both                |
| `--max-items <n>`           | `50`                       | Maximum items per feed                                      |
| `--feed-title <text>`       | —                          | Override the feed title (default: `"<site_name> — <blog>"`) |
| `--feed-description <text>` | —                          | Override the feed description (default: per-language)       |
| `--help`                    | —                          | Print usage and exit                                        |

If `--site-url` is not provided and `SITE_URL` is not in the environment, the script exits with an error.

<a name="output"></a>

## Output

For each active language, the generator writes to the blog's output directory:

| File                     | Format        | Description                |
| ------------------------ | ------------- | -------------------------- |
| `dist/blog/feed.xml`     | RSS 2.0       | Default language RSS feed  |
| `dist/blog/feed.json`    | JSON Feed 1.1 | Default language JSON feed |
| `dist/en/blog/feed.xml`  | RSS 2.0       | English RSS feed           |
| `dist/en/blog/feed.json` | JSON Feed 1.1 | English JSON feed          |

The output path mirrors the per-language structure: default language at root, others under `/{lang}/`.

<a name="per-post-frontmatter"></a>

## Per-Post Frontmatter

Each blog post (`.md` file) can configure its feed entry through frontmatter:

| Field          | Type             | Default           | Description                                                  |
| -------------- | ---------------- | ----------------- | ------------------------------------------------------------ |
| `title`        | string           | First `# Heading` | Post title for the feed entry                                |
| `published_at` | date string      | File mtime        | Publication date. Format: `YYYY-MM-DD` or full ISO datetime  |
| `date`         | date string      | File mtime        | Alias for `published_at`                                     |
| `description`  | string           | First paragraph   | Post summary for the feed. Falls back to the first paragraph |
| `summary`      | string           | First paragraph   | Alias for `description`                                      |
| `abstract`     | string           | First paragraph   | Alias for `description` (academic layout)                    |
| `author`       | string or object | —                 | Single author: `"Name"` or `{name: "Name", email: "..."}`    |
| `authors`      | array            | —                 | Multiple authors: `[{name: "A", email: "..."}, {name: "B"}]` |
| `rss`          | boolean          | `true`            | Set to `false` to exclude this post from the feed            |
| `noindex`      | boolean          | `false`           | Also excludes from the feed (shared with sitemap)            |

### Author Resolution

The author resolution chain:

1. `authors` array (first entry used for RSS `<author>`, all entries for JSON Feed)
2. `author` (string or `{name, email, url}` object)
3. No author — the `<author>` element is omitted

For the RSS `<author>` element, the format is `email (Name)`:

```xml
<author>matic@reepolee.com (Matic Jurglic)</author>
```

### Description Resolution

The description resolution chain:

1. `description` frontmatter field
2. `summary` frontmatter field (alias)
3. `abstract` frontmatter field (alias)
4. First paragraph of the markdown body (stripped of formatting)

Descriptions are truncated to 320 characters with an ellipsis.

<a name="feed-formats"></a>

## Feed Formats

### RSS 2.0

Each feed item includes:

| Element             | Source                                      |
| ------------------- | ------------------------------------------- |
| `<title>`           | Post title                                  |
| `<link>`            | Full URL to the post                        |
| `<guid>`            | Same as `<link>` (isPermaLink="true")       |
| `<pubDate>`         | RFC 822 date                                |
| `<description>`     | Post description (truncated, CDATA-wrapped) |
| `<content:encoded>` | Full post HTML (CDATA-wrapped)              |
| `<author>`          | First author, if present                    |

### JSON Feed 1.1

Each feed item includes:

| Field            | Source                                          |
| ---------------- | ----------------------------------------------- |
| `id`             | Full URL to the post                            |
| `url`            | Full URL to the post                            |
| `title`          | Post title                                      |
| `summary`        | Post description                                |
| `content_html`   | Full post HTML                                  |
| `date_published` | ISO 8601 date                                   |
| `authors`        | Array of `{name, url}` (if any authors are set) |

<a name="how-it-works"></a>

## How It Works

1. **Loads translations** to build the route map for localized URLs
2. **Collects markdown files** from the blog directory, excluding index files
3. **Resolves language variants** for each post (`{name}.{lang}.md` → `{name}.md`)
4. **Parses frontmatter** for title, date, description, authors, and exclusion flags
5. **Renders markdown body** to HTML using `Bun.markdown.html()` (same as the static build)
6. **Strips markdown** from descriptions for plain-text feed content
7. **Sorts posts** by date (newest first)
8. **Builds feeds** for each active language in the requested formats
