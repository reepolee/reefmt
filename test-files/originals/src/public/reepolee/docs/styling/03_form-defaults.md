---
title: "Form Defaults"
---

# Form Defaults

<a name="introduction"></a>

## Introduction

Forms touch enough native HTML elements that fighting their defaults page-by-page gets tiring. Reepolee's `css/forms.css` sets up consistent styling for every input, textarea, select, and submit button so the form components don't need per-element class names. The file is imported from `css/app.css` and lives in version control — you're free to edit or replace it.

Because the rules target the elements themselves (and not Tailwind classes added in markup), the styling applies to every form in the application by default. Templates focus on layout and behaviour; the visual treatment of native controls is one concern in one file.

<a name="text-inputs"></a>

## Text Inputs

The base rule covers every input type, every textarea, and any element with the `.input` class:

```css
form input,
form textarea,
form .input {
	@apply bg-bg-card rounded border border-neutral-300 px-3 py-1;
}
```

The token reference (`bg-bg-card`) means the input colour follows the theme — change `--color-bg-card` once in `@theme` and every input in the app updates. The neutral border keeps focus on the content rather than the chrome.

`.input` is for elements that aren't `<input>` but should look like one — a content-editable `<div>` used as a rich-text editor surface, for example. Adding `class="input"` makes it visually consistent with native inputs without restyling the editor library's internals.

<a name="textareas-that-grow"></a>

## Textareas That Grow

<media-frame label="Animation — textarea auto-growing with content" ratio="16/9"></media-frame>

Reepolee's textarea component uses the modern `field-sizing: content` property, which sizes the element to its content automatically:

```css
form textarea {
	field-sizing: content;
	min-height: 5lh;
}
```

`5lh` is "five line heights" — the minimum height is five lines of text in the current font size, and the textarea grows beyond that as the user types. No scrollbar inside the textarea, no JavaScript to listen for input events. Browsers without support fall back to a fixed-height textarea, which is no worse than the default.

If you want a fixed-height textarea on a specific form, add an inline class that overrides the rule:

```html
<textarea class="h-32! field-sizing-fixed!">...</textarea>
```

The trailing `!` is Tailwind's `!important` modifier, which beats the form-default rule in specificity. Use it sparingly.

<a name="submit-buttons"></a>

## Submit Buttons

Every `type="submit"` button picks up the `primary` utility automatically:

```css
form button[type="submit"] {
	@apply primary cursor-pointer rounded px-3 py-2 hover:scale-105;
}
```

This means a typical form ends with the simplest possible button markup:

```html
<button type="submit">Save</button>
```

…and gets the brand colour, contrast text, padding, rounded corners, and a small hover-scale effect. No `class="primary rounded px-3 py-2 ..."` to remember.

For non-submit buttons (cancel, delete, secondary actions), the utility classes work as usual:

```html
<button type="button" class="secondary rounded px-3 py-2">Cancel</button>
```

Or use the `[role="button"]` selector from `@layer components` to make a link look like a button:

```html
<a href="/users" role="button">Back to users</a>
```

<a name="selects"></a>

## Selects

<media-frame label="Animation — select picker-icon rotating on open/close" ratio="16/9"></media-frame>

The `<select>` element gets `appearance: base-select` — a modern CSS API that gives the native dropdown a custom look without losing its native behaviour:

```css
select {
	appearance: base-select;

	&::picker-icon {
		content: "❯";
		transition: 0.3s rotate;
		rotate: 90deg;
	}

	&:open::picker-icon {
		rotate: 270deg;
	}
}
```

The chevron rotates between closed (pointing right) and open (pointing down) states using only CSS — no `<svg>` to manage, no `aria-expanded` to listen for. Browsers without support fall back to the default OS-rendered dropdown.

The `&::picker-icon` and `&:open` nested selectors are CSS-native nesting (Tailwind v4 supports it as part of the spec, not as a preprocessor feature).

<a name="checkboxes-and-radios"></a>

## Checkboxes and Radios

Checkboxes and radio buttons keep their native appearance but get a project-themed accent colour. The rule lives in `@layer components` in `app.css`:

```css
@layer components {
	input[type="checkbox"] {
		@apply accent-brand h-5 w-5;
	}
}
```

`accent-color` is the standardised way to recolour native checkboxes and radios — it touches the check mark, the focus ring, and the selected radio dot without breaking accessibility or screen-reader behaviour. Older browsers fall back to the system colour, which is still readable.

<a name="field-wrappers"></a>

## Field Wrappers

The shipped input components wrap each field in a `<field-wrapper>` custom element. The element has no JavaScript behaviour — it's a styling hook so labels, inputs, and validation errors can be laid out together without polluting the template with extra div-class boilerplate:

```html
<field-wrapper class="grid">
	<label for="email">Email</label>
	<input type="email" id="email" name="email" />
	<validation-error class="mt-1" id="error-email"></validation-error>
</field-wrapper>
```

The `class="grid"` puts the label, input, and error in a single-column CSS grid. To switch to a side-by-side layout on a specific form, change the class on the wrapper:

```html
<field-wrapper class="grid grid-cols-[12rem_1fr] gap-3 items-baseline"> ... </field-wrapper>
```

Field-wrapper styling has no global default rule, so each form is free to use the grid (or flex, or whatever) layout that fits.

<a name="customising-the-form-look"></a>

## Customising the Form Look

A few common adjustments and how to make them:

- **Larger inputs across the application** — change `py-1` to `py-2` in the base rule.
- **A different border colour** — replace `border-neutral-300` with a custom token reference, e.g. `border-border` (where `--color-border` lives in `@theme`).
- **Rounded pill inputs** — replace `rounded` with `rounded-full` for that variant; create a `.input.pill` rule if you want it on some inputs but not others.
- **Disable the hover-scale on submit** — drop the `hover:scale-105` from the submit-button rule. The `transition` is implicit.

All of these are one-line edits in `forms.css`. There is no theme system that needs updating elsewhere — `forms.css` is the theme system for form controls.

<a name="forms-without-the-defaults"></a>

## Forms Without the Defaults

A form outside the `<form>` element doesn't pick up the rules (the selector is `form input`). This is occasionally useful — a search input in the header, a "type to filter" box on a list page — where you want a more compact, less form-y look:

```html
<input type="search" placeholder="Filter..." class="rounded border px-2 py-1 text-sm" />
```

The input still uses Tailwind utilities directly. Inputs inside a `<form>` get the defaults; inputs outside get whatever you write in the class list.
