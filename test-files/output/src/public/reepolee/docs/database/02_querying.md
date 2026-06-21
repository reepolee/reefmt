---
title: "Querying"
---

# Querying

<a name="introduction"></a>

## Introduction

All queries go through the `db` tagged template literal exported from `config/db.ts`. Values interpolated into the template are automatically parameterised — you cannot accidentally introduce a SQL injection by interpolating a variable. The tagged template covers most of what you'll write; for the cases it can't handle (dynamic column names, dynamic `ORDER BY`), `db.unsafe()` is the escape hatch.

```ts
import { db } from "$config/db";

const records = await db`SELECT * FROM users WHERE id = ${id} LIMIT 1`;
const record = records[0];
```

Results come back as plain objects keyed by column name. The shape is whatever the database returns — Bun does not impose a schema layer of its own.

<a name="select-queries"></a>

## SELECT Queries

The most common shape — a parameterised SELECT — reads exactly like the example above. Multiple values, including arrays for `IN` clauses, work the same way:

```ts
const ids = [1, 2, 3];
const rows = await db`SELECT * FROM users WHERE id IN ${db(ids)}`;
```

Wrap an array in `db(...)` to expand it into a parameter list. Without the wrapper, the array is passed as a single value and the query fails.

For one-row lookups, destructure the result:

```ts
const [user] = await db`SELECT * FROM users WHERE email = ${email} LIMIT 1`;
if (!user) {
	return render("notfound", { data: { title: "User not found", ...translated }, status: 404, ctx });
}
```

The pattern of `[row] = await db\`...\``is what you'll see throughout the generated`sql.ts`files. The destructuring is cheaper than calling`.then((rows) => rows[0])`and the`LIMIT 1` makes the intent clear.

<a name="insert-queries"></a>

## INSERT Queries

Inserts return a result object whose shape differs slightly between drivers:

```ts
const result = await db`
    INSERT INTO users (email, name) VALUES (${email}, ${name})
`;
```

| Driver | Identifier of the new row |
| ------ | ------------------------- |
| SQLite | `result.lastInsertRowid`  |
| MySQL  | `result.insertId`         |

The cleanest pattern is to fetch the inserted row immediately so the rest of the handler sees the full record:

```ts
const result = await db`
    INSERT INTO users (email, name) VALUES (${email}, ${name})
`;
const id = result.lastInsertRowid ?? result.insertId;
const [created] = await db`SELECT * FROM users WHERE id = ${id}`;
```

This way the route handler doesn't have to care which driver is in use — the `??` falls back to whichever property the active driver provides.

<a name="update-and-delete"></a>

## UPDATE and DELETE

Updates and deletes follow the same shape as selects. Both return a result object with the affected row count, which you can use to detect "this id didn't exist":

```ts
const result = await db`
    UPDATE users SET name = ${name}, updated_at = ${now} WHERE id = ${id}
`;

if (result.affectedRows === 0) {
	return render("notfound", { data: { title: "User not found", ...translated }, status: 404, ctx });
}
```

The `affectedRows` field is consistent across both drivers.

<a name="dynamic-queries"></a>

## Dynamic Queries

The tagged template syntax only works with value interpolation — you cannot parameterise column names, table names, or `ORDER BY` directions. For those, use `db.unsafe()`, which takes a plain SQL string and an optional array of values:

```ts
const query = `SELECT * FROM users ORDER BY ${sort_field} ${direction} LIMIT ? OFFSET ?`;
const records = await db.unsafe(query, [limit, offset]);
```

Always validate any identifier that comes from user input before using it in an `unsafe` query. The generated `search_records` function shows the standard pattern — accept only valid identifier characters and fall back to a known-safe default for anything else:

```ts
if (!/^[a-zA-Z_][a-zA-Z0-9_]*$/.test(sort_field)) {
	return search_records(search, offset, limit, "id::asc"); // safe default
}
```

If the validator fails the input, _return early with a known-good query_ rather than trying to sanitise. Sanitising user input is harder than it looks; rejecting it is reliable.

<a name="the-sql-ts-file"></a>

## The sql.ts File

Every route folder has a `sql.ts` that owns all the database queries for that feature. The pattern is straightforward — import `db`, define TypeScript interfaces for the row shapes, and export functions that wrap the queries:

```ts
// routes/users/sql.ts
import { db } from "$config/db";

export interface User_record {
	id: number;
	email: string;
	name: string;
	created_at: string;
	updated_at: string | null;
}

export async function get_user_by_email(email: string): Promise<User_record | undefined> {
	const rows = await db`SELECT * FROM users WHERE email = ${email} LIMIT 1`;
	return rows[0] as User_record | undefined;
}

export async function get_user_by_id(id: number): Promise<User_record | undefined> {
	const rows = await db`SELECT * FROM users WHERE id = ${id} LIMIT 1`;
	return rows[0] as User_record | undefined;
}
```

Route handlers import these named functions rather than calling `db` directly. This keeps the query surface auditable — every database access point for a feature is in one file — and gives you a single place to add caching, logging, or migration shims later.

<a name="generated-functions"></a>

## Generated Functions

The CRUD generator produces a consistent set of functions in every route folder's `sql.ts`. Route handlers import these by name:

```ts
import { get_record_by_id, get_all_records, search_records, create_record, update_record, delete_record } from "./sql";
```

| Function                                         | Description                                 |
| ------------------------------------------------ | ------------------------------------------- |
| `get_record_by_id(id)`                           | Fetch a single row by primary key           |
| `get_all_records()`                              | Fetch all rows, ordered by `id ASC`         |
| `search_records(query, offset, limit, order_by)` | Paginated search with total count           |
| `create_record(data)`                            | Insert a new row, return the created record |
| `update_record(id, data)`                        | Update a row, return the updated record     |
| `delete_record(id)`                              | Delete a row, return `true` on success      |

Results are parsed through a Zod schema before being returned, so TypeScript types stay in sync with what came back from the database. You can edit the generated `sql.ts` after generation — the generator only writes it once and won't overwrite your changes on a re-run. See [Generators](/database/generators).

The exact `search_records` signature depends on the route's pagination strategy — `(query, offset, limit, order_by)` for the default offset strategy, or a cursor-based variant for keyset pagination. The generator can also emit a streaming (DPU) handler instead of a synchronous one. See [Pagination](/database/generators#pagination) and [Streaming List Views](/database/generators#streaming).

<a name="transactions"></a>

## Transactions

Bun's `SQL` instance supports transactions through `db.begin()`, which takes a callback and commits or rolls back based on whether the callback throws:

```ts
await db.begin(async (tx) => {
	const result = await tx`INSERT INTO orders (user_id, total) VALUES (${user_id}, ${total})`;
	const order_id = result.lastInsertRowid ?? result.insertId;

	for (const item of items) {
		await tx`INSERT INTO order_items (order_id, sku, qty) VALUES (${order_id}, ${item.sku}, ${item.qty})`;
	}
});
```

The `tx` argument is a transaction-scoped query function. If the callback throws, the transaction rolls back; if it returns normally, it commits. Don't pass `tx` to other functions that close over `db` — those will run outside the transaction.

For SQLite, transactions are serialised; for MySQL they use the connection-pool's transactional connection. Both behave the same from the application's perspective.

<a name="error-handling"></a>

## Error Handling

Database errors are real exceptions — they bubble up unless you catch them. The two cases that come up daily are unique-key violations on insert and foreign-key violations on delete:

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
		data: { record, form_errors: error_message, ...translated },
		ctx,
	});
}
```

The error messages from SQLite and MySQL differ in wording — `"UNIQUE constraint failed"` vs `"Duplicate entry"`, `"foreign key constraint"` vs `"foreign key constraint fails"`. Keep your `.includes()` checks loose enough to match both. Full patterns for surfacing these errors to users are in [Error Handling](/the-basics/error-handling).
