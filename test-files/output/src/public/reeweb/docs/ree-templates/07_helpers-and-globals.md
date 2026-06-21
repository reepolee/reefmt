---
title: "Helpers & Globals"
layout: "reeweb/docs/docs.layout"
---

# Helpers & Globals

<a name="introduction"></a>

## Introduction

Every template is rendered with more than just the `props` you pass. Two other categories are available automatically: **helpers**, which are functions you call to format or transform values, and **globals**, which are values the build/render layer injects on every render. This page covers both.

<a name="built-in-helpers"></a>

## Built-in Helpers

Helpers are functions you call directly in your templates. Reeweb ships with helpers from two sources:

### Base Helpers (from `lib/template_helpers.ts`)

These are always available via `props.helpers.xxx`:

| Helper                                                | Returns                                        | Example                                                   |
| ----------------------------------------------------- | ---------------------------------------------- | --------------------------------------------------------- |
| `url(path)`                                           | Ensures a path starts with `/`                 | `<a href="{= url('profile') }">`                          |
| `localized_path(canonical)`                           | Localised path for the active language         | `<a href="{~ props.helpers.localized_path('/blog') }">`   |
| `nav_label(key)`                                      | Translated nav label from `props.nav`          | `{= nav_label('users') }`                                 |
| `is_current(url)`                                     | Active nav class if current URL matches        | `<a class="{= is_current('/about') }">`                   |
| `key_values(obj)`                                     | Spreads an object's entries as HTML attributes | `<div ...rest>` (shorthand)                               |
| `js_date_to_locale_string(val)`                       | Locale-formatted date                          | `{= js_date_to_locale_string(post.date) }`                |
| `js_time_to_locale_string(val)`                       | Locale-formatted time                          | `{= js_time_to_locale_string(event.time) }`               |
| `js_datetime_to_locale_string(val)`                   | Locale-formatted date + time                   | `{= js_datetime_to_locale_string(record.updated_at) }`    |
| `js_timestamp_to_locale_string(val)`                  | Locale-formatted timestamp incl. seconds       | `{= js_timestamp_to_locale_string(log.created_at) }`      |
| `js_date_to_iso_string(val)`                          | ISO date string (`YYYY-MM-DD`)                 | `<time datetime="{= js_date_to_iso_string(post.date) }">` |
| `js_datetime_to_iso_string(val)`                      | ISO datetime (`YYYY-MM-DDTHH:mm`)              | `<input value="{= js_datetime_to_iso_string(val) }">`     |
| `js_timestamp_to_iso_string(val)`                     | Full ISO timestamp                             | machine-readable timestamps                               |
| `display_currency(val, locale?, hide_zero?, symbol?)` | Currency string (default `€`)                  | `{~ display_currency(record.price) }`                     |
| `display_percent(val)`                                | Percent string, locale-aware                   | `{= display_percent(rate) }`                              |
| `yes_no(val, type?)`                                  | Yes/No badge (HTML)                            | `{~ yes_no(record.is_active) }`                           |
| `pill(text, class)`                                   | Single-pill HTML span                          | `{~ pill(status, 'pill-info') }`                          |
| `tags(csv, class?)`                                   | Renders comma-separated string as pills        | `{~ tags(user.tags) }`                                    |
| `human_bytes(bytes)`                                  | Human-readable byte count                      | `{= human_bytes(file.size) }`                             |
| `urlencode(str)` / `urldecode(str)`                   | URL component coding                           | `<a href="?q={= urlencode(query) }">`                     |

### Project-Specific Helpers (`src/lib/project_helpers.ts`)

`src/lib/project_helpers.ts` is where you add your own project-specific helpers. In a fresh Reeweb project the file is an empty stub with no exports — it exists as a designated place for your additions rather than shipping with built-in helpers.

To add a helper, export it from this file and register it in the object passed to `create_template_helpers`. See the [Custom Helpers](#custom-helpers) section below for the pattern.

### Calling Helpers in Templates

Helpers are available as bare names in `.ree` templates — no prefix needed. The template engine injects them automatically at render time via the helpers object:

```html
<a href="{~ localized_path('/blog') }">Blog</a>
<time datetime="{= js_date_to_iso_string(post.date) }">{= js_date_to_locale_string(post.date) }</time>
{~ yes_no(record.is_active) }
```

A few details worth knowing:

- **Date/time helpers default to `props.locale`**, which is derived from the active language (`"en-US"` for English, `"sl-SI"` for Slovenian). Pass an explicit second argument to override: `js_date_to_locale_string(date, "fr-FR")`.
- **`display_currency` takes optional arguments**: `display_currency(val, locale?, hide_zero?, symbol?)`. `hide_zero = true` returns the empty string for zero values.
- **`yes_no` returns HTML**, so use the raw output tag (`{~ }`) when you call it.
- **`...rest` shorthand** in templates spreads an object's entries as HTML attributes via `key_values()` — used in components for attribute passthrough.

<a name="custom-helpers"></a>

## Custom Helpers

To add a project-wide helper that's available at build time, edit `src/lib/project_helpers.ts` and add your function to the `project_helper_functions` object:

```ts
// src/lib/project_helpers.ts
export const project_helper_functions: Record<string, unknown> = {
	// ... add your helpers here
	uppercase: (text: string) => text.toUpperCase(),
};
```

Once registered via `create_template_helpers(props, project_helper_functions)`, the helper is available in every template:

```html
{#each props.records as user }
<tr>
	<td>{= uppercase(user.name) }</td>
</tr>
{/each }
```

A few patterns that come up often:

- **Formatting** — single-purpose transforms (`price`, `slug`).
- **Conditional display** — return one string in one case, another in another. Keeps the template free of nested `{#if}` blocks.
- **HTML generation** — return a small chunk of markup (a badge, a status pill). Always use `{~ }` to output the result.

<a name="helper-rules"></a>

### Helper Rules

- **Helpers are functions, always called with `()`**. `{= uppercase(name) }` works; `{= uppercase }` does not.
- **Helpers receive only their arguments**. They cannot read template variables they weren't passed.
- **Helpers run during rendering**. Return values are inserted into the template output.

The most common mistake is referencing a helper that isn't registered — the template throws "helper is not defined" at render time. The fix is adding the function to `project_helper_functions` in `src/lib/project_helpers.ts` or to `create_default_helpers()` in `lib/template_helpers.ts`.

<a name="global-variables"></a>

## Global Variables

The build/render layer injects a set of values into `props` automatically. You access them the same way as anything else:

| Variable                 | Source                        | Description                           |
| ------------------------ | ----------------------------- | ------------------------------------- |
| `props.lang`             | Config                        | Active language code (`"en"`, `"sl"`) |
| `props.locale`           | Derived from `props.lang`     | Locale string (`"en-US"`, `"sl-SI"`)  |
| `props.request_url`      | Current request               | Relative URL of the current page      |
| `props.rendered_at`      | `new Date().toISOString()`    | Render timestamp (ISO 8601)           |
| `props.active_languages` | `$config/supported_languages` | List of available language codes      |
| `props.language_names`   | `$config/supported_languages` | Map of code → display name            |
| `props.year`             | `new Date().getFullYear()`    | Current year for copyright            |
| `props.helpers`          | `create_template_helpers()`   | Object of template helper functions   |
| `props.is_dev`           | Build mode                    | `true` when running `bun dev`         |

These read just like any other field in templates:

```html
<footer>© {= props.year } Reeweb</footer>

{#if props.is_dev }
<div class="dev-banner">Development mode</div>
{/if }
```

<a name="rendering-to-a-string"></a>

## Rendering to a String

In the static build flow, templates are rendered to HTML files via `scripts/build.ts`. For programmatic use, you can use the template engine directly:

```ts
const html = await engine.render(template_name, data);
```
