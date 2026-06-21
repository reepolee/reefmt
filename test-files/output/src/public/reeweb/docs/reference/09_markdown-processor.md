---
title: "Markdown Docs Processor"
layout: "reeweb/docs/docs.layout"
---

# Markdown Docs Processor

<a name="introduction"></a>

## Introduction

Reeweb's `process_docs_markdown()` function in `lib/markdown_docs.ts` post-processes the raw HTML produced by `Bun.markdown.html()` in the build pipeline. It transforms plain markdown-rendered HTML into syntax-highlighted, navigation-ready content: it collects headings for the TOC, runs server-side syntax highlighting, adds `data-intersect` markers for client-side interactivity, opens external links in a new tab, and injects per-project CSS classes onto the rendered elements.

The function is called automatically during the static build (`scripts/build.ts`) and the dev server (`scripts/dev.ts`) for every `.md` file. There is no configuration needed — it activates by default.

### Pipeline vs. styling

`lib/markdown_docs.ts` is the **upstream-generic pipeline** and carries no project styling — do not edit it, so it stays upgradeable. The CSS class strings it injects are supplied by the caller via a `MarkdownStyles` object, which lives in the project-owned file **`src/lib/markdown_styles.ts`** (safe to edit). Both `build.ts` and `dev.ts` import `markdown_styles` from there and pass it in:

```ts
import { markdown_styles } from "$root/src/lib/markdown_styles";

const { html, headings } = process_docs_markdown(raw_html, markdown_styles);
```

If you call `process_docs_markdown()` without a `styles` argument, it uses `default_markdown_styles` — neutral defaults that inject **no** classes, producing plain semantic HTML. To restyle your docs/blog markdown, edit the class strings in `src/lib/markdown_styles.ts`; you never touch the pipeline.

<a name="what-it-does"></a>

## What It Does

`process_docs_markdown()` runs after `Bun.markdown.html()` completes and performs these transformations:

1. **Heading styling + TOC tracking** — adds `data-intersect` markers, records headings, and injects the class from `styles.heading(level)`
2. **Syntax highlighting** — server-side highlight.js (no client JavaScript needed)
3. **Link styling + external link handling** — opens external links in new tabs, injects `styles.anchor`
4. **Element class injection** — applies the per-project `MarkdownStyles` classes to paragraphs, lists, tables, blockquotes, and code

All class strings in steps 1, 3, and 4 come from the `styles` argument (the project's `markdown_styles`), not from the pipeline itself. The tables below show this project's values from `src/lib/markdown_styles.ts`.

<a name="1-heading-processing"></a>

## 1. Heading Processing

Every heading (`<h1>` through `<h6>`) is processed to:

- **Record for table of contents** — each heading's `id`, `text`, and `level` is collected and returned alongside the processed HTML. The build pipeline can use this for auto-generating a table of contents.
- **Inject classes** — the class is produced by `styles.heading(level)`. This project's `markdown_styles.heading` returns:

| Level | Classes                                                |
| ----- | ------------------------------------------------------ |
| `h1`  | `font-display text-4xl italic mb-6 scroll-mt-30`       |
| `h2`  | `font-display text-3xl italic mt-12 mb-6 scroll-mt-30` |
| `h3+` | `font-semibold text-lg mt-8 mb-3 scroll-mt-30`         |

- **Add `data-intersect`** — `h2` and above get `data-intersect="{id}"` attributes, consumed by the signals system for active-section highlighting in the sidebar TOC.

The returned `headings` array:

```ts
type Heading = { id: string; text: string; level: number };
```

This array is returned as part of the `process_docs_markdown()` result: `{ html: string; headings: Heading[] }`.

<a name="2-syntax-highlighting"></a>

## 2. Syntax Highlighting

Code blocks are highlighted **server-side** using the vendored `highlight.js` library from `vendor/highlight.min.js`. There is no client-side JavaScript for syntax highlighting — the highlighted HTML is baked into the page at build time.

### Auto-Detection vs Explicit Language

````markdown
```javascript
const x = 1;
```
````

`````

- If a language is specified (e.g., ````javascript`), the processor uses `hljs.highlight()` with that language
- If the language is not recognised by highlight.js, it falls back to `hljs.highlightAuto()`
- If no language is specified, `hljs.highlightAuto()` is used

### Code Block Structure

```html
<pre class="code-block relative rounded-xl overflow-hidden bg-code-bg border border-white/5 p-5 mb-6">
    <code class="hljs language-javascript">...</code>
</pre>
`````

The `styles.pre` class string is applied to every `<pre>` element for consistent styling. The inner `<code>` always receives `hljs language-{lang}` (or just `hljs` for auto-detected blocks) regardless of the project styles.

<a name="3-link-processing"></a>

## 3. Link Processing

Every `<a href>` element is processed:

- **Classes added** from `styles.anchor` — this project uses `text-accent underline underline-offset-2 decoration-accent/40 hover:decoration-accent transition-colors`
- **External links** (starting with `http://` or `https://`) get `target="_blank" rel="noopener noreferrer"` for security and usability

<a name="4-element-class-injection"></a>

## 4. Element Class Injection

Classes from the `MarkdownStyles` object are injected on common block elements (empty values are skipped — the element keeps its bare tag). Tables are a special case: when `styles.table_wrapper` is set, each `<table>` is wrapped in a styled `<div>`; otherwise the wrapper is omitted and only `styles.table` is applied. This project's `src/lib/markdown_styles.ts` defines:

| Tag               | Classes                                                                                                                          |
| ----------------- | -------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `<p>`             | `text-muted leading-relaxed mb-6`                                                                                                |
| `<code>` (inline) | `font-mono text-xs bg-warm px-1.5 py-0.5 rounded`                                                                                | Note: this class is applied to ALL `<code>` elements, including those inside syntax-highlighted `<pre><code>` blocks. The syntax-highlighting styles from highlight.js live on the wrapper `<pre>`, so the inline code classes on the inner `<code>` element can safely coexist. |
| `<blockquote>`    | `border-l-4 border-accent pl-4 py-2 italic text-muted mb-6`                                                                      |
| `<ul>`            | `list-disc list-inside space-y-2 mb-6 text-muted`                                                                                |
| `<ol>`            | `list-decimal list-inside space-y-2 mb-6 text-muted`                                                                             |
| `<li>`            | `leading-relaxed`                                                                                                                |
| `<table>`         | Wrapped in `<div class="mb-6 rounded-xl border border-divider overflow-hidden">` with `w-full text-sm text-left border-collapse` |
| `<thead>`         | `bg-warm`                                                                                                                        |
| `<th>`            | `px-4 py-3 font-semibold text-ink align-bottom`                                                                                  |
| `<td>`            | `px-4 py-3 align-top text-muted leading-relaxed wrap-anywhere`                                                                   |
| `<tbody>`         | `divide-y divide-divider`                                                                                                        |

<a name="input-and-output"></a>

## Input and Output

The function signature:

```ts
function process_docs_markdown(
	raw_html: string,
	styles?: MarkdownStyles, // defaults to default_markdown_styles (no classes)
): { html: string; headings: Heading[] };
```

**Input:**

- `raw_html` — the raw HTML string from `Bun.markdown.html()`.
- `styles` — optional `MarkdownStyles` object supplying the CSS class strings. Omitting it yields plain, unstyled semantic HTML. The project passes `markdown_styles` from `src/lib/markdown_styles.ts`.

**Output:** An object with:

- `html` — the processed HTML string with all classes and attributes injected
- `headings` — an array of `{id, text, level}` objects for table-of-contents generation

### The `MarkdownStyles` type

Each field is the raw contents of a `class="..."` attribute; an empty string omits the attribute entirely. `heading` is a function of the level (1–6); the rest are plain strings. `table_wrapper`, when non-empty, wraps tables in a styled `<div>`.

```ts
export type MarkdownStyles = {
	heading: (level: number) => string;
	pre: string;
	anchor: string;
	paragraph: string;
	inline_code: string;
	blockquote: string;
	ul: string;
	ol: string;
	li: string;
	table: string;
	table_wrapper: string;
	thead: string;
	tbody: string;
	th: string;
	td: string;
};
```

Both `MarkdownStyles` and the neutral `default_markdown_styles` are exported from `lib/markdown_docs.ts`; the project's concrete `markdown_styles` value lives in `src/lib/markdown_styles.ts`.

<a name="markdown-rendering-options"></a>

## Markdown Rendering Options

Before `process_docs_markdown()` runs, `Bun.markdown.html()` is called with these options:

```ts
Bun.markdown.html(markdown_body, {
	tables: true,
	strikethrough: true,
	tasklists: true,
	autolinks: { url: true, www: true, email: true },
	headings: { ids: true },
});
```

| Option            | Value  | Effect                                    |
| ----------------- | ------ | ----------------------------------------- |
| `tables`          | `true` | GitHub-style markdown tables              |
| `strikethrough`   | `true` | `~~text~~` renders as strikethrough       |
| `tasklists`       | `true` | `- [ ]` and `- [x]` render as checkboxes  |
| `autolinks.url`   | `true` | Bare URLs are auto-linked                 |
| `autolinks.www`   | `true` | `www.example.com` is auto-linked          |
| `autolinks.email` | `true` | `email@example.com` is auto-linked        |
| `headings.ids`    | `true` | Auto-generate `id` attributes on headings |
