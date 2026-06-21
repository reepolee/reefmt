---
title: "SMTP Configuration"
---

# SMTP Configuration

<a name="introduction"></a>

## Introduction

Reepolee's email sender lives in `lib/smtp.ts`. It connects to a remote SMTP server using Bun's Node.js compatibility layer (`node:net` and `node:tls`), authenticates, and sends a single message. There's no library to install and no background queue — `send_mail()` is an async function call that resolves when the recipient SMTP server has accepted the message.

Once Bun adds a native SMTP client to its standard library, the implementation in `lib/smtp.ts` will be swapped out. The `send_mail()` call signature will stay the same.

This page covers the configuration. Sending a message is on [Sending Email](/email/sending-email); the database-backed audit log is on [Email Module](/email/email-module).

<a name="environment-variables"></a>

## Environment Variables

SMTP credentials live in `.env`. The five variables `send_mail()` reads:

| Variable        | Required | Description                                                                               |
| --------------- | -------- | ----------------------------------------------------------------------------------------- |
| `SMTP_HOST`     | yes      | The SMTP server hostname (e.g. `smtp.sendgrid.net`, `email-smtp.eu-west-1.amazonaws.com`) |
| `SMTP_PORT`     | yes      | `587` for STARTTLS (the common case) or `465` for implicit TLS                            |
| `SMTP_USERNAME` | yes      | Auth username supplied by your provider                                                   |
| `SMTP_PASSWORD` | yes      | Auth password or API key                                                                  |
| `SMTP_FROM`     | yes      | Default `From:` address; usually a verified domain address with your provider             |

A typical `.env` block:

```
SMTP_HOST="smtp.sendgrid.net"
SMTP_PORT=587
SMTP_USERNAME="apikey"
SMTP_PASSWORD="SG.xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"
SMTP_FROM="hello@example.com"
```

Bun loads `.env` automatically — there's no `dotenv` package to install or initialise. See [Configuration](/getting-started/configuration) for the broader env-var picture.

<a name="when-credentials-are-missing"></a>

## When Credentials Are Missing

If `SMTP_HOST` is unset when you call `send_mail()`, the process exits immediately with a loud error. This is intentional — silently swallowing send failures hides bugs that don't show up until a user reports a missing notification.

```
🧨 Set SMTP vars in .ENV
```

In production, configure your SMTP credentials before starting the server. In development, either set up a sandbox SMTP service (Mailtrap, Ethereal, MailHog) or only run the code paths that send mail when you've configured them.

<a name="starttls-vs-implicit-tls"></a>

## STARTTLS vs Implicit TLS

The client picks its connection mode based on the port:

- **Port 587** → STARTTLS. Connects in plaintext, then upgrades to TLS via the `STARTTLS` command. The most common SMTP submission port.
- **Port 465** → implicit TLS. Wraps the TCP connection in TLS from the first byte. Sometimes called "SMTPS."

For most providers, port 587 is the right choice. Use port 465 only when your provider explicitly recommends it.

The third historical port, **25**, is for server-to-server SMTP and is blocked by most cloud providers' egress firewalls. Don't use it for application sending.

<a name="tls-options"></a>

## TLS Verification

By default, the client accepts the SMTP server's TLS certificate as long as it's valid. To accept self-signed or otherwise invalid certificates (for a local sandbox), pass `tls.rejectUnauthorized: false` per call:

```ts
await send_mail({
	to: "user@example.com",
	subject: "Test",
	body: "...",
	tls: { rejectUnauthorized: false },
});
```

Don't set this in production — it disables the certificate check that protects credentials in transit. For local development with a sandbox like Mailtrap that uses real certs, you don't need it.

<a name="picking-a-provider"></a>

## Picking a Provider

Reepolee is provider-agnostic; anything that speaks SMTP works. The trade-offs to know:

| Provider type                                                                | When to use                                                                                                                                         |
| ---------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Transactional services** (SendGrid, Postmark, Resend, Mailgun, Amazon SES) | Almost always. Run by people who specialise in deliverability, handle DMARC/SPF/DKIM, give you reputation monitoring and bounce/complaint handling. |
| **Self-hosted SMTP** (Postfix, Exim)                                         | When you have specific compliance needs that prevent using a third party. Operationally expensive — IP reputation alone is a full job.              |
| **Gmail / personal SMTP**                                                    | Development sandboxes and one-off scripts. Hits send-rate limits quickly; not viable for production.                                                |
| **Sandbox services** (Mailtrap, Ethereal)                                    | Development and testing. Catches mail without delivering it; lets you preview HTML rendering without spamming real addresses.                       |

For most projects, picking a transactional service in the same region as your application server (latency matters less than people assume, but staying in-region is one less variable) and configuring SPF + DKIM is the whole story.

<a name="testing-the-config"></a>

## Testing the Configuration

The simplest way to confirm SMTP is wired up correctly is to send yourself a message from a one-off route or a script:

```ts
// scripts/send-test.ts
import { send_mail } from "$lib/smtp";

await send_mail({
	to: "you@example.com",
	subject: "Reepolee SMTP test",
	body: "If you see this, SMTP is configured correctly.",
});

console.log("Sent.");
```

Run it with `bun scripts/send-test.ts`. A successful run logs the SMTP exchange (the `[SMTP] Connecting to ...` line) and exits cleanly. A failure throws with the SMTP server's error message — usually clear enough to fix the credentials or server name.

<a name="rate-limits-and-retries"></a>

## Rate Limits and Retries

`send_mail()` is a single attempt. If the SMTP server rejects the message — temporary failure, rate limit, transient network error — it throws. Reepolee does not ship a retry queue.

For low-volume use (registration emails, password resets, individual notifications), a try/catch with a user-friendly fallback is enough:

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

For higher volume — newsletters, bulk notifications, anything that should retry on transient failure — a job queue is the right tool. Bun's built-in scheduling (or a small worker process backed by the same database) handles this without an external dependency. The queue calls `send_mail()`; the rest of your application doesn't know retries are happening.
