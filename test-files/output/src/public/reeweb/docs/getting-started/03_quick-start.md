---
title: "Quick Start"
layout: "reeweb/docs/docs.layout"
---

# Quick Start

<a name="introduction"></a>

## Introduction

This page walks from a fresh setup to a running Reeweb site. It assumes you've already installed Bun — if not, work through [Installation](/reeweb/docs/getting-started/installation) first.

<a name="development-server"></a>

## Development Server

```bash
bun dev
```

This runs both the dev server and the Tailwind CSS watcher concurrently. Open `http://localhost:3000` in a browser. Edit any `.ree` or `.md` file in `src/public/` and the page reloads automatically.

<a name="building-for-production"></a>

## Building for Production

```bash
bun run build
```

The static build script (`scripts/build.ts`) renders every template and markdown file for every language, copies static assets, generates redirects, and writes the output to `dist/`. Preview it:

```bash
bun run preview
```

<a name="creating-a-new-page"></a>

## Creating a New Page

### Template (.ree)

Create `src/public/mypage/index.ree`:

```html
{#layout("layout") }

<h1>Hello, world!</h1>
<p>This is my first Reeweb page.</p>
```

Rebuild and the page is available at `/mypage/`. For data-driven pages, add a sibling `index.ts` that exports `load_template_data()` — see [Data Loading](/reeweb/docs/getting-started/project-structure#data-loading).

### Markdown (.md)

Create `src/public/mypage/index.md` with frontmatter:

```markdown
---
title: "My Page"
---

# My Page

This is rendered as HTML with syntax highlighting, auto-generated heading IDs, and Tailwind classes.
```

Markdown files support YAML frontmatter, syntax highlighting (via highlight.js, server-side), automatic heading IDs, and external links that open in new tabs.

<a name="creating-a-new-language"></a>

## Adding a Language

Reeweb ships with English and Slovenian. To add a new language:

1. Add it to `config/supported_languages.ts` — add to `languages`, `active_languages`, `language_names`, `language_locales`, and optionally change `default_language`
2. Create a `{lang}.json` translation file next to your templates with the new language's strings
3. Rebuild

Each language gets its own URL prefix — default language at root (`/`), others under `/{lang}/`. See the [Configuration](/reeweb/docs/getting-started/configuration) page.

<a name="deploying-the-site"></a>

## Deploying the Site

Copy the contents of `dist/` to any static file host — Cloudflare Pages, Netlify, Vercel, S3, or a simple VPS with nginx. There's no server process to run and no database connection to configure. If using Cloudflare Pages, the emitted `dist/_redirects` file is read automatically.

<a name="next-steps"></a>

## Next Steps

From here, the recommended reading order:

- **[Project Structure](/reeweb/docs/getting-started/project-structure)** — how a Reeweb project is organised.
- **[Ree Templates](/reeweb/docs/ree-templates/introduction)** — the templating language in detail.
- **[Configuration](/reeweb/docs/getting-started/configuration)** — environment variables, languages, and build options.
