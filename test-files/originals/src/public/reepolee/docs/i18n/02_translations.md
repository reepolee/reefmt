---
title: "Translations"
---

# Translations

<a name="introduction"></a>

## Introduction

Reepolee's translation system follows a **DB-first** model: the `translations` database table is the source of truth. JSON files in `routes/` still exist as a secondary seed and are loaded at startup, but DB values win on merge — a key that lives only in the database does not need to appear in any JSON file. There is no translation library, no extraction pipeline, and no compilation step; templates pick up the strings through a small `translated_from_request()` helper.

> **JSON files in `routes/` are legacy seeds.** Don't add or edit translations by hand-writing `routes/**/*.json` — those values are overridden by the DB on merge. Manage translations through the database instead (see [The Database Is the Source of Truth](#runtime-overrides) below). The `public/` folder is the exception: JSON files there hold static page data, not translations.

This page first describes the _shape_ a translation namespace takes (the same shape whether it lives in a JSON seed or a DB row), then how handlers read it, and finally how to manage strings the DB-first way.

Strings are namespaced by where they're consumed. The login form's strings live under the `system.auth.login` namespace; global strings — navigation labels, common button text, the language-mismatch dialog — live in the special `root` namespace (seeded from `routes/en.json`). When the login route handler renders the form, it gets both layers merged automatically.

<a name="file-layout"></a>

## File Layout

The convention is one `<lang>.json` file per route folder, plus one at the root of `routes/` for global strings:

```
routes/
├── en.json                          ← global English strings
├── sl.json                          ← global Slovenian strings
├── home/
│   ├── en.json                      ← home-page English
│   └── sl.json
├── system/
│   └── auth/
│       ├── en.json                  ← auth-section English
│       ├── sl.json
│       ├── login/
│       │   ├── en.json              ← login-form English
│       │   └── sl.json
│       └── register/
│           ├── en.json
│           └── sl.json
└── users/
    ├── translations/                ← alternative: a "translations" subfolder
    │   ├── en.json
    │   └── sl.json
    ├── index.ts
    └── ...
```

Either layout works — the loader walks the directory recursively looking for `.json` files whose name matches one of the supported languages. The `translations/` subfolder convention is what the generator produces; hand-written routes typically put the JSON next to the handler.

<a name="file-contents"></a>

## What Goes in a Translation File

A typical file groups labels, error messages, and metadata under a small set of category keys. Here's `routes/en.json` (global strings):

```json
{
	"actions": {
		"save": "Save",
		"cancel": "Cancel",
		"back": "Back",
		"delete": "Delete",
		"confirm_delete": "Yes, do delete",
		"abort_delete": "No, do not delete"
	},
	"errors": {
		"required": "Required",
		"email_required": "Email is required",
		"email_invalid": "Must be a valid email address",
		"duplicate_key": "Code already exists"
	},
	"messages": {
		"record_created": "Record created",
		"record_updated": "Record updated",
		"record_deleted": "Record deleted"
	},
	"nav": {
		"home": "Home",
		"users": "Users",
		"email": "Email"
	},
	"search": {
		"search_term": "Search term ...",
		"submit": "Search"
	},
	"selectors": {
		"all": "All records",
		"per_page": "per page",
		"select": "-- Select --"
	},
	"ui": {
		"lang_mismatch_title": "Language mismatch",
		"lang_mismatch_body": "This page is in",
		"language_names": { "en": "English", "sl": "Slovenian" }
	},
	"route_name": ""
}
```

And `routes/system/auth/login/en.json` (route-specific):

```json
{
	"route_name": "login",
	"ui": {
		"title": "Login"
	},
	"fields": {
		"email": { "label": "Email" },
		"password": { "label": "Password" }
	},
	"actions": {
		"submit": "Login"
	},
	"errors": {
		"invalid_email_or_password": "Invalid email or password.",
		"account_not_verified": "Your account is not verified yet."
	}
}
```

Three things to notice:

- **`route_name`** is a reserved key — its value drives URL localisation. The Slovenian login file has `"route_name": "prijava"`, which makes `/login` reachable as `/prijava` in Slovenian. See [Localized Routes](/i18n/localized-routes).
- **Strings are grouped under intent buckets** (`actions`, `errors`, `messages`, `ui`, `fields`, `search`, `selectors`). The generator emits this shape, the handler validators pass `translated.errors` to `validate()`, and the templates read `props.actions.submit`, `props.errors.email_required`, etc. The grouping keeps a single category translatable as a unit and avoids name collisions between, say, a label and an error.
- **Nested objects work fine.** `fields.email.label` is the convention generated CRUD code expects — the form template reads `props.fields.email.label` for the email input's label.

<a name="conventions-for-keys"></a>

## Conventions for Keys

The generator produces a consistent shape across every route's translation file. Following it keeps your own translations easy to navigate and your handler code consistent:

| Top-level key | Contains                                                                 | Common consumer                             |
| ------------- | ------------------------------------------------------------------------ | ------------------------------------------- |
| `route_name`  | The localised URL segment (omit to keep canonical)                       | URL resolver                                |
| `ui`          | Headings, body copy, page title (`ui.title`), descriptive labels         | `props.ui.title`, `props.ui.*` in templates |
| `actions`     | Button text — `submit`, `save`, `cancel`, `delete`, `back`               | `props.actions.*`                           |
| `errors`      | Validation messages by rule key (`email_required`, `password_too_short`) | `validate(data, translated.errors)`         |
| `messages`    | Toast and confirmation strings (`record_created`, `successful_save`)     | Toast cookie payload                        |
| `fields`      | Per-field metadata: `{ field_name: { label, ... } }`                     | Form templates                              |
| `search`      | Search-form copy                                                         | List page templates                         |
| `selectors`   | Dropdown options (yes/no, per-page)                                      | List page templates                         |
| `nav`         | Navigation menu entries (often nested)                                   | Layout templates, `nav_label()` helper      |

For your own keys, prefer:

- Snake_case for keys (`record_updated`, `email_not_sent`).
- Lowercase, no punctuation. The capitalisation of the displayed string lives in the value.
- One translation file per natural unit of work — a route handler's concerns, mostly. Avoid sprinkling the same translation across multiple files because then changing it requires changing both.

<a name="reading-translations-in-handlers"></a>

## Reading Translations in Handlers

`translated_from_request(req, import.meta.dir)` from `$lib/helpers` is the single function every handler calls. It does three things:

1. Reads the active language from the request (`X-Lang` header → cookie → default).
2. Looks up the route's namespace from `import.meta.dir` (e.g. `system/auth/login`).
3. Merges the route-specific translations on top of the global ones.

```ts
import { translated_from_request } from "$lib/helpers";
import { render } from "$lib/render";
import { create_ctx } from "$lib/request_context";

export async function get_auth_login(req: BunRequest): Promise<Response> {
	const ctx = await create_ctx(req);
	const translated = await translated_from_request(req, import.meta.dir);

	return render("system/auth/login/form", {
		data: {
			action: "/login",
			...translated,
		},
		ctx,
	});
}
```

Spreading `translated` into `data` means the template can access every key directly:

```html
<h1>{= props.ui.title }</h1>
<label>{= props.fields.email.label }</label>
<button type="submit">{= props.actions.submit }</button>
```

No nested object lookups, no per-key fallbacks. If a key is missing, the merge step ([Fallback Merge](#fallback-merge) below) handles it; if it's truly absent in every language, the template renders the key path wrapped in braces — e.g. `{system.auth.login.ui.title}` — so the gap is obvious on the page during testing rather than showing as a silent blank.

<a name="loading-and-namespaces"></a>

## How Files Are Loaded

`lib/i18n.ts` walks `routes/` at server startup. For each JSON file it finds:

1. The language code comes from the filename (`en.json` → `en`).
2. The namespace comes from the directory path. `routes/system/auth/login/en.json` becomes the namespace `system.auth.login`; `routes/en.json` becomes the namespace `routes` (a special name that holds the global strings).
3. The file contents are nested under that namespace in the language's tree.

For files inside a `translations/` subfolder, the `translations` segment is stripped — `routes/users/translations/en.json` becomes the namespace `users`, not `users.translations`.

The final structure looks like:

```js
{
    en: {
        routes: {                    // global strings from routes/en.json
            actions: { save: "Save", ... },
            errors:  { required: "Required", ... },
            nav:     { home: "Home", ... },
        },
        system: {
            auth: {                  // from routes/system/auth/en.json
                route_name: "auth",
                login: {             // from routes/system/auth/login/en.json
                    ui:      { title: "Login" },
                    fields:  { email: { label: "Email" }, ... },
                    actions: { submit: "Login" },
                    errors:  { invalid_email_or_password: "...", ... },
                },
            },
        },
        users: { ... }
    },
    sl: { ... }
}
```

`translated_from_request()` looks up the route's namespace (e.g. `system.auth.login`) and merges it on top of the global `routes` namespace.

<a name="fallback-merge"></a>

## Fallback Merge

After all files are loaded, the system fills in missing keys across languages so a partially-translated file doesn't produce empty strings. The rule is straightforward: for each language and each top-level namespace, any key that's missing or empty in that language is filled from any other language that has it.

The exception is `route_name` — it is _never_ inherited from another language. A missing `route_name` means "use the canonical English segment for the URL," not "use the Slovenian segment." This is what lets you ship the English version of an app before translating any URLs and still have working routes.

In practice, this means:

- **Untranslated strings show up in another language** rather than as blanks. You see them while testing and know they need translation.
- **A key absent in _every_ language** renders as its braced key path (`{namespace.key.path}`) — a loud, greppable marker rather than an empty string.
- **Untranslated URLs stay canonical**. You can add Slovenian support to a route incrementally — translate the strings first, leave the URLs canonical, add a localised `route_name` later.
- **The fallback is implicit, not configured.** There's no preferred-fallback-language setting; whichever language has the value provides it.

<a name="runtime-overrides"></a>

## The Database Is the Source of Truth

After the JSON seeds are loaded and the fallback merge runs, Reepolee merges the `translations` database table on top — and **DB values win.** Because a key can live in the database without existing in any JSON file, the table is the authoritative store: you add, correct, or remove strings there (through an admin UI, no redeploy) and the version-controlled seed SQL is just the baseline a fresh database starts from.

The table is simple:

| Column        | Purpose                                                                       |
| ------------- | ----------------------------------------------------------------------------- |
| `lang`        | Language code (`en`, `sl`, …)                                                 |
| `namespace`   | Dot-path namespace (`system.auth.login`, or empty/`root` for global `routes`) |
| `key_path`    | Dot-path within the namespace (`ui.title`, `actions.submit`)                  |
| `translation` | The string                                                                    |

To add or change a translation, do **one** of:

- **`UPDATE` / `INSERT` directly** against the `translations` table — fix an existing key or add a new one.
- **`/system/translations` admin UI** — browse namespaces and edit strings through the app (described [below](#translations-admin)).
- **`bun run sync:languages`** — AI-powered sync that scans the DB and fills missing keys across languages (described [below](#synchronising-keys)).

A few namespace conventions are handled specially during the merge (`lib/i18n.ts`):

- `key_path = "nav"` on a route namespace is placed at `routes.nav.{namespace}` so `nav_label()` resolves it.
- `key_path = "nav_prefix_title"` on a module namespace is placed at `routes.nav_prefix_title.{module}` so the layout can label navigation groups.
- An empty `namespace` (or `"root"`) targets the global `routes` namespace. **Root keys are fallbacks:** when a template references `{= actions.cancel }` and the route's own namespace lacks that key, the merge falls back to `root::actions.cancel`. This is why a handful of common keys live only in `root`.

The merge is wrapped in a try/catch — if the `translations` table doesn't exist yet (a fresh project before its first init), the DB layer is silently skipped and you run on the JSON seeds alone.

The seed SQL lives at `sql/<dialect>/02-init-translations-en.sql` and `sql/<dialect>/02-init-translations-sl.sql` (one file per language), and uses `INSERT IGNORE` / `INSERT OR IGNORE` so re-running it against an existing database never throws on duplicate keys.

<a name="translations-admin"></a>

### The Translations Admin Module

<media-frame label="Screenshot — translations admin module" ratio="16/9"></media-frame>

The system module at `/system/translations` (route `routes/system/translations/`) is a CRUD UI over the `translations` table. It lets an operator browse namespaces, edit any string, add overrides, and delete them — all writing to the DB layer described above. After a change it calls `reload_all_translations()` and rebuilds the route maps so the new value (and any changed `route_name`) takes effect without a restart.

This is why most route folders in a Reepolee project no longer ship `en.json` / `sl.json` files: their strings live in the `translations` table and are seeded from `sql/<dialect>/02-init-translations-<lang>.sql`. A JSON file next to a route still works as a seed, but the DB is authoritative and overrides it. To move existing JSON files into the table, run `bun generator/migrate_translations_to_db`.

<a name="pluralization"></a>

## Pluralization

For counts, a single translation string holds all plural forms separated by pipes, and the `plural()` helper in `$lib/helpers` picks the right one using `Intl.PluralRules` for the active locale:

```ts
import { plural } from "$lib/helpers";

// translation value: "no items|one item|{count} items|{count} items|{count} items"
plural(props.messages.item_count, count, locale); // → "3 items"
```

The string is five pipe-separated forms (matching the CLDR plural categories); the helper selects by category and substitutes `{count}`. `format_bulk_delete_message()` builds on this to compose a deleted-count message plus a pluralized error suffix — used by the generated bulk-delete handlers.

<a name="synchronising-keys"></a>

## Synchronising Keys Across Languages

Because the database is authoritative, key-sync works against the DB, not JSON files. `sync:languages` scans the `translations` table for every namespace, finds keys that exist in one language but are missing in another, translates them with an LLM, and writes the results **back to the database**:

```bash
bun run sync:languages
```

(This is `bun generator/sync_translations.ts --translate` under the hood — see [Generators](/database/generators#translations) for the `--translate` flag in the wider generator context.) You still review the result through the admin UI, but the first pass is automatic. There is no `::missing::` placeholder step and no per-file editing.

<a name="maintenance-prune-sync"></a>

## Maintenance: Pruning and Filling Gaps

Two interactive tools in the TUI (`bun tui` → **Tools & Maintenance**) keep the `translations` table aligned with what the templates actually reference. Both scan every `.ree` template for `{= … }` references, map each file to its DB namespace by path, compare against the DB, and **write a timestamped `.sql` file for you to review before applying** — neither tool touches the database directly.

| TUI command                   | Direction                              | Output                                                                 |
| ----------------------------- | -------------------------------------- | ---------------------------------------------------------------------- |
| **Prune unused translations** | DB keys not referenced by any template | `DELETE` statements for the orphaned `(namespace, key_path)` rows      |
| **Sync missing translations** | Template refs not present in the DB    | `INSERT` statements (empty values, one per active language) to fill in |

Apply the generated file via TUI → **Database & Config** → **Run SQL file**, or pipe it to your DB CLI:

```bash
mysql -u root -p < prune_translations_<timestamp>.sql
```

A few safeguards worth knowing:

- **Root fallbacks are protected.** If _any_ template references a `key_path`, the prune tool marks the corresponding `root::key_path` as in-use, so global fallback keys aren't deleted.
- **Indirect keys are protected.** `nav`, `nav_prefix_title`, `route_name`, `parent_label`, `_tags` and the `modules_tags.*` keys are resolved through helpers like `nav_label()` rather than literal `{= … }` refs, so they're never pruned.
- **Dynamic references can't be detected statically** (`{= labels[key] }`) and may show up as false positives — always review the preview before running the SQL. `components/` is skipped because its placeholder keys (`__field.name__`) don't match real DB entries.

<a name="adding-new-strings"></a>

## Adding New Strings

The flow for adding a string to an existing route:

1. **Reference it in your template** — `{= props.your_new_key }` (escaped) or `{~ props.your_new_key }` if it contains trusted HTML. Until the key exists, it renders as its braced key path, so the gap is loud.
2. **Add the key to the database.** Insert one row per language into `translations`, or run **Sync missing translations** (above) to scaffold empty `INSERT`s for every active language, then fill the values through `/system/translations`.
3. **Read it in your handler.** `translated.your_new_key` is available as soon as you've spread `translated` into `data` — no extra wiring.

No build step, no extraction tool, no codegen. For a global string that should be available on every page, add it under the `root` namespace (empty `namespace`); for a string specific to one route, use that route's namespace.

<a name="reloading-translations-in-development"></a>

## Reloading Translations in Development

Translation files are read at server startup. A change to `en.json` while the server is running won't appear until the server reloads. In development, the file watcher already restarts the process on file changes, so editing a translation file and refreshing the page picks up the new value automatically.

For manual reloads (without restarting), call `reload_all_translations()` from `$lib/i18n`:

```ts
import { reload_all_translations } from "$lib/i18n";

// in a development-only route
export async function get_dev_reload_translations(req: BunRequest): Promise<Response> {
	await reload_all_translations();
	return new Response("Reloaded", { status: 200 });
}
```

This is useful for editors that don't trigger Bun's file watcher (a remote editor, a script that writes the file from outside the project). For local development with the default watcher, you don't need it.

The server also exposes an HTTP hot-reload endpoint, **`/__reload-translations`**, which calls `reload_all_translations()` in the running process — used by the generators and the queue worker to pick up new translations (JSON or DB) without a restart. When the `RELOAD_SECRET` environment variable is set, the endpoint requires a matching `X-Reload-Secret` header; leave it unset to disable the endpoint in environments where you don't want it reachable. The translations admin module ([above](#translations-admin)) triggers this reload automatically after every edit.
