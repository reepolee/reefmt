---
title: "Caching"
---

# Caching

<a name="introduction"></a>

## Introduction

Reepolee can cache the results of CRUD list queries (`search_records`) in Redis, keyed by the exact query parameters and invalidated automatically whenever the underlying table changes. The cache is **off by default** and **opt-in** — it only activates when `CACHE_ENABLED=true` and a `REDIS_URL` is configured. With it off, the cache layer is a transparent no-op: every call runs the query directly.

The implementation lives in `$lib/cache`. It is deliberately small — one `search()` wrapper, one `invalidate()` call, plus a few admin helpers. There is no per-route configuration, no manual key management, and no stale-data risk from forgetting to clear a key: writes invalidate the cache by table dependency.

<a name="enabling"></a>

## Enabling the Cache

Two environment variables control it:

```env
CACHE_ENABLED=true
REDIS_URL=redis://localhost:6379
```

The guard is **fail-loud**: if `CACHE_ENABLED=true` but `REDIS_URL` is unset, the process exits at startup with an error rather than silently running without a cache. When `CACHE_ENABLED` is unset or `false`, `cache.search()` simply calls the query function and `cache.invalidate()` does nothing — so generated routes that call into the cache work identically whether or not caching is turned on.

<a name="how-it-works"></a>

## How It Works

Generated CRUD routes already wire the cache into their `search_records` path. Each generated `sql.ts` exports two constants the cache relies on:

- **`TABLE_NAME`** — the table this route owns, used as the invalidation key.
- **`VIEW_DEPENDENCIES`** — every table a list query reads from (the table itself plus any joined foreign-key tables).

The caching is wired **inside the generated `search_records()` function** — the route handler just calls `search_records(...)` as usual and gets caching for free. Inside `search_records`, the query is wrapped in `timed_query()` and then `cache.search()`:

```ts
// inside the generated routes/<table>/sql.ts
return await timed_query(TABLE_NAME, "search_records", async () =>
	cache.search(
		"products", // the route key
		{ search, after, before, is_last, limit, order_by, scope_clause },
		VIEW_DEPENDENCIES, // e.g. ["products", "categories"]
		async () => {
			// ...the actual cursor-paginated SQL...
		},
	),
);
```

`cache.search()`:

1. Builds a cache key from the route and the sorted, non-empty params:
   `sql:cache:{route}:after:{...}:limit:{...}:order_by:{...}:search:{...}`. The key is plain concatenation, not a hash — it is human-readable and debuggable in `redis-cli`.
2. Returns the cached JSON on a hit.
3. On a miss, runs the query, stores the result with a 5-minute TTL (`DEFAULT_TTL_S = 300`), and registers the cache key in a Redis SET per dependency table (`sql:deps:{table}`).

If Redis errors at any point, `cache.search()` logs a warning and falls back to running the query directly — a Redis outage degrades to "no cache," never to an error.

<a name="invalidation"></a>

## Invalidation

Caching read queries is easy; the hard part is keeping them fresh. Reepolee uses **dependency-set invalidation**: every cached search registers itself under each table it depends on. When a write happens, every cached entry depending on that table is dropped.

Generated create, update, delete, and bulk-delete handlers call `cache.invalidate(TABLE_NAME)` after the mutation succeeds:

```ts
await cache.invalidate(TABLE_NAME);
```

`invalidate()` reads the `sql:deps:{table}` SET, deletes every cache key in it, then deletes the SET itself. Because a list query that joins `products` and `categories` registers under **both** `sql:deps:products` and `sql:deps:categories`, editing a category correctly busts the product list that displayed its name.

<a name="admin-ui"></a>

## The Cache Admin Module

<media-frame label="Screenshot — cache admin module" ratio="16/9"></media-frame>

The system module at `/system/cache` (route `routes/system/cache/`) gives a live view of the cache:

- Whether caching is enabled (`cache.is_enabled()`).
- Total `sql:cache:*` keys and `sql:deps:*` dependency sets (`cache.get_status()`).
- A per-table breakdown of how many cached entries each table holds.
- A button to flush everything (`cache.invalidate_all()`), which deletes every `sql:*` key and returns the count removed.

This is useful for confirming the cache is warming up as expected and for manually clearing it after a bulk data import that bypassed the normal write handlers.

<a name="tuning"></a>

## Tuning

The TTL is a single constant in `$lib/cache` (`DEFAULT_TTL_S`, 300 seconds). It is a safety net, not the primary freshness mechanism — invalidation keeps data correct, and the TTL only bounds how long an orphaned entry (one whose invalidation somehow failed) can live. Raise it if your data is mostly read-only; lower it if you want a tighter ceiling.

The cache only wraps generated CRUD `search_records` queries. Hand-written queries and single-record lookups are not cached — if you want to cache those, call `cache.search()` yourself with an appropriate route key and dependency list, and add a matching `cache.invalidate()` to whatever writes that data.

> **Note:** The legacy `routes/system/users` route uses offset pagination and is **not** cache-integrated — it has no `VIEW_DEPENDENCIES` export and runs its query directly. Treat it as a known exception, not a template.
