---
title: "Supported statements"
---

# Supported statements

SQLfmt classifies each statement and applies a dedicated formatter. Anything it
doesn't have a specialised formatter for still gets generic keyword uppercasing
and spacing.

| Statement      | Formatting                                                                  |
| -------------- | --------------------------------------------------------------------------- |
| `SELECT`       | Generic formatting with recursive subquery indentation                      |
| `CREATE TABLE` | Aligned column definitions, uppercased types, constraints on separate lines |
| `CREATE VIEW`  | Aligned `SELECT` columns with `AS`, indented `JOIN`s                        |
| `CREATE INDEX` | Uppercased, compact                                                         |
| `INSERT`       | Multi-line value tuples for long inputs                                     |
| `UPDATE`       | `SET` assignments on indented lines, `WHERE` on its own line                |
| `DELETE`       | `WHERE` clause on its own line                                              |
| `DROP`         | Uppercased, compact                                                         |

## Operators

Comparison operators (`>`, `<`, `>=`, `<=`, `!=`, `<>`) are preserved and spaced
correctly. Note that `<>` is normalised to `!=`.

## Comments

`--` line comments, `/* */` block comments, and `#` hash comments are all
preserved and positioned consistently relative to the statements around them.

## How it works

1. **Tokenization** — the raw SQL is split into tokens: keywords, identifiers,
   operators, comments, and strings.
2. **Statement splitting** — tokens are split at semicolons into individual
   statements.
3. **Type detection** — each statement is classified (CREATE TABLE, SELECT,
   INSERT, UPDATE, and so on).
4. **Formatting** — each statement type has a dedicated formatter; subqueries are
   formatted recursively with indentation.
