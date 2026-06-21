---
title: "Utilities Reference"
---

# Utilities Reference

<a name="introduction"></a>

## Introduction

Reepolee ships a handful of small utility modules that don't warrant their own section but are useful across many code paths. This page collects them in one place for reference.

<a name="environment-helpers"></a>

## Environment Helpers (`$lib/env`)

```ts
import { require_env, sanitize_env_value, get_storage_mode } from "$lib/env";
```

### `require_env(name: string): string`

Reads a required environment variable. If the variable is unset, prints a red error message and exits the process immediately. Use it at the point of need — the function call is right next to where the value is used, so it's clear which variable is required and why:

```ts
const connection_string = require_env("CONNECTION_STRING");
```

The exit happens synchronously with `process.exit(1)`, so you'll see the error immediately at startup rather than as a mysterious failure later.

### `sanitize_env_value(raw: string): string`

Strips surrounding quotes and whitespace from an environment variable value. Bun's `.env` parser preserves literal quote characters, so a `.env` file with `FOO="some value"` would include the quotes in the string. This helper removes them:

```ts
const clean = sanitize_env_value('"sqlite:app.db"'); // → "sqlite:app.db"
const clean = sanitize_env_value("'sqlite:app.db'"); // → "sqlite:app.db"
```

`require_env()` calls this internally, so you don't need to call it explicitly when using `require_env()`.

### `get_storage_mode(): StorageMode | null`

Returns the storage mode from the `STORAGE` environment variable:

- `"local"` — local filesystem storage
- `"s3"` — S3-compatible object storage
- `null` — not set (auto-detect from presence of S3 credentials)

If the env var is set to an unrecognised value, exits with an error message listing the valid options.

```ts
const mode = get_storage_mode();
if (mode === "s3") {
	// Use S3 client
} else {
	// Use local filesystem
}
```

---

<a name="sql-timing"></a>

## SQL Timing Wrapper (`$lib/timed_sql`)

```ts
import { timed_query } from "$lib/timed_sql";
```

### `timed_query(component, label, fn, extra?)`

Wraps any async SQL operation with automatic duration logging. The component and label identify the operation in the logs, while the optional `extra` parameter can be a static object or a function that derives extra data from the result:

```ts
const records = await timed_query(
	"partners",
	"get_all",
	() => db`SELECT * FROM partners ORDER BY id ASC`,
	(result) => ({ count: result.length }),
);

await timed_query(
	"partners",
	"update",
	() => db`UPDATE partners SET title = ${title} WHERE id = ${id}`,
	{ id }, // logged as extra data
);
```

The output goes through `log_info()` in `$lib/logger`, so it appears in the systemd journal with the `[INFO]` level and the component/label/duration fields visible.

---

<a name="local-storage"></a>

## Local Storage (`$lib/local_storage`)

```ts
import { get_local_storage_dir, delete_local_file } from "$lib/local_storage";
```

### `get_local_storage_dir(): string | null`

Returns the resolved local storage directory path. The behaviour depends on the `STORAGE` env var:

- `STORAGE=local` — requires `LOCAL_STORAGE_DIR`, returns the resolved absolute path
- `STORAGE=s3` — returns `null` (local storage not available)
- Unset — auto-detects from `LOCAL_STORAGE_DIR` if set, returns `null` otherwise

```ts
const dir = get_local_storage_dir();
if (dir) {
	// e.g. "/home/deploy/app/storage"
}
```

### `delete_local_file(bucket: string, key: string): Promise<void>`

Deletes a file from the local storage directory at `{local_storage_dir}/{bucket}/{key}`. Silently skips if local storage is not configured:

```ts
await delete_local_file("users", "avatars/uuid.webp");
// Deletes {local_storage_dir}/users/avatars/uuid.webp
```

This is the local-storage equivalent of `delete_from_s3()` from `$lib/s3`. For a uniform interface regardless of storage mode, use `delete_from_local()` from `$lib/s3` instead — it delegates here automatically.

---

<a name="logger-api"></a>

## Logger API (`$lib/logger`)

```ts
import { log_info, log_warn, log_error, duration_ms, create_file_logger } from "$lib/logger";
```

### Structured logging functions

```ts
log_info("component", "message", { optional: "data" });
log_warn("component", "message", { optional: "data" });
log_error("component", "message", error, { optional: "data" });
```

Format: `[{timestamp}] [{level}] [{component}] {message} {JSON-data}`

The component is a short identifier for the logical module (e.g. `"s3"`, `"auth"`, `"partners"`). The data object is serialised as JSON and appended to the log line.

### `duration_ms(start: bigint): string`

Returns a human-readable duration string from a `process.hrtime.bigint()` start value:

```ts
const start = process.hrtime.bigint();
// ... do work ...
console.log(`Took ${duration_ms(start)}`); // "Took 1.42ms"
```

### `create_file_logger(path: string): (data, req) => Promise<void>`

Creates a file-based appender that writes structured JSON lines to date-rotated files:

```ts
const log_request = create_file_logger("logs/requests.ndjson");

// In a handler
await log_request({ method: req.method, path: url.pathname }, req);
```

The logger creates a subdirectory named after the file's base name (e.g. `logs/requests/`), writes to a date-stamped file (e.g. `logs/requests/2026-06-03.ndjson`), and includes the current user's email from the session in every entry.

---

<a name="translation-merge"></a>

## Translation Merge Utilities (`$lib/translation_merge`)

```ts
import {
	sort_object,
	clean_for_translation,
	extract_untranslated,
	apply_translations,
	merge_into_english,
	merge_with_missing_prefix,
	sync_lang_to_en,
	count_leaves,
	collect_leaf_paths,
} from "$lib/translation_merge";
```

These are pure functions for manipulating nested translation JSON objects. They're used internally by the generator's `sync_translations` and extraction scripts, and can be useful for custom translation tooling:

```ts
// Count how many strings need translation
const untranslated = extract_untranslated(en_obj, sl_obj);
const missing_count = count_leaves(untranslated);

// Apply machine-translated results back, preserving already-translated keys
const merged = apply_translations(sl_obj, ai_translated_obj);

// Collect all dot-notation paths for auditing
const paths = collect_leaf_paths(en_obj);
// ["ui.title", "actions.save", "errors.required", ...]

// Sort keys alphabetically (deep)
const sorted = sort_object(messy_obj);
```

These functions are all synchronous, pure (no filesystem or network calls), and operate only on JSON objects in memory.
