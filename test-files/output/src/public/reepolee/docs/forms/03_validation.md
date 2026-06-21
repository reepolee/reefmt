---
title: "Validation"
---

# Validation

<a name="introduction"></a>

## Introduction

Reepolee uses [Zod](https://zod.dev) for all validation. The library is vendored at `vendor/zod.min.js` so it's part of your codebase — no registry dependency, no surprise version bumps. Each route folder has a `validation_server.ts` that owns three schemas for that resource: one for parsing database rows on list pages, one for parsing rows into edit-form shape, and one for validating user input. Error messages are written as translation keys so the same schema works in every supported language.

The same schema runs in two places: on the server when the form is submitted, and on a JSON validate endpoint that `FormController` calls on every blur. The server check is the source of truth; the live check exists for UX.

<a name="the-three-schemas"></a>

## The Three Schemas

### index_schema

Parses raw database rows for list views. Most fields are optional or nullable because the list page only needs to display what's there:

```ts
export const index_schema = z.object({
	id: z.coerce.number().optional(),
	email: z.string(),
	name: z.nullable(z.string().optional()),
	verified_at: z.nullable(z.string().optional()),
	tags: z.nullable(z.string().optional()),
});
```

### form_schema

Parses a single record into the shape an edit form expects. This is where you apply codecs that convert database timestamps into the format HTML date inputs need:

```ts
export const form_schema = z.object({
	id: z.coerce.number().optional(),
	email: z.string(),
	verified_at: z.nullable(datetime_codec.optional()),
	created_at: z.nullable(z.coerce.date().optional()),
	updated_at: z.nullable(z.coerce.date().optional()),
});

// in the route handler:
const raw = await get_record_by_id(id);
const [, record] = validate(raw);
```

The generated list handlers also run every row through `index_schema` before passing them to the template, so columns reach the template in the same TypeScript types the schema declares (numbers as numbers, dates as `Date`, nullable strings normalised). This keeps the template free of ad-hoc `String()`/`Number()` coercion and means the list view and the edit form agree on field types.

### schema

Validates user input. Business rules live here — required fields, format checks, minimum lengths. Error messages are translation keys, not literal strings:

```ts
export const schema = z.object({
	id: z.coerce.number().optional(),
	email: z.string().min(1, "email_required"),
	name: z.nullable(z.string().optional()),
	tags: z.nullable(z.string().optional()),
});
```

<a name="running-validation"></a>

## Running Validation

`validation_server.ts` exports two convenience wrappers around `validate_schema()`:

```ts
export const validate = (data, messages) => validate_schema(schema, data, undefined, messages);

export const validate_touched = (data, touched, messages) => validate_schema(schema, data, touched, messages);
```

Both return a tuple `[errors, valid_data]`. If validation passes, `errors` is `{}` and `valid_data` is the parsed input. If it fails, `errors` is keyed by field name and `valid_data` is `null`:

```ts
import { validate } from "./validation_server";

const [errors, valid_data] = validate(data, translated.errors);

if (Object.keys(errors).length > 0) {
	return render("users/form", {
		data: { record: data, errors, ...translated },
		ctx,
	});
}

await create_record(valid_data);
return Response.redirect("/users", 303);
```

The second argument — `translated.errors` — is the translation map for the active language. `validate_schema()` looks up each Zod error message string (the translation key) in that object and replaces it with the human-readable text. The key `"email_required"` becomes whatever lives at `errors.email_required` in the active language file.

Use `valid_data` (the parsed, type-coerced object) — not the raw `data` you started with — when you hand the input on to the database or other services. `valid_data` has numbers coerced from strings, dates decoded into `Date`, optional empty strings normalised to `null`, and any field that isn't in the schema dropped. The shipped auth handlers (`login`, `register`, `password`, `profile`) all read `_valid_data.email`, `_valid_data.password`, etc. on the success path for that reason.

<a name="partial-validation"></a>

## Partial Validation

`validate_touched` only reports errors for the fields listed in the `touched` array. This is what the live validation endpoint uses — when a user has just tabbed off the email field, you only want to surface the email error, not flag every other empty required field:

```ts
export async function post_users_validate(req: BunRequest): Promise<Response> {
	const body = await req.json();
	const touched: string[] = body.touched || [];
	const [errors] = validate_touched(body, touched, translated.errors);

	return new Response(JSON.stringify({ success: Object.keys(errors).length === 0, errors }), {
		headers: { "Content-Type": "application/json" },
	});
}
```

The response shape — `{ success: boolean, errors: Record<string, string> }` — is what `FormController` expects. The same endpoint serves the live per-field validation and the all-fields validation that runs on submit.

<a name="client-side-form-controller"></a>

## Client-Side: FormController

`FormController` is a small vanilla JS class in `static/form-controller.js` that wires up live validation without any framework. Point it at the form element and its validate endpoint:

```html
<script src="/form-controller.js" defer></script>
<script>
	document.addEventListener("DOMContentLoaded", () => {
		new FormController({
			form: "#edit-form",
			validate_url: "/users/validate",
		});
	});
</script>
```

From that point on, `FormController` does three things:

1. **Tracks input values** — it caches the form's initial values and listens to `input` events on every named input, textarea, and select.
2. **Validates on blur** — when the user tabs off a field (`focusout`), it posts the current values plus a `touched: [field_name]` array to `validate_url` and writes the response's `errors[field_name]` into the `<validation-error>` element with id `error-{field_name}`.
3. **Validates on submit** — when the form is submitted, it `preventDefault`s, posts all fields to `validate_url`, and only allows the native form submission to fire if the server returns `success: true`. If there are errors, they're rendered inline and the POST to the server never happens.

<a name="the-validation-error-element"></a>

### The validation-error Element

<media-frame label="Screenshot — inline field error appearing after blur" ratio="16/9"></media-frame>

Validation errors render inside a `<validation-error>` custom element. The element is a thin shadow-DOM wrapper that displays its slotted content and re-renders when the content changes:

```html
<input type="text" id="email" name="email" value="{= props.record.email }" />
<validation-error id="error-email"> {= props.errors.email } </validation-error>
```

Two things happen here:

- **On first render**, the server fills in the error from `props.errors.email`. If the user submitted the form with JavaScript disabled, this is the only validation feedback they see.
- **On live validation**, `FormController` finds `#error-email` and replaces its `innerHTML` with the new error string.

The element renders nothing when its content is empty, so an unused `<validation-error>` is invisible.

The convention `id="error-{field_name}"` is what makes the wiring automatic — there is no registration step beyond matching the id. All of the shipped input components follow this convention.

<a name="error-messages-and-translations"></a>

## Error Messages and Translations

Validation error messages live as keys in `routes/en.json` under the `errors` object:

```json
{
	"errors": {
		"required": "Required",
		"email_required": "Email is required",
		"email_invalid": "Must be a valid email address",
		"duplicate_key": "Code already exists"
	}
}
```

Your Zod schema references these keys as the message string:

```ts
email: z.string().min(1, "email_required").email("email_invalid"),
```

At validation time, `validate_schema()` swaps each key for the translated string. Add a new key to your language files and reference it in the schema — translation flows automatically. The full localisation story is in [Translations](/i18n/translations).

<a name="date-codecs"></a>

## Date Codecs

Database timestamps and HTML date inputs use different formats. `$lib/validation_helpers` exports codecs (built on Zod's `z.codec()`) that handle conversion in both directions. Apply them in `form_schema` for fields that need the transformation:

| Codec             | Use case                                                        |
| ----------------- | --------------------------------------------------------------- |
| `date_codec`      | `DATE` column ↔ `date` input (`YYYY-MM-DD`)                     |
| `datetime_codec`  | `DATETIME` column ↔ `datetime-local` input (`YYYY-MM-DDTHH:mm`) |
| `timestamp_codec` | `TIMESTAMP` column ↔ ISO string                                 |
| `empty_string`    | Treat empty strings as `null` on both sides                     |

Note: the codecs are driver-agnostic — both SQLite and MySQL use the same encoders. Bun's native `SQL` API normalises the row shape for you.

```ts
import { datetime_codec } from "$lib/validation_helpers";

export const form_schema = z.object({
	verified_at: z.nullable(datetime_codec.optional()),
});
```

The codec's `decode` direction converts the raw database value to the HTML-friendly format. The `encode` direction handles the reverse when writing back to the database.

<a name="custom-rules"></a>

## Custom Rules

The schemas the generator produces cover the common cases. Project-specific rules — "country code must be one of SI, DE, HR", "release date must be in the future", "this user can't be assigned to this team" — go in your route handler after the standard validation runs:

```ts
const [errors, valid_data] = validate(data, translated.errors);

if (!["SI", "DE", "HR"].includes(valid_data.country_code)) {
	errors.country_code = translated.errors.country_invalid;
}

if (Object.keys(errors).length > 0) {
	return render("users/form", { data: { record: data, errors, ...translated }, ctx });
}
```

You can also add the check inside the Zod schema with `.refine()` if it's purely shape-based, but for rules that depend on database lookups (uniqueness, foreign-key existence) the handler is the right place.
