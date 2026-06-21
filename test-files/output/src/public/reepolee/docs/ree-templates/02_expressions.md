---
title: "Expressions"
---

# Expressions

<a name="introduction"></a>

## Introduction

Ree templates render dynamic content through expressions — small pieces of JavaScript embedded in your markup that output values, run logic, or assign temporary variables. There are three expression types, each with a different escaping behaviour and use case.

| Tag          | Escaped?           | Use case                                                                                        |
| ------------ | ------------------ | ----------------------------------------------------------------------------------------------- |
| `{= expr }`  | Yes (HTML-escaped) | Outputting user-controlled text — names, descriptions, any string that might contain `<` or `&` |
| `{~ expr }`  | No (raw)           | Outputting trusted HTML — rendered markdown, component output, template bodies                  |
| `{{ code }}` | No output          | Running arbitrary JavaScript — variable assignments, computation, preprocessing                 |

<a name="escaped-output"></a>

## Escaped Output — `{= }`

The most common expression. The value is converted to a string and HTML-escaped — `&` becomes `&amp;`, `<` becomes `&lt;`, and so on. Use it for everything that could originate from user input, API responses, database content, or any source you don't control:

```html
<h1>{= props.page.title }</h1>
<p>{= props.description }</p>
<span>{= record.status }</span>
```

The escaping happens at render time, so the value in your props is the original unescaped string — safe to use in multiple contexts.

Any valid JavaScript expression works inside the braces:

```html
<p>{= props.user.display_name ?? "Guest" }</p>
<p>{= Math.round(props.progress * 100) + "%" }</p>
<p>{= props.items.length > 0 ? `${props.items.length} items` : "Empty" }</p>
```

<a name="raw-output"></a>

## Raw Output — `{~ }`

Same as `{= }` but without HTML escaping. Use it only when you control the content and know it's safe — rendered markdown, layout body content, helper functions that return HTML:

```html
<main>{~ props.body }</main>

<div>{~ markdown_to_html(props.content) }</div>

<span>{~ pill(record.status, "pill-info") }</span>
```

The `{~ }` tag is also required for helpers that return HTML fragments — `yes_no()`, `pill()`, `tags()` — because escaping would turn the `<span>` elements into visible text:

```html
<!-- Right -->
<td>{~ yes_no(record.is_active, "both") }</td>

<!-- Wrong — the HTML is escaped and shows as text -->
<td>{= yes_no(record.is_active, "both") }</td>
```

A good rule: use `{= }` by default and switch to `{~ }` only when you explicitly want to inject markup. If you're ever unsure, `{= }` is safer — escaped HTML is ugly but harmless; unescaped user input is a cross-site scripting vulnerability.

<a name="inline-javascript"></a>

## Inline JavaScript — `{{ }}`

`{{ }}` runs arbitrary JavaScript without outputting anything. Use it for computations, variable assignments, and data transformations before the template's output phase:

```html
{{ const total = props.items.reduce((sum, item) => sum + item.price, 0); const formatted = display_currency(total);
const show_discount = total > 1000; }}

<div class="total">
	<span>Total: {~ formatted }</span>
	{#if show_discount }
	<span class="discount">10% bulk discount available</span>
	{/if }
</div>
```

Variables declared in `{{ }}` are scoped to the nearest template or block — they're available to everything after the block but don't leak to included templates or components.

Multiple `{{ }}` blocks can appear in the same template, and they execute in order:

```html
{{ const base_url = "/uploads/" }} {{ const image_url = base_url + props.record.image_filename }}

<img src="{= image_url }" alt="{= props.record.image_alt }" />
```

<a name="the-rest-shorthand"></a>

## The `...rest` Shorthand

Inside a component or include, you often want to destructure known props from `props.attributes` and spread the rest onto an HTML element. The `...identifier` shorthand (without `{ }`) is sugar for `{~ key_values(identifier) }`:

```html
{{ const { children } = props; const { class: _class, type, text, ...rest } = props.attributes; }}
<h1 class="{= _class }" ...rest>{~ text }</h1>
```

The `...rest` outputs each remaining attribute as `key="value"` pairs, properly escaped. It's the template equivalent of JavaScript's spread operator for HTML attributes.

The `key_values()` helper can also be called explicitly if you need to combine attribute spreads conditionally:

```html
<div {~ key_values(rest) } {~ key_values(extra_attrs) }></div>
```

<a name="expressions-in-attributes"></a>

## Expressions in Attributes

Both `{= }` and `{~ }` work inside HTML attribute values:

```html
<a href="{= localized_path('/profile') }">Profile</a>
<input type="text" name="username" value="{= props.record.username }" />
<img src="{~ props.image_url }" alt="{= props.image_alt }" />
```

For raw output in attributes (`{~ }`), use the same caution as in content — only use it when you trust the value. The attribute value is still quoted by the HTML, but `{~ }` doesn't escape characters within it.

<a name="data-xxx-prefix"></a>

## The `props.` Prefix

All template variables accessed inside expressions use the `props.` prefix:

```html
<h1>{= props.title }</h1>
<p>{= props.description }</p>
{#if props.user }
<span>{= props.user.display_name }</span>
{/if }
```

The `props` object contains everything passed to the template: your handler's data, global variables like `props.lang` and `props.user`, translation strings, and any built-in helpers (which are accessed by calling them as functions, not via `props.`). This is covered in detail in [Helpers & Globals](/ree-templates/helpers-and-globals).

<a name="expression-limitations"></a>

## Limitations

A few things expressions cannot do:

- **No statements** — `if`, `for`, `while` cannot appear inside `{= }` or `{~ }`. Use `{{ }}` for computation and `{#if}` / `{#each}` for control flow.
- **No semicolons needed** — inside `{= }` and `{~ }`, a semicolon at the end is valid but does nothing. Inside `{{ }}`, semicolons work as usual.
- **No async/await** — expressions are synchronous during rendering. For async data fetching, load the data in your route handler before calling `render()`.
