---
title: "Getting Started"
---

# Database

<a name="introduction"></a>

## Introduction

Reepolee uses Bun's built-in `SQL` API directly. There is no ORM, no query builder, no migration runner. You write SQL, Bun executes it, and you get typed results back. The same API works with both SQLite and MySQL — switching between them is a one-line change in your environment file.

The database connection is exported from `config/db.ts` as a single `db` instance that every part of the application imports. The rest of the database section walks through the four pieces of the picture:

- [Querying](/database/querying) — the `db` tagged template, dynamic queries, and the per-route `sql.ts` convention
- [Schema & Initialization](/database/schema-initialization) — the per-dialect SQL files under `sql/sqlite/` and `sql/mysql/` and how to apply them
- [Generators](/database/generators) — scaffolding CRUD modules from your database schema
- [Sessions & KV](/database/sessions-and-kv) — the small key-value layer that backs auth sessions and stays swappable

<a name="choosing-a-driver"></a>

## Choosing a Driver

<media-frame label="TUI — set database type (SQLite / MySQL)" ratio="4/3"></media-frame>

Reepolee ships with first-class support for both SQLite and MySQL. `config/db.ts` is a single unified file that creates a Bun `SQL` connection directly from your `CONNECTION_STRING` and uses its prefix to select timezone configuration:

| Driver     | Trigger                                   | When to use                                                                                                                                                                    |
| ---------- | ----------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| **SQLite** | `CONNECTION_STRING` starts with `sqlite:` | Single-process apps, low to moderate write volume, simple deployment (the database is a single file on disk). The default for development and for many production deployments. |
| **MySQL**  | `CONNECTION_STRING` starts with `mysql:`  | Multi-process or multi-server deployments, higher concurrency, when you already operate MySQL elsewhere. Connects to a separate database server.                               |

If `CONNECTION_STRING` is unset or starts with anything else, `config/db.ts` exits with a clear error — so a typo gets caught at startup, not on the first query. Switching later is a single `.env` change; your route handlers, queries, and schema files are identical between the two. Where SQL genuinely differs between engines (fulltext search, for example), that difference is isolated in `lib/sql_dialect.ts` — a small per-dialect map the generated queries call into, so route code stays dialect-agnostic.

Schema initialization SQL lives under a per-dialect folder: `sql/sqlite/` for SQLite and `sql/mysql/` for MySQL. Each holds a numbered, ordered set of files — the base schema (`01-init-sqlite.sql` / `01-init-mysql.sql`) plus seed data such as the per-language translation seeds `02-init-translations-en.sql` / `02-init-translations-sl.sql` (the seeded `translations` table — see [Translations](/i18n/translations#runtime-overrides)). Run the files in the folder matching your `CONNECTION_STRING` prefix, in numeric order, on first setup.

<a name="configuring-the-connection"></a>

## Configuring the Connection

There's nothing to copy. Set `CONNECTION_STRING` in your `.env`:

```
# SQLite — the database is a single file
CONNECTION_STRING="sqlite:app.db"

# MySQL — the connection string for your server
CONNECTION_STRING="mysql://login:pass@localhost/reepolee_dev"
```

The interactive setup tool can flip this for you — run `bun tui`, pick **Set database type**, and it toggles the `CONNECTION_STRING` lines in `.env` between the SQLite and MySQL variants.

When the server boots, `config/db.ts` inspects the prefix, selects the appropriate timezone constants from its built-in configuration map, and creates a `SQL` instance. You'll see `Using DB SQLITE` or `Using DB MySQL` logged in blue confirming which one loaded.

<a name="what-the-driver-does-on-startup"></a>

## What the Driver Does on Startup

When the server boots and imports `config/db.ts`, it:

1. **Reads `CONNECTION_STRING`** and determines the database type from its prefix.
2. **Constructs a `SQL` instance** with `new SQL(url)` — for SQLite it's a path to a file (created automatically on first connect); for MySQL it's a connection to the server.
3. **Exports `db`** for the rest of the application to import.

The driver does **not** apply your schema automatically. Running the SQL files in `sql/sqlite/` or `sql/mysql/` is a one-time manual step the first time you stand up the database — see [Schema & Initialization](/database/schema-initialization) for the exact command.

<a name="environment-variables"></a>

## Environment Variables

The two database-related environment variables:

| Variable            | Description                                                                                                                                                                                                                                                       |
| ------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `CONNECTION_STRING` | The connection string for SQLite (`sqlite:filename`) or MySQL (`mysql://user:pass@host/db`)                                                                                                                                                                       |
| `TIME_ZONE`         | **Required for all database types.** The timezone used by `config/db.ts` to configure date/time column handling (`Europe/Ljubljana`, `UTC`, etc.). For MySQL connections it is also applied with `SET time_zone='...'`; for SQLite it drives the timestamp codec. |

The `.env` file is loaded automatically by Bun — there's no `dotenv` dependency to install. See [Configuration](/getting-started/configuration) for the complete environment variable list.

<a name="logging-queries"></a>

## Logging Queries

Setting `SQL_LOGGING=true` in `.env` enables a per-query NDJSON log at `logs/sql.ndjson`. Each line is one query with its parameters and execution time — useful for tracing performance hotspots without reaching for a profiler. In production, the log rotates with whatever logrotate policy your server uses; in development it grows freely until you delete it.
