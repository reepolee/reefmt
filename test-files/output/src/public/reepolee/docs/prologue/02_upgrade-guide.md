---
title: "Upgrade Guide"
---

# Upgrade Guide

<a name="introduction"></a>

## Introduction

Reepolee upgrades work differently from most frameworks because Reepolee isn't a dependency. There's no `npm install reepolee@latest` to run; you generate your project from the Reepolee [template repository](https://docs.github.com/en/repositories/creating-and-managing-repositories/creating-a-repository-from-a-template), and the version you have is the version you're working with. Upgrading means pulling the changes from `upstream` (Reepolee's repository, registered as a second remote on your project) into your own repository — usually a clean merge, occasionally a small migration if a public-facing pattern has changed.

This page covers the general flow, what kinds of changes constitute breaking changes, and the version-by-version migration steps when one is needed. The current state of each release is in [Release Notes](/prologue/release-notes).

<a name="philosophy"></a>

## The Upgrade Philosophy

Reepolee's approach to versioning trades some convenience for a property we care about: **upgrades shouldn't surprise you.** A few things follow from that:

- **Zero runtime dependencies** — the framework can't break because a transitive dependency shipped a regression. Reepolee can only break because Reepolee changed.
- **You own the framework code** — `lib/`, `components/`, `config/db_*.ts`, the generator. If an upgrade adjusts a function signature in `lib/render.ts`, the change is right there in your repo to read; nothing in `node_modules` is mysteriously different.
- **Templates and generated code are stable** — once `bun generator/resource crud users` writes `routes/users/index.ts`, that file is yours forever. Upgrades don't reach into generated routes.
- **Breaking changes are minimal and explicit** — when one is necessary, it's documented in the release notes and on this page with the exact steps to take. The default expectation is "pull main, resolve any merge conflicts in customised files, run."

This shape is what lets a project from a year ago still build today without you doing anything other than `bun install` and `bun run css:build`.

<a name="general-upgrade-flow"></a>

## The General Upgrade Flow

The mechanical steps are the same for every release. Reepolee ships two `package.json` scripts to keep the git plumbing terse:

| Script             | What it runs                                                                        |
| ------------------ | ----------------------------------------------------------------------------------- |
| `bun git:add-sync` | `git remote add upstream git@github.com:reepolee/reepolee.git` — run once per clone |
| `bun git:sync`     | `git fetch upstream && git rebase upstream/main` — run any time you want the latest |

```bash
# Add the upstream remote (once per clone)
bun git:add-sync

# Pull the latest upstream main into your branch
bun git:sync

# Or, to pin to a specific tagged release instead of tracking main
git fetch upstream --tags
git merge v0.4.0

# Resolve any merge conflicts (see below)
# Then run the rest of the deploy sequence
bun install --frozen-lockfile
bun run css:build
```

`bun git:sync` rebases onto `upstream/main`, so it assumes you push your fork's work to a feature branch (or are comfortable with a rebased `main`). For version-pinned upgrades — picking up `v0.4.0` exactly — fetch the tags and merge by tag, as in the third block above.

Conflicts almost always happen in three places:

- **`routes.ts`** — your route registrations vs new routes the template added.
- **`sql/sqlite/`** / **`sql/mysql/`** — your schema vs the template's. Almost always wins your version; the template's seed data is for the demo.
- **Translation files** — keys you've added vs new keys the template ships.

Framework code (`lib/`, `components/`, `config/db_*.ts`) rarely conflicts because most projects don't edit those files. When it does, take the upstream version — your customisations to those files belong in your own modules anyway.

After resolving conflicts, the deploy commands stay identical. There's no "post-upgrade migration" step because there's no framework state to migrate.

<a name="what-counts-as-breaking"></a>

## What Counts as a Breaking Change

The following are breaking and get explicit migration steps below:

- **Renamed or removed functions** in `lib/` or `routes/system/auth/` that you might have imported.
- **Removed or renamed translation keys** that templates reference.
- **Schema changes** to the `sessions`, `users`, or `email` tables that an existing database needs to match.
- **Renamed configuration constants** in `config/db_structure.ts`.
- **Behaviour changes** in `render()`, `translated_from_request()`, `resolve_session()`, or any of the middleware exports.

The following are _not_ breaking and don't need migration:

- New files added under `lib/`, `components/`, `routes/system/auth/`. Take them or leave them.
- New optional fields on existing options objects (e.g., a new optional argument to `send_mail`).
- Bug fixes that change observable but undocumented behaviour.
- Internal refactors within `lib/` files where the public surface is unchanged.
- Changes to `config/db.ts`, `config/db_mysql.ts`, or `config/db_sqlite.ts` — internal refactors don't change which database you connect to; that's controlled by `CONNECTION_STRING` in your `.env`.

When in doubt, the [Release Notes](/prologue/release-notes) for the version describe what's changed.

<a name="upgrading-from-0-2-x"></a>

## Upgrading From 0.2.x to 0.3.x

The 0.2 series was Reepolee-internal and the public surface differs substantially from 0.3.x. The recommended path is **start fresh from a 0.3.x template** and port your routes and templates by hand:

1. **Create a new 0.3.x project** alongside the old one.
2. **Copy `routes/` directory contents** for your custom routes. Update import paths — many `lib/` modules were renamed in 0.3.0.
3. **Copy `init-*.sql`** for your schema. Re-run the generator to produce 0.3.x-shape route folders for any tables you want to scaffold from scratch.
4. **Copy `static/` assets** (images, custom JS). The shipped scripts (`form-controller.js`, `spa-loader.js`, etc.) are different — take the 0.3.x versions.
5. **Copy `css/app.css`** but rebuild the `@theme` block — the token names changed for consistency.
6. **Re-translate your `routes/<lang>.json`** files into the 0.3.x namespace structure.

The old 0.2.x project keeps running while you build the 0.3.x version. Cut over when the new one is ready.

If your fork only diverges from upstream by a small number of files, a direct merge with conflict resolution might be faster — but the rename-heavy nature of the 0.3.0 release means most projects find a clean rebuild less work.

<a name="upgrading-within-0-3-x"></a>

## Upgrading Within 0.3.x

Patch and minor releases within the 0.3 line follow the [general upgrade flow](#general-upgrade-flow). No version-specific migration steps are documented yet — when one is needed, it appears here with a "From 0.3.x to 0.4.0" heading.

For the current state of what's in 0.3.x, see the [release notes entry](/prologue/release-notes#version-0-3-x).

<a name="upgrading-bun"></a>

## Upgrading Bun

Reepolee tracks Bun's stable releases. Upgrading Bun itself is independent of upgrading Reepolee:

```bash
bun upgrade               # latest stable
bun upgrade --canary      # latest canary build
```

Reepolee pins a Bun version in its README's "Bun version" line. Run with a Bun version that's the pinned version or newer; older versions may be missing APIs Reepolee uses (`Bun.password`, `Bun.SQL`, the import-text attribute, `Bun.redis`).

For long-running production servers, pinning Bun explicitly avoids surprises. The [systemd unit](/deployment/systemd) calls Bun by absolute path (`/home/deploy/.bun/bin/bun`) — running `bun upgrade` updates that binary in place. To pin a specific version, install it explicitly:

```bash
curl -fsSL https://bun.sh/install | bash -s "bun-v1.3.13"
```

Then leave it alone until you've tested an upgrade in a non-production environment.

<a name="reporting-an-upgrade-issue"></a>

## Reporting an Upgrade Issue

If an upgrade breaks something that isn't documented as a breaking change, that's a bug — open an issue at [github.com/reepolee/reepolee/issues](https://github.com/reepolee/reepolee/issues) with:

- The version you upgraded from and the version you upgraded to.
- The actual error or unexpected behaviour.
- The smallest reproduction (a route, a template, a config snippet).

We track upgrade-breaking-without-warning issues with high priority because the trust that an upgrade is safe is the property the whole versioning approach rests on.
