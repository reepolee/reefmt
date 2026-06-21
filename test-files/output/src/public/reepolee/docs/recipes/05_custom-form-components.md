---
title: "Custom Form Components"
---

# Custom Form Components

<a name="introduction"></a>

## Introduction

The shipped form components cover every standard HTML input — text, email, password, date, select, checkbox, radio, textarea. For anything beyond that — a colour picker, a tag input with chips, a star rating, a date range, a rich-text editor — you write your own. The component is a `.ree` file in `components/`, follows the same data conventions as the shipped ones, and integrates with `FormController` for validation without any extra plumbing.

This recipe walks through three custom components of increasing complexity: a colour picker (single-value, native HTML), a tag input (multi-value, signal-driven), and a star rating (interactive, custom UI). By the end you'll have the patterns for any custom form input you might need.

> **A note on syntax.** Components are authored and invoked as [custom elements](/ree-templates/components#custom-elements) (`<my-component>`). Per-instance dynamic data — a field's `label` and `value` — is passed through **interpolated attributes** (`label="{= props.fields.x.label }"`), which the engine evaluates where the tag sits and delivers under `props.attributes`. (The old `{@component({...})}` function form has been removed.) In practice, generated forms inline the field markup directly; the examples below show standalone components for reuse.

<a name="the-shared-conventions"></a>

## The Shared Conventions

Every form component reads the same three values from `props.attributes` (passed as interpolated attributes on the custom element), so all components feel consistent in templates and the FormController integration is automatic:

| Attribute                | Used for                                                                                         |
| ------------------------ | ------------------------------------------------------------------------------------------------ |
| `props.attributes.name`  | The form field's `name` attribute                                                                |
| `props.attributes.label` | The visible label                                                                                |
| `props.attributes.value` | Current value (typically `props.record.<field>` from the parent template; blank for new records) |

The associated `<validation-error>` element gets the id `error-<name>` so `FormController` can find it and write the per-field error into it. The full integration is documented in [Validation](/forms/validation#client-side-form-controller).

Following these conventions means your component drops into existing forms without anyone having to read its source. It's not enforced — you can break the pattern if you have a good reason — but consistency is what makes the component catalogue easy to learn.

<a name="component-1-color-picker"></a>

## Component 1: Colour Picker

<media-frame label="Screenshot — colour picker component in a form" ratio="16/9"></media-frame>

The native `<input type="color">` is supported in every modern browser and gives you a system colour-picker dialog for free. Wrapping it in a Reepolee component takes about ten lines:

```html
<!-- components/input-color.ree -->
{{ const a = props.attributes; }}
<field-wrapper class="grid">
	<label class="px-3" for="{= a.name }">{= a.label }</label>
	<input
		type="color"
		id="{= a.name }"
		name="{= a.name }"
		value="{= a.value || '#000000' }"
		class="h-10 w-20 cursor-pointer rounded border border-neutral-300"
	/>
	<validation-error class="mt-1" id="error-{= a.name }"></validation-error>
</field-wrapper>
```

Key choices:

- **`value="{= a.value || '#000000' }"`** — falls back to black when no value is set, so the picker has something to render. `<input type="color">` requires a valid hex colour as the value; passing an empty string causes a console warning.
- **`<validation-error id="error-{= a.name }">`** — `FormController` finds this element by id and writes any per-field error into it.
- **`h-10 w-20`** — explicit sizing because browsers default to a tiny native picker.

Use it the same way as any shipped component:

```html
<form id="theme-form" method="POST" action="/themes">
	<input-text name="name" label="{= props.fields.name.label }" value="{= props.record.name }"></input-text>
	<input-color
		name="primary_color"
		label="{= props.fields.primary_color.label }"
		value="{= props.record.primary_color }"
	></input-color>
	<input-color
		name="accent_color"
		label="{= props.fields.accent_color.label }"
		value="{= props.record.accent_color }"
	></input-color>
	<button type="submit">Save</button>
</form>
```

The browser sends the hex value as a regular form field — `req.formData().get("primary_color")` returns `"#3b82f6"` and your handler stores it like any other string.

For server-side validation, treat it as a string with a regex check:

```ts
const color_pattern = /^#[0-9a-fA-F]{6}$/;
if (data.primary_color && !color_pattern.test(data.primary_color)) {
	errors.primary_color = translated.errors.invalid_color;
}
```

<a name="component-2-tag-input"></a>

## Component 2: Tag Input

<media-frame label="Animation — tag input adding and removing chips" ratio="16/9"></media-frame>

A tag input takes a comma-separated string and renders each value as a removable chip, with an input field for adding new tags. The UI is interactive — pressing Enter adds a chip, clicking × removes one — so the component pairs its markup with a small inline `<script type="module">` that drives the chip list off a `deepSignal()`.

```html
<!-- components/input-tags.ree -->
{{ const a = props.attributes; }}
<field-wrapper data-tag-input="{= a.name }" class="grid">
	<label class="px-3" for="{= a.name }">{= a.label }</label>

	<div data-tag-list class="flex flex-wrap items-center gap-2 rounded border border-neutral-300 px-3 py-2">
		<input data-tag-draft type="text" placeholder="Add tag…" class="flex-1 min-w-[8ch] outline-none" />
	</div>

	<input type="hidden" name="{= a.name }" data-tag-hidden value="{= a.value || '' }" />

	<validation-error class="mt-1" id="error-{= a.name }"></validation-error>
</field-wrapper>

<script type="module">
	import { deepSignal, watchEffect } from "/alien-deepsignals.min.js";

	const root = document.querySelector('[data-tag-input="{= a.name }"]');
	const list = root.querySelector("[data-tag-list]");
	const draft = root.querySelector("[data-tag-draft]");
	const hidden = root.querySelector("[data-tag-hidden]");

	const state = deepSignal({
		tags: hidden.value
			.split(",")
			.map((t) => t.trim())
			.filter(Boolean),
	});

	draft.addEventListener("keydown", (e) => {
		if (e.key !== "Enter" && e.key !== ",") return;
		e.preventDefault();
		const t = draft.value.trim();
		if (t && !state.tags.includes(t)) state.tags = [...state.tags, t];
		draft.value = "";
	});

	watchEffect(() => {
		const current = state.tags;
		hidden.value = current.join(",");
		list.querySelectorAll("[data-tag-chip]").forEach((n) => n.remove());
		for (const t of current) {
			const chip = document.createElement("span");
			chip.dataset.tagChip = "";
			chip.className = "inline-flex items-center gap-1 rounded-full bg-neutral-100 px-2 py-0.5 text-sm";
			chip.textContent = t;
			const x = document.createElement("button");
			x.type = "button";
			x.textContent = "×";
			x.className = "text-neutral-500 hover:text-red-600";
			x.onclick = () => (state.tags = state.tags.filter((x) => x !== t));
			chip.appendChild(x);
			list.insertBefore(chip, draft);
		}
	});
</script>
```

The pattern in three pieces:

- **The deep signal** holds the parsed tag list. The caller passes the existing tags via the interpolated `value` attribute (`props.attributes.value`) as a comma-separated string; the script splits it into an array on first read. Each `data-tag-input="..."` attribute is unique per component instance, so the script's queries only see _its own_ DOM — multiple tag inputs on the same form coexist without interference.
- **The single `watchEffect()`** is the renderer: whenever `state.tags` changes, it rebuilds the chip list and writes the joined value into the hidden input. ES modules give each `<script type="module">` block its own scope, so `state` and the rest never leak across instances.
- **The hidden input** is what actually gets submitted with the form. `FormController` discovers it by its `name` attribute the same way it discovers any other input.

The component handles add-on-Enter and add-on-comma — both common UX patterns for tag inputs. The shared keydown listener checks `e.key` directly; the chip list is rebuilt from scratch inside the `watchEffect`, which keeps the rendering logic in one place.

Server-side, the value arrives as a comma-separated string (`"design,frontend,react"`). Parse it however your data model needs:

```ts
const tags = (params.get("tags") || "")
	.split(",")
	.map((t) => t.trim())
	.filter(Boolean);

await db`UPDATE posts SET tags = ${tags.join(",")} WHERE id = ${id}`;
```

For per-tag validation (no spaces, max length, allowed character set), the same patterns from [Validation → Custom Rules](/forms/validation#custom-rules) apply.

<a name="component-3-star-rating"></a>

## Component 3: Star Rating

<media-frame label="Animation — star rating hover and select" ratio="16/9"></media-frame>

A 1–5 star rating widget — five clickable stars, the rating displayed visually, the chosen value submitted as a number. Two signals drive it: one for the committed value, one for the current hover state.

```html
<!-- components/input-rating.ree -->
{{ const a = props.attributes; }}
<field-wrapper data-rating-input="{= a.name }" class="grid">
	<label class="px-3">{= a.label }</label>

	<div class="flex gap-1">
		{#each [1, 2, 3, 4, 5] as n}
		<button
			data-rating-star="{= n }"
			type="button"
			class="text-2xl leading-none cursor-pointer transition-colors text-neutral-300"
		>
			★
		</button>
		{/each}
		<span data-rating-label class="ml-2 self-center text-sm text-neutral-500">Not rated</span>
	</div>

	<input type="hidden" name="{= a.name }" data-rating-hidden value="{= a.value || '0' }" />

	<validation-error class="mt-1" id="error-{= a.name }"></validation-error>
</field-wrapper>

<script type="module">
	import { deepSignal, watchEffect } from "/alien-deepsignals.min.js";

	const root = document.querySelector('[data-rating-input="{= a.name }"]');
	const hidden = root.querySelector("[data-rating-hidden]");
	const label = root.querySelector("[data-rating-label]");
	const stars = [...root.querySelectorAll("[data-rating-star]")];

	const state = deepSignal({ value: parseInt(hidden.value, 10) || 0, hover: 0 });

	stars.forEach((btn) => {
		const n = parseInt(btn.dataset.ratingStar, 10);
		btn.onclick = () => (state.value = n);
		btn.onmouseenter = () => (state.hover = n);
		btn.onmouseleave = () => (state.hover = 0);
	});

	watchEffect(() => {
		const v = state.value;
		const h = state.hover;
		hidden.value = String(v);
		label.textContent = v > 0 ? `${v} / 5` : "Not rated";
		for (const btn of stars) {
			const n = parseInt(btn.dataset.ratingStar, 10);
			const lit = h ? n <= h : n <= v;
			btn.classList.toggle("text-yellow-500", lit);
			btn.classList.toggle("text-neutral-300", !lit);
		}
	});
</script>
```

The interactive piece — hovering a star highlights it and every star to its left, clicking commits the rating — is the part that makes a custom component worth writing. The native HTML alternative is `<input type="number" min="1" max="5">`, which works but feels less like "rate this thing."

The hidden input pattern is the same as the tag input: the visible UI is signal-driven; the actual form value is a hidden input that `FormController` and the form submission both see. The single `watchEffect()` block re-runs whenever either `state.value` or `state.hover` changes and updates everything at once — the label text, the hidden value, and each star's class.

For nullable ratings (a "no rating yet" state), the default of `0` works — the form submits `value="0"` when no star is clicked, and your handler treats `0` as "not rated."

<a name="hooking-into-form-controller"></a>

## Hooking Into FormController

`FormController` discovers form fields by reading `name` attributes on every `<input>`, `<textarea>`, and `<select>` inside the form element. The hidden inputs in the tag and rating components are picked up automatically because they have `name` attributes — no special FormController call needed.

Live validation works the same way it does for shipped components: `FormController` posts the current values to the validate endpoint, the server returns errors keyed by field name, and the matching `<validation-error id="error-<name>">` element is filled in. Your component doesn't need to know about any of this — it just renders an empty `<validation-error>` with the right id and `FormController` writes into it.

For components where the error needs special placement (a tag input where the error should appear above the chips, not below the input), restructure the template freely. The convention is that _one_ `<validation-error>` element exists in the component with id `error-<name>`; where it lives in the markup is up to you.

<a name="organising-components"></a>

## Organising Components

For three or four custom components, drop them in `components/` alongside the shipped ones. For a project with many custom inputs, group them into subdirectories — `components/inputs/`, `components/widgets/`, etc.

A custom-element tag resolves the component name to `components/<name>.ree` (the top-level directory only), so a component in a subfolder is referenced with the full `{#include}` path instead:

```html
<input-color name="primary_color"></input-color>
<!-- components/input-color.ree -->
{#include('$components/inputs/input-color', { name: "primary_color" }) }
<!-- components/inputs/input-color.ree -->
```

The custom-element form is convenient for the top-level `components/` directory; for subfolders, the `{#include}` form is what you'd use. Most projects don't outgrow the flat layout — components/ is a single naming-collision boundary you only need to worry about when you have many of them.

<a name="when-to-write-a-custom-component"></a>

## When to Write a Custom Component

Three signals that a custom component is worth the effort:

- **The same combination of HTML elements appears in three or more forms.** A tag input that's only used once is fine to inline; one used across the site benefits from a component.
- **The native HTML element doesn't fit the use case.** A 1–5 rating with five clickable stars isn't an `<input type="number">`; a colour picker that previews live as you drag isn't an `<input type="color">`.
- **The behaviour is non-trivial enough that you don't want to copy-paste it.** Five lines of inline signal wiring is fine; thirty lines of inline signal wiring is a component.

The cost is small (one file, no registration step), so the bar to write one is low. The benefit compounds — every form that uses the component gets the same UX, the same accessibility, the same validation hookup, the same translations.

<a name="reading-the-shipped-components"></a>

## Reading the Shipped Components

The best reference for writing your own is the shipped components themselves. They're all in `components/` and most are 5–10 lines:

- **`input-text.ree`** — the canonical text-input shape.
- **`input-checkbox.ree`** — the conditional `checked` attribute pattern.
- **`input-textarea.ree`** — `field-sizing: content` for auto-growing.
- **`input-select.ree`** — `{#each}` over options with `{#if}` to mark the current selection.
- **`banner.ree`** — `{{ ... }}` JS for class computation, then a single `<div>`.

Read them when you want to know exactly what HTML you're getting or how a specific pattern fits together. They're shorter than this page describes.

<a name="whats-next"></a>

## What's Next

You have the pattern for any input the standard set doesn't cover. From here:

- **[Components](/ree-templates/components)** — custom-element invocation, interpolated attributes, and component-include mechanics.
- **[Validation](/forms/validation)** — server-side and live validation, custom rules, the FormController integration.
- **[Signals UI Patterns](/client-side/signals-ui)** — the `bind_*` helpers and the signal-driven idioms component scripts lean on.
- **[Signals](/client-side/signals)** — `deepSignal()` for form-shaped state, `watchEffect()` / `watch()` for reacting to it.
- **[Web Components](/client-side/web-components)** — when an input needs shadow DOM, encapsulation, or its own lifecycle, authoring custom HTML elements.
