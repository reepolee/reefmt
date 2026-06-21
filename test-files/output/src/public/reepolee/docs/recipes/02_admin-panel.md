---
title: "Building an Admin Panel"
---

# Building an Admin Panel

<a name="introduction"></a>

## Introduction

Most business applications have two faces: the public side that users interact with and an admin side that's hidden behind authentication. Reepolee's pattern for the admin side is `mount_prefix("/admin", ...)` plus `require_module_mw("admin")` — every admin URL gets the `/admin` prefix and every admin route requires the `admin` module in one go.

This recipe covers the full admin panel pattern: assigning the `admin` module to users, generating admin-prefixed CRUD modules, building an admin-specific layout, and wiring the navigation. By the end you'll have a working admin section with a couple of resources and a separate look that's clearly distinct from the public site.

<a name="step-1-assign-the-admin-module"></a>

## Step 1: Assign the admin Module

Users have a `modules_tags` column — a comma-separated string of module codes from the `modules` table. The convention is `"user"` for everyone and `"user,admin"` for admins. Anyone whose `modules_tags` contains `admin` can reach `/admin/*` routes.

If you're starting from a fresh project, the seeded admin already has both modules. Confirm by checking the database:

```bash
sqlite3 app.db 'SELECT id, email, modules_tags FROM users'
```

To promote an existing user, update their modules_tags directly:

```sql
UPDATE users SET modules_tags = 'user,admin' WHERE email = 'someone@example.com';
```

For a user-facing way to assign modules to others, see [Authorization → Modules](/security/authorization#modules) — the simplest pattern is letting an existing admin edit the `modules_tags` field on any user from a `/admin/users` form.

<a name="step-2-generate-an-admin-resource"></a>

## Step 2: Generate an Admin Resource

The generator's `--prefix` flag scopes a resource under a sub-path. To put `books` (from the previous recipe) under `/admin`, use `crud.ts` directly:

```bash
bun generator/crud books --prefix admin
```

The generator writes files to `routes/admin/books/` (rather than `routes/books/`) and updates `routes/routes.ts` with a `mount_prefix` call:

```ts
import { mount_prefix, require_module_mw } from "$lib/middleware";
import { admin_books_crud } from "$routes/admin/books";

export const routes = {
	"/": home_page,
	// ...
	...mount_prefix("/admin", admin_books_crud, require_module_mw("admin")),
};
```

Three things to notice:

- **The folder structure mirrors the URL.** `routes/admin/books/` corresponds to `/admin/books/*`. Future generators with the same prefix nest into the same folder.
- **`require_module_mw("admin")` wraps every handler.** Anyone without the `admin` module is redirected to login or returned 403. See [Authorization](/security/authorization#middleware-guards).
- **The export is named `admin_books_crud`** rather than `books_crud`, so it doesn't collide with a non-admin `books` module if you have one.

Visit `/admin/books` after restarting and confirm:

- **Logged out**: redirect to `/login`.
- **Logged in as a non-admin** (`modules_tags = "user"`): `403 Forbidden`.
- **Logged in as an admin**: the list page renders.

<a name="step-3-add-more-admin-resources"></a>

## Step 3: Generate Multiple Admin Resources at Once

Generating one resource at a time gets repetitive. The shorter pattern: spread multiple admin resources into a single `mount_prefix` call:

```ts
import { mount_prefix, require_module_mw } from "$lib/middleware";
import { admin_books_crud } from "$routes/admin/books";
import { admin_authors_crud } from "$routes/admin/authors";
import { admin_settings_crud } from "$routes/admin/settings";

export const routes = {
	"/": home_page,
	// ...
	...mount_prefix(
		"/admin",
		{
			...admin_books_crud,
			...admin_authors_crud,
			...admin_settings_crud,
		},
		require_module_mw("admin"),
	),
};
```

Each generated module exports its own `<name>_crud` object. Spreading them into a single object before passing to `mount_prefix` keeps the route table compact and applies the auth check to all of them at once.

To generate a new admin resource, re-run with the prefix flag:

```bash
bun generator/crud authors --prefix admin
```

The generator writes the new files, adds the import line, and your existing `mount_prefix` call expands to cover the new module.

<a name="step-4-an-admin-layout"></a>

## Step 4: A Separate Admin Layout

<media-frame label="Screenshot — admin panel with the dark header layout" ratio="16/9"></media-frame>

The shipped `routes/layout.ree` is the public layout. For an admin section that should look distinct — different navigation, an "ADMIN" badge in the header, a more dense interface — create a second layout:

```html
<!-- views/admin-layout.ree (or routes/admin/layout.ree, your choice) -->
<!DOCTYPE html>
<html lang="{= props.lang }">
	<head>
		<meta charset="UTF-8" />
		<title>Admin · {= props.ui.title }</title>
		<link rel="stylesheet" href='/app{~ props.is_dev ? "-dev" : "" }.css?v={= props.version }' />
		<script src="/web-components/validation-error.js" defer></script>
		<script src="/web-components/toasts-area.js" defer></script>
	</head>
	<body class="bg-slate-50">
		<header class="bg-slate-900 text-white px-6 py-3 flex items-center gap-4">
			<span class="font-bold">Admin</span>
			<nav class="flex gap-3 text-sm">
				<a href="/admin/books" class="{= is_current('/admin/books') }">Books</a>
				<a href="/admin/authors" class="{= is_current('/admin/authors') }">Authors</a>
				<a href="/admin/settings" class="{= is_current('/admin/settings') }">Settings</a>
			</nav>
			<div class="ml-auto text-sm">
				{= props.user.display_name }
				<form method="POST" action="/logout" class="inline">
					<button class="ml-2 underline">Log out</button>
				</form>
			</div>
		</header>

		<main class="px-6 py-8">{~ props.body }</main>

		<toasts-area></toasts-area>
	</body>
</html>
```

Then in each admin template, reference the admin layout instead of the default:

```html
<!-- routes/admin/books/index.ree -->
{#layout('admin-layout') }

<h1 class="text-2xl font-bold mb-4">{= props.ui.title }</h1>
<!-- ... rest of the list -->
```

Or — if you'd prefer not to edit every generated template — wrap the `render()` call in a small helper:

```ts
// routes/admin/render.ts
import { render } from "$lib/render";
import type { RenderOptions } from "$lib/render";
import type { BunRequest } from "bun";

export function admin_render(template: string, options: RenderOptions): Promise<Response> {
	return render(template, {
		...options,
		data: { ...(options.data ?? {}), layout: "admin-layout" },
	});
}
```

Then in the templates, reference `props.layout` dynamically:

```html
{#layout(props.layout ?? 'layout') }
```

The handler-side `admin_render` overrides `props.layout`; everything else uses the default. You don't have to touch every template.

<a name="step-5-non-crud-admin-pages"></a>

## Step 5: Adding Non-CRUD Admin Pages

Most admin sections need pages that aren't a CRUD module — a dashboard, a settings overview, a system status page. Add them as plain handlers in `routes/admin/`:

```ts
// routes/admin/dashboard/index.ts
import { resolve_session, require_module } from "$routes/system/auth/middleware";
import { translated_from_request } from "$lib/helpers";
import { render } from "$lib/render";
import { create_ctx } from "$lib/request_context";
import type { BunRequest } from "bun";

import { count_books } from "$routes/admin/books/sql";
import { count_active_users } from "$routes/admin/users/sql";

export async function get_admin_dashboard(req: BunRequest): Promise<Response> {
	const ctx = await create_ctx(req);
	const auth_ctx = await resolve_session(req);
	const guard = require_module(auth_ctx, "admin");
	if (guard) return guard;

	const translated = await translated_from_request(req, import.meta.dir);

	return render("admin/dashboard/index", {
		data: {
			ui: { title: "Admin Dashboard" },
			stats: {
				books: await count_books(),
				users: await count_active_users(),
			},
			...translated,
		},
		ctx,
	});
}

export const dashboard_routes = {
	"/dashboard": get_admin_dashboard,
};
```

Then add it to the admin section in `routes.ts`:

```ts
import { dashboard_routes as admin_dashboard_routes } from "$routes/admin/dashboard";

export const routes = {
	// ...
	...mount_prefix(
		"/admin",
		{
			...admin_dashboard_routes,
			...admin_books_crud,
			...admin_authors_crud,
		},
		require_module_mw("admin"),
	),
};
```

Visit `/admin/dashboard` — the function-form `require_tag` _and_ the middleware `require_module_mw` both run, so the check is doubled but harmless. For purely non-CRUD pages, you can drop the in-handler guard and rely on the middleware alone:

```ts
export async function get_admin_dashboard(req: BunRequest): Promise<Response> {
	// No resolve_session / require_tag — middleware handles it
	const translated = await translated_from_request(req, import.meta.dir);
	// ... but you also lose access to auth_ctx.current_user here
}
```

The trade-off is what's covered in [Authorization → Which Form to Use](/security/authorization#which-form-to-use). For dashboards that need `current_user` (showing the admin's name in the page title), the function-form guard is worth keeping.

<a name="step-6-admin-only-tables"></a>

## Step 6: Tables That Should Only Exist in Admin

A `users` table is one example — there's no public `/users` page, but there's an `/admin/users` for admins to manage accounts. The generator handles this directly: pass `--prefix admin` and only the admin module exists.

For tables that _should_ exist publicly _and_ in admin (a `posts` table where the public sees published posts and admins manage all posts), generate both:

```bash
bun generator/resource posts                # public, at /posts
bun generator/resource posts --prefix admin # admin, at /admin/posts
```

The two modules share the database table and the `sql.ts` patterns but are separate route folders. Customising one doesn't affect the other — useful when the public and admin views diverge significantly.

<a name="step-7-fine-grained-modules"></a>

## Step 7: Beyond a Single "admin" Module

The `admin` module is a single bit — either you can do everything or you can do nothing. For larger applications you'll want finer permissions: a `billing` module for the billing section, an `editor` module for content management, an `analytics` module for the dashboard. Add new module codes to the `modules` table and assign them to users via `modules_tags`.

Each `mount_prefix` can use a different module check:

```ts
export const routes = {
	"/": home_page,
	// ...
	...mount_prefix("/admin", admin_general_routes, require_module_mw("admin")),
	...mount_prefix("/billing", billing_crud, require_module_mw("billing")),
	...mount_prefix("/content", content_crud, require_module_mw("editor")),
	...mount_prefix("/analytics", analytics_routes, require_module_mw("analytics")),
};
```

A user with `modules_tags = "user,billing,editor"` reaches `/billing/*` and `/content/*` but not `/admin/*` or `/analytics/*`. Each section is gated independently.

For role hierarchies (an `admin` who can do everything plus their own modules), check both in your handler or compose middleware:

```ts
import { with_middleware, require_module_mw } from "$lib/middleware";

const require_admin_or_billing = (handler: Handler) =>
	with_middleware(handler, async (req, next) => {
		const auth_ctx = await resolve_session(req);
		const modules = (auth_ctx.current_user?.modules_tags ?? "").split(/[\s,]+/).filter(Boolean);
		if (modules.some((m) => ["admin", "billing"].includes(m))) {
			return next(req);
		}
		return new Response("Forbidden", { status: 403 });
	});
```

Custom middleware composes the same way as the built-ins. For a project with three or four roles, this stays manageable; for many roles, a `user_roles` table and a real RBAC layer is the next step.

<a name="whats-next"></a>

## What's Next

You have a working admin panel with auth, prefixed routes, a separate layout, and the start of a fine-grained permissions story. Reading from here:

- **[Authorization](/security/authorization)** — the full reference for `require_module_mw`, `require_auth`, and writing custom middleware.
- **[Generators](/database/generators)** — every flag and convention the generator supports.
- **[Custom Form Components](/recipes/custom-form-components)** — when the standard inputs aren't enough for your admin forms.
- **[Adding a New Language](/recipes/new-language)** — translating the admin alongside the public side.
