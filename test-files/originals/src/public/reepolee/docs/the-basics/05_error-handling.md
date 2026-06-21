---
title: "Error Handling"
---

# Error Handling

<a name="introduction"></a>

## Introduction

Reepolee does not ship with a global exception filter or error-handling layer to learn. Errors are handled where they happen — in the route handler that knows what the user was trying to do and what the right next step is. This page covers the four cases you'll meet most often: validation failures, missing records, database constraint violations, and uncaught exceptions.

<a name="validation-errors"></a>

## Validation Errors

When a form submission fails validation, re-render the same form with the errors and the values the user entered. The full validation flow is documented in [Validation](/forms/validation); the controller pattern is straightforward:

```ts
import { validate } from "./validation_server";

const [errors, valid_data] = validate(data, translated.errors);

if (Object.keys(errors).length > 0) {
	return render("users/form", {
		data: {
			record: data,
			errors,
			action: "/users",
			...translated,
		},
		ctx,
	});
}

await create_record(valid_data);
return Response.redirect("/users", 303);
```

`errors` is a flat object keyed by field name. Spread it into your template's `data` and the per-field `<validation-error>` components pick it up automatically.

<a name="not-found"></a>

## Not Found

When a record lookup fails, return a `404` by rendering the `notfound` template with a `status` override:

```ts
const record = await get_record_by_id(req.params.id);

if (!record) {
	return render("notfound", {
		data: { title: "404 Not Found" },
		status: 404,
		ctx,
	});
}
```

`render()` carries the status through to the `Response`, so search engines and clients see a real 404 rather than a 200 with a "not found" page.

<a name="the-fallback-404"></a>

### The Fallback 404

<media-frame label="Screenshot — 404 not-found page" ratio="16/9"></media-frame>

URLs that don't match any registered route fall through to the `fetch` function in `server.ts`, which serves static files and finally renders the same `notfound` template. The shape is:

```ts
async fetch(req) {
    const url = new URL(req.url);

    // Serve files from static/
    const file_path = join(static_dir, url.pathname);
    if (await Bun.file(file_path).exists() && file_path.startsWith(static_dir)) {
        return new Response(Bun.file(file_path), {
            status: 200,
            headers: static_headers,
        });
    }

    // 404 fallback — build ctx so the layout still has lang/user
    const ctx = await create_ctx(req);
    return render("notfound", {
        data: { title: "404 Not Found" },
        status: 404,
        ctx,
    });
}
```

Customising the fallback is just editing `routes/notfound.ree`. The same template is rendered both from handler-level 404s and the global fallback, so a single change updates both.

<a name="database-errors"></a>

## Database Errors

Wrap write operations that can fail for predictable reasons in a try/catch and inspect the error message to decide how to respond. The two cases that come up daily are unique-key violations on insert and foreign-key violations on delete:

```ts
try {
	await delete_record(id);
	return Response.redirect("/users", 303);
} catch (error) {
	const error_message =
		error instanceof Error && error.message.includes("foreign key")
			? translated.errors.foreign_key_error
			: translated.errors.error_deleting_record;

	return render("users/form", {
		data: { record, form_errors: error_message, errors: {}, ...translated },
		ctx,
	});
}
```

The error messages come from the language file rather than being hard-coded — the same try/catch works in every supported language without modification.

For unique-key violations on create, look for the driver-specific substring in the error message (`"UNIQUE constraint failed"` for SQLite, `"Duplicate entry"` for MySQL) and surface a friendly message tied to the offending field.

<a name="uncaught-exceptions"></a>

## Uncaught Exceptions

Anything thrown from a handler that isn't caught propagates to Bun's HTTP server, which returns a `500` with the error message in the body. In development that's usually what you want — the message is right there in the browser. For production you can pass an `error` handler to `Bun.serve()` that renders a friendlier page:

```ts
Bun.serve({
	routes: routed,
	async fetch(req) {
		/* ... */
	},
	error(error) {
		console.error(error);
		return new Response("Something went wrong", { status: 500 });
	},
});
```

Replace the plain text body with a rendered `error` template once your project has one. Keep the body small and avoid leaking stack traces to end users in production.

<a name="logging"></a>

## Logging

`console.log`, `console.error`, and `console.warn` are forwarded to `journalctl` when Reepolee runs as a systemd service in production. SQL queries log to `logs/sql.ndjson` when `SQL_LOGGING=true` is set — see [Logs](/deployment/logs) for the production view of these.

For a quick manual probe in development, `Bun.color()` adds ANSI colour to console output without pulling in a dependency:

```ts
console.log(Bun.color("red", "ansi") + "validation failed for", data.email);
```
