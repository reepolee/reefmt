---
title: "Redirect System"
layout: "reeweb/docs/docs.layout"
---

# Redirect System

<a name="introduction"></a>

## Introduction

Reeweb's redirect system maps old or external URLs to new destinations. Redirects are declared in `config/redirects.ts` and emitted during the build to two outputs:

1. **`dist/_redirects`** — a Cloudflare-compatible redirect rules file (with and without trailing slash variants)
2. **HTML stubs** — per-redirect `dist/{from}/index.html` pages with meta-refresh tags (fallback for non-Cloudflare hosts and local preview)

The system runs in **two phases** during the build, catching configuration errors early and ensuring no collisions with generated content.

<a name="declaring-redirects"></a>

## Declaring Redirects

Edit `config/redirects.ts`:

```ts
export const redirects: Redirect[] = [
	{ from: "/resume", to: "/files/resume-2026-v3.pdf" },
	{ from: "/old-page", to: "/new-page", status: 301 },
	{ from: "/talk", to: "https://youtube.com/watch?v=...", status: 302 },
];
```

Each redirect entry has:

| Field    | Required | Type   | Default | Description                                                    |
| -------- | -------- | ------ | ------- | -------------------------------------------------------------- |
| `from`   | ✅       | string | —       | Source path. Must start with `/`, no file extensions           |
| `to`     | ✅       | string | —       | Target path or URL. Internal paths are validated at build time |
| `status` | ❌       | number | `301`   | HTTP status: `301` (permanent) or `302` (temporary)            |

<a name="path-rules"></a>

## Path Rules

### `from` Rules

- Must start with `/` — absolute paths only
- Last segment must not contain a `.` — file extensions are not allowed (use pretty URLs only)
- Must be unique — duplicate `from` paths are rejected
- Trailing slashes are normalized for uniqueness comparison

Valid:

```
/resume
/old-about-page
/blog-old/2024
```

Invalid (will fail validation):

```
old-about-page          ← doesn't start with /
/resume.pdf             ← contains file extension
```

### `to` Rules

- Must be a non-empty string
- Can be an internal path (`/new-page`, `/files/resume.pdf`)
- Can be an external URL (`https://example.com/page`)
- Internal targets are validated for existence in `dist/` after the build is complete

<a name="how-it-works-dual-output"></a>

## How It Works: Dual Output

Every redirect produces two artifacts:

### 1. `_redirects` File (Edge-Level)

The `dist/_redirects` file contains Cloudflare-compatible redirect rules. Each redirect produces **two lines** — one with and one without trailing slash — so both `/resume` and `/resume/` redirect correctly:

```
/resume /files/resume-2026-v3.pdf 301
/resume/ /files/resume-2026-v3.pdf 301
/old-page /new-page 301
/old-page/ /new-page 301
```

On Cloudflare Pages, these rules are applied at the edge before any HTML is served — zero round-trips for redirect traffic.

### 2. HTML Stub (Fallback)

Each redirect also generates an HTML stub at `dist/{from}/index.html` with:

- A `<meta http-equiv="refresh">` tag for instant redirect
- A `<link rel="canonical">` pointing to the target (internal targets only)
- A visible link to the target for users whose browser doesn't support meta-refresh

```html
<!DOCTYPE html>
<meta charset="utf-8" />
<title>Redirecting…</title>
<meta http-equiv="refresh" content="0;url=/files/resume-2026-v3.pdf" />
<link rel="canonical" href="/files/resume-2026-v3.pdf" />
<p>Redirecting to <a href="/files/resume-2026-v3.pdf">/files/resume-2026-v3.pdf</a>…</p>
```

This ensures redirects work on any static host, not just Cloudflare Pages.

<a name="two-phase-validation"></a>

## Two-Phase Validation

The build validates redirects in two phases to catch errors early and prevent silent footguns.

### Phase 1: Schema Validation (Before Build)

Runs immediately, before any rendering work. Validates:

- `redirects` is an array
- Each entry is an object with `from` and `to` strings
- `from` starts with `/` and has no file extension
- `to` is non-empty
- `status` is `301` or `302` (if present)
- No duplicate `from` paths (trailing slashes normalized)

If any check fails, the build stops with a clear error message pointing to the specific entry.

### Phase 2: Collision & Target Validation (After Build)

Runs after all pages are rendered and static assets are copied to `dist/`. Checks:

1. **Collision with generated pages** — does the `from` path match a rendered page URL?
2. **Collision with static assets** — does the `from` path match a file that was copied to `dist/`?
3. **Target existence** — does the internal `to` path resolve to an actual file in `dist/`?

If any check fails, the build stops with a specific error message. External URL targets (`https://...`) are not fetched or validated at build time.

<a name="redirects-and-localized-routes"></a>

## Redirects and Localized Routes

Redirects use literal `from` paths — there is no per-language fan-out. A redirect from `/resume` redirects `/resume` regardless of the active language. If you need language-specific redirects, add separate entries:

```ts
export const redirects: Redirect[] = [
	{ from: "/resume", to: "/files/resume-2026-v3.pdf" },
	{ from: "/en/resume", to: "/en/files/resume-2026-v3.pdf" },
];
```

<a name="common-patterns"></a>

## Common Patterns

### Permanent Redirect (301)

```ts
{ from: "/old-blog-post", to: "/blog/new-blog-post", status: 301 }
```

Default `status` is `301`, so you can omit `status` for permanent redirects.

### Temporary Redirect (302)

```ts
{ from: "/summer-sale", to: "/products/summer", status: 302 }
```

Use `302` for time-limited promotions, event links, or any redirect you plan to revert.

### External Redirect

```ts
{ from: "/github", to: "https://github.com/reepolee/reeweb", status: 302 }
```

External targets are not validated at build time. Use `302` for external links so you're not permanently committing to the destination.

### File Download Redirect

```ts
{ from: "/resume", to: "/files/resume-2026-v3.pdf" }
```

The `from` path has no file extension (pretty URL). The target can point to a file — the build will verify it exists.
