---
title: "Page Transitions"
layout: "reeweb/docs/docs.layout"
---

# Page Transitions

<a name="introduction"></a>

## Introduction

Reeweb supports the View Transitions API — a browser-native mechanism for animating between pages. When a user navigates from one page to another, the browser captures a snapshot of the old page and smoothly morphs it into the new page. There are no JavaScript libraries, no animation frameworks, and no client-side routing to configure.

View Transitions work with regular browser navigation — every link is a full page load, and the browser handles the crossfade automatically.

<a name="how-it-works"></a>

## How It Works

The View Transitions API is available in Chromium-based browsers. When a same-origin navigation occurs, the browser:

1. Takes a screenshot of the current page state
2. Renders the new page
3. Crossfades from the old screenshot to the new page

You can customise which parts of the page animate independently by assigning `view-transition-name` CSS properties to elements that should morph rather than fade.

<a name="basic-setup"></a>

## Basic Setup

No configuration is needed. Reeweb's layout includes a `<meta name="view-transition">` tag or the `@view-transition` CSS at-rule depending on the browser. For the default crossfade, view transitions work automatically.

To opt into view transitions for cross-document (SPA-style) navigations, ensure your page includes:

```html
<meta name="view-transition" content="same-origin" />
```

Or declare it in CSS:

```css
@view-transition {
	navigation: auto;
}
```

<a name="customising-animations"></a>

## Customising Animations

For more control, assign `view-transition-name` to elements that should animate independently. A common pattern is giving the main content area its own transition name so the header and footer stay static while the content crossfades:

```css
/* In your src/css/style.css */
main {
	view-transition-name: main-content;
}

::view-transition-old(main-content) {
	animation: fade-out 0.3s ease-out;
}

::view-transition-new(main-content) {
	animation: fade-in 0.3s ease-in;
}

@keyframes fade-out {
	from {
		opacity: 1;
	}
	to {
		opacity: 0;
	}
}

@keyframes fade-in {
	from {
		opacity: 0;
	}
	to {
		opacity: 1;
	}
}
```

<a name="page-transition-names"></a>

## Assigning Transition Names in Templates

You can assign `view-transition-name` directly in your `.ree` templates using the `style` or `class` attribute:

```html
<main class="content" style="view-transition-name: main-content">{~ props.body }</main>
```

Reeweb's markdown post-processor also adds `scroll-mt-30` to headings for accessibility — when navigating to a page with an anchor, the heading has enough scroll margin to remain visible below the fixed navigation, even during the transition.

<a name="browser-compatibility"></a>

## Browser Compatibility

- **Chrome/Edge 111+** — full support for cross-document (SPA) transitions
- **Firefox** — in development (flag behind `layout.css.view-transitions.enabled`)
- **Safari** — in development

For browsers that don't support the API, navigation works normally without transitions — no polyfill needed.

<a name="related"></a>

## Related

- View Transitions API on MDN: [developer.mozilla.org/docs/Web/API/View_Transitions_API](https://developer.mozilla.org/docs/Web/API/View_Transitions_API)
