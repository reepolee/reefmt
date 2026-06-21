---
title: "Input Components"
---

# Input Components

<a name="introduction"></a>

## Introduction

Every standard HTML form field has a matching `.ree` component in `components/` that shows the canonical markup for that field type — a `<field-wrapper>` around a `<label>`, the `<input>`, and a `<validation-error id="error-{name}">` that `FormController` writes per-field errors into.

In practice, **generated CRUD forms inline this markup** directly into `form.ree` with the field name baked in, reading the field's label and value straight off the page's `props` — because the field name is known at generation time. A text field looks like this in a form:

```html
<field-wrapper class="grid">
	<label class="px-3" for="email">{= props.labels.email }</label>
	<input type="email" id="email" name="email" value="{= props.email }" />
	<validation-error class="mt-1" id="error-email"></validation-error>
</field-wrapper>
```

The component files (`components/input-email.ree`, etc.) are the reference implementations of each field type — read them to see exactly what HTML you get. The `{@input-*({...})}` function shorthand that earlier versions used to invoke them **has been removed**; build forms from the inlined markup above (or invoke a component as a `<custom-element>` with interpolated attributes — see [Components](/ree-templates/components)).

<a name="the-shared-data-shape"></a>

## The Shared Data Shape

Each field needs three pieces of data, present in the inlined markup:

| Piece   | Purpose                                                                            |
| ------- | ---------------------------------------------------------------------------------- |
| `name`  | Field name (matches the column in the database) — the `id`/`name`/`for` attributes |
| `label` | Visible label, usually pulled from translations (`props.labels.<field>`)           |
| `value` | Current value (`props.<field>` or `props.record.<field>`); blank for new records   |

Because the value comes from `props`, the same markup works for create (where the value is undefined or empty) and edit (where the existing record's field is rendered) without any branching.

The associated `<validation-error>` element gets the id `error-{name}` so `FormController` can find it and write the per-field error into it. The full validation flow is on the [Validation](/forms/validation) page.

<a name="text-inputs"></a>

## Text Inputs

The text-input family covers most everyday fields. They differ only in the `type` attribute they render, but each one is its own component so you don't have to remember which `type` value to pass:

| Component        | HTML type  | Use for                                                              |
| ---------------- | ---------- | -------------------------------------------------------------------- |
| `input-text`     | `text`     | Names, codes, generic strings                                        |
| `input-email`    | `email`    | Email addresses (gets free client validation + mobile keyboard hint) |
| `input-password` | `password` | Passwords (masked)                                                   |
| `input-tel`      | `tel`      | Phone numbers                                                        |
| `input-url`      | `url`      | URLs                                                                 |
| `input-search`   | `search`   | Search boxes                                                         |

A typical text input renders along these lines:

```html
<field-wrapper class="grid">
	<label class="px-3" for="title">{= props.labels.title }</label>
	<input type="text" id="title" name="title" value="{= props.title }" />
	<validation-error class="mt-1" id="error-title"></validation-error>
</field-wrapper>
```

`<field-wrapper>` is a bare custom element (no JavaScript registration required) that exists purely for styling — browsers treat any unknown element with a hyphenated name as `HTMLElement`, and `<field-wrapper>` wraps the label, input, and error together for grid layout. `FormController` discovers the input by its `name` attribute and writes any per-field error into the matching `<validation-error id="error-{name}">` element.

<a name="number-inputs"></a>

## Number, Date, and Time Inputs

| Component              | HTML type        | Notes                                           |
| ---------------------- | ---------------- | ----------------------------------------------- |
| `input-number`         | `number`         | Numeric input with browser-side step validation |
| `input-date`           | `date`           | Renders an ISO date string (`YYYY-MM-DD`)       |
| `input-time`           | `time`           | Renders a 24-hour time string (`HH:MM`)         |
| `input-datetime-local` | `datetime-local` | Combined date and time picker                   |

For date and datetime fields backed by SQL `DATETIME` columns, apply a codec in the form schema so the database format and the input format both work — see [Date Codecs](/forms/validation#date-codecs).

<a name="textarea"></a>

## Textarea

`input-textarea` renders a `<textarea>` element. It uses CSS `field-sizing: content` so the textarea grows with its content instead of showing a scrollbar:

```html
<field-wrapper class="grid">
	<label class="px-3" for="bio">{= props.labels.bio }</label>
	<textarea id="bio" name="bio">{= props.bio }</textarea>
	<validation-error class="mt-1" id="error-bio"></validation-error>
</field-wrapper>
```

For rich-text editing, swap in a JavaScript editor (Pell, Quill, ProseMirror) in your own custom component. Reepolee ships a Pell-based editor for the [Email Module](/email/email-module) admin form.

<a name="select-and-checkbox"></a>

## Select, Checkbox, and Radio

`input-select` renders a `<select>` with options pulled from `props.options[name]`. The parent template provides the options and loops them into the markup:

```html
<field-wrapper class="grid">
	<label class="px-3" for="role">{= props.labels.role }</label>
	<select id="role" name="role">
		{#each props.options.role as opt }
		<option value="{= opt }" {#if opt === props.role }selected{/if }>{= opt }</option>
		{/each }
	</select>
	<validation-error class="mt-1" id="error-role"></validation-error>
</field-wrapper>
```

The `input-select.ree` component reads `props.options[props.name]`, so the same component works whether you pass options inline or pre-attach them to a shared `props.options` object higher up.

`input-checkbox` renders an `<input type="checkbox">`. It checks the box when the value is truthy. Submit semantics follow native HTML: a checked box sends `name=on`; an unchecked box sends nothing. Reepolee's generated handlers translate this into a boolean by checking whether the field was present in the request body.

`input-radio` renders a single `<input type="radio">`. For a radio group, render the component multiple times with the same `name` and different `value` props. Native form submission picks the right one.

<a name="select-foreign-keys"></a>

### Foreign-Key Selects

The generator detects foreign-key relationships (either by explicit `FOREIGN KEY` constraint or by columns ending in `_id`) and renders them with `input-select` automatically. The dropdown is populated from the related table's first non-integer text column, or from a column named `title` or `name` if one exists. You can override the label source in the route's `table.ts` after generation.

<a name="banner"></a>

## Banner

<media-frame label="Screenshot — banner component, all four types (red · yellow · green · neutral)" ratio="16/9"></media-frame>

`app-banner` is for form-level messages — successes, errors, or warnings that apply to the whole form rather than a single field. It takes a `type` attribute and the message as slot content:

```html
{#if props.form_errors }
<app-banner type="red">{= props.form_errors }</app-banner>
{/if }
```

The four `type` values: `"red"` (errors), `"yellow"` (warnings), `"green"` (success), or anything else (neutral, with a light border). The component reads `props.attributes.type`, computes the final class string in a `{{ ... }}` block, and renders a styled `<div>` around `props.children`.

<a name="component-list"></a>

## Complete List

<media-frame label="Screenshot — gallery of all shipped input components" ratio="4/3"></media-frame>

For reference, the full set shipped with Reepolee:

```
components/
├── app-banner.ree
├── input-checkbox.ree
├── input-date.ree
├── input-datetime-local.ree
├── input-email.ree
├── input-number.ree
├── input-password.ree
├── input-radio.ree
├── input-search.ree
├── input-select.ree
├── input-tel.ree
├── input-text.ree
├── input-textarea.ree
├── input-time.ree
└── input-url.ree
```

Each of those is a small (~10 line) `.ree` file. Read them when you want to know exactly what HTML you're getting — they are short, self-contained, and easier to read than this documentation describes.

<a name="extending-and-customizing"></a>

## Extending and Customizing

To change how an input renders project-wide, edit the component file directly. It is part of your codebase, not a runtime dependency — the changes you make persist forever and apply to every form that uses the component.

To add a new field type — a colour picker, a slider, a tag input — drop a new file into `components/` and reference it by name. The [Custom Form Components](/recipes/custom-form-components) recipe walks through this end-to-end, including how to ensure live validation continues to work and how to add signal-driven interactivity for inputs that need it.

If you find yourself needing a one-off variant for a specific page — a wider text input on the email composer, say — add the variation as a CSS class right in the inlined markup:

```html
<field-wrapper class="grid">
	<label class="px-3" for="subject">Subject</label>
	<input type="text" id="subject" name="subject" value="{= props.record.subject }" class="w-full text-lg" />
	<validation-error class="mt-1" id="error-subject"></validation-error>
</field-wrapper>
```

If you instead invoke a reusable component as a custom element, pass the extra class as an interpolated attribute — `<input-text name="subject" input_class="w-full text-lg"></input-text>` — and read it inside the component via `props.attributes.input_class`.
