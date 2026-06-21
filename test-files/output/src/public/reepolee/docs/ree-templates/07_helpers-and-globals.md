---
title: "Helpers & Globals"
---

# Helpers & Globals

<a name="introduction"></a>

## Introduction

Every template is rendered with more than just the `props` you pass from your handler. Two other categories are available automatically: **helpers**, which are functions you call to format or transform values, and **globals**, which are values the render layer injects on every render — the current user, the active language, pending toasts, and so on. This page covers both.

<a name="built-in-helpers"></a>

## Built-in Helpers

Helpers are functions you call directly in your templates. Reepolee ships with the following. They cover URL handling, localisation-aware formatting, and a few display conveniences:

| Helper                               | Returns                                                                           | Example                                                   |
| ------------------------------------ | --------------------------------------------------------------------------------- | --------------------------------------------------------- |
| `url(path)`                          | Ensures a path starts with `/`                                                    | `<a href="{= url('profile') }">`                          |
| `localized_path(canonical)`          | Localised path for the active language                                            | `<a href="{~ localized_path('/login') }">`                |
| `nav_label(key)`                     | Translated label for a nav entry from `props.nav`                                 | `{= nav_label('users') }`                                 |
| `is_current(url)`                    | `"font-bold nav-item current"` if the current URL matches; otherwise `"nav-item"` | `<a class="{= is_current('/users') }">`                   |
| `js_date_to_locale_string(val)`      | Locale-formatted date                                                             | `{= js_date_to_locale_string(record.born_on) }`           |
| `js_time_to_locale_string(val)`      | Locale-formatted time                                                             | `{= js_time_to_locale_string(record.opens_at) }`          |
| `js_datetime_to_locale_string(val)`  | Locale-formatted date + time                                                      | `{= js_datetime_to_locale_string(record.updated_at) }`    |
| `js_timestamp_to_locale_string(val)` | Locale-formatted timestamp incl. seconds                                          | `{= js_timestamp_to_locale_string(record.logged_at) }`    |
| `js_date_to_iso_string(val)`         | ISO date string (`YYYY-MM-DD`)                                                    | `<time datetime="{= js_date_to_iso_string(post.date) }">` |
| `js_datetime_to_iso_string(val)`     | ISO datetime (`YYYY-MM-DDTHH:mm`)                                                 | input value for `datetime-local`                          |
| `js_timestamp_to_iso_string(val)`    | Full ISO timestamp                                                                | machine-readable timestamps                               |
| `display_currency(val)`              | Currency string (default `€`, locale-aware)                                       | `{~ display_currency(record.price) }`                     |
| `display_percent(val)`               | Percent string, locale-aware                                                      | `{= display_percent(record.discount_rate) }`              |
| `yes_no(val, type?)`                 | Yes/No badge (HTML)                                                               | `{~ yes_no(record.is_active, "both") }`                   |
| `pill(text, class)`                  | Single-pill HTML span                                                             | `{~ pill(record.status, 'pill-info') }`                   |
| `tags(csv, class?)`                  | Renders a comma-separated string as pills                                         | `{~ tags(user.modules_tags) }`                            |
| `human_bytes(bytes)`                 | Human-readable byte count (`1.4 MB`)                                              | `{= human_bytes(file.size) }`                             |
| `urlencode(str)` / `urldecode(str)`  | URL component coding                                                              | `<a href="?q={= urlencode(query) }">`                     |

A few details worth knowing:

- **Date/time helpers default to `props.locale`**, which the render layer derives from the active language (`"en-US"` for English, `"sl-SI"` for Slovenian). Pass an explicit second argument to override: `js_date_to_locale_string(date, "fr-FR")`.
- **`display_currency` takes optional second through fourth arguments**: `display_currency(val, locale?, hide_zero?, symbol?)`. `hide_zero = true` returns the empty string for zero values; `symbol` swaps `€` for whatever currency you need.
- **`yes_no` returns HTML**, so use the raw output tag (`{~ }`) when you call it. The `"both"` type produces both a yes and a no badge; the default `"yes_only"` shows a green badge for truthy values and nothing for falsy.

<a name="custom-helpers"></a>

## Custom Helpers

The built-in helper set is wired in by `create_template_helpers()` in `lib/template_helpers.ts`. To add a project-wide helper, edit that file and add your function to the returned object:

```ts
// lib/template_helpers.ts
export function create_default_helpers(props: any = {}): TemplateHelpers {
	const locale = props.locale;
	return {
		// ...existing helpers...
		uppercase: (text: string) => text.toUpperCase(),
		badge_color: (role: string) => (role === "admin" ? "bg-red-500" : "bg-blue-500"),
	};
}
```

Reepolee's `server.ts` can also pass a second argument to `create_template_helpers()` — custom helper functions that are project-specific. On the reepolee.com site, for example, `src/lib/project_helpers.ts` registers helpers like `md()` (inline markdown rendering), `tw_merge()` (Tailwind class merging), `avif()`/`webp()`/`jpeg()` (responsive image URL helpers), and `is_current_route()` (URL matching).

Once registered, the helper is available in every template:

```html
{#each props.records as user }
<tr>
	<td>{= uppercase(user.name) }</td>
	<td><span class="{= badge_color(user.role) }">{= user.role }</span></td>
</tr>
{/each }
```

A few patterns that come up often:

- **Formatting** — single-purpose transforms (`price`, `phone`, `slug`).
- **Conditional display** — return one string in one case, another in another. Keeps the template free of nested `{#if}` blocks.
- **HTML generation** — return a small chunk of markup (a badge, a status pill). Always use `{~ }` to output the result and never inject raw user input.

<a name="helper-rules"></a>

### Helper Rules

- **Helpers are functions, always called with `()`**. `{= uppercase(name) }` works; `{= uppercase }` does not.
- **Helpers receive only their arguments**. They cannot read template variables they weren't passed.
- **Helpers run during rendering**. Return values are inserted into the template output.

The most common mistake is referencing a helper that isn't registered — the template throws "helper is not defined" at render time. The fix is adding the function to `create_default_helpers()` in `lib/template_helpers.ts`.

<a name="global-variables"></a>

## Global Variables

When you pass `ctx` to `render()`, the render layer injects a set of values into `props` automatically. You access them the same way as anything else — `props.user`, `props.lang`, and so on:

| Variable                 | Source                                    | Description                                  |
| ------------------------ | ----------------------------------------- | -------------------------------------------- |
| `props.user`             | Session cookie → session store            | The logged-in user object, or `null`         |
| `props.lang`             | `X-Lang` header → `lang` cookie → default | Active language code (`"en"`, `"sl"`)        |
| `props.locale`           | Derived from `props.lang`                 | Locale string (`"en-US"`, `"sl-SI"`)         |
| `props.request_url`      | `req.url` (pathname + search)             | Relative URL of the current request          |
| `props.toasts`           | Cookies with `toast-` prefix              | Pending toast notifications                  |
| `props.rendered_at`      | `new Date().toLocaleString()`             | Render timestamp                             |
| `props.active_languages` | `$config/supported_languages`             | List of available language codes             |
| `props.language_names`   | `$config/supported_languages`             | Map of code → display name                   |
| `props.prefix`           | URL prefix match                          | Active prefix (`"admin"`, `"api"`) or `null` |

In templates, these read just like any other field:

```html
{#if props.user }
<a href="/logout">Log out, {= props.user.display_name }</a>
{:else }
<a href="/login">Log in</a>
{/if }
```

<a name="base-data"></a>

### Base Data

In addition to the per-request globals above, `server.ts` defines `base_data` — a small set of values that are merged into every render _before_ your handler's data. The defaults include `site_name`, `year`, `is_dev`, and a `version` string:

```html
<footer>© {= props.year } {= props.site_name }</footer>

{#if props.is_dev }
<div class="dev-banner">Development mode</div>
{/if }
```

Anything you add to `base_data` in `server.ts` becomes available in every template — feature flags, build hashes, navigation entries, environment-specific configuration.

The merge order is:

1. `base_data` from `server.ts` (lowest precedence)
2. Per-request globals (`user`, `lang`, `locale`, etc.)
3. Your handler's data (highest precedence)

So if you pass `{ year: 2099 }` to `render()`, your template sees `2099`, not the real year. The `base_data` object in `server.ts` also includes `nav_groups` — the grouped navigation menu entries built from `nav_routes`.

<a name="development-only-globals"></a>

## Development-Only Globals

When the server runs with `--dev`, two extra values are injected into every render:

| Variable             | Description                                           |
| -------------------- | ----------------------------------------------------- |
| `props.toJSON`       | Compact JSON string of the full `props` object        |
| `props.toPrettyJSON` | Pretty-printed JSON string of the full `props` object |

Stick a `<pre>{~ props.toPrettyJSON }</pre>` somewhere in a template you're debugging and the entire render context is right there. In production these are not set, so the same line renders nothing — safe to leave in place if you want.

<a name="rendering-to-a-string"></a>

## Rendering to a String

`render()` returns a `Response`. For cases where you need the rendered HTML as a string instead — composing an email body, building a feed item, generating a static page — use `get_render()`:

```ts
import { get_render } from "$lib/render";

const render_template = get_render();
const html = await render_template("emails/welcome", { name: "Alice" });
```

`get_render()` returns the underlying render function with all the same globals and helpers wired up. The result is a plain string you can pass to `send_mail()` or any other consumer.
