---
title: "Schema & Initialization"
---

# Schema & Initialization

<a name="introduction"></a>

## Introduction

Reepolee reads its database schema from per-dialect SQL files under `sql/sqlite/` (SQLite) or `sql/mysql/` (MySQL). Each folder holds a numbered, ordered set — the base schema (`01-init-sqlite.sql` / `01-init-mysql.sql`) followed by seed files such as the per-language translation seeds `02-init-translations-en.sql` and `02-init-translations-sl.sql`. These files are the source of truth for the shape of the database. There is no migration runner, no schema versioning, and no separate "create" vs "alter" commands.

The file uses `DROP TABLE IF EXISTS` followed by `CREATE TABLE` for each table, so running it gives you a clean schema and seed data every time. **You run it yourself** — the driver does not apply it automatically on startup. That keeps real user data safe across restarts. Running the file is a one-time step on a fresh database; subsequent schema edits are your responsibility.

<a name="the-init-file-format"></a>

## The init File Format

A typical table definition in `sql/sqlite/01-init-sqlite.sql`:

```sql
DROP TABLE IF EXISTS authors;

CREATE TABLE authors (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    name TEXT DEFAULT '' NULL,
    created_at DATETIME DEFAULT current_timestamp,
    updated_at DATETIME DEFAULT NULL
);

CREATE INDEX authors_name ON authors(name);

INSERT INTO authors (id, name) VALUES (1, 'Aleš'), (2, 'Evan'), (3, 'Jordan');

CREATE TRIGGER authors_update_timestamp
AFTER UPDATE ON authors
FOR EACH ROW
BEGIN
    UPDATE authors SET updated_at = current_timestamp WHERE id = NEW.id;
END;
```

The `updated_at` trigger is the convention Reepolee's generators expect — generated `sql.ts` queries don't set `updated_at` manually because the trigger handles it. If you write tables by hand, include the trigger so generated code (and hand-written code) behaves consistently across tables.

For MySQL, the trigger syntax differs slightly:

```sql
DELIMITER $$

CREATE TRIGGER authors_update_timestamp
BEFORE UPDATE ON authors
FOR EACH ROW
BEGIN
    SET NEW.updated_at = current_timestamp;
END$$

DELIMITER ;
```

The MySQL driver script handles the trigger as a single statement automatically.

<a name="domain-types"></a>

## Domain Types

Reepolee defines a small, canonical vocabulary of column types — **domain types** — that all new tables and columns should draw from. Rather than every table inventing its own width for a name or its own precision for money, you pick the domain type that matches the column's meaning and get a consistent SQL type. They're defined as typed constants in `config/domain_types/mysql.ts` and `config/domain_types/sqlite.ts` (a few differ per dialect — `pk_id`, `uuid_v7`, `currency`, `percent`, `boolean`).

| Domain type                                   | MySQL SQL                               | Use for                                                   |
| --------------------------------------------- | --------------------------------------- | --------------------------------------------------------- |
| `pk_id`                                       | `INT UNSIGNED AUTO_INCREMENT`           | Canonical primary key                                     |
| `uuid_v7`                                     | `BINARY(16)`                            | Time-ordered UUID (`Bun.randomUUIDv7Bytes()` for storage) |
| `first_name` / `last_name` / `full_name`      | `VARCHAR(100/100/255)`                  | Person/organization names                                 |
| `short_description` / `long_description`      | `VARCHAR(100/255)`                      | Labels, taglines, notes                                   |
| `text_block`                                  | `TEXT`                                  | Unbounded long text (JSON blobs, clauses, session data)   |
| `currency`                                    | `DECIMAL(18,2)`                         | All monetary values (`CURRENCY_FIELD`)                    |
| `percent`                                     | `DECIMAL(12,4)`                         | Percentage / commission rates (`PERCENT_FIELD`)           |
| `date_only`                                   | `DATE`                                  | Suffix `_on` / `_by` (`incorporated_on`)                  |
| `timestamp`                                   | `TIMESTAMP`                             | Suffix `_at` (`created_at`), default `CURRENT_TIMESTAMP`  |
| `boolean`                                     | `TINYINT(1)`                            | Prefix `is_` / `has_` / `can_`, or any `TINYINT(1)`       |
| `email` / `phone`                             | `VARCHAR(255/50)`                       | Contact fields                                            |
| `code_short` / `code_medium` / `code_long`    | `VARCHAR(3/10/64)`                      | ISO codes, lookup codes, machine tokens                   |
| `street` / `postal_code` / `city` / `country` | `VARCHAR(50/10/30/3)`                   | Address fields (`country` = ISO 3166-1 alpha-3)           |
| `username` / `password_hash` / `search_text`  | `VARCHAR(20)` / `VARCHAR(255)` / `TEXT` | Login handle, hashed password, generated fulltext column  |

The suffix and prefix conventions tie back to `config/db_structure.ts` (`DATE_SUFIXES`, `DATETIME_SUFIXES`, `BOOLEAN_PREFIXES`), so the generator already maps a `_at` column to a datetime codec and an `is_` column to a Yes/No select — see [Generators](/database/generators#configuration).

To audit an existing schema against this taxonomy, run the **Check domain compliance** tool from the TUI (`bun tui` → **Tools & Maintenance**), which reports columns whose types deviate from the canonical domain types so you can plan migrations.

<a name="applying-the-schema"></a>

## Applying the Schema

<media-frame label="TUI — Run SQL file (pick from sql/<dialect>/)" ratio="4/3"></media-frame>

The easiest path is the TUI:

```bash
bun tui
# pick "Run SQL file" → choose 01-init-sqlite.sql / 01-init-mysql.sql
```

The TUI detects your database type from `CONNECTION_STRING` and lists the `.sql` files in the matching `sql/<type>/` folder first (with a "Browse all" option for the project root and `sql/`), asks for confirmation, then connects via `CONNECTION_STRING` and runs each statement, reporting per-statement success or failure. For MySQL it wraps the run in `SET FOREIGN_KEY_CHECKS = 0/1` so DROP order doesn't matter. The "Run SQL file" option also lives inside the **Quick Start** flow that runs on a brand-new project. Run the numbered files in order — schema first, then the seed files.

If you'd rather drive the database CLIs directly, the init files are plain SQL — run them with whatever you have at hand.

**SQLite.** Run the file against the database file referenced in `CONNECTION_STRING` (`sqlite:app.db` → `app.db`):

```bash
sqlite3 app.db < sql/sqlite/01-init-sqlite.sql
sqlite3 app.db < sql/sqlite/02-init-translations-en.sql
sqlite3 app.db < sql/sqlite/02-init-translations-sl.sql
```

The file's `DROP TABLE IF EXISTS` statements clear anything that was there, and the `CREATE TABLE` / `INSERT` statements rebuild the schema and seed data. SQLite creates the database file on first connect, so you don't have to do anything else first.

**MySQL.** Wrap the run so foreign-key order doesn't bite you:

```bash
mysql -u login -p reepolee_dev < sql/mysql/01-init-mysql.sql
mysql -u login -p reepolee_dev < sql/mysql/02-init-translations-en.sql
mysql -u login -p reepolee_dev < sql/mysql/02-init-translations-sl.sql
```

If your seed file references tables in an order MySQL doesn't like, prepend `SET FOREIGN_KEY_CHECKS = 0;` to the file and `SET FOREIGN_KEY_CHECKS = 1;` to the end (the shipped `01-init-mysql.sql` already does this).

You only run this once on a fresh database. After that, the schema lives in the database itself and the application reads it on startup. Apply schema changes with hand-written `ALTER TABLE` statements (or by editing the init file and re-running it on a fresh database — see [Production Considerations](#production-considerations)).

<a name="indexes"></a>

## Indexes

Add indexes after the `CREATE TABLE` for any column you'll search or sort by:

```sql
CREATE INDEX authors_name ON authors(name);
CREATE UNIQUE INDEX authors_email_unique ON authors(email);
```

The generator looks for unique indexes when producing validation — a column with a unique index gets a "duplicate value" error message keyed off the constraint name automatically.

For composite indexes (multi-column), list the columns in the order you'll filter:

```sql
CREATE INDEX orders_user_status ON orders(user_id, status);
```

This index speeds up `WHERE user_id = ? AND status = ?` and `WHERE user_id = ?` (the leading column), but not `WHERE status = ?` alone.

<a name="foreign-keys"></a>

## Foreign Keys

Foreign keys are written inline with the `CREATE TABLE`:

```sql
CREATE TABLE frameworks (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    author_id INTEGER NOT NULL,
    language_id INTEGER NOT NULL,
    created_at DATETIME DEFAULT current_timestamp,
    updated_at DATETIME DEFAULT NULL,
    CONSTRAINT fk_frameworks_author   FOREIGN KEY (author_id)   REFERENCES authors(id)   ON UPDATE CASCADE,
    CONSTRAINT fk_frameworks_language FOREIGN KEY (language_id) REFERENCES languages(id) ON UPDATE CASCADE
);

CREATE INDEX frameworks_author_id   ON frameworks(author_id);
CREATE INDEX frameworks_language_id ON frameworks(language_id);
```

Indexes on foreign-key columns aren't free — add them explicitly so joins and lookups stay fast.

The CRUD generator detects foreign keys in two ways: explicit `FOREIGN KEY` constraints (the cleanest approach) or a column name ending in `_id`. Either way, the generated form renders the foreign-key field as a `<select>` populated from the related table. See [Generators](/database/generators#foreign-keys).

<a name="views"></a>

## Views

For pre-joined data — list views that need column data from related tables — define a SQL view next to the underlying tables:

```sql
DROP VIEW IF EXISTS v_frameworks;

CREATE VIEW v_frameworks AS
SELECT
    f.id,
    f.name,
    f.tagline,
    f.first_commit_at,
    a.name AS author_name,
    l.name AS language_name
FROM frameworks f
JOIN authors a   ON a.id = f.author_id
JOIN languages l ON l.id = f.language_id;
```

The convention `v_<table>` names views after the table they augment. Generated list templates can read from the view by overriding `select_table` in the route's `table.ts`. See [Database](/database/getting-started) for how `table.ts` ties into list rendering.

<a name="seed-data"></a>

## Seed Data

`INSERT` statements live in the same init file, so they ship as seed data alongside the schema. Hard-code primary keys when you want them to stay stable across re-runs:

```sql
INSERT INTO authors (id, name) VALUES (1, 'Aleš'), (2, 'Evan'), (3, 'Jordan');
```

For larger seed sets, a separate numbered file kept alongside the schema in `sql/sqlite/` (e.g. `03-seed-users.sql`) keeps the schema file readable. Import it the same way as the init file and run it after initialization — the numeric prefix keeps the run order obvious.

<a name="re-running-the-init-file"></a>

## Re-running the Init File

In development, re-running the init file is a fast way to reset the database — drop everything, recreate, get fresh seed data. The driver doesn't auto-run it, so wiping your data is always a deliberate command, not a side effect of restarting the server.

```bash
# SQLite — nuke and pave
rm -f app.db && sqlite3 app.db < sql/sqlite/01-init-sqlite.sql
```

For tables you want to preserve across re-runs (typically `users` in any production-ish environment), comment out the matching `DROP TABLE IF EXISTS` line and switch the create to `CREATE TABLE IF NOT EXISTS`. Once that table exists in the live database, the file becomes a no-op for it on re-run while the rest still drops and recreates:

```sql
-- DROP TABLE IF EXISTS users;

CREATE TABLE IF NOT EXISTS users (
    ...
);
```

New columns on a preserved table still need an explicit `ALTER TABLE` — there's no automatic migration.

<a name="production-considerations"></a>

## Production Considerations

When the application is past the "iterate on schema freely" phase and starts holding real data:

- **Stop re-running the full init file** in production. Treat each `DROP TABLE` as a destructive command, not a routine reset.
- **Track schema changes in version-controlled SQL files** (e.g., `migrations/001_add_users_email_index.sql`). Apply them manually on each deploy until you decide a migration runner is worth the dependency.
- **Back up before every schema change.** SQLite is a single file — `cp app.db app.db.backup` is enough. MySQL has `mysqldump` for the same purpose.
- **Test schema changes against a copy of production data**, not just an empty schema. Index changes especially can take much longer on a large table than they appear in development.

If you eventually outgrow the manual approach, dropping in a small migration runner (or a tool like Liquibase or Atlas) is the next step. Reepolee doesn't ship one because most projects don't need one for a long time, but the path from "manual SQL files" to "real migrations" is a short one when you're ready.
