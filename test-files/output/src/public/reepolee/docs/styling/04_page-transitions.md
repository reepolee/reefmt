---
title: "Page Transitions"
---

# Page Transitions

<a name="introduction"></a>

## Introduction

The CSS [View Transitions API](https://developer.mozilla.org/en-US/docs/Web/API/View_Transitions_API) lets the browser animate between two DOM states with a few lines of CSS. Reepolee's optional `css/transitions.css` enables it for navigation, so the page title slides between pages instead of snapping. The transition uses no JavaScript, no animation framework, and no per-route configuration.

The file ships with the project but is commented out in `app.css` by default. Uncomment one line to turn it on; comment it back to turn it off. Pages render the same way either way.

<a name="enabling-page-transitions"></a>

## Enabling Page Transitions

`css/app.css` imports `transitions.css` from a commented line. Enable transitions by removing the `/* */` around the import:

```css
@import "./forms.css";
@import "./toasts.css";
@import "./transitions.css"; /* uncomment this */
```

Rebuild the CSS (`bun run css:build` for production, or let the watcher pick it up in dev) and the next navigation animates. Disable transitions by re-commenting the import — no other changes needed.

<a name="how-it-works"></a>

## How It Works

<media-frame label="Animation — page title slide transition (forward navigation)" ratio="16/9"></media-frame>

The view-transitions API works by capturing the page before navigation, capturing the page after, and animating between the two snapshots. Elements that share the same `view-transition-name` across pages are interpolated as a single moving element — they slide, fade, or scale from their old position to their new one rather than being torn down and rebuilt.

The default `transitions.css` declares `view-transition-name: page-title` on `h1`:

```css
@view-transition {
	navigation: auto;
}

h1 {
	view-transition-name: page-title;
}
```

`@view-transition { navigation: auto }` opts the page into automatic transitions on same-origin navigation. The browser captures the outgoing page, executes the navigation, captures the incoming page, and animates between the captures.

Because every page has exactly one `<h1>` with the same `view-transition-name`, the browser knows to animate that element specifically — the title slides into its new position while the rest of the body fades.

<a name="direction-aware-animations"></a>

## Forward vs Backward Navigation

<media-frame label="Animation — reverse slide on back-button navigation" ratio="16/9"></media-frame>

The browser also exposes the direction of navigation, so you can animate a forward navigation differently from a back-button navigation:

```css
:root::view-transition-old(page-title) {
	animation: slide-out-left 0.3s ease forwards;
}

:root::view-transition-new(page-title) {
	animation: slide-in-right 0.3s ease forwards;
}

:root:active-view-transition-type(backward)::view-transition-old(page-title) {
	animation: slide-out-right 0.3s ease forwards;
}

:root:active-view-transition-type(backward)::view-transition-new(page-title) {
	animation: slide-in-left 0.3s ease forwards;
}

@keyframes slide-out-left {
	to {
		transform: translateX(-100%);
		opacity: 0;
	}
}
@keyframes slide-in-right {
	from {
		transform: translateX(100%);
		opacity: 0;
	}
	to {
		transform: translateX(0);
		opacity: 1;
	}
}
@keyframes slide-out-right {
	to {
		transform: translateX(100%);
		opacity: 0;
	}
}
@keyframes slide-in-left {
	from {
		transform: translateX(-100%);
		opacity: 0;
	}
	to {
		transform: translateX(0);
		opacity: 1;
	}
}
```

The `:active-view-transition-type(backward)` pseudo-class targets back-button navigation specifically. Without it, every transition animates the same way regardless of direction — usually feels wrong because users expect "going back" to slide content from the opposite side.

<a name="naming-more-elements"></a>

## Naming More Elements

Any element with a unique `view-transition-name` animates separately. Useful for hero images that move between pages, navigation indicators that slide between items, or sidebar sections that should stay in place during the transition:

```css
.nav-active-indicator {
	view-transition-name: nav-indicator;
}

header > img.logo {
	view-transition-name: logo;
}

aside.sidebar {
	view-transition-name: sidebar;
}
```

The browser captures each named element independently and interpolates between its before/after positions. If an element exists on both pages, it animates; if it only exists on one, it fades in or out.

The name must be unique on a given page. Two elements with `view-transition-name: page-title` on the same page will fall back to no transition for both. Use the `:nth-child()` selector or the data-attribute pattern (`view-transition-name: var(--vt-name)`) for cases where you have a list of items that should all animate.

<a name="opting-out"></a>

## Opting Out for Specific Pages

Some pages don't make sense to animate — a long-running report, a print view, anywhere the user needs the content to appear immediately. Disable transitions per-page with a single rule:

```css
.no-transition {
	view-transition-name: none;
}
```

Then apply the class to the page's root element:

```html
<main class="no-transition">
	<!-- This page won't pick up the title slide -->
</main>
```

Or opt out entirely for the user — respecting the `prefers-reduced-motion` media query is the right default for accessibility:

```css
@media (prefers-reduced-motion: reduce) {
	:root::view-transition-old(*),
	:root::view-transition-new(*) {
		animation-duration: 0.01s;
	}
}
```

This reduces every transition to effectively instant for users who've asked for reduced motion. The animations still happen — needed for the `view-transition-name` system to work — but they're imperceptible.

<a name="with-spa-navigation"></a>

## With SPA Navigation

The View Transitions API integrates with `spa-loader.js` (see [SPA Navigation](/client-side/spa-navigation)) but the loader needs a small change to take advantage of it. The default loader swaps the body directly:

```js
document.body.replaceWith(doc.body);
```

To trigger a view transition during the swap, wrap it in `document.startViewTransition()`:

```js
if (document.startViewTransition) {
	document.startViewTransition(() => document.body.replaceWith(doc.body));
} else {
	document.body.replaceWith(doc.body);
}
```

The browser captures the page before the callback, runs the callback (which performs the DOM swap), captures the page after, and animates between. The fallback for browsers without `startViewTransition` is to do the swap directly — no animation, no broken behaviour.

This is the recommended way to combine SPA-style navigation with view transitions. Full-page reloads handle the transition automatically via `@view-transition { navigation: auto }`; SPA swaps need the explicit wrapper because the browser doesn't see a navigation event.

<a name="browser-support"></a>

## Browser Support

The View Transitions API is supported in Chrome, Edge, and recent Safari. Firefox is rolling it out gradually. Browsers without support simply ignore the rules — the pages still navigate, just without animation.

There is no polyfill worth recommending. The fallback (instant navigation) is the same experience users had before view transitions existed; no animation is a perfectly acceptable degradation for browsers that don't support the API.
