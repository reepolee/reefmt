---
title: "Components"
layout: "reeweb/docs/docs.layout"
---

# Components

<a name="introduction"></a>

## Introduction

Components in Ree are a thin shorthand for "include a template from the `components/` directory and pass it some props." There is no class, no lifecycle, and no registration step — a component is just a `.ree` file in `components/` that expects a particular shape of props.

You invoke a component by writing a **custom HTML element** whose tag name matches the file. Any tag name containing a hyphen that resolves to `components/<tag-name>.ree` is treated as a component:

```html
<input-text name="email" label="{= props.fields.email.label }" value="{= props.record.email }"></input-text>
```

That tag resolves to `components/input-text.ree`. The attributes and slot content are merged on top of the parent's `props`, so the component can read what you passed _and_ fall back to parent state — `props.record`, global values — when it needs to.

> **The old `{@name(...)}` shorthand has been removed.** Earlier versions of the engine accepted a function-style `{@input-text({ ... })}` call. That syntax no longer compiles — components are invoked **only** through the custom-element form below. If you're migrating an older template, replace each `{@name({ ... })}` with `<name ...></name>` and move the data onto attributes (see [Passing Data](#passing-data)).

<a name="invoking-a-component"></a>

## Invoking a Component

Write the component as a paired custom element. The tag name must match `components/<name>.ree` exactly:

```html
<banner type="green" text="Your changes have been saved."></banner>
```

The dash is required (a custom element needs at least one hyphen) and is also the naming convention — `input-text`, `input-select`, `banner`, `pagination`. If no file matches the tag under `components/`, the engine leaves it untouched as a literal HTML element, so a real `<my-widget>` web component passes straight through to the browser.

<a name="passing-data"></a>

## Passing Data: Attributes and Slots

A component receives two things: **attributes** (everything on the opening tag, under `props.attributes`) and **children** (the slot content between the tags, under `props.children`).

Attribute values can be static strings or interpolated expressions:

```html
<product-card title="Featured" price="{= product.price_formatted }" on_sale="{= product.discount > 0 }"></product-card>
```

- `title="Featured"` passes the literal string `"Featured"`.
- `price="{= product.price_formatted }"` is **interpolated** — the engine strips the `{= }` and evaluates `product.price_formatted` where the tag sits, so the component receives the real value (not the literal text). Use `{~ ... }` instead of `{= ... }` for raw, non-stringified values.

Because the tag is expanded in place, an interpolated attribute can read loop variables from a surrounding `{#each}` — this is how you hand per-item data to a component (see [Loops](/reeweb/docs/ree-templates/loops)). Slot content, by contrast, is compiled in its own scope and does **not** see those loop locals.

The slot is everything between the tags:

```html
<card-panel heading="Notes">
	<p>Any markup here is available to the component as {~ props.children }.</p>
</card-panel>
```

<a name="how-props-merge"></a>

## How Props Merge

When the component renders, the engine merges what you passed on top of the parent's props:

```ts
Object.assign({}, parent_props, { children, attributes });
```

That has two consequences worth knowing:

- **The parent's props are still there.** If the parent had `props.lang`, the component still has it — there's no "pass-through" boilerplate to maintain.
- **Your attributes and slot arrive under `props.attributes` and `props.children`,** keeping per-call data namespaced and separate from the inherited parent state.

<a name="writing-a-component"></a>

## Writing a Component

A component is just a `.ree` file that reads from `props.attributes` and `props.children`. Drop a new file into `components/` and reference it by its base name — there is no file to register it in; the naming convention is the convention.

A component destructures its attributes (often with a `...rest` spread to forward the leftovers) and renders the slot:

```ree
{{
const { type, text, ...rest } = props.attributes;
// choose a class based on type …
}}
<div class="{~ final_class } …" ...rest>{~ text }</div>
```

The `...rest` spread shorthand forwards any extra attributes onto the element — it expands to `{~ key_values(rest) }`.

<a name="available-components"></a>

## Components That Ship With Reeweb

A fresh Reeweb project includes three starter components in `src/components/`:

| Component               | Purpose                                                                                                                                             |
| ----------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------- |
| `banner.ree`            | Status/notification banner — renders a styled `<div>` in green, yellow, red, or neutral based on a `type` attribute                                 |
| `my-h1.ree`             | Heading component — uppercases its slot content and renders it as an `<h1>`, demonstrating how to work with `props.children` and `props.attributes` |
| `speculation-rules.ree` | Injects a `<script type="speculationrules">` block that enables instant navigation via the browser's Speculation Rules API                          |

<a name="composing-components"></a>

## Composing Components

Components can include other components — the same merging rules apply at every level. A component reads slot content via `props.children` and attributes via `props.attributes`:

```html
<service-item>
	{#each props.services as service}
	<md-text type="h3">{~ service.title }</md-text>
	{/each}
</service-item>
```

Inside `components/service-item.ree`, the component accesses the slot via `props.children` and its attributes via `props.attributes`.

<a name="components-vs-includes"></a>

## Components vs Includes

A component custom element resolves to an include from `components/`. When the target lives **outside** `components/` — a partial inside a template folder, or a subdirectory like `svgs/` — use `{#include}` directly, which takes an explicit path and an optional data object merged onto the parent props:

```html
{#include('$routes/_partials/breadcrumbs', { trail: props.trail }) }
```

Use a `<component-name>` custom element when the target is in `components/`; use `{#include('path', data)}` when you need an explicit path elsewhere. Both produce the same kind of merged-props include — the custom element is just the ergonomic form for the common, reusable case.
