---
title: "Displaying Data"
layout: "reeweb/docs/docs.layout"
---

# Displaying Data

<a name="introduction"></a>

## Introduction

Ree has three output tags. Two of them print values into the rendered HTML — one escapes its input, the other doesn't. The third lets you run plain JavaScript inline. Most templates use the escaped form for everything and reach for the others only when needed.

<a name="escaped-output"></a>

## Escaped Output

`{= expr }` outputs an expression with HTML escaping. The characters `&`, `<`, `>`, `"`, and `'` are converted to their entity equivalents before being inserted into the page, so user-supplied content cannot inject markup or break out of an attribute:

```html
<h1>{= props.title }</h1>
<p>Welcome back, {= props.user.name }.</p>
```

This is the safe default. If you're not sure which output tag to use, use this one. Even values that you know come from your own database are best escaped — a column might one day hold something a user typed in.

The expression inside the tag is plain JavaScript. You can call methods, access nested properties, and use ternaries:

```html
<p>{= props.user.name?.toUpperCase() ?? "Anonymous" }</p>
<span class="badge {= props.is_admin ? 'badge-admin' : 'badge-user' }"> {= props.tags.join(", ") } </span>
```

<a name="raw-html-output"></a>

## Raw HTML Output

`{~ expr }` outputs an expression _without_ escaping. Use it when the value already contains trusted HTML — a markdown-rendered string, a sanitised rich-text field, a snippet you composed in your handler:

```html
{~ props.rendered_markdown }
```

Never use `{~ }` with values that came from user input. The whole point of the escaped tag is to defend against injection; the raw tag turns that defence off. If your handler computed an HTML string from a value the user submitted, escape it before passing it in (or pre-escape parts of it and concatenate trusted markup around them).

<a name="inline-javascript"></a>

## Inline JavaScript

`{{ ... }}` runs arbitrary JavaScript at render time. The block produces no output of its own — its job is to compute values that you reference later in the template:

```html
{{ const label = props.record.id ? props.labels.edit : props.labels.new; const class_map = { red: "bg-red-600
text-white", green: "bg-green-600 text-white" }; const banner_class = class_map[props.type] ?? "border
border-neutral-300"; }}

<h1>{= label }</h1>
<div class="{~ banner_class }">{~ props.text }</div>
```

Variables declared inside `{{ ... }}` are available for the rest of the template. They are scoped to the same lexical block the rest of your template runs in, so you can use `const` and `let` freely.

The most common patterns:

- **Pre-sorting or filtering a list before iterating:**

    ```html
    {{ const recent = props.posts.slice(0, 5) }} {#each recent as post }
    <article>{= post.title }</article>
    {/each}
    ```

- **Building a class string from a small lookup table:**

    ```html
    {{ const color = { admin: "red", staff: "blue", user: "gray" }[props.role] }}
    <span class="badge badge-{= color }">{= props.role }</span>
    ```

- **Pulling a frequently-used value out into a short alias:**
    ```html
    {{ const u = props.user }}
    <p>{= u.first_name } {= u.last_name } ({= u.email })</p>
    ```

Keep these blocks small. If a template starts to fill up with `{{ ... }}` blocks, the work usually belongs in the handler instead — pass the precomputed value into `props` and reference it directly.

<a name="comments"></a>

## Comments

Standard HTML comments work as you'd expect — they pass through to the rendered output:

```html
<!-- This is visible in the page source -->
```

For comments you want stripped before output, wrap them in a JavaScript block:

```html
{{ /* This is invisible to the browser */ }}
```

The block runs (it's just a JS comment, so it does nothing) and produces no output.

<a name="escaping-rules"></a>

## Escaping Rules in Detail

The escape function is single-pass and converts exactly five characters:

| Character | Replacement |
| --------- | ----------- |
| `&`       | `&amp;`     |
| `<`       | `&lt;`      |
| `>`       | `&gt;`      |
| `"`       | `&quot;`    |
| `'`       | `&#39;`     |

`null` and `undefined` are converted to the empty string. Numbers, booleans, and dates are coerced with `String(value)` first, then escaped.

The same rules apply whether the value lives in element content (`<p>{= x }</p>`), in an attribute (`<input value="{= x }">`), or in a URL (`<a href="{= x }">`). For attributes specifically, always wrap the value in double quotes — that combined with `"` being escaped means the value cannot break out of the attribute.
