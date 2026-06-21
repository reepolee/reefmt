---
title: "Changelog"
---

# Changelog

<a name="introduction"></a>

## Introduction

This page is the time-ordered journal of significant project moments — releases, public announcements, doc reorganisations, ecosystem milestones. It complements the version-organised [Release Notes](/prologue/release-notes), which is the canonical reference for what changed in each version. Use this page when you want a "what's been happening lately" view; use Release Notes when you want to know what's in a specific version.

Entries are reverse-chronological — newest first. Each entry has a date, a one-line headline, and a short paragraph of context.

<a name="2026-05"></a>

## May 2026

### 2026-05-12 — Documentation restructure

The documentation site was reorganised end-to-end into 14 top-level sections following a Laravel-shaped layout: Prologue, Getting Started, The Basics, Ree Templates, Forms, Database, Security, Client-Side, Styling, Internationalization, Email, Deployment, Recipes, Community, Enterprise. The Ree Templates section was split from a single page into seven Vue-style subpages. Forms became a top-level cluster covering input components, validation, toasts, and file uploads. The Internationalization section was created from scratch — i18n is real in the codebase but had been undocumented.

The previous URL structure (`/getting-started/why-reepolee`, `/core-concepts/...`, `/features/...`, `/guides/...`) is no longer in use. Most public links now have direct equivalents under the new structure; a few — `Why Reepolee`, the original deployment guide — kept the same URL semantics under their new section homes.

<a name="2026-04"></a>

## April 2026

### 2026-04-08 — Reepolee 0.3.1

Patch release. Fixes around session-cookie encoding when special characters are present in the session payload, and a small ergonomic improvement to `extract_params_from_url` (now defaults `count = 1`).

### 2026-04-01 — Reepolee 0.3.0 publicly recommended

The 0.3 series is the first version we recommend for new projects. Significant changes from the internal 0.2 line are documented in the [Upgrade Guide](/prologue/upgrade-guide#upgrading-from-0-2-x); the recommended path is starting fresh from a 0.3.x template rather than merging.

<a name="2026-03"></a>

## March 2026

### 2026-03-15 — VSCode extension reaches 1.0

The [Ree Templates extension](https://marketplace.visualstudio.com/items?itemName=reepolee.ree-templates) for VSCode hit a stable 1.0 release. Syntax highlighting and formatting for `.ree` files now match the engine's full grammar, including the `{@}` component shorthand and the alias-path resolution (`$components/`, `$routes/`, `$lib/`).

<a name="how-this-page-works"></a>

## How This Page Works

Three rules for what goes here:

- **Releases** of Reepolee and the VSCode extension.
- **Significant docs changes** — restructures, new tutorial sections, deprecated pages.
- **Ecosystem moments** — talks given, blog posts about Reepolee, third-party projects that integrate.

Bug fixes, internal refactors, and individual blog posts don't make the cut — they're either in the Release Notes per-version detail or on the Reepolee blog directly. The bar for an entry here is "someone tracking Reepolee lazily would want to know about this."

For a feed of these entries, subscribe to the docs repo's GitHub releases or the Reepolee blog's RSS. We don't yet publish a separate Atom feed for the changelog itself; that's on the [Roadmap](/community/roadmap).

<a name="contributing-an-entry"></a>

## Contributing an Entry

If you've built something with Reepolee that's worth a mention here — a public project, a third-party integration, a library that pairs well — open a PR against this page or send a note via [GitHub Discussions](https://github.com/reepolee/reepolee/discussions). We're happy to surface community work; the only filter is "would another Reepolee user benefit from knowing this exists."
