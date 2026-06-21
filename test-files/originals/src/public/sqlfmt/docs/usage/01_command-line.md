---
title: "Command line"
---

# Command line

SQLfmt reads SQL from **stdin or a file** and writes back consistently formatted
SQL. It is designed to drop into pipes, editor commands, and pre-commit hooks.

## Format a file in place

Pass a path and SQLfmt formats that file in place:

```bash
sqlfmt path/to/file.sql
```

## Format from stdin

With no file argument, SQLfmt reads from stdin and writes the result to stdout:

```bash
cat query.sql | sqlfmt
```

## Pipe directly

```bash
echo "SELECT * FROM users WHERE active = 1;" | sqlfmt
```

## Editor and hook integration

Because SQLfmt is a plain stdin/stdout filter, it plugs into most editors as an
external formatter and into Git as a pre-commit step. For example, to format and
fail CI if anything changed:

```bash
sqlfmt schema.sql
git diff --exit-code
```

## What SQLfmt does

- **Uppercases keywords** — `SELECT`, `FROM`, `WHERE`, `CREATE`, `INSERT`, and
  the rest.
- **Aligns definitions** — `CREATE TABLE` columns and types line up; `CREATE
VIEW` select columns align on `AS`.
- **Structures statements** — `UPDATE` puts each `SET` assignment on its own
  line; `WHERE` clauses move to their own line; long `INSERT` value tuples wrap.
- **Indents subqueries** — `(SELECT ...)` subqueries are formatted recursively.
- **Preserves comments** — `--`, `/* */`, and `#` comments are kept and
  repositioned.
- **Normalises line endings** — `\r\n` and `\n` are handled consistently.

For the full list of statement types see
[Supported statements](/sqlfmt/docs/formatting/supported-statements), and for
before/after samples see [Examples](/sqlfmt/docs/formatting/examples).
