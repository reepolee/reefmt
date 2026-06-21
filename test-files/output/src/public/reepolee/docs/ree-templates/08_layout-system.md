---
title: "Layout System"
---

# Layout System

<a name="overview"></a>

## Overview

Both reepolee and reeweb use the same `TemplateEngine` (inspired by Eta.js and Svelte) that supports:

- Layout wrapping via `{#layout('path')}`
- Partial includes via `{#include('path')}`
- Custom HTML element → component auto-discovery
- Language-variant template fallback
- Relative, absolute, and alias path resolution

A layout is a `.ree` template that wraps a page's rendered content. The page declares its layout as the **first line** of the template, and the layout receives the page's HTML in `{~ props.body }`.

---

<a name="how-layouts-are-declared"></a>

## How Layouts Are Declared

```ree
{#layout("layout")}
<h1>{= props.ui.title }</h1>
```

Everything after `{#layout(...)}` becomes the **body content**. The layout template renders first, calling `{~ props.body }` where the page content should appear.

Layouts can also receive extra data:

```ree
{#layout("layout", { title: "Custom" })}
```

The second argument merges into the layout's data: `Object.assign({}, data, extraData, { body: body })`.

---

<a name="layout-stacks"></a>

## Layout Stacks

**reepolee** has **two separate layout worlds** because it creates two `TemplateEngine` instances with different `viewsDir` configurations. **reeweb** has only one.

<a name="server-side-routes"></a>

### Server-side route templates — `views: ./routes/`

Configured in `reepolee/lib/template.ts`:

```ts
new TemplateEngine({
	views: join(import.meta.dir, "..", "routes"), // → ./routes/
});
```

```
routes/
  layout.ree              ← App shell: nav sidebar, auth, toasts, language switcher
  notfound.ree            ← Standalone 404 (FULL HTML, no {#layout})
  home/home.ree           → {#layout("layout")}
  system/users/index.ree  → {#layout("layout")}
  system/users/form.ree   → {#layout("layout")}
```

All route templates reference `{#layout("layout")}`, which resolves to **`routes/layout.ree`**.

<a name="public-static-templates"></a>

### Public/static templates — `views: ./public/`

Used by the dev server (`scripts/dev.ts`) and the static build (`scripts/static_build.ts`):

```
public/
  layout.ree              ← Docs/homepage shell: header, nav, sidebar, footer
  academic.layout.ree     → {#layout('layout')}  (NESTED — wraps itself in layout.ree)
  index.ree               → {#layout("layout")}
  about/index.en.ree      → {#layout("layout")}
  contact/index.ree       → {#layout("layout")}
```

<a name="reeweb-single-stack"></a>

### reeweb — single layout stack

reeweb only has one `viewsDir` (`./src/public/`):

```
src/public/
  layout.ree
  plain.layout.ree
  academic.layout.ree     → {#layout('layout')} → src/public/layout.ree
  index.ree               → {#layout("layout")}
  ...
```

---

<a name="layout-nesting"></a>

## Layout Nesting

Layouts nest arbitrarily via recursive `__include` at compile time.

<a name="code-generation"></a>

### How it works at the code generation level

When the compiler encounters `{#layout('path')}`, it wraps the generated code:

```js
// Normal template compilation produces __output
const __body = __output;
const __layoutData = Object.assign({}, props, extraData, { body: __body });
__output = await __include("path", __layoutData);
```

This means the layout template is rendered with the captured body as `props.body`, and the layout's output becomes the final output (which itself could trigger another layout).

<a name="nesting-example"></a>

### Real example: `academic.layout.ree` → `layout.ree`

```
academic.layout.ree                  public/layout.ree
┌─────────────────────────────┐      ┌───────────────────────────────────────────┐
│ {#layout('layout')}         │ ──→  │ <html>                                   │
│                             │      │   <body>                                 │
│ <article class="paper">     │      │     <header>...</header>                 │
│   {~ props.body }            │      │     <main>{~ props.body }</main>           │
│ </article>                  │      │     <footer>...</footer>                  │
└──────────────┬──────────────┘      └──────────────────┬────────────────────────┘
               │                                         │
    Rendered body passed as props.body        Receives academic layout's HTML
    to layout.ree via {#layout('layout')}    as props.body, wraps in full page
```

**At runtime:**

1. Page template renders → produces `<h1>Content</h1>`
2. `{#layout('layout')}` triggers → `layout.ree` renders, receives page HTML as `props.body`
3. `layout.ree` wraps it in `<html>/<header>/<main>/<footer>`
4. Final output: full page

If **another template nests `academic.layout.ree`** → `public/layout.ree`:

1. Page renders → produces content
2. `academic.layout.ree` wraps it in `<article class="paper">` + captures as `props.body`
3. `academic.layout.ree`'s `{#layout('layout')}` triggers → `layout.ree` renders
4. `layout.ree` wraps academic layout output in `<html>/<header>/<footer>`
5. Final output

---

<a name="path-resolution"></a>

## Path Resolution

The `resolve_include()` function (in `lib/template/include_resolver.ts`) handles all path types:

| Syntax                            | Resolves to                           | Example                       |
| --------------------------------- | ------------------------------------- | ----------------------------- |
| `{#layout("layout")}`             | `{viewsDir}/layout.ree`               | `routes/layout.ree`           |
| `{#layout("pages/home")}`         | `{viewsDir}/pages/home.ree`           | `routes/pages/home.ree`       |
| `{#layout("./partial")}`          | `{dirname(current)}/partial.ree`      | Relative to current template  |
| `{#layout("../shared/header")}`   | `{parent(current)}/shared/header.ree` | Parent traversal              |
| `{#layout("/components/card")}`   | `{viewsDir}/components/card.ree`      | Absolute from views root      |
| `{#layout("$components/card")}`   | `{projectRoot}/components/card.ree`   | Alias path (outside viewsDir) |
| `{#include("./data.json")}`       | Raw file loaded as-is (unescaped)     | Non-`.ree` extension          |
| `<card-header>slot</card-header>` | `components/card-header.ree`          | Auto-component                |

<a name="alias-paths"></a>

### Alias paths (`$` prefix)

Resolve relative to the **project root** (one level up from `viewsDir`):

| Alias              | Maps to                 |
| ------------------ | ----------------------- |
| `$components/card` | `./components/card.ree` |
| `$routes/examples` | `./routes/examples.ree` |
| `$lib/helpers`     | `./lib/helpers.ree`     |

<a name="extension-rules"></a>

### Extension rules

- **`.ree`** → compiled template (extension stripped, rendered via `engine.render`)
- **Other extensions** (`.json`, `.html`, `.svg`) → raw file, injected as text (unescaped)
- **No extension** → treated as `.ree` template

<a name="path-security"></a>

### Path Security

Path traversal is blocked: resolved paths must stay within the base directory (views root for views-relative, project root for alias paths), or an error is thrown.

---

<a name="fallback-chains"></a>

## Fallback Chains

<a name="language-variant-fallback"></a>

### Language variant fallback

Every template load goes through this chain:

```
{name}.{requestedLang}.ree  →  {name}.{default_language}.ree  →  {name}.ree
```

Example: with `lang=de` and `default_language=sl`:

1. Try `about/index.de.ree` → not found
2. Try `about/index.sl.ree` → found ✓ (or)
3. Try `about/index.ree` → found ✓ (fallback)

This applies to **every** template load — pages, layouts, includes, and components.

<a name="renderts-path-fallback"></a>

### `render.ts` path fallback (server-side only)

In `reepolee/lib/render.ts`, when rendering a route template:

```ts
// First try: route_dir-prefixed path (from ctx.route_dir)
resolved_template = ctx.route_dir + "/" + clean_name;

// On "Template not found" error:
resolved_template = clean_name; // fall back to views root
```

Example: route handler calls `render("layout", ...)` with `ctx.route_dir = "routes/home"`:

1. Tries `routes/home/layout.ree` → not found
2. **Falls back** to `routes/layout.ree` → found ✓

<a name="markdown-layout-fallback"></a>

### Markdown layout fallback (static build)

For `.md` files rendered during static build:

```ts
const raw_layout = frontmatter.layout || "layout";
const base_layout = raw_layout.replace(".ree", "").replace(".layout", "");
const candidates = [`${base_layout}.layout`, base_layout];
```

1. Frontmatter `layout: academic` → tries `academic.layout.ree` → `academic.ree`
2. No frontmatter `layout` → default `"layout"` → tries `layout.layout.ree` → `layout.ree`

<a name="layout-data-merge"></a>

### Layout data merge

```ts
Object.assign({}, props, extraData, { body: body });
```

The layout receives **all** page data, plus any extra data from `{#layout('path', extraData)}`, with `body` always taking precedence (last write wins).

---

<a name="include-vs-layout"></a>

## `{#include}` vs `{#layout}`

|                   | `{#include('path')}`                | `{#layout('path')}`                   |
| ----------------- | ----------------------------------- | ------------------------------------- |
| **Body capture**  | No — rendered inline                | Yes — captures output as `props.body` |
| **Use case**      | Partials, components, raw files     | Page structure wrapping               |
| **HTML escaping** | Never escaped (trusted HTML output) | Body unescaped via `{~ }`             |
| **Nesting**       | Any depth                           | Any depth                             |
| **Raw files**     | Yes — `.html`, `.svg`, `.json`      | No — must be `.ree` template          |

---

<a name="component-auto-discovery"></a>

## Component Auto-Discovery

Templates can use kebab-case HTML elements (`<card-header>`, `<toasts-area>`). The pre-processor:

1. Checks if `components/{tag-name}.ree` exists in the project root
2. **If found** → emits an internal marker that the compiler resolves into a component include — `<card-header attr="val">slot</card-header>` becomes an `__rtInclude("$components/card-header", { children: ..., attributes: { attr: "val" } })` call
3. **If not found** → passes through as a native HTML element, with slot content compiled inline

Slot content is compiled and rendered at runtime. HTML attributes on the tag are passed as `props.attributes` inside the component, and the slot as `props.children`. An attribute written as `attr="{= expr }"` is interpolated — the engine evaluates `expr` where the tag sits, so the component receives the real value rather than the literal string.

---

<a name="layout-file-convention"></a>

## Layout File Convention

| File                         | Purpose                                                  | Views Dir   | Used by                  |
| ---------------------------- | -------------------------------------------------------- | ----------- | ------------------------ |
| `routes/layout.ree`          | App shell (nav sidebar, auth, toasts, lang switcher)     | `./routes/` | Server-side routes       |
| `public/layout.ree`          | Public site shell (header, nav, footer, sidebar)         | `./public/` | Dev server, static build |
| `public/academic.layout.ree` | Academic paper layout (nests inside `public/layout.ree`) | `./public/` | Markdown docs            |
| `public/plain.layout.ree`    | Minimal layout                                           | `./public/` | Simple pages             |

**Naming convention:** Layouts end in `.layout.ree` or can be plain `layout.ree`. The `.layout` suffix distinguishes named layouts (e.g., `academic.layout.ree`) from page templates. When resolving, the engine tries `{name}.layout.ree` first, then `{name}.ree`.

---

<a name="reeweb-vs-reepolee"></a>

## reeweb vs reepolee

| Aspect                   | reeweb                  | reepolee                          |
| ------------------------ | ----------------------- | --------------------------------- |
| **Layout stacks**        | 1 (`./src/public/`)     | 2 (`./routes/` + `./public/`)     |
| **Server-side layout**   | N/A                     | `routes/layout.ree` (app shell)   |
| **Public layout**        | `src/public/layout.ree` | `public/layout.ree` (docs shell)  |
| **Nesting**              | ✅                      | ✅                                |
| **Language fallback**    | ✅ 3-level chain        | ✅ 3-level chain                  |
| **Relative paths**       | ✅ `./`, `../`          | ✅ `./`, `../`, `$` aliases       |
| **`render.ts` fallback** | N/A                     | ✅ route_dir + bare name fallback |
| **Raw file includes**    | ✅ with extension       | ✅ with extension                 |
| **Components**           | ✅ auto-discovery       | ✅ auto-discovery                 |
