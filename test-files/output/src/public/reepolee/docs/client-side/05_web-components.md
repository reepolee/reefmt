---
title: "Web Components"
---

# Web Components

<a name="introduction"></a>

## Introduction

Reepolee ships four custom elements: `<validation-error>`, `<toasts-area>`, `<title-display>`, and `<image-editor>`. They're plain web components — no framework, no shadow DOM dependency beyond what the platform provides — and they're loaded as static scripts in your layout. Each one solves a specific problem that isn't worth a framework component but is worth abstracting away from every page that needs it.

This page is the inventory. The first two have full coverage in their dedicated sections; this is the place to look when you want to know what's available and what each element does.

<a name="loading"></a>

## Loading

Three elements live in `static/web-components/` (`validation-error`, `toasts-area`, `title-display`); `image-editor` lives in `static/` directly. Load whichever ones the page needs in your layout's `<head>`:

```html
<script src="/web-components/validation-error.js" defer></script>
<script src="/web-components/toasts-area.js" defer></script>
<script src="/web-components/title-display.js" defer></script>
<script src="/image-editor.js" defer></script>
```

Each script registers its custom element with `customElements.define()` on load. After that, the element is usable anywhere on the page — there's no per-page initialisation.

<a name="validation-error"></a>

## validation-error

Displays a single per-field validation error inline. Used in every form throughout the application; full integration details are in [Validation](/forms/validation).

```html
<input type="text" id="email" name="email" />
<validation-error id="error-email"> {= props.errors.email } </validation-error>
```

The element is a thin shadow-DOM wrapper around a `<slot>`. It re-renders when its slotted content changes, which is what makes the live-validation pattern work — `FormController` sets the element's `innerHTML` and the element picks up the change.

When the slotted content is empty, the element renders nothing visible. There's no `display: none` to manage; an unused `<validation-error>` simply takes up no visual space.

<a name="toasts-area"></a>

## toasts-area

A fixed-position container that displays toast notifications stacked at the bottom of the viewport. Used for confirming saves, surfacing errors after a redirect, and any other "something happened, here's a brief notification" pattern. Full integration details are in [Toast Notifications](/forms/toast-notifications).

```html
<toasts-area id="toasts-area"></toasts-area>
```

Place it once near the end of your `<body>`. The element exposes an `add_toast(toast)` method and a global `window.addToast(toast)` alias for adding notifications from any script:

```js
addToast({
	type: "green",
	message: "Saved successfully",
	duration: 2000,
});
```

The element manages auto-expiry (each toast self-removes after its `duration`) and stacks multiple toasts with a small staggered animation.

<a name="title-display"></a>

## title-display

A small element for projecting a string with optional capitalization or pluralization, useful for keeping headings in sync with feature names without duplicating the string in code.

```html
<title-display capitalize pluralize>user</title-display>
<!-- renders: Users -->

<title-display capitalize>email</title-display>
<!-- renders: Email -->

<title-display pluralize>category</title-display>
<!-- renders: categories -->
```

The two boolean attributes:

- **`capitalize`** — uppercase the first character.
- **`pluralize`** — naive English pluralization (`y` → `ies`, otherwise add `s`).

The element observes its slotted text content with a `MutationObserver`, so changing the text dynamically re-renders the output:

```js
document.querySelector("title-display").textContent = "project";
```

The pluralization is intentionally simple — it covers most database table names (`user` → `users`, `category` → `categories`) and falls down on irregular plurals (`person` → `persons`, not `people`). For anything beyond that, render the right string on the server using the route's translations.

<a name="image-editor"></a>

## image-editor

<media-frame label="Screenshot — image-editor (canvas with crop controls)" ratio="16/9"></media-frame>

An in-browser image editor backed by `Bun.Image` on the server. The element renders a canvas, exposes crop / resize controls, and posts the resulting bytes back to your handler when the user saves. Used by the shipped Email Module to crop hero images and by the avatar form in `/profile` when configured.

```html
<image-editor name="hero" src="/uploads/{= props.record.hero_filename }"></image-editor>
```

The full set of attributes and the server-side endpoint it expects are in `static/image-editor.js`. Most projects won't reach for it directly — the avatar and email flows wire it up for you.

<a name="why-shadow-dom"></a>

## Why Shadow DOM

`<validation-error>` and `<title-display>` use shadow DOM for style encapsulation. The shadow root keeps Tailwind's global resets and the application's CSS from accidentally restyling the element's internals — so the same `<validation-error>` looks the same whether it's on a CRUD form or a marketing page. `<toasts-area>` doesn't use shadow DOM; its children pick up the page's styles intentionally so toasts match the surrounding theme.

If you're writing your own custom element, the rough rule:

- **Shadow DOM** if the element has internal markup that must look the same regardless of where it's used.
- **No shadow DOM** if the element's children should inherit page styles (theming, typography).

<a name="writing-your-own"></a>

## Writing Your Own Custom Element

A minimal custom element is around twenty lines. The element class extends `HTMLElement`, optionally attaches a shadow root in the constructor, and renders in `connectedCallback`:

```js
class CopyButton extends HTMLElement {
	connectedCallback() {
		const text = this.getAttribute("text") || "";

		this.innerHTML = `<button>Copy</button>`;
		this.querySelector("button").addEventListener("click", () => {
			navigator.clipboard.writeText(text);
			this.querySelector("button").textContent = "Copied!";
			setTimeout(() => {
				this.querySelector("button").textContent = "Copy";
			}, 1500);
		});
	}
}

customElements.define("copy-button", CopyButton);
```

```html
<copy-button text="https://reepolee.com"></copy-button>
```

A few conventions worth following:

- **Element names must contain a hyphen.** That's the rule that keeps custom elements from colliding with future native elements.
- **Initialise in `connectedCallback`, not the constructor.** Attributes and children may not be available yet when the constructor runs.
- **Read attributes with `getAttribute`** rather than properties — properties on a custom element only exist if you define them.
- **Save the file in `static/web-components/`** and load it in your layout for project-wide availability, or inline the script in a single template if it's only used on one page.

For elements that need to react to attribute changes after registration, implement `static get observedAttributes()` and `attributeChangedCallback` — `<validation-error>` and `<title-display>` both use this pattern; their source is the most readable starting point.

<a name="other-bundled-utilities"></a>

## Other Bundled Utilities

Reepolee also ships a small set of vanilla JS classes that aren't custom elements but live alongside them in `static/`:

- **`form-controller.js`** — live validation and submit interception. See [Validation](/forms/validation).
- **`checkbox-group.js`** — bulk-select tables (select all / deselect all / enable action buttons when any row is checked). Used in generated CRUD list views.
- **`spa-loader.js`** — SPA-style navigation. See [SPA Navigation](/client-side/spa-navigation).
- **`dialog-confirm.js`** — tiny global handler that turns `[data-dialog-confirm]` buttons into a `"confirm"` event on their enclosing `<dialog>`. Open/close use native HTML `commandfor` + `command` attributes (no JS). See [Dialogs](/client-side/dialogs).
- **`helpers-client.js`** — `$`, `$$`, and a few small DOM helpers loaded globally.

Each one is small enough to read in one sitting. When you need to know exactly what's happening, the source is in `static/`.
