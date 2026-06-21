---
title: "Upgrade Guide"
layout: "reeweb/docs/docs.layout"
---

# Upgrade Guide

<a name="introduction"></a>

## Introduction

Reeweb is currently in alpha (v0.1.x). During this phase, breaking changes are expected and documented here. Each entry covers the change, what you need to update, and migration commands where applicable.

For the latest release, see the [Release Notes](/reeweb/docs/prologue/release-notes).

<a name="v0-1-0-initial"></a>

## Upgrading to 0.1.0

This is the initial release. There are no previous versions to upgrade from. See the [Installation](/reeweb/docs/getting-started/installation) page for setup instructions.

<a name="keeping-up-to-date"></a>

## Keeping Up to Date

Reeweb is updated via the starter template's git repository. To pull in the latest changes:

```bash
# If you cloned the starter repository:
git pull origin main
bun install
```

### What to watch for

- **`lib/` directory** — this mirrors the upstream library and should not be modified. If the upstream updates these files, `git pull` will attempt to merge changes. Resolve conflicts by accepting the upstream version and re-applying your customisations in `src/lib/project_helpers.ts`.
- **`config/` files** — if the shape of `supported_languages.ts` or `redirects.ts` changes, the build or dev server may fail until you update your config to match.
- **`scripts/` directory** — build, dev, and preview scripts are part of the upstream. If you've modified them, `git pull` will attempt to merge your changes with upstream changes.

### Safe files to customise

These files are yours and won't be overwritten by upstream updates:

| Path                            | Purpose                           |
| ------------------------------- | --------------------------------- |
| `src/public/`                   | Templates, markdown, translations |
| `src/components/`               | Site-specific components          |
| `src/css/style.css`             | Tailwind source and theme         |
| `src/lib/project_helpers.ts`    | Custom template helpers           |
| `config/supported_languages.ts` | Language configuration            |
| `config/redirects.ts`           | Site redirects                    |
| `.env`                          | Environment variables             |

<a name="reporting-issues"></a>

## Reporting Issues

If an upgrade breaks your project, open a [GitHub Discussion](https://github.com/reepolee/reeweb/discussions) with:

- Your Reeweb version (from `package.json`)
- The version you're upgrading **to**
- The error message or unexpected behaviour
- A minimal reproduction if possible
