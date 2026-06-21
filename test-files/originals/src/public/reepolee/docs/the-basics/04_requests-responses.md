---
title: "Requests & Responses"
---

# Requests & Responses

<a name="introduction"></a>

## Introduction

Every Reepolee handler takes a `BunRequest` and returns a `Response`. Both types are part of the standard fetch-style API that Bun's HTTP server exposes, so anything you'd do with `Request` and `Response` in a worker or edge function works the same way here. This page covers the request-side helpers you'll reach for daily — URL parameters, query strings, and form bodies — and the most common patterns for building responses, including redirects.

<a name="url-parameters"></a>

## URL Parameters

Routes that include `:param` segments expose those values on `req.params`. The names match the segment names in the route definition:

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

Values arrive as strings. Coerce them yourself if you need a number:

```ts
const id = Number(req.params.id);
```

<a name="extract-params-helper"></a>

### The extract_params_from_url Helper

Generated CRUD handlers use a small helper that pulls one or more numeric IDs from the URL path in order. It's useful when you have nested routes like `/projects/:project_id/tasks/:task_id` and want to destructure both at once:

```ts
import { extract_params_from_url } from "$lib/helpers";

const [project_id = 0, task_id = 0] = extract_params_from_url(req, 2);
```

The first argument is the request, the second is how many IDs to extract (defaults to `1`). For prefixed routes mounted via `mount_prefix`, pass the number of prefix segments as the third argument so the helper skips them.

<a name="query-strings"></a>

## Query Strings

Read query parameters directly from the URL:

```ts
const url = new URL(req.url);
const query = url.searchParams.get("query") || "";
const offset = parseInt(url.searchParams.get("offset") || "0", 10);
```

The list view in every generated CRUD module reads `query`, `sort`, `direction`, `offset`, and `limit` this way, which is what makes filtered views bookmarkable.

<a name="form-bodies"></a>

## Form Bodies

For `POST` handlers that receive `application/x-www-form-urlencoded` (the default for HTML forms without an `enctype`), read the body as text and parse it with `URLSearchParams`:

```ts
export async function post_users_index(req: BunRequest): Promise<Response> {
	const body = await req.text();
	const params = new URLSearchParams(body);

	const data = {
		email: params.get("email")?.trim() || "",
		name: params.get("name")?.trim() || "",
	};
	// ...
}
```

For multipart forms (file uploads) use `req.formData()` — see [File Uploads](/forms/file-uploads).

<a name="json-bodies"></a>

## JSON Bodies

For endpoints called from JavaScript with `Content-Type: application/json`, use `req.json()`:

```ts
export async function post_users_validate(req: BunRequest): Promise<Response> {
	const body = await req.json();
	const touched: string[] = body.touched || [];
	// ...
}
```

The live validation endpoint that `FormController` calls on every blur uses exactly this shape — see [Validation](/forms/validation).

<a name="building-responses"></a>

## Building Responses

The `render()` helper returns a `Response` with `Content-Type: text/html` and status `200` by default. To override either, pass them in the options object:

```ts
return render("dashboard/index", {
	data: { user },
	status: 200,
	headers: {
		"Cache-Control": "no-store",
		"X-Frame-Options": "DENY",
	},
	ctx,
});
```

For non-HTML responses, construct the `Response` directly:

```ts
return new Response(JSON.stringify({ ok: true }), {
	headers: { "Content-Type": "application/json" },
});
```

```ts
return new Response("plain text reply", {
	headers: { "Content-Type": "text/plain" },
});
```

`Bun.file()` is the cleanest way to stream a file back — set the appropriate `Content-Type` and Bun handles the rest:

```ts
return new Response(Bun.file("/tmp/report.pdf"), {
	headers: { "Content-Type": "application/pdf" },
});
```

<a name="redirects"></a>

## Redirects

Use `Response.redirect()` with status `303` after a successful write operation. `303 See Other` ensures the browser follows the redirect with a `GET` request regardless of the original method, preventing duplicate submissions on refresh:

```ts
return Response.redirect("/users", 303);
```

If your form included a `_return_url` hidden field — populated by the client with the URL the user came from — you can redirect back to that page after a save:

```ts
const return_url = params.get("_return_url");
const redirect_url = return_url?.includes("/users") ? return_url : "/users";
return Response.redirect(redirect_url, 303);
```

Always validate the `_return_url` value before using it. The check above ensures the URL belongs to the same feature and prevents an open-redirect issue.

<a name="redirects-with-cookies"></a>

### Redirects That Set Cookies

`Response.redirect()` does not let you attach extra headers, so redirects that need to set a cookie (like a [toast notification](/forms/toast-notifications) or a session cookie after login) are built manually:

```ts
const headers = new Headers({ Location: "/users" });
headers.append("Set-Cookie", session_cookie.toString());

return new Response(null, { status: 303, headers });
```

The body is `null`, the status is `303`, and any number of `Set-Cookie` headers can be appended.
