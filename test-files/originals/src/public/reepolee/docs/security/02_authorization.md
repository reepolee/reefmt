---
title: "Authorization"
---

# Authorization

<a name="introduction"></a>

## Introduction

Authentication is "is this user logged in?". Authorization is "can this user do this?". Reepolee handles authorization the same way it handles most things — with small functions you compose in your route table. There are two flavours: **middleware guards** (recommended for protecting whole routes and sections) and **function guards** (when the handler needs to read the session itself).

The model is intentionally narrow: every user has a `modules_tags` field — a comma-separated string of module codes — and protected routes check whether a specific module is present. The modules table is seeded with `user`, `admin`, `system`, `images`, and `examples`, but you can add any you need.

<a name="middleware-guards"></a>

## Middleware Guards

`require_auth_mw()` and `require_module_mw(tag)` from `$lib/middleware` are proper middleware functions. They compose with `with_middleware()` and `mount_prefix()`, which makes them the right choice for protecting individual routes or entire sections:

```ts
import { with_middleware, mount_prefix, require_auth_mw, require_module_mw } from "$lib/middleware";

export const routes = {
	"/profile": with_middleware(profile_page, require_auth_mw()),
	"/settings": with_middleware(settings_page, require_auth_mw()),
	...mount_prefix("/admin", admin_crud, require_module_mw("admin")),
};
```

What each middleware does:

- **`require_auth_mw()`** — redirects unauthenticated requests to the localized `/login` URL with a `?redirect=` back to the original location.
- **`require_module_mw(module_code)`** — first runs the auth check. If the user's `modules_tags` includes `module_code`, the handler runs; if not, returns `403 Forbidden`.

Both middlewares also re-emit the `X-Lang-Preferred` header from the language cookie so downstream `render()` calls can detect language mismatches (the dialog that appears when a user lands on a Slovenian URL but has English as their preferred language). See [Localized Routes](/i18n/localized-routes).

For an admin section that's also auth-gated, `require_module_mw("admin")` is enough — the module check implicitly requires authentication.

<a name="function-guards"></a>

## Function Guards

When your handler needs the session object itself — to pass `current_user` into a template, to look up something tied to the user, to make a decision based on session data — use the function-form guards along with `resolve_session()`:

```ts
import { resolve_session, require_auth } from "$routes/system/auth/middleware";

export async function get_profile(req: BunRequest): Promise<Response> {
	const ctx = await create_ctx(req);
	const auth_ctx = await resolve_session(req);
	const guard = require_auth(auth_ctx, req);
	if (guard) return guard;

	// auth_ctx.current_user is guaranteed non-null here
	return render("auth/profile/form", {
		data: { ...auth_ctx.current_user },
		ctx,
	});
}
```

The pattern is always the same:

1. Resolve the session.
2. Call the guard. If it returns a `Response`, return that immediately — the user's not authenticated (or not authorized).
3. Otherwise the guard returns `null` and you proceed with full access to the session.

`require_module(auth_ctx, module_code)` works the same way but also verifies the user has the module:

```ts
const guard = require_module(auth_ctx, "admin");
if (guard) return guard;
```

`require_module` calls `require_auth` first, so it both redirects unauthenticated users to login _and_ returns 403 for authenticated users without the module.

<a name="which-form-to-use"></a>

## Which Form to Use

A rough rule:

- **Use middleware guards** when the handler doesn't need to read the session. Most CRUD pages, most admin routes, anywhere "if you're not authed, you can't be here" is the only check.
- **Use function guards** when the handler needs `current_user` (or `session_id`, or `session`) for its own logic. Profile edits, password changes, "show me my own records," and similar.

The middleware form is more declarative — the protection is visible in the route table without reading the handler. The function form is more flexible because the handler has the session object in hand and can pass it onward.

You can mix the two: middleware guards the route, the handler still calls `resolve_session()` to read the session. Both are fast (a single SELECT against the sessions table); calling `resolve_session()` twice in one request is rarely noticeable.

<a name="modules"></a>

## Modules

A module is a code attached to a user. Reepolee stores user-to-module assignments as a comma-separated string in `users.modules_tags` — e.g. `"user"`, `"user,admin"`, `"user,editor,billing"`. The two checks (`require_module_mw` and `require_module`) both split that string on commas and look for an exact match.

The set of valid module codes lives in the `modules` table, seeded with `default`, `user`, `system`, `admin`, `images`, `examples`. `lib/modules.ts` loads them on startup into a cached list — `get_available_prefixes()` exposes the result for URL-prefix detection (the `RequestContext.prefix` field is set by matching the URL against this list). Add a row to `modules` and re-run the SQL when you need a new one.

```sql
-- in sql/sqlite/01-init-sqlite.sql
INSERT INTO modules (code, name) VALUES ('billing', 'Billing');

-- and on individual users
INSERT INTO users (id, email, modules_tags) VALUES
    (1, 'admin@example.com', 'user,admin'),
    (2, 'editor@example.com', 'user,editor'),
    (3, 'normal@example.com', 'user');
```

The two conventions worth following:

- **Everyone has `user`.** Tagging every account with `user` lets you use `require_module_mw("user")` as a slightly stronger "authenticated and a regular user" check (versus `require_auth_mw()`, which accepts any session). In practice the distinction rarely matters and `require_auth_mw()` is enough for most routes.
- **Adding modules to a user is just editing the string.** No migration, no separate join table. For the rare project that needs role hierarchies or per-feature permissions, a separate `user_roles` table is the next step — but most applications get far on modules alone.

Module assignments are stored once in the session payload, so the check is in-memory after `resolve_session()` — no extra database query per request.

<a name="building-an-admin-section"></a>

## Building an Admin Section

The canonical pattern for an admin area is `mount_prefix` with `require_module_mw("admin")`:

```ts
import { mount_prefix, require_module_mw } from "$lib/middleware";
import { admin_users_crud, admin_email_crud, admin_settings_crud } from "$routes/admin";

export const routes = {
	"/": home_page,
	...auth_crud,
	...mount_prefix(
		"/admin",
		{
			...admin_users_crud,
			...admin_email_crud,
			...admin_settings_crud,
		},
		require_module_mw("admin"),
	),
};
```

Every URL inside is prefixed with `/admin` and every handler is wrapped with the tag check. A handler that lives in `routes/admin/users/index.ts` registered as `"/users"` becomes `/admin/users` in the route table, and visitors without the `admin` tag get a `403` before the handler runs.

The generator can produce admin-scoped resources directly. `bun generator/crud users --prefix admin` writes files to `routes/admin/users/` and updates `routes/routes.ts` with the right `mount_prefix` call — see [Generators](/database/generators#prefixed-routes) and the [Building an Admin Panel](/recipes/admin-panel) recipe.

<a name="custom-checks"></a>

## Custom Authorization Logic

For checks beyond "is this tag present?" — "does this user own this record?", "is this user in the same organisation?", "is this feature flag enabled for this account?" — write the check inline in the handler. There's no decorator system to learn:

```ts
export async function get_invoice_edit(req: BunRequest): Promise<Response> {
	const ctx = await create_ctx(req);
	const auth_ctx = await resolve_session(req);
	const guard = require_auth(auth_ctx, req);
	if (guard) return guard;

	const invoice = await get_invoice_by_id(req.params.id);
	if (!invoice || invoice.user_id !== auth_ctx.session!.user_id) {
		return render("notfound", {
			data: { title: "Not found", ...translated },
			status: 404,
			ctx,
		});
	}

	// ... render the edit form
}
```

The 404 (rather than 403) is intentional — telling someone "this exists but you can't see it" is a small information leak. For internal admin tools the 403 is fine; for public-facing apps, 404 on access-denied is the safer default.

<a name="global-scopes"></a>

## Row-Level Scopes (Global Scopes)

The tag model above controls _which routes_ a user can reach. **Global scopes** control _which rows_ a list query returns — a data-driven way to add a reusable `WHERE` filter to a table's CRUD index without editing the generated SQL.

A scope is a named, stored `WHERE` clause attached to a table. They live in the `global_scopes` table and are managed through the system module at `/system/global_scopes`. Each scope has:

| Column         | Purpose                                               |
| -------------- | ----------------------------------------------------- |
| `table_name`   | The table the scope applies to                        |
| `scope_key`    | Stable identifier passed in the URL (`?scope=active`) |
| `display_name` | Human label shown in the scope selector               |
| `where_clause` | The SQL fragment appended to the list query           |
| `is_default`   | Whether this scope is selected by default             |
| `sort_order`   | Order in the selector dropdown                        |

The helpers in `$lib/global_scopes` are what generated routes call:

```ts
import { get_global_scopes, get_scope_clause } from "$lib/global_scopes";

const scopes = await get_global_scopes("orders"); // for the selector UI
const where = await get_scope_clause("orders", scope_key); // the WHERE fragment
```

`get_scope_clause()` returns an empty string for an unknown or empty `scope_key`, so an absent scope simply means "no extra filter." The selected `scope` is also part of the [cache key](/database/caching), so cached list results never bleed across scopes.

Because the clause is stored as raw SQL, scopes are an **admin-authored** feature — only trusted operators should have access to `/system/global_scopes`. Treat the `where_clause` value as you would any other privileged SQL: it is appended to queries verbatim. Use scopes for operator-defined views ("active only", "this region", "unpaid"), not as a substitute for the per-user ownership checks shown above.

<a name="when-the-user-changes"></a>

## When the User Changes

Some user changes need the session to refresh — a renamed nickname should appear in the layout immediately, a tag added by an admin should grant access without forcing a logout. Two patterns:

**Refreshing the current user's session in the same handler** — call `refresh_session(session_id, partial)` after the database write so the next page load sees the new values:

```ts
await update_user_profile(user_id, { nickname: new_nickname });
await refresh_session(auth_ctx.session_id!, { nickname: new_nickname });
```

**Forcing a re-login** — for changes that should invalidate the user's session entirely (a password change, a security-sensitive operation), destroy the session and let the user log in again:

```ts
await destroy_session(auth_ctx.session_id!);
return Response.redirect("/login", 303);
```

For password changes specifically, Reepolee's default is to keep the user logged in but refresh the session cookie with a new UUID — see [Profile & Password](/security/profile-and-password).
