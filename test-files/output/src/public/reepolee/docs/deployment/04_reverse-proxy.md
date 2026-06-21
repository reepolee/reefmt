---
title: "Reverse Proxy"
---

# Reverse Proxy

<a name="introduction"></a>

## Introduction

Reepolee listens on plain HTTP on whatever port `PORT` is set to (`2338` by default). To serve it on the internet over HTTPS, you put a reverse proxy in front of it that terminates TLS, listens on port 443, and forwards requests to the application over the loopback interface. Nginx is the path of least resistance — it's on every Linux distribution, the configuration is short, and Certbot integrates with it directly for free TLS certificates.

This page covers the Nginx config and the Certbot setup. The same pattern works for Caddy, Apache, HAProxy, or any other reverse proxy — only the syntax differs.

<a name="why-a-reverse-proxy"></a>

## Why Not Just Bind to 443

Bun can listen on port 443 directly with TLS. For a single-application server with no other concerns, that's a viable setup. A reverse proxy in front buys you a few things that are useful as you grow:

- **TLS certificate management** without touching the application. Certbot handles renewal automatically; the application doesn't need to restart.
- **Multiple applications on one server.** Run staging on `2339`, production on `2338`, a marketing site on `2340` — Nginx routes by hostname.
- **Static-file serving with finer cache control.** The proxy can serve assets directly from disk faster than the application can.
- **Request logging, gzip, header rewriting** — all configurable in the proxy without touching the application.

For a small project where none of those matter, you can skip the proxy. For most production deployments, the proxy is worth the ten lines of configuration.

<a name="nginx-config"></a>

## A Minimal Nginx Site

A site configuration that proxies HTTPS to the application:

```nginx
server {
    listen 80;
    server_name yourdomain.com;
    return 301 https://$host$request_uri;
}

server {
    listen 443 ssl http2;
    server_name yourdomain.com;

    ssl_certificate     /etc/letsencrypt/live/yourdomain.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/yourdomain.com/privkey.pem;

    # Strong defaults; the Certbot-managed include adds Mozilla's recommendations.
    include /etc/letsencrypt/options-ssl-nginx.conf;
    ssl_dhparam /etc/letsencrypt/ssl-dhparams.pem;

    location / {
        proxy_pass http://127.0.0.1:2338;
        proxy_set_header Host              $host;
        proxy_set_header X-Real-IP         $remote_addr;
        proxy_set_header X-Forwarded-For   $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_http_version 1.1;
    }
}
```

Save this to `/etc/nginx/sites-available/yourdomain.com`, symlink it into `sites-enabled`, and reload Nginx:

```bash
sudo ln -s /etc/nginx/sites-available/yourdomain.com /etc/nginx/sites-enabled/
sudo nginx -t                  # validate the config
sudo systemctl reload nginx
```

Two `server` blocks: one to redirect plain HTTP to HTTPS, one to terminate TLS and proxy to the application. The redirect is what makes typing `yourdomain.com` (without `https://`) work — the browser hits port 80 first, gets a `301`, and the second request goes to 443.

The `proxy_set_header` lines forward the original client information to the application. Without them, the application sees every request as coming from `127.0.0.1` — useful for debugging, useless for analytics or geo-IP. `X-Forwarded-Proto` is what tells the application "this connection started as HTTPS even though I'm now talking to you on HTTP."

<a name="getting-a-certificate"></a>

## Getting a TLS Certificate

[Certbot](https://certbot.eff.org) gets and renews free certificates from Let's Encrypt. Install it with the Nginx plugin:

```bash
sudo apt install certbot python3-certbot-nginx
```

Run it once and answer the questions:

```bash
sudo certbot --nginx -d yourdomain.com
```

Certbot asks for your email (used for renewal-failure warnings) and whether you want to redirect HTTP to HTTPS automatically (yes). It updates the Nginx config in place to add the `ssl_certificate` lines, fetches the certificate, reloads Nginx, and walks away. Subsequent renewals happen via a systemd timer that runs twice a day; certificates are renewed when they're within 30 days of expiry.

Verify the cert is in place by curling the site:

```bash
curl -I https://yourdomain.com
```

A `200 OK` (or whatever your home page returns) confirms the chain is working. A `Could not resolve host` means DNS isn't pointing at the server yet; a TLS error means the certificate isn't installed correctly.

<a name="multiple-domains"></a>

## Multiple Domains on One Server

Each application gets its own `server` block and its own port. To run a second site:

```nginx
# /etc/nginx/sites-available/staging.yourdomain.com
server {
    listen 80;
    server_name staging.yourdomain.com;
    return 301 https://$host$request_uri;
}

server {
    listen 443 ssl http2;
    server_name staging.yourdomain.com;

    ssl_certificate     /etc/letsencrypt/live/staging.yourdomain.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/staging.yourdomain.com/privkey.pem;

    location / {
        proxy_pass http://127.0.0.1:2339;   # different port for staging
        proxy_set_header Host              $host;
        proxy_set_header X-Real-IP         $remote_addr;
        proxy_set_header X-Forwarded-For   $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

`certbot --nginx -d staging.yourdomain.com` adds a separate certificate for the new hostname. Symlink, validate, reload Nginx — the new site is up alongside the existing one.

For a wildcard certificate (one cert that covers `*.yourdomain.com`), use Certbot's DNS challenge with your DNS provider's API. The Nginx plugin handles only single-hostname certs.

<a name="websockets"></a>

## WebSockets

Reepolee's live-reload uses WebSockets in development. To proxy WebSockets through Nginx, add the `Upgrade` headers:

```nginx
location / {
    proxy_pass http://127.0.0.1:2338;
    proxy_set_header Host              $host;
    proxy_set_header X-Real-IP         $remote_addr;
    proxy_set_header X-Forwarded-For   $proxy_add_x_forwarded_for;
    proxy_set_header X-Forwarded-Proto $scheme;

    # WebSocket upgrade
    proxy_http_version 1.1;
    proxy_set_header Upgrade    $http_upgrade;
    proxy_set_header Connection "upgrade";

    proxy_read_timeout 86400;
}
```

The increased `proxy_read_timeout` keeps long-lived WebSocket connections alive; the default is 60 seconds, which closes idle WebSocket connections too aggressively.

In production, live-reload is disabled (`is_dev` is false) and there are no WebSocket routes by default. If your application uses WebSockets for real-time features, this block is what makes them work behind the proxy.

<a name="static-files-from-nginx"></a>

## Serving Static Files From Nginx

For a small performance boost, let Nginx serve `/static/` directly from disk rather than going through the application. Add a `location` block before the catch-all:

```nginx
location ^~ /static/ {
    alias /home/deploy/app/static/;
    add_header Cache-Control "public, max-age=31536000, immutable";
    access_log off;
}
```

`^~` makes this location win over regex matches; `alias` rewrites `/static/foo.css` to `/home/deploy/app/static/foo.css` on disk. The cache header matches what the application serves in production. `access_log off` skips writing each static file fetch to the access log.

For most projects this is a micro-optimisation — the application's static-file serving is already fast and adds the same cache headers. Only worth doing when static-file traffic dominates and you're tuning out the last bit of latency.

<a name="rate-limiting"></a>

## Rate Limiting

Nginx can apply per-IP rate limits at the proxy layer, before requests reach the application:

```nginx
http {
    limit_req_zone $binary_remote_addr zone=app_limit:10m rate=30r/s;
}

server {
    location / {
        limit_req zone=app_limit burst=60 nodelay;
        proxy_pass http://127.0.0.1:2338;
        # ... other proxy headers
    }
}
```

This limits each client IP to 30 requests per second with a burst of 60. Excess requests get a `503 Service Unavailable`. The numbers are starting points — tune to your traffic.

For login forms specifically, a tighter limit on `/login` is worth adding to slow down credential-stuffing attacks:

```nginx
location = /login {
    limit_req zone=login_limit burst=5 nodelay;
    proxy_pass http://127.0.0.1:2338;
    # ... other proxy headers
}
```

With `limit_req_zone ... rate=5r/m` declared at the `http` block. Five attempts per minute per IP is generous for a real user and tight for an attacker.

<a name="alternatives"></a>

## Alternatives

[Caddy](https://caddyserver.com) is a single-binary alternative that handles TLS automatically — no Certbot setup, no config-file dance. The equivalent of the Nginx config above is two lines:

```caddy
yourdomain.com {
    reverse_proxy 127.0.0.1:2338
}
```

Caddy fetches the certificate on first start and renews it transparently. It's a great choice for new deployments. The reason most documentation defaults to Nginx is install base — Nginx is everywhere, Caddy is younger and not yet packaged on every distro.

For very large deployments, [HAProxy](https://www.haproxy.org/) is the proxy of choice for its specifically-designed load-balancing and observability features. None of that matters until you're balancing across many backends; for a single-instance Reepolee deployment, Nginx or Caddy is the right size.
