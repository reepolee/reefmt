---
title: "Module System"
---

# Module System

<a name="introduction"></a>

## Introduction

Reepolee's module system lets you group routes under URL prefixes that are gated by user permissions. A typical use is an admin panel at `/admin/` — only users with the `admin` module in their `modules_tags` field can see it. The modules themselves are stored in a database table, so adding or removing a module is a row insert, not a code change.

The system has three parts:

- A **`modules` database table** that stores the available module codes.
- A **URL prefix resolution** step that detects which module the current request belongs to.
- A **middleware guard** (`require_module_mw()`) that checks the user's permissions before the handler runs.

Routes are organised using `mount_prefix()` from `$lib/middleware`, which scopes a route table under a prefix and optionally wraps each handler with module-guarding middleware.

<a name="the-modules-table"></a>

## The Modules Table

Modules are defined in the `modules` database table:

```sql
CREATE TABLE modules (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    code TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    description TEXT DEFAULT '' NULL,
    created_at DATETIME DEFAULT current_timestamp
);
```

Each row is one module. The `code` column is what you reference in middleware — it must be lowercase. The `name` column is the human-readable label.

The shipped seed data includes the `default` module (always accessible, no prefix) and any project-specific modules you add:

```sql
INSERT INTO modules (code, name) VALUES
    ('default', 'Default'),
    ('admin', 'Admin Panel'),
    ('system', 'System Settings');
```

The `default` module has special meaning — it's the catch-all for routes that aren't under any prefix. Every user implicitly has access to the default namespace.

<a name="loading-modules-at-startup"></a>

## Loading Modules at Startup

At server startup, `lib/modules.ts` loads module codes from the database:

```ts
import { load_modules } from "$lib/modules";

// In server.ts during initialisation
await load_modules();
```

`load_modules()` reads all non-default module codes from the `modules` table and caches them in memory for the lifetime of the process. If the database is unavailable, it logs an error and sets the cache to an empty array — effectively disabling all module-prefixed routes.

The cached prefixes are then available via `get_available_prefixes()`:

```ts
import { get_available_prefixes } from "$lib/modules";

const prefixes = get_available_prefixes();
// ["admin", "system"]
```

This is used internally by `create_ctx()` to detect which module prefix the current request URL matches.

<a name="defining-routes-with-prefixes"></a>

## Defining Routes With Prefixes

`mount_prefix()` from `$lib/middleware` scopes a route table under a URL prefix and optionally wraps each handler with middlewares:

```ts
import { mount_prefix, require_module_mw } from "$lib/middleware";
import { admin_users_crud } from "$routes/admin/users";

export const routes = {
	"/": home_page,
	...mount_prefix(
		"/admin",
		{
			...admin_users_crud,
		},
		require_module_mw("admin"),
	),
};
```

The second argument to `mount_prefix` is a `RouteTable` — the same shape as the top-level `routes` export. The third argument (and beyond) are middlewares applied to every handler in the table.

`require_module_mw("admin")` does two things:

1. Checks for a valid authenticated session. If none exists, redirects to `/login`.
2. Checks that the user's `modules_tags` field contains the required module code (e.g., `"admin"`). If not, returns a `403 Forbidden`.

Users' module assignments are stored as a comma-separated string in the `users.modules_tags` column:

```sql
-- A user with admin and system access
UPDATE users SET modules_tags = 'admin,system' WHERE id = 1;
```

The field is parsed by splitting on commas and whitespace, so `"admin, system"` and `"admin, system"` both work.

<a name="the-route-builder-pattern"></a>

## The Route Builder Pattern

For larger applications with many module-routed resources, `lib/route_builder.ts` provides a declarative pattern:

```ts
import { build_routes, build_nav_routes } from "$lib/route_builder";

const user_routes: RouteDef[] = [
	{
		url: "/admin/users",
		crud: admin_users_crud,
		module: "admin",
		nav_title_key: "users",
	},
	{
		url: "/admin/partners",
		crud: admin_partners_crud,
		module: "admin",
		nav_title_key: "partners",
	},
	{
		url: "/api/public",
		handler: api_handler,
		module: null, // no auth required
	},
];

export const routes = build_routes(user_routes);
export const nav_routes = build_nav_routes(user_routes.filter((r) => r.module !== null));
```

Each `RouteDef` can specify:

| Field           | Purpose                                                          |
| --------------- | ---------------------------------------------------------------- |
| `url`           | The URL path (e.g. `/admin/users`)                               |
| `handler`       | A single handler function                                        |
| `methods`       | A method map (`{ GET: ..., POST: ... }`)                         |
| `resource`      | A pre-built route table to merge in                              |
| `crud`          | A pre-built CRUD route table (mounted via `mount_prefix`)        |
| `module`        | The module code for middleware guarding (`null` = no guard)      |
| `nav_title_key` | Translation key for the navigation menu entry                    |
| `is_menu_entry` | Whether to include this route in the navigation (default `true`) |

When `crud` is provided, `build_routes()` automatically wraps it with `mount_prefix()` using the URL's parent path as the prefix and the specified module's middleware. So `/admin/users` with `crud: admin_users_crud` and `module: "admin"` becomes a `mount_prefix("/admin", admin_users_crud, require_module_mw("admin"))` entry.

<a name="url-prefix-detection"></a>

## URL Prefix Detection at Request Time

When `create_ctx()` processes a request, it checks the URL path against the loaded module prefixes:

```ts
const ctx = await create_ctx(req);
// ctx.prefix → "admin" for /admin/users, null for /login
```

The `ctx.prefix` value is available in templates as `props.prefix`. This lets you conditionally render UI elements based on which module section the user is in:

```html
{#if props.prefix === "admin" }
<nav class="admin-sidebar">...</nav>
{/if }
```

<a name="generating-module-routes"></a>

## Generating Module Routes

The resource generator supports the `--prefix` flag for creating routes under a module:

```bash
bun generator/crud users --prefix admin
```

This writes files to `routes/admin/users/` and updates `routes/routes.ts` with the correct `mount_prefix` and `require_module_mw("admin")` wrapper. See [Generators](/database/generators#prefixed-routes) for the full command reference.

<a name="adding-a-new-module"></a>

## Adding a New Module

Adding a new section to your application:

1. Insert a row into the `modules` table:

```sql
INSERT INTO modules (code, name) VALUES ('analytics', 'Analytics Dashboard');
```

2. Restart the server so `load_modules()` picks up the new code.
3. Create route handlers under `routes/analytics/`.
4. Register the routes with `mount_prefix("/analytics", ..., require_module_mw("analytics"))`.
5. Assign the module to users via their `modules_tags` field.
