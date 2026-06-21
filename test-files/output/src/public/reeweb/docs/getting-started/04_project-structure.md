---
title: "Project Structure"
layout: "reeweb/docs/docs.layout"
---

# Project Structure

<a name="introduction"></a>

## Introduction

Reeweb follows a simple, opinionated directory layout where every folder has a clear, single purpose. Once you've worked on one Reeweb project, you'll feel immediately at home in any other.

<a name="the-root-directory"></a>

## The Root Directory

A fresh Reeweb project looks like this:

```
my-site/
├── config/                ← site configuration
│   ├── supported_languages.ts
│   └── redirects.ts
├── lib/                   ← template engine, helpers, build utilities
│   ├── template_engine.ts      ← orchestrator
│   ├── template/               ← engine modules (compiler, custom_elements, includes, types)
│   ├── template_helpers.ts
│   ├── i18n.ts
│   ├── markdown_docs.ts
│   ├── static_site.ts
│   ├── route_aliases.ts
│   └── redirects.ts
├── scripts/               ← build and dev tooling
│   ├── build.ts
│   ├── dev.ts
│   ├── preview.ts
│   ├── dev_watcher.ts
│   ├── generate_sitemap.ts
│   └── generate_rss.ts
├── src/
│   ├── public/            ← templates, markdown, translations, static assets
│   │   ├── index.ree      ← homepage
│   │   ├── index.ts       ← homepage data loader
│   │   ├── layout.ree     ← default layout
│   │   ├── en.json        ← English translations
│   │   ├── sl.json        ← Slovenian translations
│   │   ├── reepolee/      ← Reepolee framework docs section
│   │   ├── reeweb/        ← Reeweb docs section
│   │   ├── blog/          ← blog section
│   │   ├── about/         ← about page
│   │   └── css/           ← page-specific CSS
│   ├── components/        ← reusable .ree components (source)
│   │   ├── banner.ree
│   │   ├── my-h1.ree
│   │   └── speculation-rules.ree
│   ├── css/               ← Tailwind CSS source
│   │   └── style.css
│   └── lib/               ← project-specific helpers (safe to edit)
│       ├── project_helpers.ts
│       └── markdown_styles.ts  ← Tailwind classes for rendered markdown
├── static/                ← optional: compiled CSS, JS, images, fonts (served at /)
├── vendor/                ← vendored third-party libraries (e.g., highlight.min.js)
├── dist/                  ← build output (generated)
└── package.json
```

<a name="the-src-public-directory"></a>

## The src/public/ Directory

The `src/public/` directory is where your content lives. Every `.ree` file and `.md` file here becomes a page on your site. Files are organised by URL path:

- `src/public/index.ree` → `/`
- `src/public/about/index.ree` → `/about/`
- `src/public/blog/post-title/index.md` → `/blog/post-title/`

Ordering prefixes (`01_`, `02_`) on filenames are stripped when generating the canonical URL — `docs/01_getting-started/03_quick-start.md` becomes `/docs/getting-started/quick-start`.

### Templates (.ree)

`.ree` files are your page templates. They use the Ree templating language with tags for output (`{= }`, `{~ }`), control flow (`{#if}`, `{#each}`, `{#with}`), layouts (`{#layout()}`), includes (`{#include()}`), and components (`<custom-element>` tags). See [Ree Templates](/reeweb/docs/ree-templates/introduction) for the full reference.

### Markdown (.md)

`.md` files are rendered as HTML by Bun's built-in markdown processor. They support:

- YAML frontmatter (title, layout, description, sidebar, etc.)
- Syntax highlighting via highlight.js (server-side, no client JS needed)
- Auto-generated heading IDs for table-of-contents links
- Tailwind CSS classes injected on headings, tables, code blocks, lists, blockquotes, and links
- External links automatically open in new tabs

### Data Loading (.ts)

Templates can have sibling `.ts` files that export `load_template_data()` — called at build time to inject dynamic data. For example, `src/public/index.ts` provides the homepage's services, testimonials, and product data by language:

```ts
export async function load_template_data(): Promise<Record<string, any>> {
    return {
        services: { en: [...], sl: [...] },
        testimonials: { en: [...], sl: [...] },
        home: { ... },
    };
}
```

The returned data is merged into the render context and accessed via `props.xxx` in templates.

### Translation Files ({lang}.json)

Each language gets a `.json` file alongside templates. Translation keys are merged from the directory hierarchy — a key in `src/public/en.json` is available globally, while a key in `src/public/blog/en.json` overrides it for the blog section. The `i18n.ts` loader handles cross-language fallback: missing keys inherit from other languages (except `route_name`, which is always language-specific).

<a name="the-src-components-directory"></a>

## The src/components/ Directory

`src/components/` holds reusable `.ree` partials. A fresh Reeweb project includes three starter components — `banner.ree` (status notifications), `my-h1.ree` (styled heading), and `speculation-rules.ree` (instant navigation via the browser's Speculation Rules API). Add your own by dropping `.ree` files into this directory.

You include a component by writing it as a custom HTML element whose tag matches the file name:

```html
<banner type="green" text="Saved!"></banner> <my-h1>Hello</my-h1>
```

The template engine resolves `<banner>` to `src/components/banner.ree` automatically via the `$components/` alias. Attributes arrive as `props.attributes` and slot content as `props.children`. See [Components](/reeweb/docs/ree-templates/components).

<a name="the-lib-directory"></a>

## The Lib Directory

Plain TypeScript modules with zero runtime dependencies:

| File                      | Purpose                                                                                                                                                                                                                   |
| ------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `lib/template_engine.ts`  | Orchestrates `.ree` rendering (load, render, cache); delegates compilation to `lib/template/` (`compiler.ts`, `custom_elements.ts`, `include_handler.ts`, `include_resolver.ts`, `types.ts`)                              |
| `lib/template_helpers.ts` | Template helper functions (`localized_path`, `nav_label`, `key_values`, date formatters, currency, etc.)                                                                                                                  |
| `lib/i18n.ts`             | Translation file loader — walks directories, discovers `{lang}.json` files, builds merged translations with cross-language fallback                                                                                       |
| `lib/markdown_docs.ts`    | Generic markdown post-processor pipeline — TOC scan, syntax highlighting, external-link handling, class injection. Style-free; the Tailwind classes come from `src/lib/markdown_styles.ts` (do **not** edit the pipeline) |
| `lib/static_site.ts`      | Static build helpers — `walk_dir`, `parse_frontmatter`, `template_to_canonical`, `build_static_route_map`, `collect_page_files`                                                                                           |
| `lib/route_aliases.ts`    | URL slugification — `slugify()` transliterates Unicode to ASCII for localized route URLs                                                                                                                                  |
| `lib/redirects.ts`        | Redirect validation and emission — schema validation, collision checks, HTML stubs + `_redirects` file                                                                                                                    |

<a name="the-config-directory"></a>

## The Config Directory

| File                            | Purpose                                                             |
| ------------------------------- | ------------------------------------------------------------------- |
| `config/supported_languages.ts` | Language list, locale mappings, default language                    |
| `config/redirects.ts`           | URL redirects (301/302) — emitted as `dist/_redirects` + HTML stubs |

<a name="the-scripts-directory"></a>

## The Scripts Directory

| Script                        | Purpose                                                                                                                                                     |
| ----------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `scripts/build.ts`            | **Static site builder** — renders templates + markdown, copies assets, emits redirects                                                                      |
| `scripts/dev.ts`              | **Development server** — serves pages with live reload via WebSocket                                                                                        |
| `scripts/dev_watcher.ts`      | **Launcher** — alternative dev launcher that watches `scripts/dev.ts`, `.env`, and `bunfig.toml` for changes and auto-restarts (run via `bun run launcher`) |
| `scripts/preview.ts`          | Preview server for `dist/` — a custom Bun HTTP server, not `bun x serve`                                                                                    |
| `scripts/generate_sitemap.ts` | Generates `sitemap.xml`                                                                                                                                     |
| `scripts/generate_rss.ts`     | Generates RSS feed from blog markdown                                                                                                                       |

<a name="the-scripts-build-flow"></a>

## The Build Flow (scripts/build.ts)

The build script runs in phases:

1. **Schema-validate redirects** — catch config errors early
2. **Clear and recreate `dist/`** — starts fresh
3. **Load translations** — walks `src/public/` for `{lang}.json` files
4. **Collect files** — discovers `.ree` templates, `.md` files, and static assets
5. **Load dynamic data** — calls `load_template_data()` from sibling `.ts` files
6. **Build route map** — resolves canonical → localized paths per language
7. **Render templates** — each `.ree` file rendered per language with merged translations
8. **Build sidebar navigation** — from markdown frontmatter `has_sidebar: true`
9. **Render markdown** — each `.md` file rendered through `process_docs_markdown`
10. **Copy static files** — images, fonts, JS, etc. copied verbatim
11. **Emit redirects** — `dist/_redirects` + HTML stubs
12. **Summary** — total rendered, errors, static files copied
