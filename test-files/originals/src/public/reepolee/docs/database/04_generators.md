---
title: "Generators"
---

# Generators

<a name="introduction"></a>

## Introduction

The Reepolee generator introspects your database schema and produces a complete, working CRUD module — route handlers, SQL queries, form and list templates, Zod validation, and translation files — in a single command. The output is real code you own and can edit freely; the generator is a starting point, not a runtime dependency.

```bash
bun generator/resource all
```

<a name="via-the-tui"></a>

## Via the TUI

<media-frame label="TUI — Resource Manager grouped menu" ratio="4/3"></media-frame>

The friendliest way to drive the generator is the project's interactive tool, the **reepolee Resource Manager** (`bun tui`). It presents a grouped menu of generators, page scaffolds, database/config tasks, and maintenance actions — no CLI flags to remember:

```bash
bun tui
```

On a fresh database (no `users` table yet) it offers to run **Quick Start** first. After each action you press Enter and return to the menu; choose **Exit** to quit.

### CRUD Generators

| Menu option                       | What it does                                                                                                              |
| --------------------------------- | ------------------------------------------------------------------------------------------------------------------------- |
| **Single table**                  | Full pipeline — schema introspection **and** CRUD — for one table you pick                                                |
| **Schema only**                   | Introspect the DB and write schema files only (no CRUD)                                                                   |
| **CRUD only**                     | Generate CRUD from schema files that already exist                                                                        |
| **Bulk CRUD**                     | Select multiple tables (those without existing CRUD) and batch-generate them                                              |
| **All tables**                    | Full pipeline for every table not in `IGNORE_TABLES`                                                                      |
| **Nested children (auto-detect)** | Pick a parent table; the TUI auto-discovers its FK children and batch-generates [nested CRUD](#nested-resources) for them |

### Simple Pages

| Menu option           | What it does                                                                    |
| --------------------- | ------------------------------------------------------------------------------- |
| **Simple Table Page** | Scaffold a simple route backed by a DB query (from a template)                  |
| **Simple Page**       | Scaffold a static page that reads from a local `data.json` — no database needed |

### Database & Config

| Menu option                              | What it does                                                                                                                                        |
| ---------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Set database type**                    | Switch between MySQL and SQLite and update `CONNECTION_STRING` in `.env`                                                                            |
| **Run SQL file**                         | Pick and execute a `.sql` file against the database (lists `sql/<dialect>/` first — see [Schema & Initialization](/database/schema-initialization)) |
| **Set session driver**                   | Switch the session store between Redis and the DB-backed default                                                                                    |
| **Quick Start** / **Reset the database** | Guided setup — DB type → run SQL file → session driver → admin user. Shown as "Reset the database" once a `users` table exists                      |

### Tools & Maintenance

Route management lives here too, alongside the schema and translation maintenance tools:

| Menu option                     | What it does                                                                                                                                                                  |
| ------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Remove route**                | Delete a registered route — folder, imports, and nav entry (skips system routes)                                                                                              |
| **Remove module/prefix folder** | Delete an entire prefixed route folder and all its sub-routes                                                                                                                 |
| **Refresh CRUD**                | Regenerate CRUD for an existing route — overwrites the generated files, keeps your schema files                                                                               |
| **Check domain compliance**     | Audit column types against the canonical [domain types](/database/schema-initialization#domain-types) and report deviations to migrate                                        |
| **Add language**                | Add a language to `config/supported_languages.ts`, generate stub translations, and optionally run the AI translation pass ([providers](/i18n/dynamic-translations#providers)) |
| **Remove language**             | Remove a language from the project — strips it from `config/supported_languages.ts`, prunes its `translations` rows, and cleans localized route maps                          |
| **Prune unused translations**   | Scan templates for `{= … }` refs and emit `DELETE` SQL for DB keys no longer referenced ([details](/i18n/translations#maintenance-prune-sync))                                |
| **Sync missing translations**   | Scan templates for refs missing from the DB and emit `INSERT` SQL to fill them ([details](/i18n/translations#maintenance-prune-sync))                                         |

The CRUD flows prompt interactively for options like prefix and parent where relevant. For scripted runs with `--prefix`, `--parent`, or `--pagination`, use the CLI below.

<a name="commands"></a>

## CLI Commands

All generator commands go through `generator/resource.ts`:

```bash
bun generator/resource all                       # introspect + generate CRUD for every table
bun generator/resource schema <table>            # introspect one table and write schema files only
bun generator/resource schema all-tables         # introspect every table
bun generator/resource crud <table>              # generate CRUD from an existing schema
bun generator/resource crud all                  # generate CRUD for every existing schema
```

For prefixed routes (e.g. `/admin/...`), `bun generator/crud <table> --prefix admin` is the lower-level invocation — see [Prefixed Routes](#prefixed-routes) below.

`schema` and `crud` are intentionally separate steps so you can review the introspected types before committing to the generated UI.

<a name="flags"></a>

## Flags

| Flag                  | Description                                                                                                                                                                                                                                   |
| --------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `--force`             | Overwrite existing generated files without prompting (also injects the marker comments `--refresh-fields` relies on)                                                                                                                          |
| `--translate`         | Auto-translate generated language files via the configured AI provider                                                                                                                                                                        |
| `--refresh-fields`    | Regenerate **only** the field sections inside `form.ree` / `index.ree` (between the `crud:fields` marker comments), leaving your customisations outside the markers untouched. Requires an initial `--force` generation to inject the markers |
| `--parent <table>`    | Generate the resource as a **nested child** of `<table>` (master-detail). See [Nested Resources](#nested-resources)                                                                                                                           |
| `--pagination <type>` | Pagination strategy for the generated list view: `offset` (default) or `cursor`. Recorded as the `pagination_strategy` export in `schema/table.ts`. See [Pagination](#pagination)                                                             |

`--prefix <name>` is supported when calling `crud.ts` directly (not via `resource.ts`) — see [Prefixed Routes](#prefixed-routes).

<a name="what-gets-generated"></a>

## What Gets Generated

Running the generator against a `users` table produces:

```
routes/users/
├── schema/
│   ├── table.generated.ts      # types, field metadata — regenerated on each run
│   ├── table.ts                # column definitions you can edit — written once
│   └── validation_server.ts   # Zod schemas — regenerated on each run
├── translations/
│   ├── en.json
│   └── sl.json                 # one file per supported language
├── index.ts                    # route handlers
├── sql.ts                      # database query functions
├── index.ree                   # list template with search, sort, pagination
└── form.ree                    # create / edit template
```

`routes/routes.ts` is also updated automatically with the import and route entry for the new module.

<a name="schema-generation"></a>

## Schema Generation

The schema step connects to your database using `CONNECTION_STRING` from `.env` and introspects the table structure. It detects column types, nullability, foreign key relationships, and any field metadata embedded in column comments.

Column comments drive field metadata. They come in two forms, and JSON takes precedence when both are present:

**JSON comments** — an object of extra hints:

```sql
ALTER TABLE products ADD COLUMN price DECIMAL(10,2) COMMENT '{"min": 0, "max": 99999}';
ALTER TABLE users    ADD COLUMN bio   TEXT          COMMENT '{"type": "textarea"}';
```

**Plain-word comments** — a bare keyword naming the field type (`autocomplete`, `textarea`, `tags`, …). This replaces the older `field_overrides` pattern where hints were generated commented-out in `table.ts` and uncommented by hand.

<a name="visibility-flags"></a>

### Column Visibility Flags (ICU / CU)

A plain-text comment may also carry a visibility flag controlling where the field appears:

| Flag  | Meaning                                                                                                     |
| ----- | ----------------------------------------------------------------------------------------------------------- |
| `ICU` | **I**ndex + **C**reate + **U**pdate — visible in the list and both forms. This is the default.              |
| `CU`  | **C**reate + **U**pdate only — excluded from the index list (`omit_index`), still on the create/edit forms. |

The flag is matched as a standalone word, so `CU` only takes effect when `ICU` isn't also present. This keeps wide tables readable: only the most relevant columns show in the list, while every column remains editable. Set the flag in the column comment (MySQL only; SQLite has no column comments), or omit it to take the `ICU` default and pare the list down later with [`IGNORE_INDEX_FIELDS`](#configuration) or the `CU` flag per column.

> **Generated columns** (MySQL `VIRTUAL`/`STORED`, SQLite hidden columns) are detected during introspection and excluded from `fields` (they can't be inserted or updated), but still shown in list/index views when a view exists.

The generator maps database types to HTML input types and TypeScript types automatically:

| Database type           | HTML input       | TypeScript |
| ----------------------- | ---------------- | ---------- |
| `INT`, `BIGINT`         | `number`         | `number`   |
| `VARCHAR`, `CHAR`       | `text`           | `string`   |
| `TEXT`                  | `textarea`       | `string`   |
| `DATE`                  | `date`           | `string`   |
| `DATETIME`, `TIMESTAMP` | `datetime-local` | `string`   |
| `BOOLEAN`, `TINYINT(1)` | `checkbox`       | `boolean`  |

<a name="foreign-keys"></a>

## Foreign Keys

Foreign key relationships are detected three ways: explicit `FOREIGN KEY` constraints, the `{singular_table}_{column}` naming convention, or a column name ending in `_id` whose target table+column exist. Either way, the generator renders that field as a dropdown populated from the related table — a plain `<select>`, or the searchable `<auto-complete>` component (`components/auto-complete.ree`) when the column is marked `autocomplete`, which gives live search, keyboard navigation, and autoscroll for large reference tables.

The dropdown label comes from the related table's first non-integer text column (or a column named `title` or `name` if one exists). You can override this by editing `table.ts` after generation.

<a name="crud-generation"></a>

## CRUD Generation

<media-frame label="Screenshot — generated list view (search · sort · pagination)" ratio="16/9"></media-frame>

The CRUD step reads the schema files and writes the route handlers, SQL functions, and templates. The following URL pattern is registered for each resource:

| Method | Path                | Handler                            |
| ------ | ------------------- | ---------------------------------- |
| `GET`  | `/<table>`          | List with search, sort, pagination |
| `GET`  | `/<table>/new`      | Empty create form                  |
| `POST` | `/<table>`          | Create record                      |
| `GET`  | `/<table>/:id/edit` | Populated edit form                |
| `POST` | `/<table>/:id/edit` | Update or delete record            |
| `POST` | `/<table>/validate` | Real-time field validation         |

The list view supports URL-driven state — search query, sort field, sort direction, pagination position (offset, or cursor params), and per-page limit are all carried as query parameters, making every filtered view bookmarkable and shareable. The pagination row in `index.ree` renders four navigation controls — first, previous, next, last — driven by `first_url`, `prev_url`, `next_url`, and `last_url` returned from the handler. Each one renders as a disabled icon when there's nothing to navigate to, so the bar stays visually stable on the first and last page rather than reflowing when a button appears or disappears.

A few details the generator handles automatically:

- **Two pagination strategies.** Offset (the default) and cursor (keyset), selectable per-route — see [Pagination](#pagination) below.
- **Sort options from indexed columns.** The sort dropdown is built from every column that has a DB index (stored as `indexed_columns` in `table.ts`), not a hardcoded list — so adding an index makes a column sortable. Columns in `IGNORE_ORDER_FIELDS` are excluded.
- **Fulltext search.** When a table has a `search_text` column, search uses a fulltext query (MySQL `MATCH … AGAINST`, SQLite `LIKE`) instead of `LIKE '%term%'`, via `lib/sql_dialect.ts`. Requires a MySQL FULLTEXT index.
- **Cache + scope integration.** Each generated `sql.ts` exports `TABLE_NAME` and `VIEW_DEPENDENCIES`, and list queries run through the optional [SQL cache](/database/caching) with the active [global scope](/security/authorization#global-scopes) folded into the cache key.

<a name="pagination"></a>

## Pagination

Generated list views support two pagination strategies, chosen at generation time with `--pagination` and recorded as a `pagination_strategy` export in `schema/table.ts`:

```ts
export const pagination_strategy: "cursor" | "offset" = "offset";
```

The generator branches on this value at codegen time — there is no runtime switch — producing the matching SQL, URL-param parsing, pagination-URL building, and template rendering. It defaults to `"offset"` when the export is absent (the same once-and-edit pattern as `enable_delete`).

| Strategy             | SQL                                                      | URL params                                                       | Position display          | Notes                                                                                                   |
| -------------------- | -------------------------------------------------------- | ---------------------------------------------------------------- | ------------------------- | ------------------------------------------------------------------------------------------------------- |
| **Offset** (default) | `LIMIT ? OFFSET ?`                                       | `offset`, `limit`, `order_by`, `query`, `scope`                  | range, e.g. `21-40 / 100` | Supports [global scopes](/security/authorization#global-scopes). Nested children always use offset.     |
| **Cursor** (keyset)  | `WHERE (sort > val OR (sort = val AND id > id)) LIMIT N` | `after`, `before`, `last`, `limit`, `order_by`, `query`, `scope` | count, e.g. `40 / 100`    | Deep pages stay fast and don't skip/duplicate rows when data shifts underneath. No numbered page links. |

Both render the same first/prev/next/last icon-button bar. Offset is the better default for admin grids where a stable, jump-to-page range display matters; choose cursor for large or fast-changing tables where deep-page performance matters more than positional display.

<a name="streaming"></a>

## Streaming List Views (DPU)

<media-frame label="Animation — streaming list: shell first, rows fill in" ratio="16/9"></media-frame>

A generated list view can render synchronously (the default) or **stream** its records as Declarative Partial Updates (DPU), selectable per-route via the `render_strategy` export in `schema/table.ts`:

```ts
export const render_strategy: "stream" | "load" = "load";
```

- **`"load"`** (default) — runs the DB queries, then returns one fully-rendered `Response`.
- **`"stream"`** — renders the page shell (layout, nav, controls) immediately as the first chunk of a `ReadableStream`, then streams the pagination bar and record rows as `<template for="…">` chunks once the DB queries resolve. The shell appears instantly even when the query is slow; the rows fill in when ready.

Like pagination, the generator branches on this value at codegen time (no runtime branching), selecting the streaming GET-handler template (offset or cursor variant) and wrapping the records/pagination regions in `<?start name="…">` / `<?end>` DPU markers in `index.ree`.

Streaming relies on a small vendored polyfill (`static/dpu.min.js`, GoogleChromeLabs' HTML `<template>` setters polyfill, fetched by `bun get:dpu`). Its `<script>` tag in `routes/layout.ree` ships **commented out** — uncomment it when you deploy streaming to production.

<a name="prefixed-routes"></a>

## Prefixed Routes

The `--prefix` flag scopes a resource under a sub-path. This is the standard way to build an admin panel:

```bash
bun generator/crud users --prefix admin
```

Files are written to `routes/admin/users/` and `routes/routes.ts` is updated to mount them with `mount_prefix`:

```ts
...mount_prefix("/admin", admin_users_crud, require_module_mw("admin")),
```

The second argument to `mount_prefix` is an optional middleware guard. `require_module_mw("admin")` redirects any request from a user who doesn't have the `admin` module in their `users.modules_tags` field before the handler runs. You can swap it for `require_auth_mw()` or any custom middleware. Available modules are loaded from the `modules` table — see [Authorization](/security/authorization).

<a name="nested-resources"></a>

## Nested Resources (Master-Detail)

<media-frame label="Screenshot — parent edit form with inline child list" ratio="16/9"></media-frame>

The `--parent <table>` flag generates a resource as a **child** of another table — records scoped to a single parent and managed under nested URLs like `/orders/:order_id/items`:

```bash
bun generator/crud items --parent orders
```

The parent foreign key is auto-detected from the child table's FK constraints at introspection time and recorded as a `parent` export in `schema/table.ts`, which you can override:

```ts
export const parent = {
	table: "orders", // parent table
	fk_column: "order_id", // FK column in THIS table
	route_param: "order_id", // URL param name
};
```

What nesting produces:

- **Child routes** carry the parent segment: `POST /orders/:order_id/items` (create), `GET /orders/:order_id/items/new`, and `GET|POST /orders/:order_id/items/:item_id/edit`. There is **no dedicated child index page** and no bulk-delete — children are viewed on the parent's edit form.
- **Scoped SQL.** Every child query takes the parent id and filters `WHERE parent_fk = ?`; single-record lookups also verify the parent matches, so you can't reach a child scoped to a different parent.
- **Parent integration.** The generator edits the parent's files too: the parent `sql.ts` gains a `get_<children>_by_<fk>()` query, the parent `index.ts` edit handler loads the children, and the parent `form.ree` gets a managed `crud:children` marker section rendering an **inline child list** (all rows, no pagination/search/sort, with edit + delete actions).
- **Return-URL flow.** Child new/edit forms hide and pre-fill the parent FK and set `_return_url` back to the parent edit page, so create/update/delete redirects land on `/orders/:order_id/edit`.

Nesting is single-level (parent → child); a parent can have multiple children. The parent FK field in the child form renders as a hidden input inside a `<field-wrapper>` and is excluded from validation since it's set programmatically.

<a name="crud-ignore"></a>

## Excluding Tables with `crud-ignore`

Besides `IGNORE_TABLES` in config, you can exclude a table directly from the database by adding the tag `crud-ignore` to its **table comment**. Tagged tables drop out of the TUI table listings, the bulk CRUD generator, and the `all` generation path — handy for internal/system tables and join tables you annotate in the schema rather than in code. The tag is case-insensitive. (MySQL only — SQLite has no table comments.)

<a name="configuration"></a>

## Configuration

The generator respects several settings in `config/db_structure.ts`:

```ts
export const IGNORE_TABLES = ["modules", "sessions", "email", "images", "users", "translations"] as const;

export const MAINTENANCE_FIELDS = ["created_at", "updated_at"] as const;

export const DATE_SUFIXES = ["_on", "_by"] as const;
export const DATETIME_SUFIXES = ["_at"] as const;

export const IGNORE_INDEX_FIELDS = [
	"option_title",
	"search_text",
	"hashed_password",
	"previous_hashed_password",
] as const;

export const IGNORE_ORDER_FIELDS = ["search_text", "hashed_password", "previous_hashed_password"] as const;

export const BOOLEAN_PREFIXES = ["is_", "has_", "can_"] as const;

export const MIN_PASSWORD_LENGTH = Bun.argv.includes("--dev") ? 1 : 8;
```

**`IGNORE_TABLES`** — Tables the generator skips entirely. Add tables that don't need a UI here. The shipped list excludes the framework's own tables (`sessions`, `modules`, `images`, `users`, `translations`) and the `email` table that has its own hand-written admin. You can also exclude a table from the database side with the [`crud-ignore`](#crud-ignore) table comment.

**`MAINTENANCE_FIELDS`** — Fields managed by the database itself. Excluded from form schemas and from user-supplied write payloads; still readable on edit pages as read-only timestamps.

**`DATE_SUFIXES` / `DATETIME_SUFIXES`** — Column-name suffixes that signal date or datetime fields. A column ending in `_on` or `_by` (`incorporated_on`, `payment_due_by`) gets the date codec applied; a column ending in `_at` (`created_at`, `verified_at`) gets the datetime codec. The codecs handle conversion between database format and the format HTML date inputs expect.

**`IGNORE_INDEX_FIELDS`** — Fields hidden from the list table. Still present on the form. Useful for fields that exist in the schema but aren't meaningful at a glance — sensitive values (`hashed_password`, `previous_hashed_password`) and free-text columns used for search (`search_text`, `option_title`). For per-column control without a global list, use the [`CU` visibility flag](#visibility-flags) in the column comment instead.

**`IGNORE_ORDER_FIELDS`** — Columns excluded from the sort dropdown even when they're indexed. Mirrors `IGNORE_INDEX_FIELDS` but for ordering — keeps internal columns like `search_text` and password hashes out of the sort options.

**`BOOLEAN_PREFIXES`** — Column names starting with these prefixes are rendered as a Yes/No select instead of a checkbox. Useful when you want the value to be explicit ("Yes" or "No") rather than depending on whether a checkbox was checked.

**`MIN_PASSWORD_LENGTH`** — Minimum length enforced by the auth flows. Relaxed to 1 in development (`--dev` flag) so test accounts are easy to create; hardened to 8 in production.

<a name="translations"></a>

## Translations

The generator produces auto-generated labels for each field (e.g. `created_at` becomes `"Created at"`). It also syncs the resource's **navigation label directly into the `translations` database table** (namespace = the route's nav key, `key_path = "nav"`), so a freshly generated resource shows up in the sidebar without editing JSON. The database is the source of truth for translations — see [Translations](/i18n/translations#runtime-overrides).

To fill in missing translations automatically, pass `--translate`:

```bash
bun generator/resource users --translate
```

This calls the configured AI provider (OpenRouter/Claude Haiku by default, or Ollama / Hugging Face — see [Dynamic Translations](/i18n/dynamic-translations#providers)) to translate every missing key.

To fill keys missing across languages after the fact, `sync:languages` scans the `translations` **database** table (not JSON), AI-translates any key present in one language but missing in another, and writes the results back to the DB:

```bash
bun run sync:languages
```

To find and fill keys referenced in templates but not yet in the database — or to prune DB keys no longer referenced by any template — use the **Sync missing translations** and **Prune unused translations** TUI tools. Both emit a reviewable `.sql` file rather than touching the DB directly. See [Translations](/i18n/translations#maintenance-prune-sync) for the full DB-first workflow.

<a name="editing-generated-files"></a>

## Editing Generated Files

Not all generated files are equal:

| File                          | Safe to edit?                         |
| ----------------------------- | ------------------------------------- |
| `schema/table.ts`             | Yes — written once, never overwritten |
| `schema/table.generated.ts`   | No — regenerated on every schema run  |
| `schema/validation_server.ts` | No — regenerated on every schema run  |
| `index.ts`                    | Yes — written once                    |
| `sql.ts`                      | Yes — written once                    |
| `index.ree`                   | Yes — written once                    |
| `form.ree`                    | Yes — written once                    |
| `translations/en.json`        | Seed only — the DB is authoritative   |

`table.ts` is the intended customisation point for the schema layer. Change column labels, swap an input type, adjust the grid layout, or add display-only computed fields — none of those changes will be lost when you re-run the generator.
