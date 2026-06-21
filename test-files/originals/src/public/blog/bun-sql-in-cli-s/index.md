---
title: "Why Bun's SQL Dies in CLI Scripts (and How to Fix It)"
layout: "blog"
track: "Engineering"
published_at: 2026-06-08
author: "Aleš Vaupotič"
image: "/images/responsive/hero-privacy.png"
excerpt: "Bun's native SQL uses an internal connection pool whose idle connections don't keep the event loop alive — so standalone scripts exit mid-query. Here's why, and two clean workarounds."
description: "Bun's native SQL uses an internal connection pool whose idle connections don't keep the event loop alive — so standalone scripts exit mid-query. Here's why, and two clean workarounds."
---

I lost an hour to this one before the cause clicked, so here it is written down. A migration script that ran two queries kept dropping the second one — sometimes silently, sometimes with a `Connection closed` error — even though the exact same code ran fine inside a server. The difference isn't the SQL. It's the event loop.

## The Problem

Bun's native `sql` uses a **connection pool** internally, and the pool's idle connections don't register persistent I/O on the event loop. Long-lived handles like `Bun.serve()` keep the loop alive because they hold an open server socket; an idle pool connection does not.

So in a standalone script the sequence is:

1. Your first query runs and its promise resolves.
2. The event loop sees no more pending I/O and the process exits.
3. The SQL connection is torn down mid-execution.

That's why the second query never runs, or dies halfway through.

---

## The Fix

Two clean options, depending on what the script does.

**Option A — Keep the event loop alive with a dummy timer, then exit manually:**

```typescript
import { sql } from "bun";

const db = sql`mysql://user:pass@localhost:3306/mydb`;

const stay_alive = setInterval(() => {}, 2_147_483_647);

async function run() {
	const users = await db`SELECT id, name FROM users`;
	console.log(users);

	const orders = await db`SELECT id, total FROM orders`;
	console.log(orders);

	clearInterval(stay_alive);
	await db.end();
	process.exit(0);
}

run().catch((err) => {
	console.error(err);
	process.exit(1);
});
```

**Option B — Use `sql.reserve()` to hold a dedicated, persistent connection:**

```typescript
import { sql } from "bun";

const db = sql`mysql://user:pass@localhost:3306/mydb`;

async function run() {
	const conn = await db.reserve();

	try {
		const users = await conn`SELECT id, name FROM users`;
		console.log(users);

		const orders = await conn`SELECT id, total FROM orders`;
		console.log(orders);
	} finally {
		conn.release();
		await db.end();
		process.exit(0);
	}
}

run();
```

---

Which one you reach for comes down to the work. `reserve()` is the right call when you need transaction-level connection affinity — anything wrapped in `BEGIN` / `COMMIT`. The `setInterval` trick is simpler for fire-and-forget scripts that just run a few queries and quit. And both become unnecessary the moment this code lives inside `Bun.serve()`, because the server socket holds the loop open for as long as the process runs.
