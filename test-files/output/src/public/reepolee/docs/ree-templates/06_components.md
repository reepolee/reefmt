---
title: "Components"
---

# Components

<a name="introduction"></a>

## Introduction

A component is just a `.ree` file in the `components/` directory — there is no class, no lifecycle, and no registration step. You use one the way you'd use any HTML element: as a **custom element**.

```html
<site-nav></site-nav> <field-wrapper class="grid">…</field-wrapper>
```

Custom-element syntax is the form for every component — it reads as HTML and matches the convention the rest of the project uses (`<field-wrapper>`, `<validation-error>`, `<auto-complete>`, `<app-banner>`). Per-call data goes on attributes (which can be interpolated) and in the slot.

> **The old `{@name({...})}` function shorthand has been removed.** Earlier versions of the engine accepted a function-style component call for passing evaluated data. That syntax no longer compiles — a literal `{@…}` now passes straight through as text. Components are invoked **only** as custom elements, and dynamic data is passed through interpolated attributes (see below). Reepolee's own templates already use this form throughout.

<a name="custom-elements"></a>

## Custom Elements

Any tag whose name matches a file in `components/` is rendered by including that file. `<my-card>` resolves to `components/my-card.ree`. If no matching file exists, the tag is passed through as a literal HTML element, so custom elements never collide with real HTML.

A custom element compiles to an include that hands the component three things:

- **The full parent `props`** — everything the surrounding template can see (including globals like `lang`, `localized_path`, `md`, `tw_merge`) is in scope inside the component.
- **`props.children`** — the slot content between the opening and closing tags.
- **`props.attributes`** — the tag's attributes, as a plain object.

```html
<!-- components/info-box.ree -->
<aside class="info-box {= props.attributes.tone }">{~ props.children }</aside>
```

```html
<!-- usage -->
<info-box tone="warning"> {~ props.notice_html } </info-box>
```

<a name="attribute-interpolation"></a>

### Attributes can be static or interpolated

An attribute value is either a literal string or an interpolated expression:

- **Static:** `tone="warning"` passes the string `"warning"`.
- **Interpolated:** `tone="{= props.level }"` — the engine strips the `{= }` and evaluates `props.level` **where the tag sits**, so the component receives the real value, not the literal text. Use `{~ … }` for raw (non-stringified) values.

Both arrive under `props.attributes` inside the component (`props.attributes.tone`). Because the tag is expanded in place, an interpolated attribute can read a surrounding `{#each}` loop variable — this is how you hand per-item data to a component:

```html
{#each props.rows as row }
<field-cell label="{= row.label }" value="{= row.value }"></field-cell>
{/each }
```

> **Slot scope:** slot content (between the tags) compiles in an isolated scope that sees only `props` and globals — **not** page-local `const`s or `{#each}` loop variables. When a component needs per-iteration data, pass it through interpolated attributes (above) rather than the slot.

<a name="includes"></a>

## When to Use `{#include}` Instead

A custom element resolves to an include from `components/`. When the target lives **outside** `components/` — a partial inside a route folder, or a subdirectory — use `{#include}` directly, which takes an explicit path and an optional data object merged on top of the parent's props:

```html
{#include('$routes/dashboard/_partials/summary', { totals: props.totals }) }
```

The merge is `Object.assign({}, parent_props, your_obj)`, so you can override anything the parent had and you don't have to forward props you didn't change.

<a name="writing-a-component"></a>

## Writing a Component

A component reads from `props.attributes` (per-call config) and `props.children` (slot), plus any inherited parent props it needs. The shipped `app-banner` is a good model — `type` comes in as an attribute, the message as the slot:

```html
<!-- components/app-banner.ree -->
{{ const tone = props.attributes.type || "neutral"; }}
<div class="banner banner-{= tone }">{~ props.children }</div>
```

```html
<!-- usage -->
<app-banner type="red">{= props.form_error }</app-banner>
```

Note that a component body is itself built from custom elements — `<field-wrapper>` and `<validation-error>` are components used as plain tags. To write your own, drop a file into `components/` and reference it by its base name (kebab-case is the convention — `field-wrapper`, `auto-complete`, `app-banner`). There is no file to register it in.

<a name="forms-inline-fields"></a>

## Forms Inline Their Fields

Generated CRUD forms do **not** call per-field input components. The generator **inlines** the field markup — `<field-wrapper>`, `<label>`, `<input>`, `<validation-error>` — directly into `form.ree`, with the field name baked in. The `<validation-error>` gets the id `error-{name}`, which is the convention `FormController` uses to find it and write the per-field error into it (see [Validation](/forms/validation)):

```html
<field-wrapper class="grid">
	<label class="px-3" for="name">{= props.labels.name } *</label>
	<input type="text" id="name" name="name" value="{= props.name }" required />
	<validation-error class="mt-1" id="error-name"></validation-error>
</field-wrapper>
```

This is why the input markup reads directly off `props` (`props.name`, `props.labels.name`) — it lives inline in the page template, not inside a separate component scope.

<a name="composing-components"></a>

## Composing Components

Components nest freely — a component's body can use other components as custom elements, passing data through interpolated attributes:

```html
<!-- components/card.ree -->
<article class="card">
	<header>
		<user-avatar></user-avatar>
		<!-- reads props.user, inherited -->
		<h3>{= props.attributes.title }</h3>
	</header>
	{~ props.children }
</article>
```

```html
<!-- usage: title via attribute, body via slot -->
<card title="{= props.headline }"> {~ props.announcement_html } </card>
```

The same merging rules apply at every level — the inner component sees the parent's props plus its own attributes and children.
