---
title: "SPA Navigation"
---

# SPA Navigation

<a name="introduction"></a>

## Introduction

`spa-loader.js` intercepts internal link clicks, fetches the destination as HTML, swaps the `<body>`, and pushes a new history entry — giving navigation the feel of a single-page app while keeping the server doing the rendering. There's no client-side router, no virtual DOM, no manifest of routes for the loader to know about. The browser still asks the server for fully-rendered HTML; the loader just avoids the full-page reload between requests.

It's around 60 lines of code in `static/spa-loader.js` and entirely opt-in. Include the script in your layout to enable it; remove the script and the application reverts to standard full-page navigation.

<a name="enabling-it"></a>

## Enabling It

Load the script once in your layout's `<head>`:

```html
<script src="/spa-loader.js" defer></script>
```

That's the entire setup. The loader registers a single delegated `click` handler on `document` and a `popstate` handler on `window`. There's no API to call, no per-link opt-in, and no configuration.

<a name="what-it-intercepts"></a>

## What It Intercepts

The loader handles any click on an `<a>` element whose `href` starts with `/`. It deliberately opts out of these cases:

- **Modifier keys**: holding `cmd`, `ctrl`, `shift`, or `alt` (the user wants to open in a new tab/window).
- **Middle-click**: same reason.
- **External links**: anything not starting with `/` falls through to the browser's default behaviour.
- **Already-handled clicks**: if `event.defaultPrevented` is true (some other handler called `preventDefault`), the loader doesn't override.

For the cases it does intercept, the flow is:

1. Add a `loading` class to `<html>` so you can show a progress indicator if you want one.
2. Fetch the URL with `credentials: "same-origin"` — cookies (sessions, language, toasts) flow through.
3. Parse the response with `DOMParser`, pull out the new `<title>`, and replace `document.body` with the fetched body.
4. Re-execute every `<script>` in the new body — both external and inline. (Browsers don't re-run scripts inserted via `replaceWith`, so the loader does it manually.)
5. Push a new entry into the browser history with `history.pushState()`.
6. Remove the `loading` class.

If anything fails — network error, 5xx response, parse error — the loader falls back to a plain `window.location.href = url` reload so the user always reaches the destination.

<a name="back-and-forward"></a>

## Back and Forward Navigation

The loader handles the back/forward buttons by registering a `popstate` listener that triggers a full page reload. This is intentionally simple — caching arbitrary previous DOM states is what tripped up most early SPA implementations, and a full reload from the server keeps the page fresh and correct. The trade-off is that back navigation isn't quite as fast as forward navigation; the trade-back is that you don't have to think about cache invalidation.

If you're navigating between heavy pages where the back-button reload is slow, consider scoping the SPA loader to specific sections of your app (load the script only on those pages) rather than fighting the model.

<a name="loading-indicator"></a>

## A Loading Indicator

<media-frame label="Animation — SPA navigation with top loading indicator" ratio="16/9"></media-frame>

Because the loader adds `class="loading"` to `<html>` during navigation, a single CSS rule gives you a basic indicator. The simplest version is a thin progress bar that grows across the top:

```css
html.loading::before {
	content: "";
	position: fixed;
	top: 0;
	left: 0;
	height: 2px;
	background: var(--color-accent);
	width: 0;
	animation: progress 1.2s ease-out forwards;
	z-index: 9999;
}

@keyframes progress {
	0% {
		width: 0;
	}
	50% {
		width: 60%;
	}
	100% {
		width: 90%;
	}
}
```

The animation maxes out at 90% so it doesn't look "complete" while the request is still in flight. When the new page loads and the `loading` class is removed, the bar disappears.

<a name="re-running-scripts"></a>

## Re-running Scripts

Every page that the loader fetches has its scripts re-executed in order:

- **External scripts** (`<script src="...">`) are re-inserted by creating a fresh `<script>` element with the same `src`. The browser fetches and runs them.
- **Inline scripts** (`<script>...</script>`) are run via `eval()` of their text content.

This means initialisation code in your page templates (like `new FormController({ ... })`) runs every time the page loads, including after SPA navigation. The pattern is:

```html
<script>
	document.addEventListener("DOMContentLoaded", () => {
		new FormController({ form: "#edit-form", validate_url: "/users/validate" });
	});
</script>
```

`DOMContentLoaded` won't fire on SPA navigations (the document doesn't reload), but the inline script's eval runs after the body swap, so the `new FormController(...)` line executes anyway. If you want code to run _only on first page load and not on SPA navigations_, register your code on `DOMContentLoaded` directly without the `addEventListener` wrapper.

For a script that wires up signal-driven UI, the same rule applies — register your bindings inline at the bottom of the page template. The SPA loader evaluates inline `<script>` blocks after every body swap, so `effect()` and the `bind_*` helpers from `signals-ui.js` are re-attached to the fresh DOM on every navigation. Anything with one-time initialisation (a chart library, a third-party widget) should be re-initialised in the inline script of the page that needs it.

<a name="view-transitions"></a>

## Pairing With View Transitions

The CSS [View Transitions API](https://developer.mozilla.org/en-US/docs/Web/API/View_Transitions_API) lets the browser animate between DOM states automatically. Combined with SPA navigation, you can animate the title between pages, slide content in from the side, fade entire sections — all from CSS, no JavaScript per-transition.

The shipped `css/transitions.css` (imported by `css/app.css`) enables it on the `h1` element:

```css
@view-transition {
	navigation: auto;
}

h1 {
	view-transition-name: page-title;
}
```

Because every page has exactly one `<h1>` with the same `view-transition-name`, the browser interpolates between them — the title slides into place as the rest of the page swaps.

For the SPA loader to trigger view transitions, the body swap needs to happen inside `document.startViewTransition()`. The current `spa-loader.js` doesn't do this automatically; if you want it, modify the loader's body-swap section:

```js
if (document.startViewTransition) {
	document.startViewTransition(() => document.body.replaceWith(doc.body));
} else {
	document.body.replaceWith(doc.body);
}
```

<a name="when-to-use-it"></a>

## When to Use It

SPA navigation is a UX upgrade — pages don't flash white between loads, scroll position stays where you left it (within a page), and the perceived speed is noticeably better. The cost is one small script in your layout and the discipline of remembering that scripts re-execute on every navigation.

The trade-offs you might want to think about:

- **Analytics**: page-view tracking that hooks into `DOMContentLoaded` won't fire on SPA navigations. Hook into `popstate` and the loader's `pushState` calls instead, or remove the loader on pages where you need traditional analytics behaviour.
- **Memory**: the loader doesn't dispose of objects when it swaps the body. Long-lived event listeners, intervals, or WebSocket connections in inline scripts will accumulate across navigations. Clean them up explicitly if they're not garbage-collected with the body.
- **Print styles, screen readers, etc.**: most assistive technology and browser features handle DOM swaps fine, but it's worth testing the navigation flow with a screen reader on at least once.

For most server-rendered apps, the upgrade is worth it. For pages where you absolutely need a clean slate per request — long-running JavaScript that's hard to clean up, embedded third-party content with its own lifecycle — the loader is opt-out by simply not loading the script in the layout for those pages.
