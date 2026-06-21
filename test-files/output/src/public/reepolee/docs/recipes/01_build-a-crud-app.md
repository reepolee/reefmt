---
title: "Build a CRUD App"
---

# Build a CRUD App

<a name="introduction"></a>

## Introduction

This recipe walks from an empty Reepolee project to a fully working CRUD module: list with search and pagination, create form, edit form, delete with confirmation, server-side and live validation, translated labels. The example is a `books` resource — a table with a title, an author, an ISBN, a publication date, and a "currently in stock" boolean.

The point isn't to teach you SQL — it's to show how the pieces of Reepolee fit together. By the end, you'll have a feel for the database-first workflow that the rest of the framework is shaped around.

Total time: about 15 minutes if you're following along, less once you've done it twice.

<a name="step-1-define-the-schema"></a>

## Step 1: Define the Schema

Open `sql/sqlite/01-init-sqlite.sql` (or `sql/mysql/01-init-mysql.sql` if you're on MySQL) and add the `books` table:

```sql
DROP TABLE IF EXISTS books;

CREATE TABLE books (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    author TEXT DEFAULT '' NULL,
    isbn TEXT DEFAULT '' NULL,
    published_on DATE DEFAULT NULL,
    is_in_stock INTEGER DEFAULT 1,
    created_at DATETIME DEFAULT current_timestamp,
    updated_at DATETIME DEFAULT NULL
);

CREATE INDEX books_title ON books(title);
CREATE UNIQUE INDEX books_isbn_unique ON books(isbn);

CREATE TRIGGER books_update_timestamp
AFTER UPDATE ON books
FOR EACH ROW
BEGIN
    UPDATE books SET updated_at = current_timestamp WHERE id = NEW.id;
END;

INSERT INTO books (title, author, isbn, published_on, is_in_stock) VALUES
    ('Pride and Prejudice', 'Jane Austen',  '978-0141439518', '1813-01-28', 1),
    ('Dune',                'Frank Herbert', '978-0441172719', '1965-08-01', 1),
    ('The Left Hand of Darkness', 'Ursula K. Le Guin', '978-0441478125', '1969-03-01', 0);
```

A few of the conventions matter for the generator:

- **`id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT`** — the standard primary key shape Reepolee expects.
- **`is_in_stock`** starts with `is_` — the generator recognises it as a boolean and renders it as a Yes/No select. See [`BOOLEAN_PREFIXES`](/database/generators#configuration).
- **`published_on`** ends with `_on` — the generator applies the date codec automatically (`DATE_SUFIXES`).
- **`created_at` / `updated_at`** plus the trigger — the generator excludes them from form schemas and the list view edits them as read-only timestamps.
- **The unique index on `isbn`** lets the generator surface a "duplicate" error message when you try to add two books with the same ISBN.
- **The seed `INSERT`** rows give the generated list page something to display on first load. Drop them later when you have real data.

<a name="step-2-initialise-the-database"></a>

## Step 2: Initialise the Database

Apply the schema manually — the driver doesn't run init SQL on startup. For SQLite:

```bash
sqlite3 app.db < sql/sqlite/01-init-sqlite.sql
```

That drops and recreates every table the file defines, then inserts the seed data. The `app.db` file now contains the `books` table with the three seed rows. Start the dev server (`bun dev`) and the application picks up the new tables on its next query.

<a name="step-3-generate-the-resource"></a>

## Step 3: Generate the Resource

<media-frame label="Screenshot — generated /books list with seed rows" ratio="16/9"></media-frame>

In a second terminal:

```bash
bun generator/resource all
```

Pass `all` to introspect every table the database knows about (excluding the ones in `IGNORE_TABLES`) and generate both the schema files and the CRUD module in one pass. To target just `books`, use `bun generator/resource crud books` after running `bun generator/resource schema books` first.

The generator writes a complete CRUD module to `routes/books/`:

```
routes/books/
├── schema/
│   ├── table.generated.ts      ← types and field metadata (regenerated each run)
│   ├── table.ts                ← user-editable column definitions (written once)
│   └── validation_server.ts    ← Zod schemas (regenerated each run)
├── translations/
│   ├── en.json                 ← English labels and validation messages
│   └── sl.json                 ← Slovenian labels (empty until translated)
├── index.ts                    ← route handlers
├── sql.ts                      ← database queries
├── index.ree                   ← list template
└── form.ree                    ← create / edit template
```

The generator also adds an entry to `routes/routes.ts` (using the [declarative `RouteDef` shape](/the-basics/routing#declarative-route-defs)):

```ts
import { build_routes, build_nav_routes, type RouteDef } from "$lib/route_builder";
import { books_crud } from "$routes/books";
// ...
const route_defs: RouteDef[] = [
	{ url: "/", handler: home_page },
	{ url: "/books", crud: books_crud, nav_title_key: "books" },
	// ...
];

export const nav_routes = build_nav_routes(route_defs);
export const routes = { ...build_routes(route_defs), ...auth_crud };
```

The dev server picks up the new files and re-registers the routes. Visit `http://localhost:2338/books` and you have:

- A list page with the three seed rows, search, pagination, and per-page-limit dropdown.
- "New" button → `/books/new` for adding a record.
- Each row's edit link → `/books/:id/edit` for updates and deletes.
- The toast confirmation after saves.

That's a working CRUD module in three commands.

<a name="step-4-what-was-generated"></a>

## Step 4: What Was Generated

Open `routes/books/index.ts` to see the route handlers. The list handler reads the pagination params, resolves the active [table scope](/security/authorization#global-scopes), and calls `search_records`:

```ts
export async function get_books_index(req: BunRequest): Promise<Response> {
    const ctx = await create_ctx(req, import.meta.dir);
    ctx.toasts = get_cookies_by_prefix(req, "toast-");

    // Offset pagination params (the generator default)
    const { query, offset, limit, order_by, scope } = parse_pagination_params(req.url);
    const scope_clause = scope ? await get_scope_clause("books", scope) : "";

    const result = await search_records(query, offset, limit, order_by, scope_clause);

    // ...build first/prev/next/last pagination URLs from offset + result.total...

    const translated = await translated_from_request(req, import.meta.dir);
    return render("index", {
        data: { records: result.records, total: result.total, query, order_by, /* …pagination… */, ...translated },
        ctx,
    });
}
```

The generator defaults to **offset** pagination; pass `--pagination cursor` (or set `pagination_strategy: "cursor"` in `schema/table.ts`) to generate the keyset variant instead, which parses `after`/`before` cursor params. See [Pagination](/database/generators#pagination).

Open `routes/books/sql.ts` for the queries — `get_record_by_id`, `get_all_records`, `search_records`, `create_record`, `update_record`, `delete_record`. `search_records` runs through the [Redis cache](/database/caching) (a no-op unless `CACHE_ENABLED`); the write functions call `cache.invalidate(TABLE_NAME)`. The functions are typed against the row shape that `table.generated.ts` exports.

Open `routes/books/index.ree` for the list template, `routes/books/form.ree` for the form. Both use the [shipped form components](/forms/input-components) and the [validation flow](/forms/validation), so they're concise — most of the markup is the page chrome.

The output is meant to be edited. Generated files are written once unless you re-run with `--force`; your edits stick.

<a name="step-5-customise-a-field"></a>

## Step 5: Customise a Field

The `published_on` field is currently a `<input type="date">`, which is fine but maybe we want a year-only display in the list view. The customisation point is `routes/books/schema/table.ts`:

```ts
// schema/table.ts
export type { books_type } from "./table.generated";
import { fields, v_fields } from "./table.generated";

const columns = "10ch_30ch_25ch_15ch_10ch_auto";

export { fields, v_fields, columns };
```

`columns` is a CSS grid column definition the list template uses. Adjust it to give the title more room and the ISBN less:

```ts
const columns = "5ch_40ch_25ch_18ch_12ch_8ch";
```

Save, refresh the list page, the columns rebalance. No restart needed — the dev server picks up the change.

For deeper customisation — overriding an input type, hiding a column, adding a derived field — `table.ts` re-exports `fields` from the generated file. Override entries by re-declaring them:

```ts
import { fields as gen_fields } from "./table.generated";

export const fields = {
	...gen_fields,
	isbn: { ...gen_fields.isbn, label: "ISBN", input: "text", input_class: "font-mono" },
};
```

The form template reads `props.fields[field_name].label` and respects whatever you put there.

<a name="step-6-add-custom-validation"></a>

## Step 6: Add Custom Validation

The generated Zod schema in `validation_server.ts` covers the shape — required fields, types, optional vs not. For project-specific rules (an ISBN format check, a "publication date can't be in the future" rule), add the check in the route handler after the standard `validate()` call.

Open `routes/books/index.ts` and find `post_books_index`:

```ts
export async function post_books_index(req: BunRequest): Promise<Response> {
	const translated = await translated_from_request(req, import.meta.dir);
	const params = new URLSearchParams(await req.text());
	const data = {
		/* ... */
	};

	const [errors, valid_data] = validate(data, translated.errors);

	// ↓ add custom checks here
	if (data.isbn && !/^[\d-]{10,17}$/.test(data.isbn)) {
		errors.isbn = translated.errors.isbn_invalid;
	}

	if (data.published_on && new Date(data.published_on) > new Date()) {
		errors.published_on = translated.errors.publication_in_future;
	}

	if (Object.keys(errors).length > 0) {
		return render("books/form", {
			/* ... */
		});
	}

	await create_record(valid_data);
	return Response.redirect("/books", 303);
}
```

Then add the translation keys to `routes/books/translations/en.json`:

```json
{
	"errors": {
		"isbn_invalid": "ISBN must be 10–17 digits with optional dashes",
		"publication_in_future": "Publication date cannot be in the future"
	}
}
```

Run `bun run sync:languages` to fill the missing keys across languages — it scans the `translations` database table and AI-translates anything missing, writing the results back to the DB.

<a name="step-7-add-to-the-navigation"></a>

## Step 7: Add to the Navigation

The shipped layout's left-hand nav is built from `nav_routes`, which `build_nav_routes()` derives from the same `RouteDef` array that builds the route table. If the generator's entry already has `nav_title_key`, the link is already in the menu. Otherwise, add the key:

```ts
// routes/routes.ts
const route_defs: RouteDef[] = [
	{ url: "/", handler: home_page },
	{ url: "/books", crud: books_crud, nav_title_key: "books" },
	// ...
];
```

Add the navigation label to your translation files:

```json
// routes/en.json
{
	"nav": {
		"books": "Books"
	}
}
```

The layout's left-hand nav now shows a "Books" entry that links to `/books`. To group the entry under a module heading (e.g. an `admin` section), add `module: "admin"` to the same `RouteDef`.

<a name="step-8-test-the-flow"></a>

## Step 8: Walk the Whole Flow

<media-frame label="Screenshot — edit form showing an inline validation error" ratio="16/9"></media-frame>

In the browser, click through the actions:

1. **List** at `/books` — search for "Dune", confirm the result filters down.
2. **New** → fill in a fake book → save. The redirect lands on `/books` with the toast confirmation.
3. **Edit** any row → change the title → save. The list reflects the new title.
4. **Edit** with bad data — try an ISBN of "abc" — confirm the inline validation error appears as you tab off the field, and the form doesn't submit.
5. **Try a duplicate ISBN** — the unique-index violation surfaces as a "duplicate" form error from the database error handler.
6. **Delete** a row from the edit page — confirm the dialog, accept, the row disappears from the list.

If everything works, you have a fully functional CRUD module in roughly fifteen minutes.

<a name="whats-next"></a>

## What's Next

The same pattern scales to dozens of resources. A few directions to take it:

- **[Building an Admin Panel](/recipes/admin-panel)** — generate the same module under `/admin/books` with admin authorization.
- **[Adding a New Language](/recipes/new-language)** — translate the module into another language with auto-localised URLs.
- **[Custom Form Components](/recipes/custom-form-components)** — write a custom input for fields the standard set doesn't cover (a colour picker, a tag input, a rich-text editor).
- **Foreign keys** — add an `author_id` column to `books` referencing an `authors` table, regenerate, and the form renders a dropdown populated from the related table. See [Schema & Initialization](/database/schema-initialization#foreign-keys).

The deeper docs to read once you're comfortable with the basic flow:

- **[Generators](/database/generators)** — the full reference for what the generator can do.
- **[Validation](/forms/validation)** — the Zod schema layout, custom rules, the live-validation endpoint.
- **[Database — Querying](/database/querying)** — direct SQL when you need it.
