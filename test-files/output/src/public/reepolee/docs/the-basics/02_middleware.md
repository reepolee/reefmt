---
title: "Middleware"
---

# Middleware

<a name="introduction"></a>

## Introduction

Middleware lets you wrap one or more route handlers with shared behaviour — logging, language detection, authentication, response headers — without touching the handlers themselves. A Reepolee middleware is a plain async function. There is no class to extend and no registration step beyond importing the function and applying it.

Every middleware has the same signature: it receives the incoming `BunRequest` and a `next` function that calls the wrapped handler. It can inspect or modify the request, short-circuit with an early response, or post-process the outgoing response before returning it:

```ts
import type { Middleware } from "$lib/middleware";

const timing: Middleware = async (req, next) => {
	const start = performance.now();
	const res = await next(req);
	const ms = (performance.now() - start).toFixed(1);

	const headers = new Headers(res.headers);
	headers.append("Server-Timing", `app;dur=${ms}`);
	return new Response(res.body, { ...res, headers });
};
```

`$lib/middleware` ships with a small set of built-in middlewares and three composition helpers: `with_middleware`, `wrap_all_routes`, and `mount_prefix`.

<a name="composing-middleware"></a>

## Composing Middleware

`with_middleware()` wraps a single handler with one or more middlewares. Multiple middlewares compose right-to-left — the rightmost middleware runs closest to the handler:

```ts
import { with_middleware } from "$lib/middleware";

"/profile": with_middleware(profile_page, require_auth_mw(), timing),
```

In that example, `timing` runs first (starting the clock), then `require_auth_mw()` checks the session, then `profile_page` produces the response. On the way back, `require_auth_mw()` is the first to see the response, then `timing` appends the `Server-Timing` header last.

<a name="wrapping-every-route"></a>

## Wrapping Every Route

`wrap_all_routes()` applies one or more middlewares to every handler in a route table at once. This is how the global pipeline is applied in `server.ts`:

```ts
import { wrap_all_routes, rate_limit_mw, set_lang, csrf_mw } from "$lib/middleware";
import { active_languages } from "$config/supported_languages";

const routed = wrap_all_routes(routes, rate_limit_mw(), set_lang(active_languages), csrf_mw());

Bun.serve({ routes: routed, fetch(req) { ... } });
```

The global chain runs in order `rate_limit_mw → set_lang → csrf_mw`: the rate limiter runs first (before any language or CSRF work, so a throttled request is rejected as cheaply as possible), `set_lang` resolves the language, and `csrf_mw` validates the double-submit token last. `wrap_all_routes()` also registers trailing-slash redirect variants for every route.

`wrap_all_routes()` understands both shapes a route can take — a single handler function and a method map — so wrapping a CRUD module wraps every method on every URL inside it.

<a name="prefixed-routes"></a>

## Prefixed Routes

`mount_prefix()` mounts an entire route table under a URL prefix and optionally wraps every handler with one or more middlewares. This is the standard way to build an admin section — every route picks up the prefix and the auth check in a single call:

```ts
import { mount_prefix, require_module_mw } from "$lib/middleware";
import { admin_crud } from "$routes/admin";

export const routes = {
	"/": home_page,
	...mount_prefix("/admin", admin_crud, require_module_mw("admin")),
};
```

`/users` becomes `/admin/users`, `/users/new` becomes `/admin/users/new`, and `require_module_mw("admin")` is applied to every handler automatically.

`mount_prefix()` without any middleware arguments just adds the prefix:

```ts
...mount_prefix("/api", api_routes),
```

<a name="built-in-middleware"></a>

## Built-in Middleware

Reepolee ships with a handful of middlewares for the most common cross-cutting concerns. All of them live under `$lib/middleware`.

| Middleware                       | Purpose                                                                                                                                                                                                                                               |
| -------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `timing`                         | Appends a `Server-Timing` header showing how long the handler took                                                                                                                                                                                    |
| `rate_limit_mw(client?)`         | Sliding-window rate limiting backed by Redis. First in the global chain. Disabled unless `RATE_LIMITING=true`; fails loud if enabled without `REDIS_URL`. Accepts an optional `RedisClient` for testing. See [Rate Limiting](/security/rate-limiting) |
| `set_lang(languages, options?)`  | Resolves the active language from query string, cookie, or URL path; sets the `lang` cookie when needed                                                                                                                                               |
| `csrf_mw()`                      | Double-submit cookie CSRF protection on state-changing requests; skips `/validate` endpoints. See [CSRF Protection](/security/csrf)                                                                                                                   |
| `add_cors(options?)`             | Adds CORS headers to responses for cross-origin clients                                                                                                                                                                                               |
| `require_auth_mw()`              | Redirects unauthenticated requests to the localized `/login`                                                                                                                                                                                          |
| `require_module_mw(module_code)` | Redirects unauthenticated requests to the localized `/login`; returns `403` if the user's `modules_tags` doesn't include `module_code`                                                                                                                |

The two auth middlewares are covered in depth in [Authorization](/security/authorization); rate limiting and CSRF have their own pages under [Security](/security/rate-limiting).

<a name="writing-your-own"></a>

## Writing Your Own

Any function with the `Middleware` signature is a valid middleware. The pattern for adding a response header is the most common shape:

```ts
import type { Middleware } from "$lib/middleware";

export const cache_for_an_hour: Middleware = async (req, next) => {
	const res = await next(req);
	const headers = new Headers(res.headers);
	headers.set("Cache-Control", "public, max-age=3600");
	return new Response(res.body, { status: res.status, headers });
};
```

A few things to keep in mind:

- Always clone `res.headers` into a new `Headers` object before mutating. The original headers belong to the response coming back from `next` and should not be modified in place.
- Wrap the body with `new Response(res.body, ...)` rather than reading and re-emitting it — this preserves streaming semantics.
- If your middleware needs to short-circuit (block the request, redirect, etc.), return a `Response` directly without calling `next`.

<a name="middleware-and-request-state"></a>

## Sharing Data Between Middleware and Handler

Middleware can stash values on the request by setting headers — `req.headers.set()` is mutable, and downstream handlers (or other middlewares) can read what you set. The built-in `set_lang` middleware uses this pattern: it writes the resolved language to `X-Lang` so `render()` can pick it up later:

```ts
req.headers.set("X-Lang", finalLang);
return next(req);
```

The same convention applies to anything else you want to pass through — feature flags, request IDs, computed user context.
