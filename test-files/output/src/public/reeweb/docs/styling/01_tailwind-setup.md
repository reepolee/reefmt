---
title: "Tailwind CSS Setup"
layout: "reeweb/docs/docs.layout"
---

# Tailwind CSS Setup

<a name="introduction"></a>

## Introduction

Reeweb uses Tailwind CSS v4 for styling. Tailwind is configured as a dev dependency — the build step compiles your source CSS into a minified output file that's served alongside your static pages. There is no runtime dependency and no client-side style computation.

<a name="how-it-works"></a>

## How It Works

Tailwind v4 runs as a standalone CLI tool. It scans your template and markdown files for class names, generates only the CSS those classes reference, and outputs a single minified stylesheet. The pipeline is:

1. Source CSS lives in `src/css/style.css`
2. Tailwind processes it against `src/public/` (templates and markdown)
3. Output goes to `src/public/css/style.min.css` (both `css:build` and `css:watch`)
4. `build:dist` also writes a `style.css` variant alongside as part of its `build:css` step

The source file imports the framework and declares the theme:

```css
@import "tailwindcss";
@source not ".git/**/*";
@source not ".vscode/**/*";
@source not "node_modules/**/*";

@source "../lib/**/*";

@theme {
	--color-brand: #b40000;
	--color-bg-page: #fafafa;
	--color-bg-card: #fefefe;
}
```

The `@source` directives tell Tailwind where to scan for class name usage. Templates live in `src/public/`, and project helpers in `src/lib/`. Directories like `.git` and `node_modules` are explicitly excluded to keep the scan fast.

<a name="commands"></a>

## Commands

```bash
# Build CSS once (minified) — output to src/public/css/style.min.css
bun run css:build

# Watch mode for development
bun run css:watch

# Full production build: fresh CSS + static site + sitemap + RSS
bun run build:dist
```

Note: `bun run build` does **not** automatically run `css:build`. If you want a complete production build with fresh CSS, use `bun run build:dist`. To compile CSS separately before a plain `bun run build`, run `bun run css:build` first.

The scripts in `package.json` map to:

```bash
# Minified CSS — css:build and css:watch both output to style.min.css
tailwindcss -i ./src/css/style.css -o ./src/public/css/style.min.css --minify

# Watch mode for development (same output path, no --minify)
tailwindcss -i ./src/css/style.css -o ./src/public/css/style.min.css --watch --verbose
```

In development (`bun dev`), the Tailwind watcher runs concurrently with the dev server. Changes to your CSS source or template files trigger a recompilation automatically.

<a name="theme-customization"></a>

## Theme Customization

The `@theme` directive in your source CSS defines project-specific design tokens. These extend Tailwind's default theme:

```css
@theme {
	/* Brand colours */
	--color-brand: #b40000;
	--color-bg-page: #fafafa;
	--color-bg-card: #fefefe;

	/* Typography */
	--font-display: "Instrument Serif", serif;
	--font-sans: "Montserrat", system-ui, sans-serif;
	--font-mono: "DM Mono", monospace;
}
```

Custom colours become available as Tailwind utilities — `bg-brand`, `text-brand`, `bg-bg-page`, and so on. The `--color-` prefix is Tailwind's convention for mapping to `bg-`, `text-`, `border-`, etc.

<a name="utility-layers"></a>

## Utility Layers

Tailwind v4 supports `@layer` directives for organising your custom CSS:

```css
@layer base {
    /* Base styles applied to HTML elements */
    html { scrollbar-gutter: stable; }
}

@layer components {
    /* Component-specific styles extracted from repeated patterns */
    .btn { ... }
}

@layer utilities {
    /* Custom utility classes */
    .scroll-mt-30 { scroll-margin-top: 7.5rem; }
}
```

Reeweb's `process_docs_markdown` post-processor also injects Tailwind classes directly into rendered HTML — headings, tables, code blocks, and links all receive classes at build time, so your documentation gets styled without manual class annotations. Those class strings live in `src/lib/markdown_styles.ts` (a project-owned file you can edit freely); the post-processor pipeline in `lib/markdown_docs.ts` stays style-free so it remains upstream-upgradeable. See the [Markdown Docs Processor](/reeweb/docs/reference/markdown-processor) reference for the full list.

<a name="best-practices"></a>

## Best Practices

- **Use Tailwind utilities directly in `.ree` templates** — `class="text-lg font-bold text-brand"` is the idiomatic approach.
- **Extract repeated patterns into `@layer components`** when the same combination of utilities appears in more than three places.
- **Keep custom CSS in `src/css/style.css`** — this is the single source file Tailwind processes.
- **For page-specific overrides**, add a `src/public/css/style.css` that your layout loads after the main stylesheet.
- **Preview the built site** (`bun run preview`) to verify styles look correct — the dev server applies Tailwind's output, but the production build may use slightly different optimisation.
