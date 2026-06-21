---
title: "Custom Components"
layout: "reeweb/docs/docs.layout"
---

# Custom Components

<a name="introduction"></a>

## Introduction

Components in Reeweb are just `.ree` files in `src/components/`. There is no registration step, no lifecycle, and no framework to learn. This recipe walks through creating, using, and composing custom components.

<a name="the-simplest-component"></a>

## The Simplest Component

Create `src/components/greeting.ree`:

```ree
<div class="greeting">
    <h2>Hello, {= props.attributes.name }!</h2>
</div>
```

Use it in any template by writing a custom HTML element whose tag matches the file name:

```html
<greeting name="World"></greeting>
<!-- Renders: <div class="greeting"><h2>Hello, World!</h2></div> -->
```

The element passes HTML attributes as `props.attributes` and slot content (the markup between the tags) as `props.children`. To pass a dynamic value rather than a literal string, interpolate the attribute: `<greeting name="{= props.user.first_name }"></greeting>`.

<a name="component-props-pattern"></a>

## Component Props Pattern

A well-designed component destructures its expected props explicitly, then spreads the rest:

```ree
{{ const { children } = props;
   const { class: _class, href, ...rest } = props.attributes || {}; }}

<a href="{= href }" class="btn {= _class || '' }" ...rest>
    {~ children }
</a>
```

This pattern:

- Destructures known attributes (`class`, `href`) for special handling
- Spreads any additional attributes via `...rest` (converts to `key_values()`)
- Renders slot content via `{~ children }` (raw output — it's trusted template content)

Usage:

```html
<link-button href="/about" class="large" data-track="nav-click"> About us </link-button>
```

<a name="using-project-helpers-in-components"></a>

## Using Project Helpers in Components

Project helpers from `src/lib/project_helpers.ts` are available in components just like in templates:

```ree
{{ const { text, type } = props.attributes; }}

<div class="badge badge-{= type }">
    {~ md(text) }
</div>
```

This renders inline markdown inside the component:

```html
<badge type="info" text="**New** feature available!"></badge>
```

<a name="composing-components"></a>

## Composing Components

Components can include other components. A card component that uses the badge component:

```ree
{{ const { children } = props;
   const { title, badge_text, badge_type, ...rest } = props.attributes || {}; }}

<div class="card" ...rest>
    {#if title }
        <h3>{= title }</h3>
    {/if }
    {#if badge_text }
        <badge type="{= badge_type || 'info' }" text="{= badge_text }"></badge>
    {/if }
    <div class="card-body">
        {~ children }
    </div>
</div>
```

Usage:

```html
<card title="Getting Started" badge_text="New" badge_type="green">
	<p>Here's how to get started with Reeweb.</p>
</card>
```

<a name="accessing-translations-in-components"></a>

## Accessing Translations in Components

Components have access to the full render context — including translations — through `props`:

```ree
<footer class="site-footer">
    <p>{= props.ui.site_name } — {= props.nav.home }</p>
</footer>
```

There's no need to forward translation strings explicitly. The component inherits the parent's data automatically.

<a name="testing-components-locally"></a>

## Testing Components Locally

To test a component during development:

1. Create a test page in `src/public/test/` that uses the component
2. Run `bun dev`
3. Open `http://localhost:3000/test/`
4. Edit the component and see changes immediately (dev server reloads on file changes)

Once the component works as expected, use it from your real pages and remove the test page.
