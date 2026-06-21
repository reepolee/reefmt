---
title: "Controllers"
---

# Controllers

<a name="introduction"></a>

## Introduction

In Reepolee, a controller is a plain async function. There is no class to extend, no base controller to inherit from. Each route handler lives in `routes/<feature>/index.ts` alongside the templates, queries, and translations it works with.

Every handler receives a `BunRequest` and must return a `Response`. Everything else — what data you load, how you validate it, what you render — is up to you.

```ts
export async function get_users_index(req: BunRequest): Promise<Response> {
	const ctx = await create_ctx(req);
	const translated = await translated_from_request(req, import.meta.dir);
	const records = await get_all_records();

	return render("users/index", {
		data: { records, ...translated },
		ctx,
	});
}
```

That's the whole pattern. Build the request context, load the translations, load the data, call `render()`. Reading the request body, building responses, and handling errors each have their own pages — see [Requests & Responses](/the-basics/requests-responses) and [Error Handling](/the-basics/error-handling).

<a name="the-request-context"></a>

## The Request Context

`create_ctx(req)` from `$lib/request_context` returns a `RequestContext` populated with everything the rest of the request needs in one place:

| Field            | Source                                                                  |
| ---------------- | ----------------------------------------------------------------------- |
| `req`            | The raw `BunRequest`                                                    |
| `request_url`    | `pathname + search` of the incoming URL                                 |
| `prefix`         | The URL prefix the route is mounted under (e.g. `admin`) or `null`      |
| `lang`           | Active language from `X-Lang` header → `lang` cookie → default          |
| `locale`         | Locale string for the active language (e.g. `en-US`)                    |
| `preferred_lang` | The user's explicit cookie language preference, if any                  |
| `user`           | The current user row from `resolve_session()` — `null` if not signed in |
| `toasts`         | Toast notifications (populated by `with_toasts` middleware when used)   |

Resolving the session inside `create_ctx` means every handler that builds a `ctx` gets the current user for free with a single database hit. The layout, navbar, and any template that reads `props.user` work without each handler having to wire the session up.

<a name="rendering-a-response"></a>

## Rendering a Response

The `render()` helper from `$lib/render` compiles a `.ree` template and returns an HTML `Response`. Pass the template path and an options object with your data and the incoming request:

```ts
import { render } from "$lib/render";
import { create_ctx } from "$lib/request_context";

export async function get_users_edit(req: BunRequest): Promise<Response> {
	const ctx = await create_ctx(req);
	const id = req.params.id;
	const record = await get_record_by_id(id);
	const translated = await translated_from_request(req, import.meta.dir);

	return render("users/form", {
		data: {
			title: `Edit ${record.email}`,
			record,
			action: `/users/${record.id}/edit`,
			...translated,
		},
		ctx,
	});
}
```

`ctx` is what populates the current user, active language, locale, prefix, and any pending toast notifications in the template. The full options object:

| Option    | Type             | Description                                                                                         |
| --------- | ---------------- | --------------------------------------------------------------------------------------------------- |
| `data`    | `object`         | Template-specific data merged on top of the global base data                                        |
| `ctx`     | `RequestContext` | Output of `create_ctx(req)` — populates `user`, `lang`, `locale`, `prefix`, `toasts`, `request_url` |
| `status`  | `number`         | HTTP status code, defaults to `200`                                                                 |
| `headers` | `object`         | Extra response headers                                                                              |

The full set of values that `render()` injects into every template — `user`, `lang`, `locale`, `toasts`, `request_url`, and so on — is covered in [Helpers & Globals](/ree-templates/helpers-and-globals).

<a name="translations"></a>

## Translations

Call `translated_from_request()` at the top of any handler that renders a template. It reads the active language from the request and merges the global and route-level translation files into a single flat object:

```ts
import { translated_from_request } from "$lib/helpers";

export async function get_users_index(req: BunRequest): Promise<Response> {
	const ctx = await create_ctx(req);
	const translated = await translated_from_request(req, import.meta.dir);

	return render("users/index", {
		data: {
			title: translated.ui.title,
			...translated,
		},
		ctx,
	});
}
```

Spread `translated` into `data` and your template has access to every label, heading, and error message without any additional lookups. The full localisation flow — adding a language, layering route-level and global keys, generating localised URLs — is covered in [Translations](/i18n/translations).

<a name="the-action-convention"></a>

## The \_action Convention

HTML forms only support `GET` and `POST`. For destructive operations like delete, Reepolee uses a `_action` hidden field to communicate intent through the same `POST` handler that processes updates:

```html
<form id="delete-form" method="POST" action="/users/{= props.record.id }/edit">
	<input type="hidden" name="_action" value="delete" />
</form>
```

The `POST` handler checks `_action` first and branches accordingly:

```ts
export async function post_users_edit(req: BunRequest): Promise<Response> {
	const params = new URLSearchParams(await req.text());
	const action = params.get("_action");

	if (action === "delete") {
		await delete_record(req.params.id);
		return Response.redirect("/users", 303);
	}

	// otherwise treat as an update
	// ...
}
```

This keeps the route table small — one URL per resource action rather than separate `/edit`, `/delete`, `/restore` endpoints — and matches the pattern used throughout the generated CRUD code.

<a name="confirming-feedback"></a>

## Confirming Feedback to the User

After a successful create, update, or delete, redirect with a [toast notification](/forms/toast-notifications) so the next page load surfaces the result. Toasts are sent as cookies on the redirect response and consumed by the `<toasts-area>` web component in your layout — the controller does not have to know anything about how they're displayed.
