---
title: "Signals"
---

# Signals

<a name="introduction"></a>

## Introduction

Client-side reactivity in Reepolee is built on [alien-deepsignals](https://github.com/heyrict/alien-deepsignals) — a small object-reactivity layer over the [alien-signals](https://github.com/stackblitz/alien-signals) push-pull engine. The [Signals UI Patterns](/client-side/signals-ui) page covers the common DOM bindings layered on top (`bind_show`, `bind_class`, `bind_collapse`, etc.). This page covers reaching for deepsignals directly: wrapping page state in a `deepSignal()` and reacting to it with `watchEffect()` and `watch()`.

The library is vendored at `public/alien-deepsignals.min.js` after running `bun get:signals` (it's fetched from a CDN at setup time, not shipped in the repo). The bundle is self-contained — alien-signals is inlined — so there's no bundler step and no `node_modules` involvement. The pattern is opt-in per page; pages that don't need it never load it.

A working demo lives in `routes/examples/signals/signals.ree`. This page walks through the same building blocks and explains when to reach for them.

<a name="when-to-use-signals"></a>

## When to Reach for Deepsignals

For most UI work — toggles, dropdowns, accordions, modal open/close — the `bind_*` helpers in [Signals UI Patterns](/client-side/signals-ui) already wrap the reactivity primitives and there's no reason to reach past them.

Reach for `deepSignal()` directly when:

- A form has many fields that derive from each other — quantity × price = total, country selection changes available regions, a slider drives a live preview.
- State needs to flow across non-adjacent elements and you want precise "this value changed, only this view should update" behaviour without a DOM-binding helper applying.
- You're building something that's mostly derived state — a configurator, a calculator, a form that previews its output as you type.

If the server can render it, render it on the server. Signals are for state that genuinely lives only on the page.

<a name="primitives"></a>

## The Core API

alien-deepsignals centres on three functions:

```js
import { deepSignal, watchEffect, watch } from "/alien-deepsignals.min.js";
```

| Function            | Purpose                                                                                       |
| ------------------- | --------------------------------------------------------------------------------------------- |
| `deepSignal(obj)`   | Wraps an object and makes every property reactive on read and on write.                        |
| `watchEffect(fn)`   | Runs `fn` immediately, then re-runs whenever any deepsignal property it read changes.          |
| `watch(source, cb)` | Watches a specific source (a getter or deepsignal) and runs `cb(next, prev)` when it changes.  |

The simplest possible example — a counter that updates the DOM whenever its value changes:

```html
<input id="counter_input" value="100" disabled />
<button id="plus_button">Add 1</button>

<script type="module">
	import { deepSignal, watchEffect } from "/alien-deepsignals.min.js";

	const counter = deepSignal({ count: parseInt(counter_input.value) });

	plus_button.onclick = () => counter.count++;

	watchEffect(() => (counter_input.value = counter.count));
</script>
```

A few things to notice:

- **Properties read and write like plain object access.** `counter.count` reads (and tracks the read); `counter.count++` writes and notifies anything that read it.
- **`watchEffect` runs once immediately**, then again whenever a property it read changes. It ignores its callback's return value, so the `() => (el.value = x)` shorthand is safe.
- **There's no virtual DOM.** The effect does whatever side effect you write — set an attribute, replace text, draw on a canvas.

The page also references HTML elements by id directly (`counter_input`, `plus_button`) — browsers expose any element with an id as a global variable, so for short scripts you can skip the `document.getElementById` call.

<a name="deriving-state"></a>

## Deriving State

alien-deepsignals doesn't ship a standalone `computed()`. Derive values inside a `watchEffect` and write the result straight to the DOM (or into another deepsignal property). Dependency tracking is automatic — whatever properties you read inside the effect become its dependencies:

```js
import { deepSignal, watchEffect } from "/alien-deepsignals.min.js";

const cart = deepSignal({ quantity: 2, price: 9.99 });

watchEffect(() => {
	total_display.textContent = (cart.quantity * cart.price).toFixed(2);
});

cart.quantity = 5; // total recomputes to 49.95
cart.price = 12.5; // total recomputes to 62.50
```

When you need to react to one specific source instead of re-running a whole effect — fire an XHR when a select changes, say — reach for `watch`:

```js
watch(
	() => cart.quantity,
	(next, prev) => console.log(`quantity went ${prev} → ${next}`),
);
```

<a name="deep-signals-for-forms"></a>

## Deep Signals for Forms

A form usually has a handful of fields, and a single `deepSignal()` over an object keeps them all reactive without declaring a separate signal per field:

```html
<form id="profile_form">
	<input id="first_name_input" name="first_name" value="Aleš" />
	<input id="last_name_input" name="last_name" value="Vaupotič" />
</form>
<pre id="debug"></pre>

<script type="module">
	import { deepSignal, watchEffect } from "/alien-deepsignals.min.js";

	const profile = deepSignal({
		first_name: first_name_input.value,
		last_name: last_name_input.value,
	});

	first_name_input.oninput = () => (profile.first_name = first_name_input.value);
	last_name_input.oninput = () => (profile.last_name = last_name_input.value);

	watchEffect(() => {
		debug.textContent = JSON.stringify(profile, null, 4);
	});
</script>
```

`profile.first_name` and `profile.last_name` are reactive on read and on write. The `watchEffect` block re-runs whenever either changes and updates the debug output.

The cleanest pattern for forms with several fields is to bind every input the same way — listen to `input`, write to the deep signal — then derive everything else (validation results, totals, button states) from those values inside a `watchEffect`.

<a name="signals-and-form-controller"></a>

## Signals and FormController

`FormController` (covered in [Validation](/forms/validation)) handles live validation and submit interception by tracking values in its own internal `values` object. It's not built on signals — it uses plain event listeners and a manual `render_errors()` call.

The two systems coexist fine. Use `FormController` for the standard "fields + per-field validation + submit" flow; reach for deepsignals when you need derived state on top of that. A common pattern: let `FormController` manage the field values and validation, and use a deep signal to drive a live preview or computed totals that don't belong in the form payload.

For projects where the form is mostly derived state (a configurator, a calculator, a form that previews its output as you type), it's reasonable to skip `FormController` entirely and build the whole thing with deep signals + native form submission.

<a name="when-not-to-use-signals"></a>

## When Not to Use Signals

The default rule, stricter than for the `bind_*` helpers because reaching for the library directly is slightly more involved:

- **If the server can render it**, render it on the server. A list of records doesn't need signals; render the `<ul>` server-side.
- **If `bind_show` / `bind_class` / `bind_text` is enough**, use them and skip the explicit `watchEffect()`. The helpers exist for a reason.
- **If you're unsure**, start with the helpers. Switching from a `bind_*` call to an explicit `watchEffect()` later is one line of code; the other direction is the same.

<a name="loading"></a>

## Loading

`alien-deepsignals` ships vendored at `public/alien-deepsignals.min.js` (re-fetch with `bun get:signals`). The file is a self-contained bundle — alien-signals is inlined — so imports resolve to `/alien-deepsignals.min.js` directly, with no CDN at runtime:

```js
import { deepSignal, watchEffect } from "/alien-deepsignals.min.js";
```

The demo in `routes/examples/signals/signals.ree` imports the same library from `esm.sh` instead, so it stays self-contained without a `bun get:signals` step:

```js
import { deepSignal, watchEffect } from "https://esm.sh/alien-deepsignals@0.4.2";
```

For your own pages, prefer the vendored path. Either way, pin the version — don't use a bare `latest` tag, so the page doesn't break the day the library publishes a major release.
