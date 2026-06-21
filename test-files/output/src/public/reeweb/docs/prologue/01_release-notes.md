---
title: "Release Notes"
layout: "reeweb/docs/docs.layout"
---

# Release Notes

<a name="introduction"></a>

## Introduction

This page documents each release of Reeweb, organised by version number. Releases are chronological within each major version, with the most recent first. For a timeline view of project milestones, see the [Changelog](/reeweb/docs/community/changelog).

<a name="v0-1-0"></a>

## 0.1.0 — 2026-05-12

Initial public release of the Reeweb static site generator.

### Features

- **`.ree` template engine** — custom template engine with output tags (`{= }`, `{~ }`), control flow (`{#if}`, `{#each}`, `{#with}`), layouts (`{#layout}`), includes (`{#include}`), and components via `<custom-element>` tags
- **Multi-language support** — per-language translation files with cross-language fallback, language-prefixed URLs, hreflang links
- **Localized routes** — translate URL paths per language via `route_name` in translation JSON, with Unicode slugification
- **Markdown rendering** — server-side rendering via `Bun.markdown.html()` with frontmatter, syntax highlighting (highlight.js), auto-generated heading IDs, Tailwind class injection
- **Tailwind CSS v4** — standalone CLI with `@theme` customisation, watch mode for development
- **Static site build** — `scripts/build.ts` renders templates + markdown to static HTML with redirect support
- **Dev server** — `scripts/dev.ts` with live reload via WebSocket, file watching, multi-language routing
- **Redirect system** — schema-validated redirects from `config/redirects.ts`, emitted as `_redirects` (Cloudflare format) + HTML stubs
- **Sitemap & RSS** — `scripts/generate_sitemap.ts` and `scripts/generate_rss.ts` for SEO and blog feeds
- **Component system** — custom HTML element auto-discovery resolves `<my-component>` to `components/my-component.ree`
- **Data loading** — sibling `.ts` files export `load_template_data()` for injecting dynamic data at build time
- **Props convention** — all template data accessed via `props.xxx`, custom element attributes grouped under `props.attributes`, `...rest` spread shorthand
- **Zero runtime dependencies** — only dev dependencies (Tailwind CSS v4, `@types/bun`)

### Known limitations

- Search is not yet implemented (search modal is a placeholder)
- No plugin system — template engine extensions require editing source files directly
- No visual editor — templates are hand-authored
