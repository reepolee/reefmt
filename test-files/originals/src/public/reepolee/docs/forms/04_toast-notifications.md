---
title: "Toast Notifications"
---

# Toast Notifications

<a name="introduction"></a>

## Introduction

After a successful create, update, or delete, the user needs feedback — but the response is a redirect, so there's no template to render a "Saved!" banner into. Reepolee's solution is a cookie-based toast: the handler attaches a small cookie to the redirect, the next page load reads the cookie, renders the toast on the page, and immediately expires the cookie so it doesn't show again.

The pattern survives page refreshes, works without JavaScript (the toast simply doesn't appear, but the redirect still succeeds), and never accumulates state on the server. Everything that needs to be remembered is in the cookie itself.

<a name="creating-a-toast"></a>

## Creating a Toast

`create_toast_cookie()` from `$lib/helpers` builds the cookie. It takes a small options object:

```ts
import { create_toast_cookie } from "$lib/helpers";

const cookie = await create_toast_cookie({
	record_id: record.id,
	feature: "users",
	message: translated.messages.record_updated,
	type: "green",
	req,
});
```

| Option      | Type               | Default            | Description                                                                            |
| ----------- | ------------------ | ------------------ | -------------------------------------------------------------------------------------- |
| `record_id` | `number \| string` | —                  | Used to build the cookie name so multiple toasts can coexist                           |
| `feature`   | `string`           | —                  | Feature name (e.g. `"users"`); included in the toast payload                           |
| `message`   | `string`           | `"record_updated"` | The text shown to the user                                                             |
| `type`      | `string`           | `"yellow"`         | `"green"` (success), `"red"` (error), `"yellow"` (warning), or anything else (neutral) |
| `duration`  | `number`           | `2500`             | Milliseconds the toast stays visible                                                   |
| `req`       | `BunRequest`       | —                  | If provided, the current user's display name is included on the toast                  |

`create_toast_cookie` returns a `Cookie` object whose `toString()` produces the `Set-Cookie` header value.

<a name="attaching-the-cookie-to-a-redirect"></a>

## Attaching the Cookie to a Redirect

`Response.redirect()` doesn't let you attach extra headers, so toast-bearing redirects are built manually:

```ts
const cookie = await create_toast_cookie({
	record_id: record.id,
	feature: "users",
	message: translated.messages.record_updated,
	type: "green",
	req,
});

const headers = new Headers({ Location: "/users" });
headers.append("Set-Cookie", cookie.toString());

return new Response(null, { status: 303, headers });
```

Status `303` ensures the browser follows the redirect with a `GET` regardless of the original method. The body is `null`. Any number of `Set-Cookie` headers can be appended — pair multiple toasts with a session cookie if you need to.

<a name="how-the-toast-is-displayed"></a>

## How the Toast Is Displayed

<media-frame label="Screenshot — green success toast, bottom-centre" ratio="16/9"></media-frame>

The layout includes the `<toasts-area>` custom element once, near the end of `<body>`:

```html
<toasts-area id="toasts-area"></toasts-area>
```

Right after that, the layout iterates `props.toasts` and registers each one with the element:

```html
{#if props.toasts?.length > 0 }
<script>
	{#each props.toasts as toast }
	    addToast({~ JSON.stringify(toast.value) });
	{/each }
</script>
{/if }
```

`props.toasts` is populated automatically by `render()` — when you pass `req`, the render layer reads every cookie with the `toast-` prefix, hands them to the template as `props.toasts`, and **adds `Set-Cookie: <name>=; Max-Age=0` headers to the response** to expire them immediately. The user sees the toast exactly once.

<a name="the-toasts-area-element"></a>

## The toasts-area Element

`<toasts-area>` is a vanilla custom element in `static/web-components/toasts-area.js`. It:

- Positions itself fixed at the bottom-centre of the viewport.
- Stacks multiple toasts with a small staggered animation.
- Provides an `add_toast(toast)` method (and a global `window.addToast` convenience alias).
- Removes each toast automatically when its `duration` elapses.
- Adds a close button so the user can dismiss a toast early.

The toast object the element accepts:

```ts
{
    id: string,           // optional; auto-generated if missing
    type: "green" | "red" | "yellow" | "neutral",
    message: string,      // shown as the toast body
    duration: number,     // ms before auto-dismiss
    user: string,         // optional; appended in parentheses
}
```

Load the element once in your layout `<head>`:

```html
<script src="/web-components/toasts-area.js" defer></script>
```

<a name="triggering-toasts-from-client-code"></a>

## Triggering Toasts From Client Code

`addToast()` is a global function exposed by the element. Any JavaScript on the page can call it to show a toast without involving the server:

```js
addToast({
	type: "green",
	message: "Copied to clipboard",
	duration: 1500,
});
```

Useful for things that complete entirely in the browser — clipboard actions, drag-and-drop confirmations, signal-driven UI feedback.

<a name="error-toasts"></a>

## Error Toasts

For failures — a save that succeeded in part but couldn't send the confirmation email, for example — use `type: "red"` and a longer duration so the user has time to read it:

```ts
const cookie = await create_toast_cookie({
	record_id: record.id,
	feature: "email",
	message: translated.errors.email_partial_failure,
	type: "red",
	duration: 6000,
	req,
});
```

If the failure is bad enough that you don't want to redirect at all — for example, validation errors on submit — re-render the form with `props.form_errors` and the `banner` component instead of a toast. Toasts are for "the action completed, here's a status update"; the banner is for "the action did not complete, here's why."

<a name="multiple-toasts"></a>

## Multiple Toasts

<media-frame label="Screenshot — multiple toasts stacked" ratio="1/1"></media-frame>

The cookie name uses `record_id` as a discriminator (`toast-updated-<record_id>`), so attaching two toast cookies with different record IDs in the same response shows two toasts. The `<toasts-area>` element stacks them with a staggered animation.

If you're attaching multiple toasts deliberately, pass distinct `record_id` values so the cookie names don't collide. For truly unrelated toasts, the value can be anything unique — a `crypto.randomUUID()`, a timestamp, the feature name plus a counter.

<a name="without-javascript"></a>

## Without JavaScript

If the browser has JavaScript disabled, the toast cookie still gets set and the render layer still expires it on the next page load — the user just doesn't see the toast appear. The redirect completes, the action succeeded, and the user lands on the right page. The toast is a UX nicety, not a load-bearing part of the flow.

For applications where positive confirmation matters more than that (a payment flow, an account deletion), reach for the [`banner`](/forms/input-components#banner) component inside a real page render instead — it shows up the same way for everyone.
