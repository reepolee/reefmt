---
title: "Introduction"
layout: "reeweb/docs/docs.layout"
---

# Ree Templates

<a name="introduction"></a>

## Introduction

Ree is the templating language Reeweb uses for rendering HTML on the server. Files end in `.ree` and look like HTML with a small set of tags woven through them — for output, control flow, layouts, includes, and components. The whole language fits in a single page; the rest of this section walks through each piece in detail.

Templates compile to async JavaScript functions on first use and are cached in production. There is no separate build step and no precompilation phase — the engine reads the file, builds a function, and renders the output. In development, caching is off so changes appear immediately.

<a name="why-a-new-language"></a>

## Why a New Language

Ree is not novel. The syntax draws from Eta.js for output tags and from Svelte for control-flow blocks, both of which we found pleasant to read and easy to scan. Building it as part of Reeweb rather than reaching for an existing dependency keeps the runtime requirement at zero — there is nothing to install, nothing to update, and nothing in `node_modules` that can break six months from now.

The implementation lives in `lib/template_engine.ts` — a thin orchestrator (load, render, cache) that delegates to focused modules under `lib/template/`: `compiler.ts` (directives → render function), `custom_elements.ts` (comment stripping, `<tag-name>` and spread pre-processing), `include_handler.ts` and `include_resolver.ts` (includes and path resolution), and `types.ts`. This is the same engine code Reepolee ships, so templates behave identically on both. If you ever want to know exactly what a tag compiles to, you can read it.

> **HTML comments are stripped at build time.** Anything inside `<!-- ... -->` is removed before compilation, and template tags inside a comment are never evaluated. That makes commenting-out a block safe (a `{= props.missing }` inside a comment won't error), but it also means comments never reach the rendered HTML — don't rely on them being in the output.

<a name="the-data-object"></a>

## The props Object

Every template is rendered with a `props` object. You supply that object from a `.ts` sibling file next to the template — export a `load_template_data` function and whatever it returns becomes `props` in the page:

```ts
// src/public/blog/index.ts
export async function load_template_data({ lang }: { lang: string }) {
	return {
		posts: await fetch_posts(lang),
		ui: { title: "Blog" },
	};
}
```

```html
<h1>{= props.ui.title }</h1>
{#each props.posts as post }
<p>{= post.title }</p>
{/each}
```

The build pipeline also injects a handful of values into every template automatically — `props.lang`, `props.locale`, `props.canonical_path`, `props.rendered_at`, `props.is_dev`, `props.lang_url_prefix` — so you don't have to pass them yourself. Those are documented in [Helpers & Globals](/reeweb/docs/ree-templates/helpers-and-globals).

<a name="syntax-at-a-glance"></a>

## Syntax at a Glance

The whole language is six tag families. The rest of this section gives each one its own page.

| Tag                                              | Purpose                                 | Page                                                                  |
| ------------------------------------------------ | --------------------------------------- | --------------------------------------------------------------------- |
| `{= expr }`                                      | Output, HTML-escaped (the safe default) | [Displaying Data](/reeweb/docs/ree-templates/displaying-data)         |
| `{~ expr }`                                      | Output, raw HTML (no escaping)          | [Displaying Data](/reeweb/docs/ree-templates/displaying-data)         |
| `{{ ... }}`                                      | Inline JavaScript block                 | [Displaying Data](/reeweb/docs/ree-templates/displaying-data)         |
| `{#if cond } ... {:else} ... {/if }`             | Conditional rendering                   | [Conditionals](/reeweb/docs/ree-templates/conditionals)               |
| `{#each list as item } ... {:else} ... {/each }` | Iteration                               | [Loops](/reeweb/docs/ree-templates/loops)                             |
| `{#with expr } ... {/with }`                     | Scope block (unqualified member access) | [Loops](/reeweb/docs/ree-templates/loops)                             |
| `{#layout('name') }`, `{#include('name') }`      | Composition                             | [Layouts & Includes](/reeweb/docs/ree-templates/layouts-and-includes) |
| `<component-name>...</component-name>`           | Component (custom element)              | [Components](/reeweb/docs/ree-templates/components)                   |

A complete page using most of these:

```html
{#layout('layout') } {{ const sorted = props.posts.sort((a, b) => b.date - a.date) }}

<h1>{= props.ui.title }</h1>

{#if props.is_dev }
<p class="text-xs text-gray-400">Rendered at {= props.rendered_at }</p>
{/if }

<ul>
	{#each sorted as post, i }
	<li>{= i + 1 }. {= post.title }</li>
	{:else }
	<li>No posts yet.</li>
	{/each }
</ul>

<banner type="info" text="{= props.ui.notice }"></banner>
```

<a name="editor-support"></a>

## Editor Support

A VSCode extension provides syntax highlighting and formatting for `.ree` files. Install [Ree Templates for VSCode](https://marketplace.visualstudio.com/items?itemName=reepolee.ree-templates) from the marketplace.

For other editors, treating `.ree` as HTML gets you most of the way — the tag syntax is intentionally distinct enough that the HTML highlighter ignores it cleanly.
