---
title: "Layout System"
layout: "reeweb/docs/docs.layout"
---

# Layout System

<a name="overview"></a>

## Overview

Reeweb's `TemplateEngine` (inspired by Eta.js and Svelte) supports:

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

<a name="layout-stack"></a>

## Layout Stack

Reeweb has a single `viewsDir` (`./src/public/`). All layouts and templates resolve within it:

```
src/public/
  layout.ree              ← Default shell: header, nav, footer
  plain.layout.ree        ← Minimal layout (no sidebar/nav)
  academic.layout.ree     → {#layout('layout')}  (NESTED — wraps inside layout.ree)
  index.ree               → {#layout("layout")}
  about/index.ree         → {#layout("layout")}
  blog/post.ree           → {#layout("layout")}
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
academic.layout.ree                  src/public/layout.ree
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

If **another template nests `academic.layout.ree`** → `layout.ree`:

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
| `{#layout("layout")}`             | `{viewsDir}/layout.ree`               | `src/public/layout.ree`       |
| `{#layout("pages/home")}`         | `{viewsDir}/pages/home.ree`           | `src/public/pages/home.ree`   |
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

Templates can use kebab-case HTML elements (`<card-header>`, `<site-nav>`). The pre-processor:

1. Checks if `components/{tag-name}.ree` exists in the project root
2. **If found** → emits an internal marker that the compiler resolves into a component include — `<card-header attr="val">slot</card-header>` becomes an `__rtInclude("$components/card-header", { children: ..., attributes: { attr: "val" } })` call
3. **If not found** → passes through as a native HTML element, with slot content compiled inline

Slot content is compiled and rendered at runtime. HTML attributes on the tag are passed as `props.attributes` inside the component, and the slot is passed as `props.children`. Attribute values written as `attr="{= expr }"` are interpolated — the engine evaluates `expr` where the tag sits, so the component receives the real value rather than the literal string.

---

<a name="layout-file-convention"></a>

## Layout File Convention

| File                             | Purpose                                           | Used by       |
| -------------------------------- | ------------------------------------------------- | ------------- |
| `src/public/layout.ree`          | Default shell (header, nav, footer)               | Most pages    |
| `src/public/academic.layout.ree` | Academic paper layout (nests inside `layout.ree`) | Markdown docs |
| `src/public/plain.layout.ree`    | Minimal layout                                    | Simple pages  |

**Naming convention:** Layouts end in `.layout.ree` or can be plain `layout.ree`. The `.layout` suffix distinguishes named layouts (e.g., `academic.layout.ree`) from page templates. When resolving, the engine tries `{name}.layout.ree` first, then `{name}.ree`.
