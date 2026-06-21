---
title: "Generic Static Hosting"
layout: "reeweb/docs/docs.layout"
---

# Generic Static Hosting

<a name="introduction"></a>

## Introduction

Reeweb's output is standard static HTML — no server-side rendering, no runtime, no database. Any web server or static host can serve it. This page covers the most common alternatives to Cloudflare Pages.

<a name="netlify"></a>

## Netlify

Netlify is a popular static hosting platform with its own `_redirects` support (same format as Cloudflare).

### Git Integration

1. Push your project to GitHub/GitLab/Bitbucket
2. Go to [app.netlify.com](https://app.netlify.com) → **Add new site** → **Import an existing project**
3. Select your repository
4. Configure:

| Setting                   | Value                                 |
| ------------------------- | ------------------------------------- |
| **Build command**         | `bun run build`                       |
| **Publish directory**     | `dist`                                |
| **Environment variables** | Add `SITE_URL=https://yourdomain.com` |

Netlify supports Bun natively — no need to specify a Node.js version.

### Manual Upload

```bash
bun run build
# zip dist/ and upload via Netlify UI
```

<a name="vercel"></a>

## Vercel

Vercel is another zero-config option:

1. Push to GitHub
2. Import in the [Vercel Dashboard](https://vercel.com)
3. Vercel auto-detects the project structure

Set `SITE_URL` in the Vercel project environment variables. The framework preset is "Other" — Vercel serves static files from `dist/`.

<a name="github-pages"></a>

## GitHub Pages

For free hosting on public repositories:

```yaml
# .github/workflows/deploy.yml
name: Deploy to GitHub Pages
on:
    push:
        branches: [main]

jobs:
    deploy:
        runs-on: ubuntu-latest
        steps:
            - uses: actions/checkout@v4
            - uses: oven-sh/setup-bun@v1
            - run: bun install
            - run: bun run build
              env:
                  SITE_URL: https://username.github.io
            - uses: peaceiris/actions-gh-pages@v3
              with:
                  github_token: ${{ secrets.GITHUB_TOKEN }}
                  publish_dir: ./dist
```

Then enable GitHub Pages in your repository settings, pointing to the `gh-pages` branch.

<a name="s3-and-cloudfront"></a>

## S3 + CloudFront

For full control over infrastructure:

1. Build: `bun run build`
2. Upload `dist/` to an S3 bucket:

```bash
aws s3 sync dist/ s3://my-site-bucket/ --delete
```

3. Configure the bucket for static website hosting
4. Place CloudFront in front of the bucket for HTTPS and CDN

The `_redirects` file doesn't work with S3 directly — you'll need to configure CloudFront Functions or Lambda@Edge to handle redirects, or use S3's own routing rules.

<a name="nginx-on-a-vps"></a>

## nginx on a VPS

```nginx
server {
    listen 80;
    server_name example.com;
    root /var/www/my-site/dist;

    # Directory index
    index index.html;

    # Pretty URLs — serve index.html for directory paths
    location / {
        try_files $uri $uri/ =404;
    }

    # Gzip
    gzip on;
    gzip_types text/html text/css application/javascript image/svg+xml;

    # Cache static assets
    location ~* \.(css|js|ico|svg|woff2)$ {
        expires 1y;
        add_header Cache-Control "public, immutable";
    }
}
```

For automatic builds on push, set up a GitHub Action that SSH-es into the VPS, pulls the latest code, and rebuilds.
