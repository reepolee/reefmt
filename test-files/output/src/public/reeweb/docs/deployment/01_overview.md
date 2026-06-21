---
title: "Deployment Overview"
layout: "reeweb/docs/docs.layout"
---

# Deployment Overview

<a name="introduction"></a>

## Introduction

Reeweb generates a fully static site — HTML, CSS, JavaScript, images, and fonts. There is no server process to keep running, no database connection to maintain, and no runtime to monitor. The output in `dist/` can be deployed to any static file host.

<a name="the-build-output"></a>

## The Build Output

Running `bun run build` produces a `dist/` directory with this structure:

```
dist/
├── index.html              ← Default language homepage
├── about/
│   └── index.html          ← Default language /about/
├── en/
│   ├── index.html          ← English homepage
│   └── about/
│       └── index.html      ← English /about/
├── blog/
│   └── post-title/
│       └── index.html      ← Blog post
├── css/
│   └── style.css           ← Compiled Tailwind CSS
├── _redirects              ← Cloudflare-style redirect rules
├── favicon.ico
└── ...
```

Each page is rendered as `index.html` inside a directory named after the URL path. This is the standard pattern for static hosting — `example.com/about/` serves `about/index.html` automatically on most hosts.

<a name="choosing-a-host"></a>

## Choosing a Host

Any static file host that serves `index.html` from directories works with Reeweb. The most common options:

| Host                 | Notes                                                           |
| -------------------- | --------------------------------------------------------------- |
| **Cloudflare Pages** | Reads `_redirects` natively, global CDN, generous free tier     |
| **Netlify**          | Reads `_redirects` natively, form handling available            |
| **Vercel**           | Zero-config, automatic HTTPS, global CDN                        |
| **GitHub Pages**     | Free for public repositories, push `dist/` to `gh-pages` branch |
| **S3 + CloudFront**  | Full control, pay-per-request pricing                           |
| **Any VPS**          | Serve with nginx, Caddy, or any static file server              |

<a name="deploying-with-bun-run-preview"></a>

## Previewing Locally

Before deploying, preview the built site:

```bash
bun run build
bun run preview
```

The preview server (`scripts/preview.ts`) serves `dist/` on `http://localhost:3000` with proper MIME types and directory index resolution. It detects language subdirectories automatically.

<a name="environment-variables-for-production"></a>

## Environment Variables for Production

Set `SITE_URL` in your production environment before building:

```env
SITE_URL=https://example.com
```

When set, the build generates hreflang alternate links for each language — required by Google for multi-language SEO. Without it, hreflang links are skipped (the site still builds and works correctly).

<a name="common-tasks"></a>

## Common Tasks

- **Rebuilding on every push** — see the individual deployment pages for CI/CD configuration
- **Custom domains** — configure through your host's dashboard
- **HTTPS** — all modern static hosts provide automatic HTTPS
- **Cache headers** — set long cache durations on the versioned CSS file, short cache on HTML
