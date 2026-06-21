---
title: "Release Notes"
---

# Release Notes

<a name="introduction"></a>

## Introduction

This page tracks what changes between Reepolee releases. Each entry covers the version's user-facing additions, the bug fixes worth knowing about, and any breaking changes that need follow-up in your project (cross-linked to the [Upgrade Guide](/prologue/upgrade-guide) when an upgrade isn't a no-op).

Reepolee follows semantic versioning loosely while we're pre-1.0. The shape:

- **Patch releases** (`0.3.1` → `0.3.2`) — bug fixes, documentation updates, internal refactors. Always safe to upgrade in place.
- **Minor releases** (`0.3.x` → `0.4.0`) — new features, deprecations, occasional small breaking changes that have a documented one-step migration. Upgrade by reviewing the entry below.
- **Major releases** (`0.x` → `1.0`, eventually) — substantial breaking changes. Won't happen often.

After 1.0, breaking changes will move to major releases only — minor releases become additive-only. Until then, treat the entry below as the contract for what each release contains.

<a name="versioning-and-distribution"></a>

## Versioning and Distribution

Reepolee isn't published to a package registry. There's no `npm install reepolee` — you generate your project from the Reepolee [template repository](https://docs.github.com/en/repositories/creating-and-managing-repositories/creating-a-repository-from-a-template) on GitHub, and the version your repo was seeded from is the one you're working with. `package.json` tracks the version for diagnostic purposes (it appears in `props.version` and on the production logs); `bun run release` bumps the patch number.

To pull a specific Reepolee version into your project, register the upstream remote once with the shipped script, fetch the tag, and merge:

```bash
bun git:add-sync          # one-time: adds upstream → reepolee/reepolee
git fetch upstream --tags
git merge v0.4.0
```

To track upstream `main` instead of pinning to a tag, run `bun git:sync` (it fetches and rebases onto `upstream/main` in one step). Either way, resolve any conflicts with files you've customised (most often: `routes.ts`, your `sql/` schema files, your translation files). The framework code under `lib/` and `components/` rarely conflicts because most projects don't edit those files.

<a name="how-to-stay-informed"></a>

## How to Stay Informed

Three ways to know when a new release ships:

- **GitHub releases** — every release tag publishes notes on `github.com/reepolee/reepolee/releases`. Watching the repository (with the "Releases only" option) sends an email per release.
- **The `production` branch on this docs repo** — when these release notes are updated, that's a release. Subscribing to the RSS feed of [the docs site changelog](https://docs.rebun.com/community/changelog) covers it.
- **Following [@reepolee](https://twitter.com/reepolee) on Twitter** — release announcements with one-paragraph summaries, no other noise.

For deeper "what's about to ship" visibility, [the Roadmap](/community/roadmap) tracks larger initiatives in flight.

<a name="version-0-3-x"></a>

## 0.3.x — Current

The 0.3 series is the current line. Highlights:

- **Server-rendered Ree templating** with layouts, partials, components, and the `{@}` shorthand for `components/`. See [Ree Templates](/ree-templates/introduction).
- **Bun's native SQL** for SQLite and MySQL with a single `db` instance imported from `config/db.ts`. The driver is a one-line file copy. See [Database](/database/getting-started).
- **A complete CRUD generator** that introspects your database and produces routes, queries, validation, templates, and translations from the schema alone. See [Generators](/database/generators).
- **Invite-only authentication** with sessions in the database, a small KV abstraction so the session store is swappable, and tags for authorization. See [Authentication](/security/authentication).
- **Tailwind v4 styling** with design tokens in `@theme`, semantic utilities (`primary`/`secondary`/`tertiary`), and form-control defaults that mean templates rarely need per-input class lists. See [Styling](/styling/tailwind-setup).
- **Full internationalisation** — per-route translation files with auto-fallback across languages, and **localised URLs** generated automatically from `route_name` translation keys. See [Internationalization](/i18n/languages-and-locales).
- **Optional client-side enhancements** — fine-grained reactivity with `alien-deepsignals` for object reactivity (paired with a small self-contained `signals-ui.js` helper module for DOM bindings), an SPA-style page loader, and a small set of shipped web components. See [Client-Side](/client-side/signals-ui).
- **Production-ready deployment** with a systemd unit, GitHub Actions workflow, and Nginx configuration documented end-to-end. See [Deployment](/deployment/preparing-the-server).

The 0.3 series is the first publicly-recommended version. Everything in the documentation reflects what 0.3.x ships.

<a name="recent-additions"></a>

### Recent additions

Capabilities added across the latest 0.3.x point releases:

- **Redis-backed SQL cache** — opt-in caching of generated `search_records` queries with automatic dependency-set invalidation and a `/system/cache` admin view. Enable with `CACHE_ENABLED=true` + `REDIS_URL`. See [Caching](/database/caching).
- **Rate limiting** — sliding-window request limiting as the first global middleware, with per-scope tiers in `config/rate_limit.ts`. Enable with `RATE_LIMITING=true` + `REDIS_URL`. See [Rate Limiting](/security/rate-limiting).
- **CSRF protection** — always-on double-submit-cookie protection on state-changing requests; generated forms already include the token. See [CSRF Protection](/security/csrf).
- **DB-first translations** — the `translations` table is now the source of truth (JSON files in `routes/` are legacy seeds it overrides), edited live through the `/system/translations` admin module and seeded per-language from `sql/<dialect>/02-init-translations-<lang>.sql`. `sync:languages` AI-fills gaps directly in the DB; the **Prune unused** / **Sync missing** TUI tools keep the table aligned with templates. Missing keys render as their braced key path. See [Translations](/i18n/translations#runtime-overrides).
- **Pluralization** — `plural()` selects CLDR plural forms via `Intl.PluralRules` from pipe-separated translation strings.
- **More AI translation providers** — Ollama (local, highest priority) and Hugging Face Inference join OpenRouter. See [Dynamic Translations](/i18n/dynamic-translations#providers).
- **Global (row-level) scopes** — admin-defined, named `WHERE` filters per table via `/system/global_scopes`, folded into list queries and the cache key. See [Authorization](/security/authorization#global-scopes).
- **Nested CRUD (master-detail)** — `--parent <table>` generates parent-scoped child resources with inline child lists on the parent's edit form. A parent can have multiple children. See [Generators](/database/generators#nested-resources).
- **Generator improvements** — `ICU`/`CU` column-visibility flags, plain-word and JSON column-comment field types, `--refresh-fields` to regenerate field sections only, sort options from indexed columns, fulltext search via `lib/sql_dialect.ts`, and the `crud-ignore` table tag.
- **Pagination strategies & streaming** — generated list views choose **offset** (default) or **cursor** pagination via `--pagination` / the `pagination_strategy` schema export, and can stream records as Declarative Partial Updates (DPU) via `render_strategy: "stream"`. See [Pagination](/database/generators#pagination) and [Streaming](/database/generators#streaming).
- **Domain types** — a canonical column-type vocabulary in `config/domain_types/{mysql,sqlite}.ts`, with a **Check domain compliance** TUI tool that audits a schema against it. See [Schema & Initialization](/database/schema-initialization#domain-types).
- **Agent mode** — `--agent` runs the dev server headless, impersonating a user via the `X-Agent-User-Email` header or `AGENT_USER_EMAIL` (dev + localhost only). See [Authentication](/security/authentication#agent-mode).
- **CSS design system** — the `@theme` block expands into a full token set (brand/surface/primary/semantic colours, typography, radii), and generated list views gain a right slide-in **filter panel** (`css/filters.css`). See [Theming & Tokens](/styling/theming-and-tokens).
- **Add/remove language** — both are one-step TUI actions that seed or strip a language's rows directly in the `translations` table. See [Adding a New Language](/recipes/new-language).
- **SQL layout & dialect isolation** — schema/seed SQL moved to per-dialect folders (`sql/sqlite/`, `sql/mysql/`) with numbered files; engine-specific SQL is isolated in `lib/sql_dialect.ts`.
- **`{#with}` template directive** — scopes bare variable names against a nested object, used widely in generated templates. See [Displaying Data](/ree-templates/displaying-data#with-scope).
- **libvips installer** — `bun scripts/cli.ts vips` fetches a prebuilt libvips for the image-processing pipeline. See [Installation](/getting-started/installation#libvips).

<a name="version-0-2-x"></a>

## 0.2.x

Internal-use releases used at Reepolee. Not publicly documented; if you're upgrading from a fork of 0.2.x, the structural changes are large enough that a fresh start from a 0.3.x template is usually less work than merging the diff. See the [Upgrade Guide](/prologue/upgrade-guide) for the broad strokes.

<a name="version-0-1-x"></a>

## 0.1.x

Pre-public prototype releases. Not documented.

<a name="reporting-issues"></a>

## Reporting Issues

Found something broken? Open an issue at [github.com/reepolee/reepolee/issues](https://github.com/reepolee/reepolee/issues). Include the Reepolee version (the `version` field in `package.json`), the Bun version (`bun --version`), and the smallest reproduction you can manage.

For security issues, email security@reepolee.com directly rather than opening a public issue.
