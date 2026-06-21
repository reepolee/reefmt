---
title: "Conditionals"
---

# Conditionals

<a name="introduction"></a>

## Introduction

Conditional rendering in Ree uses `{#if}` blocks. The condition is plain JavaScript — any expression that evaluates to a truthy value enters the block, anything falsy skips it. The same rules JavaScript uses everywhere else apply here: `0`, `""`, `null`, `undefined`, `false`, and `NaN` are falsy; everything else is truthy.

<a name="basic-if-block"></a>

## The if Block

```html
{#if props.user }
<p>Welcome back, {= props.user.display_name }.</p>
{/if }
```

Every `{#if}` must be closed by a matching `{/if}`. If you forget, the engine throws a clear error at compile time pointing to the unclosed block.

Conditions are also resilient at render time: if the expression _throws_ — say you reach through a property of something that turned out to be `undefined` — the engine catches it and treats the condition as falsy rather than crashing the whole render. So `{#if props.user.profile.verified }` renders the `{:else}` branch (or nothing) when `props.user.profile` is missing, instead of erroring. Treat this as a safety net, not a license to skip optional chaining — `props.user?.profile?.verified` still states intent more clearly and is the recommended form.

<a name="if-else"></a>

## if / else

Add an `{:else}` branch for the falsy case:

```html
{#if props.user }
<p>Welcome back, {= props.user.display_name }.</p>
{:else }
<a href="/login">Log in</a>
{/if }
```

Only one `{:else}` is allowed per `{#if}`. There is no `{:elseif}` keyword — chain conditions with nested blocks or compute the value with `{{ ... }}` first.

<a name="chaining-conditions"></a>

## Multiple Conditions

For more than two branches, the cleanest approach is to compute a label or a class in a `{{ ... }}` block and output it:

```html
{{ const label = props.status === "draft" ? "Draft" : props.status === "review" ? "In review" : props.status === "live"
? "Live" : "Unknown"; }} <span class="badge">{= label }</span>
```

If you need different _markup_ per branch rather than different text, nest the blocks:

```html
{#if props.record.status === "draft" }
<button class="primary">Submit for review</button>
{:else } {#if props.record.status === "review" }
<button class="secondary">Approve</button>
{:else }
<span class="text-muted">Already live</span>
{/if } {/if }
```

This reads awkwardly past two levels — that's usually a sign that the branching belongs in the handler rather than the template. Compute the right state on the server, pass a single field through `props`, and let the template render one shape.

<a name="conditions-in-attributes"></a>

## Conditions Inside Attributes

`{#if}` works inline as well as in element content. A common case is conditionally adding an attribute:

```html
<option value="{= option.value }" {#if option.value === props.selected }selected{/if }>
    {= option.label }
</option>
```

```html
<button class="primary" {#if props.submitting }disabled{/if }>
    Save
</button>
```

For conditional class names specifically, building the string with a `{{ ... }}` block is often more readable than scattering `{#if}` tags through a `class="..."` attribute:

```html
{{ const row_class = props.record.is_archived ? "row archived" : "row" }}
<tr class="{= row_class }">
	...
</tr>
```

<a name="truthiness-pitfalls"></a>

## Truthiness Pitfalls

The condition uses regular JavaScript truthiness, which catches a few people:

- An **empty array** (`[]`) is **truthy**. Use `props.records.length > 0` if you only want the block when there are items.
- An **empty string** (`""`) is **falsy**. If a database column might return `""` for a missing value, `{#if props.field }` will treat that as missing — which is usually what you want, but worth being aware of.
- **The number `0`** is **falsy**. For numeric IDs starting from `0`, check `props.id != null` rather than `props.id`.

The `{#each}` block has its own `{:else}` branch for the "list is empty" case, so you rarely need `{#if props.records.length > 0 }` around an each block — write the `{:else}` inline instead. See [Loops](/ree-templates/loops).

<a name="negation-and-coalescing"></a>

## Negation and Nullish Coalescing

Plain JS operators are the right tool — there is no special syntax for "if not":

```html
{#if !props.user }
<a href="/login">Log in</a>
{/if } {#if props.user && props.user.modules_tags?.split(",").includes("admin") }
<a href="/admin">Admin panel</a>
{/if }
```

Optional chaining (`?.`) and the nullish coalescing operator (`??`) work everywhere expressions are accepted — inside conditions, inside `{= }`, and inside `{{ ... }}`.
