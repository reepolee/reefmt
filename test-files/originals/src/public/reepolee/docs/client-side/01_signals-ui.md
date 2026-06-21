---
title: "Signals UI Patterns"
---

# Signals UI Patterns

<a name="introduction"></a>

## Introduction

Reepolee renders pages on the server and uses fine-grained signals on the client for the small pieces of behaviour that need to live in the browser — dropdowns, modals, toggles, tabs, anything that doesn't justify a full client-side framework. These patterns are powered by `signals-ui.js`, a small self-contained module that provides a minimal `signal()` / `effect()` pair plus DOM-binding utilities on top. It's around 3 KB, has no external dependencies, and needs no build step. For heavier object- and form-shaped state, reach for [alien-deepsignals](/client-side/signals).

This page covers the patterns we reach for in practice. For reaching for the reactivity library directly — `deepSignal()`, `watchEffect()`, `watch()` — see [Signals](/client-side/signals).

<a name="loading-signals-ui"></a>

## Loading signals-ui

`signals-ui.js` (~3 KB) ships under `static/` and is fully self-contained — it carries its own minimal `signal()` / `effect()` implementation, so there's nothing to fetch, no `node_modules`, and no bundler step.

Pages that need client behaviour drop an inline ES module at the bottom of the template and import what they need:

```html
<button id="toggle">Open</button>
<div id="panel" hidden>Hidden content</div>

<script type="module">
	import { signal, bind_show } from "/signals-ui.js";

	const open = signal(false);
	toggle.onclick = () => open(!open());
	bind_show(panel, open);
</script>
```

Pages without behaviour load nothing — the cost is paid per page, not per project.

<a name="mental-model"></a>

## The Mental Model

A `signal()` is a reactive value. Call it with no arguments to read; call it with one argument to write:

```js
const open = signal(false);
open(); // → false
open(true); // sets to true
open(); // → true
```

`effect()` runs a function and tracks every signal read inside it. Whenever any of those signals change, the effect re-runs:

```js
effect(() => {
	panel.hidden = !open();
});
```

The `bind_*` helpers in `signals-ui.js` are thin wrappers around `effect()` — each one connects one piece of state to one piece of DOM. They exist so you don't have to write the `effect(() => …)` boilerplate for the common cases.

There is no scope concept like Alpine's `x-data`. State lives wherever you declare it — usually inside a page-bottom `<script type="module">`. Sharing state across non-adjacent parts of the page means declaring the signal once and passing it to every helper that reads or writes it.

<a name="the-bindings"></a>

## The Bindings

| Helper                           | Behaviour                                                                                                              |
| -------------------------------- | ---------------------------------------------------------------------------------------------------------------------- |
| `signal(initial)`                | Reactive value; getter when called bare, setter when called with an argument.                                          |
| `computed(() => …)`              | Derived value, recomputed when its signal dependencies change.                                                         |
| `effect(() => …)`                | Side effect re-run when its tracked signals change.                                                                    |
| `bind_show(el, getter)`          | Toggles the native `hidden` attribute.                                                                                 |
| `bind_text(el, getter)`          | Sets `textContent`.                                                                                                    |
| `bind_class(el, getter, name)`   | Toggles a class.                                                                                                       |
| `bind_attr(el, attr, getter)`    | Sets/removes an attribute. `null`/`false`/`undefined` removes it; `true` writes an empty value.                        |
| `bind_collapse(el, getter)`      | Show/hide alias for accordion-like groups (toggles `hidden`).                                                          |
| `on_intersect(el, setter, opts)` | IntersectionObserver wrapper. Calls `setter()` whenever the element enters the viewport.                               |
| `bind_focus_trap(el, getter)`    | When the getter is truthy, traps Tab inside the element and adds `scroll-locked` to `<body>`. Restores focus on close. |
| `on_escape(handler)`             | Listens for Escape on `window`.                                                                                        |
| `restore_scroll(el, key)`        | Reads/writes `el.scrollTop` from `sessionStorage[key]`.                                                                |

Imports are plain named imports — pull in only what the page uses:

```js
import { signal, computed, effect, bind_show, bind_class, on_intersect } from "/signals-ui.js";
```

<a name="events"></a>

## Events

Events use `addEventListener` (or the older `el.onclick =` shortcut). There is no `@click` shorthand; there is also no scope to inherit from, so handlers are just closures over the signals they reference:

```html
<button id="incr">+1</button>
<button id="reset">Reset</button>
<output id="display">0</output>

<script type="module">
	import { signal, bind_text } from "/signals-ui.js";

	const count = signal(0);
	incr.onclick = () => count(count() + 1);
	reset.onclick = () => count(0);
	bind_text(display, count);
</script>
```

Common Alpine modifiers map to native event idioms:

| Alpine                   | Native equivalent                                                                                      |
| ------------------------ | ------------------------------------------------------------------------------------------------------ |
| `@click.prevent`         | `(e) => { e.preventDefault(); … }`                                                                     |
| `@click.stop`            | `(e) => { e.stopPropagation(); … }`                                                                    |
| `@click.outside`         | a document-level click handler that bails when `el.contains(e.target)`                                 |
| `@keydown.escape.window` | `on_escape(() => …)` from `signals-ui.js`                                                              |
| `@submit.prevent`        | `form.addEventListener("submit", (e) => { e.preventDefault(); … })`                                    |
| `@input.debounce.300ms`  | wrap the handler with a small debounce; nothing in `signals-ui.js` ships one because it is three lines |

<a name="common-patterns"></a>

## Common Patterns

<media-frame label="Animation — signals-ui patterns (toggle · dropdown · accordion)" ratio="16/9"></media-frame>

<a name="toggle-dropdown"></a>

### Toggle and Dropdown

```html
<button id="menu_toggle">Menu</button>
<ul id="menu" hidden>
	…
</ul>

<script type="module">
	import { signal, bind_show, on_escape } from "/signals-ui.js";

	const open = signal(false);
	menu_toggle.onclick = () => open(!open());
	bind_show(menu, open);
	on_escape(() => open(false));
	document.addEventListener("click", (e) => {
		if (!menu.contains(e.target) && e.target !== menu_toggle) open(false);
	});
</script>
```

<a name="accordion"></a>

### Accordion (Sidebar Groups)

A single signal holds the currently-open group slug; each panel listens for its own slug being active:

```html
<div data-group="getting-started">
	<button data-group-toggle="getting-started">Getting Started</button>
	<div data-group-panel hidden>…</div>
</div>
<div data-group="forms">
	<button data-group-toggle="forms">Forms</button>
	<div data-group-panel hidden>…</div>
</div>

<script type="module">
	import { signal, bind_collapse, bind_class } from "/signals-ui.js";

	const open_group = signal(location.pathname.split("/")[1] || "getting-started");

	document.querySelectorAll("[data-group]").forEach((wrap) => {
		const slug = wrap.dataset.group;
		const panel = wrap.querySelector("[data-group-panel]");
		const toggle = wrap.querySelector("[data-group-toggle]");
		toggle.onclick = () => open_group(open_group() === slug ? "" : slug);
		bind_collapse(panel, () => open_group() === slug);
		bind_class(toggle, () => open_group() === slug, "rotate-180");
	});
</script>
```

<a name="modal"></a>

### Modal with Focus Trap

```html
<button id="open_modal">Open</button>
<div id="modal" hidden role="dialog" aria-modal="true">
	<button id="close_modal">Close</button>
	…
</div>

<script type="module">
	import { signal, bind_show, bind_focus_trap, on_escape } from "/signals-ui.js";

	const open = signal(false);
	open_modal.onclick = () => open(true);
	close_modal.onclick = () => open(false);
	bind_show(modal, open);
	bind_focus_trap(modal, open);
	on_escape(() => open(false));
</script>
```

Add `.scroll-locked { overflow: hidden; }` to your stylesheet — `bind_focus_trap` adds that class to `<body>` while the trap is active. For most application modals, the native `<dialog>` element with `command="show-modal"` is a better fit (see [Dialogs](/client-side/dialogs)).

<a name="intersect"></a>

### Active-Section Highlighting on Scroll

```html
<section id="intro" data-intersect="intro">…</section>
<section id="usage" data-intersect="usage">…</section>
<aside>
	<a data-section="intro">Intro</a>
	<a data-section="usage">Usage</a>
</aside>

<script type="module">
	import { signal, bind_class, on_intersect } from "/signals-ui.js";

	const active = signal("intro");

	document.querySelectorAll("[data-intersect]").forEach((el) => {
		on_intersect(el, () => active(el.dataset.intersect), {
			rootMargin: "-80px 0px -60% 0px",
		});
	});

	document.querySelectorAll("[data-section]").forEach((a) => {
		bind_class(a, () => active() === a.dataset.section, "active");
	});
</script>
```

<a name="server-state-into-signals"></a>

## Bringing Server State Into Signals

Server-rendered values flow into signals via the same `{= }` and `{~ }` tags used elsewhere. The script runs as JavaScript, so server values must be valid JS literals:

```html
<script type="module">
	import { signal } from "/signals-ui.js";

	const count  = signal({= props.initial_count });
	const name   = signal("{= props.user.name }");
	const record = signal({~ JSON.stringify(props.record) });
</script>
```

Use `{~ }` (raw output) for the JSON literal — the JSON encoder already escapes the inner content, so double-escaping with `{= }` would break the literal.

<a name="preventing-fouc"></a>

## Preventing the Initial Flash

ES modules are deferred, so without precautions a freshly-rendered page can briefly show every "hidden" element before the script runs and hides them. The fix is to set the `hidden` attribute server-side on whichever elements should start hidden, then let `bind_show` toggle the same attribute reactively:

```html
<div id="panel" {= props.is_active ? "" : "hidden" }>…</div>
```

`bind_show` writes the same `hidden` attribute, so initial render and reactive updates use the same mechanism. There is no `x-cloak` equivalent because the native `hidden` attribute does the job without a stylesheet rule.

<a name="when-not-to-use-signals"></a>

## When Not to Use Signals

The default rule: **if the server can render it, render it on the server.** A list of records doesn't need signals; render the `<ul>` server-side. A page header doesn't need signals. A form field doesn't need signals; that's `FormController`'s job.

Reach for signals when the user's interaction has to feel immediate and the state lives only on the page:

- Opening and closing a modal or dropdown
- Toggling a tab or accordion
- Showing live feedback as the user types (without round-tripping the server)
- Swapping the visible state of a UI control (a "copy to clipboard" button that shows "Copied!" briefly)

For form input state shared across many fields with derived values, see [Signals](/client-side/signals) for `deepSignal()` and `watchEffect()` patterns.

<a name="dom-helpers"></a>

## DOM Helpers

Reepolee loads `helpers-client.js` alongside the signals module to expose two small globals that come up often in inline scripts:

```js
$("#email"); // shorthand for document.querySelector
$$("input[name]"); // shorthand for Array.from(document.querySelectorAll(...))
```

Both accept an optional second argument to scope the query to a parent element. They are plain functions, not signals — but they are available everywhere because the script is loaded in the layout.
