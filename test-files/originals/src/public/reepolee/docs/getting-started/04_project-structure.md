---
title: "Project Structure"
---

# Project Structure

<a name="introduction"></a>

## Introduction

Reepolee follows a simple, opinionated directory layout where every folder has a clear, single purpose. You won't need to make decisions about where things go — the structure tells you. Once you've worked on one Reepolee project, you'll feel immediately at home in any other.

<a name="the-root-directory"></a>

## The Root Directory

<media-frame label="Diagram — project directory map" ratio="4/3"></media-frame>

A freshly cloned Reepolee project looks like this:

```
my-app/
├── server.ts
├── config/
├── lib/
├── routes/
│   └── routes.ts
├── components/
├── css/
├── static/
├── generator/
└── vendor/
```

The two most important files are `server.ts` and `routes/routes.ts`. Everything else supports them.

<a name="the-server-file"></a>

### The `server.ts` File

`server.ts` is the entry point for your application. It starts Bun's HTTP server via `Bun.serve()`, wires up global middleware, serves static files from the `static/` directory, and opens a WebSocket connection for live reload in development. You rarely need to touch this file day-to-day.

<a name="the-routes-file"></a>

### The `routes/routes.ts` File

`routes/routes.ts` is where all URL paths are mapped to their handlers. It imports every route module and assembles the final route table that gets handed to `Bun.serve()`. Adding a new feature means adding one import and one entry here.

CRUD modules spread multiple paths at once using the spread operator, which keeps the file concise even as your application grows:

```ts
export const routes = {
	"/": home_page,
	"/about": about_page,
	...users_crud, // registers /users, /users/new, /users/:id/edit, and more
	...auth_crud,
};
```

<a name="the-routes-directory"></a>

## The Routes Directory

The `routes/` directory is the heart of a Reepolee application. Each feature lives in its own subdirectory alongside everything it needs — the handler, templates, SQL queries, validation, and translations. Nothing leaks out into a global controllers folder or a global views folder.

<a name="route-folders"></a>

### Route Folders

A fully generated CRUD route folder looks like this:

```
routes/users/
├── schema/
│   ├── table.generated.ts   # auto-generated types and helpers
│   ├── table.ts             # user-editable column definitions
│   └── validation_server.ts # auto-generated Zod validation
├── translations/
│   ├── en.json
│   └── sl.json
├── index.ts                 # route handlers
├── sql.ts                   # database queries
├── index.ree                # list template
└── form.ree                 # create / edit template
```

Simple pages that don't need a database only need an `index.ts` and a `.ree` template.

Every route handler follows the same pattern — load translations, query the database, call `render()`:

```ts
export async function home_page(req: BunRequest): Promise<Response> {
	const ctx = await create_ctx(req);
	const translated = await translated_from_request(req, import.meta.dir);
	return render("home/home", { data: { ...translated }, ctx });
}
```

At the root of `routes/` you'll also find `layout.ree`, the main HTML shell that every page extends, and `notfound.ree`, the 404 fallback rendered by `server.ts`.

<a name="translations"></a>

### Translations

Each route folder contains one JSON file per supported language. Generated CRUD routes group them under `translations/`; simple routes can keep `en.json` and `sl.json` directly in the route folder — both layouts are picked up. The `translated_from_request()` helper reads the active language from a cookie or a `?lang=` query parameter, then merges the route-level file with the global translations found at `routes/en.json` (and `sl.json`). Your templates receive a flat `translated` object — no nested lookups, no missing-key surprises.

<a name="the-lib-directory"></a>

## The Lib Directory

The `lib/` directory contains the core utilities that power your application. These are plain TypeScript modules with no runtime dependencies — everything runs on Bun's built-in APIs.

The most important files you'll interact with are `render.ts` (the `render()` helper used by every route handler), `helpers.ts` (including `translated_from_request()`), `validation_helpers.ts` (Zod-based validation), and `middleware.ts` (which provides `wrap_all_routes()` and `set_lang()`). The template engine lives in `template_engine.ts` and is covered in depth in the [Ree Templates](/ree-templates/introduction) section.

<a name="the-config-directory"></a>

## The Config Directory

`config/db.ts` is a single file that creates a Bun `SQL` connection directly from the `CONNECTION_STRING` environment variable and exports it as `db`. It also reads `TIME_ZONE` unconditionally to configure timezone constants for both SQLite and MySQL. There are no separate `db_sqlite.ts` or `db_mysql.ts` sub-files — driver selection is handled entirely within `config/db.ts` at startup. Schema initialization SQL lives under per-dialect folders, `sql/sqlite/` and `sql/mysql/`, each holding numbered schema and seed files.

`config/supported_languages.ts` is where you add or remove languages. Changing this list is all it takes to enable a new locale across the entire application.

<a name="the-components-directory"></a>

## The Components Directory

`components/` holds reusable `.ree` partials that can be included from any template. Reepolee ships with a full set of typed form input components — text, email, password, select, checkbox, textarea, date/time pickers, and more. These live in `components/input-text.ree`, `components/input-email.ree`, etc.

You include a component by writing it as a custom HTML element whose tag matches the file name, passing data through attributes:

```html
<app-banner type="error">{= translated.error }</app-banner>
```

The template engine resolves `<app-banner>` to `components/app-banner.ree` automatically. Attributes arrive under `props.attributes` and slot content as `props.children`; an `attr="{= expr }"` attribute is interpolated. There's no registration step. (Generated CRUD forms don't call the typed input components — they inline the field markup directly; see [Components](/ree-templates/components).)

<a name="the-generator-directory"></a>

## The Generator Directory

The `generator/` directory contains the tools that make Reepolee fast to work with. Running the resource generator against a database table produces a complete, working CRUD module — list view with search, sorting and pagination, create and edit forms, server-side Zod validation, toast notifications on save, and delete with foreign key error handling.

```bash
bun generator/resource all              # full pipeline for every table
bun generator/resource schema users     # introspect DB schema only
bun generator/resource crud users       # generate CRUD from existing schema
bun generator/resource schema all-tables   # regenerate schemas for every table
```

**Flags:**

| Flag          | Description                                               |
| ------------- | --------------------------------------------------------- |
| `--force`     | Overwrite existing files without prompting                |
| `--translate` | Auto-translate generated language files via OpenRouter AI |

For prefixed CRUD modules, call `crud.ts` directly with `--prefix <name>`. Running `bun generator/crud users --prefix admin` writes files to `routes/admin/users/` and registers the route via `mount_prefix` with a matching module guard:

```ts
...mount_prefix("/admin", admin_users_crud, require_module_mw("admin")),
```

The generator writes real files and registers the new route in `routes/routes.ts`. Review the output before committing — it's meant as a starting point, not a black box.

<a name="the-static-and-css-directories"></a>

## The Static and CSS Directories

`static/app.css` is the compiled Tailwind v4 output. The source is at `css/app.css` — the Tailwind CLI compiles it to `static/app.css` for production and `static/app-dev.css` during development.

The `static/` directory also holds the vendored front-end scripts that ship with every Reepolee project: `spa-loader.js` for view transitions, `form-controller.js` for AJAX form submission with toast feedback, `helpers-client.js` for utility functions, and a `web-components/` subdirectory with `validation-error.js`, `toasts-area.js`, `title-display.js`, and `image-editor.js`. Client-side reactivity is powered by the signals system documented in the [Client-Side](/client-side/signals) section. These files are served directly from disk with aggressive caching headers in production.
