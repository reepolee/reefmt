---
title: "Sending Email"
---

# Sending Email

<a name="introduction"></a>

## Introduction

`send_mail()` from `$lib/smtp` is the one function you call to send mail from a Reepolee application. It takes an options object, opens an SMTP connection, sends the message, and resolves when the recipient server has accepted it. There's no library to learn beyond the options table below.

This page covers the call shape, multi-recipient handling, HTML messages, and the small set of patterns worth knowing for production use. SMTP credentials and provider choice are covered in [SMTP Configuration](/email/smtp-configuration); the database-backed audit log is in [Email Module](/email/email-module).

<a name="basic-usage"></a>

## Basic Usage

Import `send_mail` and call it from any handler:

```ts
import { send_mail } from "$lib/smtp";

await send_mail({
	to: "user@example.com",
	subject: "Welcome to Reepolee",
	body: "Plain text fallback.",
	html: "<h1>Welcome!</h1><p>Your account is ready.</p>",
});
```

The function returns a `Promise<void>`. `await` it so any failure surfaces as a thrown error in your handler. Forgetting the `await` gives you a fire-and-forget that swallows errors silently.

<a name="options"></a>

## Options

The full options object:

| Option                   | Type                 | Description                                                      |
| ------------------------ | -------------------- | ---------------------------------------------------------------- |
| `to`                     | `string \| string[]` | One or more recipient addresses                                  |
| `cc`                     | `string \| string[]` | Carbon copy recipients                                           |
| `bcc`                    | `string \| string[]` | Blind carbon copy recipients                                     |
| `subject`                | `string`             | The subject line                                                 |
| `body`                   | `string`             | Plain text content                                               |
| `html`                   | `string`             | HTML content — sent as `multipart/alternative` alongside `body`  |
| `tls.rejectUnauthorized` | `boolean`            | Whether to reject invalid TLS certificates (defaults to `false`) |

`subject` and `body` are required. Everything else is optional.

<a name="multipart-html"></a>

## Plain Text + HTML

When both `body` and `html` are provided, the message is sent as `multipart/alternative` so clients that don't render HTML fall back to the plain text version. Pass only `html` for HTML-only messages, or only `body` for plain text:

```ts
// HTML + text fallback (recommended for most messages)
await send_mail({
	to,
	subject: "Your invitation",
	body: "Visit https://example.com/register/...",
	html: `<p>Visit <a href="https://example.com/register/...">your invitation</a>.</p>`,
});

// Plain text only
await send_mail({
	to,
	subject: "Your invitation",
	body: "Visit https://example.com/register/...",
});
```

Most modern email clients render HTML, but providing the text fallback is good practice — accessibility tools, watch interfaces, and the rare email client that strips HTML all benefit. The body field doubles as the spam-filter-friendly version (filters tend to score messages with no plain-text part more aggressively).

<a name="multiple-recipients"></a>

## Multiple Recipients

The `to`, `cc`, and `bcc` fields all accept either a single string or an array. The client normalises both forms to an array and joins them into the SMTP recipient list:

```ts
// Single recipient
await send_mail({ to: "alice@example.com", subject, body });

// Multiple, as an array
await send_mail({
	to: ["alice@example.com", "bob@example.com"],
	cc: "manager@example.com",
	subject: "Report ready",
	body: "The monthly report is attached.",
});

// Multiple, as a comma-separated string
await send_mail({
	to: "alice@example.com, bob@example.com",
	subject,
	body,
});
```

The string form is parsed by splitting on `,` and trimming. The array form is preferred because it's harder to get wrong — a stray space or a missing comma in the string form silently malforms an address.

`bcc` recipients receive the message but are not visible in the headers — the conventional use is for sending yourself a copy without exposing your address to the primary recipients. Don't use `bcc` to send the same message to many people; legitimate bulk sending uses your provider's API for batch sends.

<a name="rendering-html-with-templates"></a>

## Rendering HTML With Templates

The cleanest way to compose an HTML message is to use the same template engine that renders your pages. `get_render()` from `$lib/render` returns a function that takes a template name and data and returns the rendered HTML as a string:

```ts
import { send_mail } from "$lib/smtp";
import { get_render } from "$lib/render";

const render_template = get_render();

const html = await render_template("emails/welcome", {
	user,
	activation_url: `${BASE_URL}/register/${user.email}/${user.invitation_code}`,
});

await send_mail({
	to: user.email,
	subject: "Welcome to Reepolee",
	body: `Your account is ready. Activate at ${activation_url}.`,
	html,
});
```

Email templates live wherever you want — `emails/`, `routes/emails/`, anywhere reachable by the template engine. They're regular `.ree` files. The shape constraints come from email clients (which inline-style most CSS, ignore JavaScript, and render less reliably than browsers) rather than from Reepolee.

For most transactional emails — registration, password reset, notification — a single `<table>`-based layout with inline `style` attributes is the safest bet. A handful of small services (Maizzle, MJML) compile a friendlier source format down to email-safe HTML; both work fine alongside Reepolee without any framework integration.

<a name="error-handling"></a>

## Error Handling

`send_mail()` throws on any SMTP error — failed connection, rejected recipient, authentication failure, server timeout. The error has a message but no specific shape; treat it as a generic `Error` and decide whether to surface it to the user or log it silently:

```ts
try {
	await send_mail({ to, subject, body });
} catch (error) {
	console.error("Email failed:", error);
	return render("form", {
		data: { form_error: translated.errors.email_not_sent, ...translated },
		ctx,
	});
}
```

The translation key `email_not_sent` is already defined in `routes/en.json` for this purpose.

For flows where the email is essential (a password reset link the user is about to need), surface the error and don't move on. For flows where the email is informational (a notification that the report finished), log the error and let the user continue — they can always re-trigger the action that would resend the email.

<a name="dont-send-from-the-request"></a>

## Don't Send From the Request

Sending mail blocks the request until the SMTP server accepts the message. For a login confirmation that takes 200ms to send, that's fine — for a bulk notification that takes a minute, the user is staring at a hung browser tab.

The pattern for anything beyond a single immediate send is to write the message to a queue (or a `pending_emails` table) and let a background process call `send_mail()`. The handler returns instantly; the message goes out when the worker picks it up. Bun's `setInterval` plus a simple "claim a row, send it, mark it sent" loop is enough for most projects — no Redis or external queue service required.

<a name="testing-without-sending"></a>

## Testing Without Sending

For automated tests or local development, the cheapest "don't actually send" path is to point `SMTP_HOST` at a sandbox like Mailtrap. Mail goes through the same code path; nothing reaches a real inbox.

For unit tests where even sandbox sending is overkill, mock `send_mail` at the import boundary:

```ts
import { mock } from "bun:test";
import * as smtp from "$lib/smtp";

const send_mail = mock(() => Promise.resolve());
mock.module("$lib/smtp", () => ({ send_mail }));

// run your handler
// assert: expect(send_mail).toHaveBeenCalledWith(expect.objectContaining({ to: "..." }));
```

Bun's mocking integrates into `bun test` directly — no separate library needed.
