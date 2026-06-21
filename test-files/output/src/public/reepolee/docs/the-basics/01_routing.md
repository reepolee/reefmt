---
title: "Routing"
---

# Routing

<a name="introduction"></a>

## Introduction

Reepolee routes are defined declaratively in `routes/routes.ts` using `RouteDef` objects. Each definition includes a URL pattern, a handler function, and optional metadata for navigation labels, module gating, and localisation. The route table is assembled by `build_routes()` and handed to Bun's built-in router at startup.

Any path that does not match a registered route falls through to the `fetch` function in `server.ts`, which serves static files from `static/` and renders the 404 page. Customising that fallback is covered in [Error Handling](/the-basics/error-handling).

<a name="defining-routes"></a>

## Defining Routes

A route handler is a function that receives a `BunRequest` and returns a `Response`. Assign one directly to a path to handle `GET` requests:

```ts
// Minimal example. The real top-level route table uses the declarative
// `build_routes()` helper, which is covered further down on this page.
export const routes = {
	"/": home_page,
	"/about": about_page,
};
```

When a path needs to respond to multiple HTTP methods, use a method map instead:

```ts
export const routes = {
	"/users": {
		GET: get_users_index,
		POST: post_users_index,
	},
	"/users/new": get_users_new,
	"/users/validate": {
		POST: post_users_validate,
	},
};
```

Every handler follows the same shape:

```ts
export async function home_page(req: BunRequest): Promise<Response> {
	const ctx = await create_ctx(req);
	const translated = await translated_from_request(req, import.meta.dir);
	return render("home/home", { data: { title: translated.ui.title, ...translated }, ctx });
}
```

<a name="url-parameters"></a>

## URL Parameters

Use `:param` segments in the path to capture dynamic values. Bun makes them available on `req.params`:

```ts
"/users/:id/edit": {
    GET: get_users_edit,
    POST: post_users_edit,
},
```

```ts
export async function get_users_edit(req: BunRequest<"/users/:id/edit">): Promise<Response> {
	const ctx = await create_ctx(req);
	const id = req.params.id;
	const record = await get_record_by_id(id);
	return render("users/form", { data: { record }, ctx });
}
```

Reading query strings, form bodies, and JSON payloads is covered in [Requests & Responses](/the-basics/requests-responses).

<a name="crud-modules"></a>

## CRUD Modules

A CRUD module is just an exported object that holds several related routes together. You spread it into the main route table, which keeps `routes.ts` concise even as your application grows:

```ts
// routes/users/index.ts
export const users_crud = {
	"/users": {
		GET: get_users_index,
		POST: post_users_index,
	},
	"/users/new": get_users_new,
	"/users/validate": {
		POST: post_users_validate,
	},
	"/users/:id/edit": {
		GET: get_users_edit,
		POST: post_users_edit,
	},
};
```

```ts
// routes/routes.ts
import { users_crud } from "$routes/users";
import { auth_crud } from "$routes/system/auth";

export const routes = {
	"/": home_page,
	...users_crud,
	...auth_crud,
};
```

The generator produces this structure automatically when you scaffold a new resource — see [Generators](/database/generators).

<a name="declarative-route-defs"></a>

## Declarative Route Definitions

The top-level `routes/routes.ts` uses a small helper layer — `build_routes()` and `build_nav_routes()` from `$lib/route_builder` — that lets you describe each route as a `RouteDef` object instead of a bare key/value pair. The payoff is that menu entries, module-based access control, and `mount_prefix` wiring all come from the same array:

```ts
import { build_nav_routes, build_routes, type RouteDef } from "$lib/route_builder";
import { about_page } from "$routes/examples/about";
import { open_free_map_page } from "$routes/examples/open_free_map";
import { authors_crud } from "$routes/authors";
import { home_page } from "$routes/home";
import { auth_crud } from "$routes/system/auth";
import { system_users_crud } from "$routes/system/users";

const route_defs: RouteDef[] = [
	{ url: "/", handler: home_page },
	{ url: "/examples/about", handler: about_page, nav_title_key: "about", module: "examples" },
	{ url: "/examples/open_free_map", handler: open_free_map_page, nav_title_key: "open_free_map", module: "examples" },
	{ url: "/system/users", crud: system_users_crud, nav_title_key: "system.users", module: "system" },
	{ url: "/authors", crud: authors_crud, nav_title_key: "authors" },
];

export const nav_routes = build_nav_routes(route_defs);

export const routes = {
	...build_routes(route_defs),
	...auth_crud,
};
```

Each `RouteDef` chooses one of four shapes:

| Field      | Used when                                                                                               |
| ---------- | ------------------------------------------------------------------------------------------------------- |
| `handler`  | A single function answers `GET` for this URL                                                            |
| `methods`  | A `{ GET, POST, ... }` map answers multiple methods                                                     |
| `resource` | An imported route table (e.g. `email_page`) is spread in as-is                                          |
| `crud`     | A generated CRUD module is mounted under a prefix derived from `url` and optionally guarded by `module` |

The extra fields drive everything else:

- **`nav_title_key`** — the translation key for this route's navigation label. `build_nav_routes()` filters to entries with this key set, so the nav menu and the routes stay in sync from a single source.
- **`module`** — gates the route behind `require_module_mw(module)`. For `crud` defs this is applied to every URL the CRUD table mounts; for nav entries it groups the link under that module in the menu. Available modules are loaded at startup from the `modules` table — see [Authorization](/security/authorization).
- **`is_menu_entry`** — set to `false` to keep `nav_title_key` for translation lookup but suppress the nav entry.

`server.ts` reads `nav_routes` into grouped menu sections — entries without a `module` come first, then grouped alphabetically by module — so adding `{ module: "admin", nav_title_key: "..." }` to a route automatically slots it into the right group.

<a name="whats-next"></a>

## What's Next

Routes by themselves only resolve URLs to handlers. To wrap groups of routes with cross-cutting behaviour — logging, language detection, authentication — see [Middleware](/the-basics/middleware). To mount an entire route table under a URL prefix (the way the admin section is built), see the prefixed routes section there.

<a name="adding-and-removing-routes-via-the-tui"></a>

## Adding and Removing Routes via the TUI

<media-frame label="TUI — add / remove route" ratio="4/3"></media-frame>

For the common shapes — a one-off page or a generated CRUD module — `bun tui` writes the boilerplate and edits `routes/routes.ts` for you:

- **Simple route** — generates a `routes/<name>/index.ts` + `index.ree` pair from the shipped template and inserts a `RouteDef` entry. Optionally takes a prefix (e.g. `admin`), in which case the entry is created with `module: "admin"` so it's gated and grouped automatically.
- **Single table** / **CRUD only** / **All tables** — runs the schema-introspection + CRUD generator and adds the `RouteDef` for each generated module (see [Generators](/database/generators)).
- **Remove route** — deletes the route folder, removes its `import` from `routes/routes.ts`, and removes its `RouteDef` entry. System routes (`/login`, `/profile`, etc.) are skipped to prevent accidents.

You can always edit `routes/routes.ts` by hand; the TUI just keeps the housekeeping in one place.
