---
title: "Frontmatter Reference"
layout: "reeweb/docs/docs.layout"
---

# Frontmatter Reference

<a name="introduction"></a>

## Introduction

Both `.md` (markdown) and `.ree` (template) files can include YAML frontmatter — a block of metadata at the top of the file, delimited by `---` lines. Frontmatter is parsed by `parse_frontmatter()` in `lib/static_site.ts` using Bun's built-in YAML parser.

```markdown
---
title: "My Document"
layout: "docs"
has_sidebar: true
---
```

## Global Fields

These fields apply to both `.md` and `.ree` files:

| Field    | Type   | Default    | Description                                                                                                        |
| -------- | ------ | ---------- | ------------------------------------------------------------------------------------------------------------------ |
| `title`  | string | —          | Page title. Used in `<title>`, sidebar navigation, and as the heading if no `# Heading` exists in markdown content |
| `layout` | string | `"layout"` | Layout template name. The engine tries `{name}.layout.ree` first, then `{name}.ree`                                |

## Markdown-Only Fields

These fields are specific to `.md` files and are consumed at different stages of the build pipeline.

### Sitemap & SEO

| Field      | Type        | Default   | Description                                                                                                                                                                  |
| ---------- | ----------- | --------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `sitemap`  | boolean     | `true`    | Set to `false` to exclude this page from the generated `sitemap.xml`. Also implied by `noindex: true`                                                                        |
| `noindex`  | boolean     | `false`   | Set to `true` to exclude this page from the sitemap. Use alongside `<meta name="robots" content="noindex">` in the layout                                                    |
| `localize` | boolean     | `true`    | Set to `false` for pages whose content is identical in every language — the sitemap emits only the default-language URL, with no per-language entries or hreflang alternates |
| `lastmod`  | date string | see below | Override the `lastmod` date in the sitemap. Format: `YYYY-MM-DD`. When not set, the sitemap falls back to `published_at`/`date`, then to the file's modification time        |

### Navigation & Sidebar

| Field             | Type    | Default | Description                                                                                                                                                                                                      |
| ----------------- | ------- | ------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `has_sidebar`     | boolean | `false` | Enable sidebar navigation for this section. Set on the section's `index.md` file. Also accepts `"true"` and `"yes"` (string values from YAML parsing). All `.md` files in the same folder become sidebar entries |
| `skip-navigation` | boolean | `false` | Exclude this page from the sidebar navigation. Useful for utility pages that shouldn't appear in the sidebar menu                                                                                                |

### RSS Feed

| Field          | Type             | Default         | Description                                                                                                             |
| -------------- | ---------------- | --------------- | ----------------------------------------------------------------------------------------------------------------------- |
| `rss`          | boolean          | `true`          | Set to `false` to exclude this post from the RSS/JSON feed                                                              |
| `published_at` | date string      | file mtime      | Publication date for RSS. Format: `YYYY-MM-DD` or full ISO datetime                                                     |
| `date`         | date string      | file mtime      | Alias for `published_at`                                                                                                |
| `description`  | text             | first paragraph | Post description/summary for the RSS feed. Falls back to the first paragraph of content if not set                      |
| `summary`      | text             | first paragraph | Alias for `description`                                                                                                 |
| `abstract`     | text             | first paragraph | Alias for `description` (academic layout)                                                                               |
| `author`       | string or object | —               | Author name or `{name, email}` object                                                                                   |
| `authors`      | array            | —               | Array of author objects `[{name, email, url}]`. First entry is used for RSS `<author>`, all entries appear in JSON Feed |

### Rendering

| Field       | Type   | Default         | Description                                   |
| ----------- | ------ | --------------- | --------------------------------------------- |
| `site_name` | string | `"Static Site"` | Override the site name for this specific page |

## Frontmatter Resolution

### Language-Variant Frontmatter

For language-specific markdown files (`file.en.md`, `file.sl.md`), frontmatter from the resolved variant is used. When checking for sitemap inclusion, `noindex: true` or `sitemap: false` on **any** language variant excludes the canonical page from the sitemap entirely.

The resolution order:

1. `{name}.{lang}.md` — exact match for the active language
2. `{name}.{default_language}.md` — fallback to the default language
3. `{name}.md` — generic fallback

### Frontmatter Merging

When the build scans language variants, frontmatter values are merged:

- Scalar values (strings, numbers, booleans) from the most specific variant win
- `sitemap: false` and `noindex: true` on any variant cause exclusion
- `lastmod` resolves to the explicit `lastmod`, then `published_at`/`date`, then the most recent mtime across all variants

## Examples

### Basic Page

```markdown
---
title: "Installation Guide"
layout: "academic"
---
```

### Blog Post with RSS Metadata

```markdown
---
title: "Introducing Reeweb 0.1"
date: 2026-05-12
author: "Matic Jurglic"
description: "The first public release of Reeweb, a new static site generator."
tags:
    - release
    - announcement
---
```

### Multi-Author Post

```markdown
---
title: "Building with Reeweb"
published_at: 2026-06-01
authors:
    - name: "Matic Jurglic"
      email: "matic@reepolee.com"
    - name: "Evan"
      url: "https://example.com"
---
```

### Page Excluded from Everything

```markdown
---
title: "Draft Notes"
sitemap: false
rss: false
skip-navigation: true
---
```

### Section with Sidebar

On the section's `index.md`:

```markdown
---
title: "Documentation"
has_sidebar: true
---
```

All `*.md` files in the same directory automatically become sidebar entries, ordered by filename.
