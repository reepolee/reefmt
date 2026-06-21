---
title: "Layouts & Includes"
---

# Layouts & Includes

<a name="introduction"></a>

## Introduction

A real application is rarely a single template. Layouts wrap a page in the surrounding chrome — `<head>`, navigation, footer, scripts. Includes pull a shared chunk of markup into a page from one place so you can update it once and have every caller follow. Ree handles both with two tags: `{#layout}` and `{#include}`.

<a name="layouts"></a>

## Layouts

Place `{#layout('name') }` at the top of a template to wrap its output inside a layout file. The template's rendered HTML is exposed to the layout as `props.body`:

```html
{#layout('layout') }

<div class="p-4">
	<h1>{= props.title }</h1>
	<p>{= props.description }</p>
</div>
```

Inside the layout file, render the body wherever the page should be inserted:

```html
<!-- routes/layout.ree -->
<!DOCTYPE html>
<html>
	<head>
		<title>{= props.title }</title>
		<link rel="stylesheet" href="/app.css" />
	</head>
	<body>
		<nav>...</nav>
		<main>{~ props.body }</main>
		<footer>...</footer>
	</body>
</html>
```

Use `{~ }` to inject the body — the body is HTML you generated and you do not want it escaped.

Only one `{#layout}` per template is supported, and it must appear at the top. The layout itself can extend another layout the same way, so you can build nested wrappers (an "admin shell" inside the main "app shell", for example).

<a name="passing-data-to-the-layout"></a>

### Passing Extra Data to the Layout

The layout receives everything in `props` from the page automatically — `props.title`, `props.user`, anything else you passed to `render()`. To override a value just for the layout or to add layout-specific props, pass a second argument to `{#layout}`:

```html
{#layout('layout', { title: "Override Title", show_sidebar: false }) }
```

The object you pass is merged on top of `props`, so any keys you specify here win over what the page received.

<a name="includes"></a>

## Includes

`{#include('path') }` pulls another template inline. The included template shares the current data by default, and you can pass extra values as a second argument:

```html
{#include('partials/header') } {#include('./sidebar', { active: props.current_section }) }
```

Use `{#include}` when:

- The chunk doesn't need to be configured with props (a static header or footer).
- The chunk lives close to the page that uses it — a partial inside a route folder.
- You'd reach for it across feature boundaries and want a stable, addressable path.

For chunks that take props and live in a shared catalogue, [Components](/ree-templates/components) (`<component-name>` custom elements) are the cleaner option — they're built specifically for that case.

<a name="path-resolution"></a>

## Path Resolution

`{#include}` and `{#layout}` accept several path styles. The engine resolves each style consistently across both tags.

| Prefix         | Resolves From                    | Example                         |
| -------------- | -------------------------------- | ------------------------------- |
| `$components/` | Project root `components/`       | `$components/button`            |
| `$routes/`     | Project root `routes/`           | `$routes/home/hero`             |
| `$lib/`        | Project root `lib/`              | `$lib/flash`                    |
| `./` or `../`  | Relative to the current template | `./sidebar`, `../shared/footer` |
| `/name`        | Views root (`routes/`, absolute) | `/layouts/base`                 |
| `name`         | Views root (`routes/`, implicit) | `layouts/base`                  |

Without an extension, the engine appends `.ree` and treats the target as a template. With an explicit extension other than `.ree` — `.css`, `.svg`, `.txt`, anything else — the file is loaded and injected as raw text, unescaped:

```html
<style>
	{#include('./styles.css') }
</style>
```

This is occasionally useful for inlining a small stylesheet or an SVG icon. The file's content is dropped in verbatim, so don't include user-controlled paths here.

<a name="path-security"></a>

### Path Security

Path traversal outside the resolved base directory is blocked at compile time. An include like `{#include('../../../../etc/passwd') }` raises an error rather than reading a file outside `routes/`. Alias paths (`$lib/`, `$routes/`, `$components/`) are similarly anchored to their respective project-root directories.

<a name="data-flow-into-includes"></a>

## Data Flow Into Includes

Includes receive a merged copy of the current props plus whatever you pass as the second argument. The second argument wins on conflict:

```html
<!-- parent template -->
{{ const items = ["alpha", "beta", "gamma"] }} {#include('./list', { items, show_index: true }) }
```

```html
<!-- routes/list.ree -->
<ul>
	{#each props.items as item, i }
	<li>{#if props.show_index }<strong>{= i + 1 }.</strong>{/if } {= item }</li>
	{/each }
</ul>
```

Anything the parent had — `props.user`, `props.title`, translations — is still available inside the include. The second argument just adds (or overrides) on top.

<a name="when-to-use-each"></a>

## When to Use Each

A rough decision tree:

- **`{#layout}`** — once per page, at the top, for the surrounding HTML shell.
- **`{#include}`** — for a chunk of markup you want to factor out of a page, with or without extra data.
- **`<component-name>` custom elements** — for reusable, prop-driven pieces shared across pages (form inputs, banners, cards). Covered in [Components](/ree-templates/components).
