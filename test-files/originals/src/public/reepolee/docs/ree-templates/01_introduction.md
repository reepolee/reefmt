---
title: "Introduction"
---

# Ree Templates

<a name="introduction"></a>

## Introduction

Ree is the templating language Reepolee uses for rendering HTML on the server. Files end in `.ree` and look like HTML with a small set of tags woven through them — for output, control flow, layouts, includes, and components. The whole language fits in a single page; the rest of this section walks through each piece in detail.

Templates compile to async JavaScript functions on first use and are cached in production. There is no separate build step and no precompilation phase — the engine reads the file, builds a function, and renders the output. In development, caching is off so changes appear immediately.

<a name="why-a-new-language"></a>

## Why a New Language

Ree is not novel. The syntax draws from Eta.js for output tags and from Svelte for control-flow blocks, both of which we found pleasant to read and easy to scan. Building it as part of Reepolee rather than reaching for an existing dependency keeps the runtime requirement at zero — there is nothing to install, nothing to update, and nothing in `node_modules` that can break six months from now.

The implementation lives in `lib/template_engine.ts` — a thin orchestrator (load, render, cache) that delegates to focused modules under `lib/template/`: `compiler.ts` (directives → render function), `custom_elements.ts` (comment stripping, `<tag-name>` and spread pre-processing), `include_handler.ts` and `include_resolver.ts` (includes and path resolution), and `types.ts`. This is the same engine code Reeweb ships, so templates behave identically on both. If you ever want to know exactly what a tag compiles to, you can read it.

> **HTML comments are stripped at build time.** Anything inside `<!-- ... -->` is removed before compilation, and template tags inside a comment are never evaluated — so commenting-out a block that references a missing field won't error. The flip side: comments never reach the rendered HTML.

<a name="the-data-object"></a>

## The props Object

Every template is rendered with a `props` object passed in from the route handler. Inside the template, everything you put in `props` is available under that name:

```ts
return render("users/index", {
	data: { records, ui: { title: "All users" } },
	ctx,
});
```

```html
<h1>{= props.ui.title }</h1>
{#each props.records as record }
<p>{= record.email }</p>
{/each}
```

The render layer also injects a handful of values into every template automatically — `props.user`, `props.lang`, `props.locale`, `props.toasts`, `props.request_url` — so you don't have to pass them yourself. Those are documented in [Helpers & Globals](/ree-templates/helpers-and-globals).

<a name="syntax-at-a-glance"></a>

## Syntax at a Glance

The whole language is six tag families. The rest of this section gives each one its own page.

| Tag                                              | Purpose                                 | Page                                                      |
| ------------------------------------------------ | --------------------------------------- | --------------------------------------------------------- |
| `{= expr }`                                      | Output, HTML-escaped (the safe default) | [Displaying Data](/ree-templates/displaying-data)         |
| `{~ expr }`                                      | Output, raw HTML (no escaping)          | [Displaying Data](/ree-templates/displaying-data)         |
| `{{ ... }}`                                      | Inline JavaScript block                 | [Displaying Data](/ree-templates/displaying-data)         |
| `{#if cond } ... {:else} ... {/if }`             | Conditional rendering                   | [Conditionals](/ree-templates/conditionals)               |
| `{#each list as item } ... {:else} ... {/each }` | Iteration                               | [Loops](/ree-templates/loops)                             |
| `{#with expr } ... {/with }`                     | Scope block (unqualified member access) | [Loops](/ree-templates/loops)                             |
| `{#layout('name') }`, `{#include('name') }`      | Composition                             | [Layouts & Includes](/ree-templates/layouts-and-includes) |
| `<component-name>...</component-name>`           | Component (custom element)              | [Components](/ree-templates/components)                   |

A complete page using most of these:

```html
{#layout('layout') } {{ const sorted = props.records.sort((a, b) => b.created_at - a.created_at) }}

<h1>{= props.ui.title }</h1>

{#if props.user }
<p>Welcome back, {= props.user.display_name }.</p>
{/if }

<ul>
	{#each sorted as record, i }
	<li>{= i + 1 }. {= record.email }</li>
	{:else }
	<li>No records yet.</li>
	{/each }
</ul>

<app-banner type="green">{= props.flash_message }</app-banner>
```

<a name="editor-support"></a>

## Editor Support

A VSCode extension provides syntax highlighting and formatting for `.ree` files. Install [Ree Templates for VSCode](https://marketplace.visualstudio.com/items?itemName=reepolee.ree-templates) from the marketplace.

For other editors, treating `.ree` as HTML gets you most of the way — the tag syntax is intentionally distinct enough that the HTML highlighter ignores it cleanly.
