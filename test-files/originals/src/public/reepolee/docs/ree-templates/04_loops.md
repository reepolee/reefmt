---
title: "Loops"
---

# Loops

<a name="introduction"></a>

## Introduction

Ree iterates with `{#each}`. The block works on both arrays and objects, exposes the current index and key as optional second and third variables, and supports an `{:else}` branch for the empty case.

```html
{#each props.records as record }
<a href="/users/{= record.id }/edit">{= record.name }</a>
{:else }
<p>No records found.</p>
{/each }
```

Like `{#if}`, every `{#each}` must be closed with `{/each}` — unclosed blocks raise a compile-time error.

<a name="basic-iteration"></a>

## Basic Iteration

The simplest form gives you each item under a name of your choosing. The name is scoped to the block, so it doesn't collide with anything outside:

```html
<ul>
	{#each props.posts as post }
	<li><a href="/posts/{= post.slug }">{= post.title }</a></li>
	{/each }
</ul>
```

If `props.posts` is `null`, `undefined`, or anything that isn't iterable, the block produces no output (and falls through to `{:else}` if you wrote one).

<a name="with-index"></a>

## With an Index

Add a second variable after a comma to capture the current zero-based index:

```html
{#each props.options as option, i }
<option value="{= option.value }">{= i + 1 }. {= option.label }</option>
{/each }
```

The index is always numeric. Use it for display ("Step 1 of 5"), for adding CSS classes to alternate rows, or for skipping the first item:

```html
{#each props.items as item, i } {#if i === 0 }
<li class="first">{= item.name }</li>
{:else }
<li>{= item.name }</li>
{/if } {/each }
```

<a name="iterating-objects"></a>

## Iterating Over Objects

`{#each}` also iterates over plain objects. The item is the value, and a third variable gives you the key:

```html
{#each props.settings as value, i, key }
<div>{= key }: {= value }</div>
{/each }
```

For arrays, the index is the same as the key, so the third variable is redundant. For objects, the third variable is where the property name shows up.

You can iterate object entries either by destructuring or by using the index-and-key form directly:

```html
{#each props.scores_by_user as score, i, user_id }
<tr>
	<td>{= user_id }</td>
	<td>{= score }</td>
</tr>
{/each }
```

<a name="empty-state"></a>

## Empty State

`{:else}` renders when the list has no items — same syntax as `{#if}/{:else}`:

```html
{#each props.records as record }
<article>{= record.title }</article>
{:else }
<p class="text-muted">No records to display.</p>
{/each }
```

This means you almost never need to wrap an each block with `{#if props.records.length > 0 }`. Write the empty case inline as `{:else}` instead — it's clearer and one less indentation level.

<a name="nested-loops"></a>

## Nested Loops

Each blocks nest cleanly. The variables introduced by each `{#each}` are scoped to that block, so inner loops can reuse outer variable names without conflict — though for readability you'll usually pick distinct names:

```html
{#each props.categories as category }
<section>
	<h2>{= category.name }</h2>
	<ul>
		{#each category.items as item }
		<li>{= item.title }</li>
		{/each }
	</ul>
</section>
{:else }
<p>No categories yet.</p>
{/each }
```

There is no limit on nesting depth, but if you find yourself three or four loops deep, consider whether the inner shape would be better computed on the server and passed in as a flatter structure.

<a name="scope-blocks"></a>

## Scope Blocks With `{#with}`

`{#with expr } ... {/with }` opens a JavaScript `with` scope so you can reference the members of an object directly, without repeating the path. It's handy when a chunk of markup reads several fields off the same nested object:

```html
{#with props.author }
<figure class="byline">
	<img src="{= avatar_url }" alt="{= name }" />
	<figcaption>{= name } — {= role }</figcaption>
</figure>
{/with }
```

Inside the block, `name`, `role`, and `avatar_url` resolve against `props.author`. Every `{#with}` must be closed with `{/with}`, and unlike `{#if}` and `{#each}` it has no `{:else}` branch — if the object might be missing, guard it with an `{#if}` first. Use it sparingly: a `with` scope can make it ambiguous which variable a bare name refers to, so reserve it for short, obviously-scoped blocks.

<a name="loops-and-includes"></a>

## Loops Around Includes and Components

Each blocks compose naturally with `{#include}` and with component custom elements. Because the component tag is expanded in place — right inside the loop body — its interpolated attributes can read the loop variable. Pass per-item data through `attr="{= expr }"` attributes, which the component reads from `props.attributes`:

```html
{#each props.products as product }
<product-card name="{= product.name }" badge="{= product.is_new ? 'NEW' : null }"></product-card>
{/each }
```

Inside `components/product-card.ree`, `props.attributes.name` and `props.attributes.badge` are available alongside everything the parent had. (Slot content — the markup _between_ the tags — is compiled in its own scope and does **not** see the loop variable, so reach for interpolated attributes when you need per-iteration values.) For a target outside `components/`, use `{#include('path', { product })}` instead, which still merges its second argument onto the parent props.

<a name="preprocessing-with-inline-js"></a>

## Preprocessing With Inline JavaScript

For sorting, filtering, or grouping before iterating, compute the working list in a `{{ ... }}` block first:

```html
{{ const sorted = props.posts.sort((a, b) => b.date - a.date) }} {{ const recent = sorted.slice(0, 5) }} {#each recent
as post }
<article>{= post.title }</article>
{/each }
```

This keeps the loop itself focused on rendering. For anything heavier — joining across multiple lists, deduplicating, paginating — do the work in the route handler and pass the final shape into `props`.
