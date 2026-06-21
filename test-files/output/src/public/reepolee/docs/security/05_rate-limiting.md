---
title: "Rate Limiting"
---

# Rate Limiting

<a name="introduction"></a>

## Introduction

Rate limiting caps how many requests a single client can make within a time window, protecting authentication endpoints from brute force and the server as a whole from abuse. Reepolee implements it as the **first** middleware in the global chain (`rate_limit_mw → set_lang → csrf_mw`), so a throttled request is rejected before any language resolution, CSRF check, or template work happens.

The implementation lives in `lib/middleware/rate_limit.ts`, the per-scope rules in `config/rate_limit.ts`, and a full test suite in `lib/middleware/rate_limit.test.ts`.

<a name="enabling"></a>

## Enabling

Rate limiting is **off by default**. Turn it on with two environment variables:

```env
RATE_LIMITING=true
REDIS_URL=redis://localhost:6379
```

The guard is **fail-loud**: with `RATE_LIMITING=true` and no `REDIS_URL`, the process exits at startup (`process.exit(1)`) with a red error. There is no silent fallback and no in-memory degradation — if you ask for rate limiting, you get Redis-backed rate limiting or a refusal to start. When `RATE_LIMITING` is unset or `false`, `rate_limit_mw()` passes every request straight through.

<a name="algorithm"></a>

## The Sliding-Window Algorithm

Reepolee uses a **sliding-window counter**, which avoids the boundary-burst problem of fixed windows (where a client can send a full window's worth of requests at the end of one window and again at the start of the next) without the memory cost of a full log of timestamps.

Each request increments a per-window counter in Redis. The effective count is a weighted blend of the current and previous windows:

```
estimate = prev_count × weight + current_count
weight   = elapsed_in_current_window / window_size
```

This is O(1) memory per key and uses only `INCR` (atomic, provides first-increment semantics) plus a `GET` of the previous window's count — no `MULTI` or `MGET` needed. Keys follow the pattern `rl:{scope}:{identity}:{window_start_epoch}` and are given a TTL of `2 × window_size` so they clean themselves up.

<a name="scopes"></a>

## Scopes and Tiers

Different endpoints get different limits. The tiers are defined in `config/rate_limit.ts`:

| Scope        | Limit     | Applies to                                      |
| ------------ | --------- | ----------------------------------------------- |
| `login`      | 5 / 60s   | `POST /login`                                   |
| `register`   | 3 / 60s   | `POST /register/*`                              |
| `password`   | 5 / 60s   | `POST /password`                                |
| `invite`     | 10 / 60s  | `POST /invite`                                  |
| `validation` | 30 / 60s  | client-side validation endpoints (`*/validate`) |
| `global`     | 300 / 60s | every other state-changing request              |

`resolve_scope()` picks the tier in priority order: (1) anything ending in `/validate` gets the `validation` tier, (2) exact matches for `/login`, `/password`, `/invite`, (3) prefix match for `/register/*`, (4) everything else state-changing falls back to `global`.

Edit `config/rate_limit.ts` to change a limit, add a window, or tune a tier for your traffic:

```ts
export const rate_limit_rules: Record<RateLimitScope, RateLimitRule> = {
	global: { max: 300, window_s: 60 },
	login: { max: 5, window_s: 60 },
	register: { max: 3, window_s: 60 },
	password: { max: 5, window_s: 60 },
	invite: { max: 10, window_s: 60 },
	validation: { max: 30, window_s: 60 },
};
```

<a name="identity"></a>

## Client Identity

`extract_identity()` is hybrid:

- **Authenticated users** are keyed by their session cookie (`sid`), so the limit follows the account across IPs.
- **Anonymous users** are keyed by IP, read from `X-Forwarded-For` then `X-Real-IP`.
- If neither is available, the request is keyed as `ip:unknown`.

Behind a reverse proxy, make sure the proxy sets `X-Forwarded-For` / `X-Real-IP` so anonymous clients aren't all collapsed into one identity. See [Reverse Proxy](/deployment/reverse-proxy).

<a name="responses"></a>

## The 429 Response

<media-frame label="Screenshot — 429 too-many-requests page" ratio="16/9"></media-frame>

When a client exceeds its limit, `rate_limited_response()` returns **HTTP 429** with rate-limit headers. The body format depends on the `Accept` header — JSON for API clients, a small inline HTML page for browsers. The middleware runs before `set_lang` and the template engine, so the 429 path has no template dependency.

Headers are sent **only on 429 responses** (not on allowed requests):

| Header                  | Meaning                             |
| ----------------------- | ----------------------------------- |
| `Retry-After`           | Seconds until the client may retry  |
| `X-RateLimit-Limit`     | The tier's `max`                    |
| `X-RateLimit-Remaining` | Always `0` on a 429                 |
| `X-RateLimit-Reset`     | Epoch second when the window resets |

<a name="testing"></a>

## Testing

`rate_limit_mw()` accepts an optional `RedisClient` for dependency injection, and the `RedisClient` interface (`incr`, `expire`, `get`) is exported from the middleware. This lets the test suite exercise the sliding-window logic with a fake client — no real Redis and no mocking of Bun's built-in `redis`:

```ts
import { rate_limit_mw, type RedisClient } from "$lib/middleware/rate_limit";

const fake: RedisClient = {
	/* incr / expire / get */
};
const mw = rate_limit_mw(fake);
```

> **Redis client note:** Reepolee uses Bun's built-in Redis — `import { redis } from "bun"` for the default client (reads `REDIS_URL`) or `import { RedisClient } from "bun"` for explicit connections. Never reach for `Bun.RedisClient`.
