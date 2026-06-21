---
title: "Email Module"
---

# Email Module

<a name="introduction"></a>

## Introduction

`routes/examples/email/` is the shipped example of putting `send_mail()` in front of a form. A small composer form takes a recipient, subject, body, and (optional) HTML; submitting either sends the message immediately or, when the worker is running, enqueues it for the worker to pick up. The composer mirrors a transactional flow without the database-backed audit log a real admin tool would add — that part is left to your project.

The example is not a complete email-admin module — there is no `email` table in the shipped schema, no draft/sent state, and no resend flow. Treat the rest of this page as a design pattern for building one when you need it.

<a name="the-email-table"></a>

## A Suggested Email Table

If you want an audit log of every message your application sends, a narrow table is enough. The intended shape:

```sql
CREATE TABLE email (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    to_address TEXT NOT NULL,
    cc TEXT DEFAULT '' NULL,
    bcc TEXT DEFAULT '' NULL,
    subject TEXT NOT NULL,
    body TEXT DEFAULT '' NULL,
    html TEXT DEFAULT '' NULL,
    sent_at DATETIME DEFAULT NULL,
    created_at DATETIME DEFAULT current_timestamp,
    updated_at DATETIME DEFAULT NULL
);
```

Each row is one message. `to_address` is named with the `_address` suffix because `to` is a reserved word in some SQL dialects. The `body` and `html` columns store both versions of the message — keeping both means the audit log can render exactly what the recipient saw, even if you change the email template later.

`sent_at` is `NULL` for messages that haven't been sent yet (drafts) and a timestamp for ones that have.

If you add this table to your project's schema (`sql/sqlite/01-init-sqlite.sql` / `sql/mysql/01-init-mysql.sql`), also add `"email"` to `IGNORE_TABLES` in `config/db_structure.ts` so the resource generator doesn't overwrite the hand-written compose form with default scaffolding.

<a name="the-shipped-example"></a>

## The Shipped Example

`routes/examples/email/index.ts` registers:

| Method | Path                       | Purpose                                   |
| ------ | -------------------------- | ----------------------------------------- |
| `GET`  | `/examples/email`          | Empty composition form                    |
| `POST` | `/examples/email`          | Send (or enqueue) the message             |
| `POST` | `/examples/email/validate` | Live per-field validation                 |
| `GET`  | `/examples/email/confirm`  | Confirmation page after a successful send |

The POST handler checks whether the in-process worker is alive (`is_worker_alive()` from `$queue/index`) — when it is, the message is added to the queue; when it isn't, the handler calls `send_mail()` directly on the request. Either way the user lands on the confirm page.

For a real admin tool, mount your own CRUD behind module authentication so non-admins can't compose mail:

```ts
import { mount_prefix, require_module_mw } from "$lib/middleware";
import { email_crud } from "$routes/email";

export const routes = {
	"/": home_page,
	...mount_prefix("/admin", { ...email_crud }, require_module_mw("admin")),
};
```

That gates `/admin/email`, `/admin/email/new`, and the rest behind the `admin` module. See [Authorization](/security/authorization).

<a name="composing-with-pell"></a>

## Composing With Pell

<media-frame label="Screenshot — email composer (Pell rich-text editor)" ratio="16/9"></media-frame>

The compose form in `routes/examples/email/form.ree` includes a small WYSIWYG editor — [Pell](https://github.com/jaredreich/pell) — for the HTML body. It loads from a CDN (no install step), provides the standard formatting controls (bold, italic, lists, headings, links), and writes the resulting HTML to a hidden input that submits with the form:

```html
<link rel="stylesheet" type="text/css" href="https://unpkg.com/pell@1.0.6/dist/pell.min.css" />
<script src="https://unpkg.com/pell@1.0.6/dist/pell.min.js"></script>

<field-wrapper class="grid">
	<label class="px-3">{= props.fields.html.label }</label>
	<div id="html-body" class="input pell"></div>
	<validation-error class="mt-1" id="error-html"></validation-error>
</field-wrapper>

<input type="hidden" name="html" id="html" />

<script>
	const editor = pell.init({
		element: $("#html-body"),
		onChange: (html) => {
			$("#html").value = html;
		},
		defaultParagraphSeparator: "p",
		actions: [
			"bold",
			"italic",
			"underline",
			"strikethrough",
			"heading1",
			"heading2",
			"paragraph",
			"olist",
			"ulist",
			"link",
			"image",
		],
	});
</script>
```

Pell is small (~3 KB) and depends on nothing. If you'd rather use a richer editor (TinyMCE, Quill, ProseMirror), the integration shape is the same — initialise the editor on a div, write the rendered HTML to the hidden `name="html"` input, submit the form. Reepolee doesn't care which editor produced the HTML.

<a name="save-then-send-pattern"></a>

## The Save-Then-Send Pattern

A typical send-from-handler flow saves the row first and then calls `send_mail()`. Saving first means the audit log captures the message even if the SMTP send fails:

```ts
import { send_mail } from "$lib/smtp";
import { create_record, update_record } from "$routes/email/sql";

const record = await create_record({
	to_address: user.email,
	subject: translated.welcome_subject,
	body: translated.welcome_body,
	html: rendered_html,
	sent_at: null,
	cc: "",
	bcc: "",
});

try {
	await send_mail({
		to: record.to_address,
		subject: record.subject,
		body: record.body,
		html: record.html,
	});
	await update_record(record.id, { sent_at: new Date().toISOString() });
} catch (error) {
	console.error("Email send failed for record", record.id, error);
	// record stays with sent_at = NULL; admin can retry from the UI
}
```

If `send_mail()` throws, the row is still in the database with `sent_at = NULL`. An admin can find it in the list view (filtering for unsent messages) and re-send from the edit form. Without the save-first step, a transient SMTP failure would lose the message entirely.

For automated sends — registration emails, password resets, system notifications — this pattern lets the audit log answer "what messages did the system send and when?" without you maintaining a separate log file.

<a name="resending"></a>

## Resending a Failed Message

The simplest "retry" handler reads the row, calls `send_mail()`, and stamps `sent_at` on success:

```ts
export async function post_email_resend(req: BunRequest): Promise<Response> {
	const ctx = await create_ctx(req);
	const auth_ctx = await resolve_session(req);
	const guard = require_module(auth_ctx, "admin");
	if (guard) return guard;

	const id = Number(req.params.id);
	const record = await get_record_by_id(id);
	if (!record) return render("notfound", { status: 404, ctx });

	try {
		await send_mail({
			to: record.to_address,
			subject: record.subject,
			body: record.body,
			html: record.html,
		});
		await update_record(id, { sent_at: new Date().toISOString() });
	} catch (error) {
		const cookie = await create_toast_cookie({
			record_id: id,
			feature: "email",
			message: translated.errors.email_send_failed,
			type: "red",
			req,
		});
		const headers = new Headers({ Location: `/admin/email/${id}/edit` });
		headers.append("Set-Cookie", cookie.toString());
		return new Response(null, { status: 303, headers });
	}

	return Response.redirect(`/admin/email/${id}/edit`, 303);
}
```

Wire it as a `POST` handler with `_action=resend` on the existing edit endpoint, or as a separate route — either works.

<a name="exclude-from-generators"></a>

## Excluding the Email Table

Because the email module is hand-written (rather than generated), the `email` table is in the `IGNORE_TABLES` constant in `config/db_structure.ts`:

```ts
export const IGNORE_TABLES = ["modules", "sessions", "email", "images", "users"] as const satisfies readonly string[];
```

This means `bun generator/resource all` skips the `email` table — it doesn't try to scaffold a new CRUD module that would conflict with the existing one. If you remove the `email` entry from `IGNORE_TABLES` and run the generator, you'll get the default scaffolding (no Pell editor, no send action) and overwrite the customised form.

For other tables you want to manage by hand rather than generate, add them to the same list. The generator gives you a starting point; the moment you customise the result, exclude the table to keep your changes safe from the next regeneration.
