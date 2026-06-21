---
title: "CSRF Protection"
---

# CSRF Protection

<a name="introduction"></a>

## Introduction

Cross-Site Request Forgery protection is built into the global middleware chain via `csrf_mw()` (`lib/middleware/csrf.ts`), which runs after `set_lang`. It uses the **double-submit cookie** pattern: a random token is stored in a cookie _and_ echoed back in every form, and the middleware rejects any state-changing request where the two don't match. Because an attacker's cross-origin page can't read your cookie, it can't forge the matching field.

Unlike rate limiting and caching, CSRF protection has **no enable flag** — it is always on. Generated forms already include the token, so for typical CRUD apps it works without any setup.

<a name="how-it-works"></a>

## How It Works

1. **Cookie is issued.** On any response that doesn't already have one, `csrf_mw()` sets a random `csrf_token` cookie (24-hour max-age).
2. **Token reaches the template.** The middleware also writes the token to the `X-CSRF-Token` request header. `render()` reads that header and exposes it to templates as `csrf_token`.
3. **Form submits it.** Every form includes a hidden field carrying the token:
    ```html
    <input type="hidden" name="_csrf_token" value="{= csrf_token }" />
    ```
4. **Middleware validates.** On `POST`, `PUT`, `PATCH`, and `DELETE`, the middleware extracts the submitted token and compares it to the cookie. A mismatch (or missing token) is rejected before the handler runs.

`GET`, `HEAD`, and `OPTIONS` are never validated (they shouldn't mutate state), and a small set of read-only validation endpoints is explicitly skipped (see below).

<a name="token-sources"></a>

## Where the Token Can Come From

`extract_csrf_token()` looks in three places, in order:

1. The **`X-CSRF-Token` header** — used by AJAX / `fetch` requests.
2. A **form field** named `_csrf_token` — for `application/x-www-form-urlencoded` and `multipart/form-data` submissions.
3. A **`_csrf_token` JSON property** — for `application/json` request bodies.

For AJAX calls, read the token from the cookie and send it as a header — exactly what generated list pages do for inline delete/bulk actions:

```js
const csrf_token = document.cookie.match(/csrf_token=([^;]+)/)?.[1] || "";

await fetch(url, {
	method: "POST",
	headers: { "X-CSRF-Token": csrf_token },
	body,
});
```

<a name="exempt-endpoints"></a>

## Exempt Endpoints

Some endpoints are `POST` but only read and validate input (live form validation), never mutating state. These are listed in `SKIP_VALIDATION_PATHS` and bypass CSRF checks:

- `/login/validate`
- `/register/validate`
- `/invite/validate`
- `/profile/validate`
- `/password/validate`

This mirrors the rate limiter's treatment of `*/validate` endpoints. If you add your own validation-only POST endpoint, add it to this set so the CSRF check doesn't block legitimate keystroke-by-keystroke validation.

<a name="writing-forms"></a>

## Adding the Token to Hand-Written Forms

If you write a form by hand (rather than generating it), include the hidden field so the submission passes validation:

```html
<form method="post" action="/your-route">
	<input type="hidden" name="_csrf_token" value="{= csrf_token }" />
	<!-- your fields -->
</form>
```

`csrf_token` is available in every template `render()` produces, because the middleware sets the header on every request. If you ever see a CSRF rejection on a hand-written form, the missing hidden field is almost always the cause.
