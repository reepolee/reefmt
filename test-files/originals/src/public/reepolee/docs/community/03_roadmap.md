---
title: "Roadmap"
---

# Roadmap

<a name="introduction"></a>

## Introduction

This page tracks the larger initiatives in flight or on the horizon. Smaller fixes and incremental improvements happen continuously and don't get individual entries here — they're in [Release Notes](/prologue/release-notes) when they ship.

Reepolee ships when things are ready, not on a fixed schedule. Items below are organised by horizon (next release, this year, longer term) rather than by date. An item moving from "longer term" to "next release" is the most reliable signal that work has actually started.

<a name="next-release"></a>

## Next Release (0.4.x)

Items being worked on for the next minor version.

### Native Bun SMTP client

The current `lib/smtp.ts` is built on Bun's Node compatibility layer (`node:net` and `node:tls`). When Bun adds a native SMTP client to its standard library, we'll swap the implementation. The `send_mail()` API will stay identical so application code doesn't need to change. See [Sending Email](/email/sending-email).

### Generator support for views

The CRUD generator currently produces routes from base tables. For complex list pages backed by SQL views (`v_frameworks` joining `frameworks`, `authors`, and `languages`), the generator will detect the view and route the list page through it automatically. The `table.ts` `select_table` override that currently does this manually becomes the default.

### `bun --parallel` for the dev script

The current `bun dev` script uses `concurrently` (installed globally) to run the Tailwind watcher and the dev server side-by-side. When `bun --parallel` ships in stable, the global `concurrently` dependency goes away — `bun dev` will run both processes natively.

<a name="this-year"></a>

## This Year (2026)

### Documentation expansion

This documentation site is at first-pass coverage as of May 2026. The next pass focuses on:

- **Cookbook recipes** for more advanced patterns: search-as-you-type, file-based attachments, multi-tenant isolation, scheduled jobs, integrating with external auth providers.
- **A "From SvelteKit / Laravel / Rails" comparison page** for each major server-side framework, showing the equivalent Reepolee pattern for common idioms.
- **A migration guide section** for the eventual 0.x → 1.0 transition.

### A test suite for the framework code

Reepolee ships with `bun test` available but doesn't currently have comprehensive coverage of the framework itself. The plan: a test suite for `lib/` modules (template engine, render layer, middleware, helpers) that runs in CI on every commit. Application-level testing patterns will get their own [recipe](/recipes/build-a-crud-app).

### A `reepolee new <project>` CLI

The current "start a new project" flow is `git clone reepolee my-app`. A small CLI that fetches the latest tagged release, drops in your project name and a fresh `.env`, and optionally runs the database driver copy step. Saves three or four commands per project setup.

<a name="longer-term"></a>

## Longer Term

Items that are real ambitions but not actively in flight.

### Reepolee 1.0

The 1.0 release is when we feel the public surface (`render()`, `translated_from_request()`, the middleware exports, the generator conventions) is stable enough that breaking changes become rare and explicit. Going by current trajectory, that's late 2026 or 2027 — depends as much on how the API holds up under real-world use as on shipping any particular feature.

The [Upgrade Guide](/prologue/upgrade-guide) covers the philosophy of upgrades pre-1.0; post-1.0, semver becomes strict and breaking changes only happen at major versions.

### Background jobs and scheduled work

Many projects need either "run this every N minutes" or "queue this to run later." Reepolee doesn't currently provide either; the workaround is a small worker process that polls a `pending_jobs` table or uses Bun's `setInterval`. We're considering a built-in lightweight job runner that uses the existing database-backed queue pattern with no additional infrastructure. The decision is whether the scope is worth the surface area it adds.

### A managed Reepolee host

The framework targets self-hosted deployments. A managed option — push to a branch, deployment happens, TLS/scaling/backups handled — would lower the floor for projects that don't want to operate a server. This is more of a Reepolee product question than a framework question; the framework would stay deployable on any Linux box regardless.

### Real-time features

WebSocket plumbing is built into Bun and has been used in Reepolee's dev-mode live-reload. Productising it as an application-level pattern (rooms, broadcast, presence) is a real opportunity but a sizeable surface area. The current direction is to wait until enough applications need it that the right shape becomes obvious, rather than designing a system in advance and hoping it fits.

<a name="not-on-the-roadmap"></a>

## Not on the Roadmap

A few things we don't currently plan to add — not because they're bad ideas, but because they conflict with what Reepolee is:

- **A separate ORM layer.** Bun's `SQL` API is the abstraction; queries are SQL. An ORM is a different framework with different trade-offs.
- **A client-side router.** The SPA loader gives you SPA-feeling navigation without a router. For applications that need a true single-page-app experience, the right tool is a true SPA framework.
- **A built-in component library.** The shipped `components/` set covers form inputs and the banner. Extending into navigation components, modals, data tables, etc., expands the project's surface in a direction we've deliberately stayed narrow on. Build them in your project — they're yours, and they fit your visual language.
- **Plugin system / extensions.** Reepolee's extension model is "edit the file." Adding plugin hooks adds API surface that has to stay stable; "edit the file" stays flexible forever.

<a name="influencing-the-roadmap"></a>

## Influencing the Roadmap

The single most useful input we get is "I tried to do X and it was awkward in this specific way." Concrete pain — a use case that didn't fit — is what moves items up the priority list. Abstract feature requests ("Reepolee should support X") are harder to act on without that context.

The forum for this is [GitHub Discussions](https://github.com/reepolee/reepolee/discussions). Tag a thread with `roadmap` if it relates to something on this page; start a new discussion if it's a fresh idea.

For sponsors, the relationship is more direct — sponsorship doesn't buy roadmap influence per se, but it does buy a regular conversation about what you're building and where it's friction-y. See [Sponsors](/community/sponsors).
