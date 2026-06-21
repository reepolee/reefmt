---
title: "Quick Start"
---

# Quick Start

<a name="introduction"></a>

## Introduction

This page walks from a fresh project to a running Reepolee application. It assumes you've already installed Bun and the global tools — if not, work through [Installation](/getting-started/installation) first.

By the end, you'll have your own repository created from the Reepolee template, the project running locally, a SQLite database initialised with seed data, a logged-in admin account, and an upstream remote pointing back at Reepolee so future releases are one `bun git:sync` away.

<a name="creating-your-repository"></a>

## Creating Your Repository

Reepolee isn't published to a package registry — it's a [GitHub template repository](https://docs.github.com/en/repositories/creating-and-managing-repositories/creating-a-repository-from-a-template). Generating a project from the template gives you a brand-new repository under your own account or organisation, with clean history and no fork relationship. From there it's yours: you commit, you push, you decide what changes go in.

Visit [github.com/reepolee/reepolee](https://github.com/reepolee/reepolee), click **Use this template → Create a new repository**, pick the owner and name (e.g., `my-app`), then clone the new repository locally:

```bash
git clone git@github.com:<your-account>/my-app.git
cd my-app
```

Your project's `origin` is now your own repository. The Reepolee template is no longer in `origin`'s history at all — that's what the next step puts back, deliberately, as `upstream`.

<a name="without-the-github-ui"></a>

### Prefer not to use GitHub's UI?

If you'd rather not click through GitHub's templating flow — or you want to stage things locally before publishing anywhere — you can do the same setup by hand: clone Reepolee, drop its `.git` directory to sever the history, initialise your own, and push to an empty repository you create on GitHub afterwards.

```bash
# Pull down the template
git clone https://github.com/reepolee/reepolee.git my-app
cd my-app

# Sever Reepolee's git history so your project starts clean
rm -rf .git
git init
git add -A
git commit -m "Initial commit from Reepolee template"
```

Now create an empty repository on GitHub (no README, no `.gitignore` — Reepolee ships its own) at `github.com/<your-account>/my-app`, then point your new local repo at it and push:

```bash
git remote add origin git@github.com:<your-account>/my-app.git
git push -u origin main
```

You end up in exactly the same state as the template-button path: your `origin` is your own repository, your history starts at one fresh commit, and `reepolee/reepolee` isn't on the remote list yet. The next step adds it back as `upstream`.

<a name="tracking-upstream"></a>

## Tracking Upstream

You created your repo from the template, so right now your `origin` knows nothing about Reepolee. That's intentional — you want your own history — but it means new Reepolee releases can't flow in unless you wire up a second remote. Run this once, immediately after cloning:

```bash
bun git:add-sync
```

That script is a one-liner — it runs `git remote add upstream git@github.com:reepolee/reepolee.git`, registering Reepolee as your `upstream`. Any time you want to pull the latest Reepolee changes into your project:

```bash
bun git:sync
```

That fetches `upstream` and rebases your current branch onto `upstream/main`, so your customisations land on top of the freshest Reepolee. The full upgrade flow, including how to pin to a tagged release instead of tracking `main`, is in the [Upgrade Guide](/prologue/upgrade-guide).

<a name="installing-dependencies"></a>

## Installing Dev Dependencies

Reepolee has zero runtime dependencies — `bun install` only pulls in tooling needed to develop:

```bash
bun install
```

<a name="the-tui"></a>

## The Quickest Path — `bun tui`

<media-frame label="TUI — bun tui main menu (Quick Start + grouped tasks)" ratio="4/3"></media-frame>

Most of the steps below — picking a database, applying the schema, choosing a session backend, creating the admin user — are also one menu pick each in the project's interactive setup tool:

```bash
bun tui
```

On a fresh project (no `users` table yet) the TUI offers a **Quick Start** flow that walks you through database type → init/seed SQL → session driver → admin user in order. On an initialised project the same tasks are available individually from the grouped menu, alongside generators ("Single table", "All tables", "Bulk CRUD", "Nested children"), "Simple Table Page", "Remove route", "Add language", and "Run SQL file".

Reading the rest of this page is still worth doing — it explains what the TUI changes — but `bun tui` is the shortest path from clone to running app.

<a name="picking-a-database-driver"></a>

## Picking a Database Driver

Reepolee ships with first-class support for both SQLite and MySQL. There's nothing to copy — `config/db.ts` is a dynamic barrel that picks the right driver based on your `CONNECTION_STRING`. Set it to one starting with `sqlite:` and you get the SQLite driver; set it to one starting with `mysql:` and you get MySQL. The TUI's "Set database type" option toggles `CONNECTION_STRING` in `.env` for you.

<a name="environment-variables"></a>

## Environment Variables

Copy the example environment file:

```bash
cp .env.example .env
```

Open `.env` and set `CONNECTION_STRING` to whichever database you want:

```
# SQLite — the database is a single file in the project directory
CONNECTION_STRING="sqlite:app.db"

# or, for MySQL
# CONNECTION_STRING="mysql://login:pass@localhost/reepolee_dev"
```

For SQLite, the `app.db` file is created automatically the first time you start the server — you don't need to create it or run any migrations manually.

For MySQL, replace `login`, `pass`, and `reepolee_dev` with your credentials. The `TIME_ZONE` variable in `.env` controls the timezone applied to every MySQL connection — set it to match your server's timezone.

The barrel exits with a clear error at startup if `CONNECTION_STRING` doesn't start with either `sqlite:` or `mysql:` — so a typo gets caught before you load a page.

The full list of environment variables is in [Configuration](/getting-started/configuration).

<a name="running-the-dev-server"></a>

## Running the Dev Server

<media-frame label="Screenshot — fresh app running at localhost:2338" ratio="16/9"></media-frame>

Start everything in development mode:

```bash
bun dev
```

This single command runs the Tailwind watcher and the Bun hot-reload server side-by-side. Open `http://localhost:2338` in a browser. The port is configurable via the `PORT` variable in `.env`.

What's running behind the scenes:

- `bun --watch server.ts --dev` — the application server with file-change detection (via the `development` script).
- `tailwindcss -i ./css/app.css -o ./static/app-dev.css --watch` — the CSS compiler.
- `concurrently` — the multiplexer that runs both with colour-coded output.

Hot reload picks up template, route, and styling changes within a few hundred milliseconds. For most edits you don't need to refresh the page.

<a name="the-seeded-admin"></a>

## The Seeded Admin Account

The fastest way to get an admin user is the admin-user step of the TUI's **Quick Start** flow (a standalone `bun generator/user <email> <password>` does the same thing):

```bash
bun tui
# pick "Quick Start" on a fresh DB, or "Reset the database" later
# the last step asks for an email + password and creates the row
```

Behind the scenes that runs `bun generator/user <email> <password>`, which writes a verified user straight into the database — no invitation round-trip needed. The **first** user created this way is granted `modules_tags = "system,examples"` (full system access); subsequent users get no modules unless you pass `--modules`, e.g. `bun generator/user editor@example.com s3cret --modules user,editor`.

For an invitation-based flow (or to add the seed yourself), the legacy seed pattern still works: write an unverified user with an invitation code into `sql/sqlite/01-init-sqlite.sql`, run the SQL, then visit `/register/<email>/<invitation_code>` to finish the registration:

```sql
INSERT INTO users (id, email, invitation_code, modules_tags) VALUES
    (1, 'you@example.com', 'invite123', 'user,admin');
```

For production, drop any seeded users entirely once you have real accounts — see [Schema & Initialization](/database/schema-initialization#production-considerations).

<a name="generating-a-resource"></a>

## Generating Your First Resource

<media-frame label="Screenshot — generated /authors list view" ratio="16/9"></media-frame>

The CRUD generator scaffolds a complete admin module from a database table — list view, create/edit form, validation, route handlers, the lot. Add a table to `sql/sqlite/01-init-sqlite.sql`:

```sql
DROP TABLE IF EXISTS authors;

CREATE TABLE authors (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    name TEXT DEFAULT '' NULL,
    email TEXT DEFAULT '' NULL,
    created_at DATETIME DEFAULT current_timestamp,
    updated_at DATETIME DEFAULT NULL
);

CREATE INDEX authors_name ON authors(name);

CREATE TRIGGER authors_update_timestamp
AFTER UPDATE ON authors
FOR EACH ROW
BEGIN
    UPDATE authors SET updated_at = current_timestamp WHERE id = NEW.id;
END;
```

Apply the new schema with `bun tui` → "Run SQL file" (or `sqlite3 app.db < sql/sqlite/01-init-sqlite.sql` for SQLite). Then scaffold the CRUD module — the TUI has options for this too:

```bash
bun tui
# pick "Single table" → enter "authors"
# or "All tables" to scaffold everything in the database in one shot
```

Behind the scenes the TUI runs `bun generator/resource crud authors`, writes a complete CRUD module to `routes/authors/`, and registers the route in `routes/routes.ts`. Visit `/authors` and you have a working list page; `/authors/new` creates a record; `/authors/:id/edit` edits one.

The full generator reference is in [Generators](/database/generators).

<a name="the-project-structure"></a>

## What's Where

A fresh Reepolee project has a small, opinionated layout. The folders you'll touch day-to-day:

| Path               | Purpose                                                                       |
| ------------------ | ----------------------------------------------------------------------------- |
| `routes/`          | Route handlers, templates, queries, and translations — one folder per feature |
| `components/`      | Reusable `.ree` components (form inputs, banners)                             |
| `lib/`             | Framework helpers (render, middleware, template engine)                       |
| `config/`          | Database driver, language list, generator settings                            |
| `css/app.css`      | The Tailwind entry point — design tokens, base styles                         |
| `static/`          | Static files served directly to the browser                                   |
| `sql/`             | Per-dialect schema + seed files (`sql/sqlite/`, `sql/mysql/`)                 |
| `server.ts`        | The HTTP server entry point — rarely needs editing                            |
| `routes/routes.ts` | The top-level route table — wires features into URL paths                     |

[Project Structure](/getting-started/project-structure) covers each in more detail.

<a name="next-steps"></a>

## Next Steps

You have a running Reepolee application. From here, the natural reading order:

- **[The Basics](/the-basics/routing)** — how routes, controllers, and middleware compose.
- **[Ree Templates](/ree-templates/introduction)** — the templating language, in detail.
- **[Forms](/forms/introduction)** — the input components, validation, toasts, and uploads patterns that drive most of what you'll build.
- **[Database](/database/getting-started)** — the SQL API, schema management, and the generator.
- **[Security](/security/authentication)** — the auth flow, authorization tags, and the invitation system.

For a tutorial-style walkthrough of building a real application, see the [Recipes](/recipes/build-a-crud-app) section.
