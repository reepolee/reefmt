---
title: "Cloudflare Pages"
layout: "reeweb/docs/docs.layout"
---

# Cloudflare Pages

<a name="introduction"></a>

## Introduction

Cloudflare Pages is the recommended deployment target for Reeweb sites. It reads the `_redirects` file natively, provides a global CDN with automatic HTTPS, and has a generous free tier. This page covers manual and automated (Git-based) deployment.

<a name="why-cloudflare-pages"></a>

## Why Cloudflare Pages

- **`_redirects` native support** — Reeweb emits `_redirects` in Cloudflare's format, so 301/302 redirects work at the edge without any configuration.
- **Global CDN** — 330+ data centres, automatic HTTPS, HTTP/3 support.
- **Free tier** — 500 builds per month, unlimited sites, 1 GB storage.
- **Automatic Git integration** — connects to your GitHub or GitLab repository.

<a name="manual-deployment"></a>

## Manual Deployment

Upload the contents of `dist/` to Cloudflare Pages manually:

1. Build the site: `bun run build`
2. Go to the [Cloudflare Dashboard](https://dash.cloudflare.com/) → Pages
3. Click **Create a project** → **Direct Upload**
4. Drag and drop the `dist/` folder
5. Set the production URL and deploy

This works for one-off deployments. For ongoing updates, use Git integration instead.

<a name="git-integration"></a>

## Git Integration

To connect your repository:

1. Push your project to GitHub or GitLab
2. Go to Cloudflare Pages → **Create a project** → **Connect to Git**
3. Select your repository
4. Configure the build settings:

| Setting                    | Value                                 |
| -------------------------- | ------------------------------------- |
| **Build command**          | `bun run build`                       |
| **Build output directory** | `/dist`                               |
| **Root directory**         | (leave blank — project root)          |
| **Environment variables**  | Add `SITE_URL=https://yourdomain.com` |

Cloudflare Pages detects Bun automatically from the `bun.lock` file. No additional configuration needed.

<a name="redirects-on-cloudflare"></a>

## Redirects on Cloudflare

Reeweb's `_redirects` file is emitted automatically when you declare redirects in `config/redirects.ts`. Cloudflare Pages reads this file at the edge — redirects are applied before any HTML is served, which means zero round-trips for redirect traffic.

The `_redirects` format includes both trailing-slash and non-trailing-slash variants:

```
/old-page /new-page 301
/old-page/ /new-page 301
```

This ensures both `/old-page` and `/old-page/` redirect correctly.

<a name="custom-domain"></a>

## Custom Domain

In the Cloudflare Pages dashboard:

1. Go to your project → **Custom domains**
2. Click **Set up a custom domain**
3. Enter your domain and follow the DNS instructions
4. Cloudflare automatically provisions an SSL certificate

If your domain is already on Cloudflare, the DNS is configured automatically. If not, Cloudflare will give you nameservers to update at your registrar.
