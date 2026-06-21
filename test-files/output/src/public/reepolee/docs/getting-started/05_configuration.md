---
title: "Configuration"
---

# Configuration

<a name="introduction"></a>

## Introduction

Reepolee has very little to configure. Everything that varies between environments — database connection, SMTP credentials, the port, the timezone — lives in `.env` and is read directly from `Bun.env`. Everything that varies between projects — supported languages, generator preferences, the database driver — lives in TypeScript files under `config/` that you edit like any other source file.

There is no JSON config file, no YAML, no `dotenv` package to install (Bun loads `.env` automatically), no environment-resolution layer. This page covers what each piece is for and where the runtime reads it.

For the common toggles — switching database, switching session backend, adding a language — `bun tui` exposes each as a menu item that edits the relevant file (`.env`, the supported-languages config, etc.) so you don't have to remember which knob lives where.

<a name="the-env-file"></a>

## The .env File

`.env` is gitignored and lives at the project root. The `.env.example` file in the repository shows the full set of variables a Reepolee project expects:

```
# --- Server -----------------------------------------------------------
PORT=2338
TIME_ZONE=Europe/Ljubljana

# --- Database ---------------------------------------------------------
CONNECTION_STRING="sqlite:app.db"
# CONNECTION_STRING="mysql://login:pass@localhost/reepolee_dev"

# --- Email (optional, but required for any send_mail call) -----------
SMTP_HOST=smtp.your-provider.com
SMTP_PORT=587
SMTP_USERNAME=...
SMTP_PASSWORD=...
SMTP_FROM=hello@yourdomain.com

# --- S3-compatible storage (optional) -------------------------------
# MinIO, AWS S3, Cloudflare R2, etc.
# If unset, file uploads (e.g. avatars) fall back to the local filesystem.
S3_ACCESSKEYID=minio
S3_SECRETACCESSKEY=minio123
S3_ENDPOINT=http://localhost:9000
# S3_REGION=auto

# --- Sessions / Redis ------------------------------------------------
SESSION_STORE=sql           # "sql" (default) or "redis"
# REDIS_URL=redis://localhost:6379

# --- Redis-backed features (all require REDIS_URL) -------------------
CACHE_ENABLED=false         # Redis cache for search_records queries
# RATE_LIMITING=true        # sliding-window request rate limiting

# --- Optional --------------------------------------------------------
SQL_LOGGING=false
MAX_UPLOAD_SIZE_MB=10
TRANSLATED_ROUTES=true
RELOAD_SECRET=my-secret     # protects the /__reload-translations endpoint

# --- AI translation providers (pick one) ----------------------------
OPENROUTER_KEY=...
# OPENROUTER_MODEL=anthropic/claude-haiku
# OLLAMA_URL=http://localhost:11434   # local LLM; takes priority if set
# HF_TOKEN=hf_xxx                      # Hugging Face inference fallback
```

Every variable is read via `Bun.env.<NAME>` from somewhere in the codebase. If you add a new one, you can read it the same way — no helper or registration step.

<a name="env-var-reference"></a>

## Environment Variable Reference

The full set of variables Reepolee reads, with the file that consumes each:

| Variable             | Required             | Read by                                                                                     | Purpose                                                                                                                                                                                           |
| -------------------- | -------------------- | ------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `PORT`               | yes                  | `server.ts`                                                                                 | The port the HTTP server listens on                                                                                                                                                               |
| `TEST_PORT`          | no                   | `server.ts`                                                                                 | Overrides `PORT` when the server runs with the `--test` flag (binds to `127.0.0.1` only). Falls back to `PORT`, then `2338`. The smoke-test script defaults to `2600`                             |
| `MCP_SERVER_PORT`    | no                   | `scripts/mcp/index.ts`                                                                      | Port the standalone MCP server listens on (it runs on its own port, separate from the app). Defaults to `2338`                                                                                    |
| `AGENT_USER_EMAIL`   | no                   | `routes/system/auth/middleware.ts`                                                          | Default authenticated user for [agent mode](/security/authentication#agent-mode) (`bun run agent`). Looked up via `get_user_by_email()`; the `X-Agent-User-Email` header overrides it per-request |
| `AGENT_SERVER_PORT`  | no                   | `server.ts`                                                                                 | Port for the agent-mode server (avoids clashing with dev on `2338`). Agent mode binds to `127.0.0.1` only and is allowed only alongside `--dev`                                                   |
| `TIME_ZONE`          | yes                  | `config/db.ts`                                                                              | Timezone used by the database connection for date/time column handling. Required for all database types — `config/db.ts` calls `require_env("TIME_ZONE")` unconditionally                         |
| `CONNECTION_STRING`  | yes                  | `config/db.ts` (dispatches to the SQLite or MySQL driver based on prefix)                   | Database connection string                                                                                                                                                                        |
| `SESSION_STORE`      | no                   | `routes/system/auth/session_store.ts`                                                       | `redis` backs sessions with Redis (requires `REDIS_URL`). Otherwise (default `sql`) sessions use the database store matching your driver — SQLite or MySQL                                        |
| `SMTP_HOST`          | for email            | `lib/smtp.ts`                                                                               | SMTP server hostname                                                                                                                                                                              |
| `SMTP_PORT`          | for email            | `lib/smtp.ts`                                                                               | `587` for STARTTLS, `465` for implicit TLS                                                                                                                                                        |
| `SMTP_USERNAME`      | for email            | `lib/smtp.ts`                                                                               | SMTP auth username                                                                                                                                                                                |
| `SMTP_PASSWORD`      | for email            | `lib/smtp.ts`                                                                               | SMTP auth password / API key                                                                                                                                                                      |
| `SMTP_FROM`          | for email            | `lib/smtp.ts`                                                                               | Default `From:` address                                                                                                                                                                           |
| `SQL_LOGGING`        | no                   | `server.ts`                                                                                 | Set to `"true"` to log every query to `logs/sql.ndjson`                                                                                                                                           |
| `OPENROUTER_KEY`     | no                   | `generator/openrouter.ts`                                                                   | Used by `bun generator/resource --translate` for AI-translated language files                                                                                                                     |
| `OPENROUTER_MODEL`   | no                   | `generator/openrouter.ts`                                                                   | Override the translation model (defaults to a Claude Haiku model)                                                                                                                                 |
| `OLLAMA_URL`         | no                   | `generator/ai-provider.ts`                                                                  | Local LLM endpoint for AI translation. Takes priority over all other providers when set                                                                                                           |
| `OLLAMA_MODEL`       | no                   | `generator/ai-provider.ts`                                                                  | Model name for the local Ollama provider                                                                                                                                                          |
| `HF_TOKEN`           | no                   | `generator/ai-provider.ts`                                                                  | Hugging Face inference token; used when set and `OPENROUTER_KEY` is not                                                                                                                           |
| `REDIS_URL`          | for Redis features   | `lib/cache.ts`, `lib/middleware/rate_limit.ts`, `routes/system/auth/session_store_redis.ts` | Redis connection string; defaults to `redis://localhost:6379`. **Required** when `SESSION_STORE=redis`, `CACHE_ENABLED=true`, or `RATE_LIMITING=true`                                             |
| `CACHE_ENABLED`      | no                   | `lib/cache.ts`                                                                              | Set to `"true"` to cache `search_records` query results in Redis (requires `REDIS_URL` — fails loud if missing). Silent no-op when unset. See [Caching](/database/caching)                        |
| `CACHE_MAX_BYTES`    | no                   | `lib/cache.ts`                                                                              | Max serialized size of a single cached value. Larger results are skipped rather than cached. Defaults to `524288` (512 KB)                                                                        |
| `CACHE_MAX_RECORDS`  | no                   | `lib/cache.ts`                                                                              | Max record count for a cached query result; larger result sets are not cached. Defaults to `500`                                                                                                  |
| `RATE_LIMITING`      | no                   | `lib/middleware/rate_limit.ts`                                                              | Set to `"true"` to enable sliding-window rate limiting (requires `REDIS_URL` — fails loud if missing). Disabled by default. See [Rate Limiting](/security/rate-limiting)                          |
| `RELOAD_SECRET`      | no                   | `server.ts`                                                                                 | If set, the `/__reload-translations` endpoint requires a matching `X-Reload-Secret` header. Generators and the queue worker use it to hot-reload translations without a restart                   |
| `MAX_UPLOAD_SIZE_MB` | no                   | `lib/middleware`                                                                            | Maximum upload size in megabytes. Defaults to `10`                                                                                                                                                |
| `S3_ACCESSKEYID`     | for uploads          | `lib/s3.ts`                                                                                 | Access key for S3-compatible storage. Also accepts `S3_ACCESS_KEY_ID` or `AWS_ACCESS_KEY_ID`                                                                                                      |
| `S3_SECRETACCESSKEY` | for uploads          | `lib/s3.ts`                                                                                 | Secret key. Also accepts `S3_SECRET_ACCESS_KEY` or `AWS_SECRET_ACCESS_KEY`                                                                                                                        |
| `S3_ENDPOINT`        | for uploads          | `lib/s3.ts`                                                                                 | Endpoint URL (e.g. `http://localhost:9000` for MinIO, `https://<account>.r2.cloudflarestorage.com` for R2). Also accepts `AWS_ENDPOINT`                                                           |
| `S3_HOSTNAME`        | for uploads          | `lib/s3.ts`                                                                                 | Hostname for split-style S3 connection (e.g. `localhost`). Takes precedence over `S3_ENDPOINT` when set                                                                                           |
| `S3_PORT`            | for uploads          | `lib/s3.ts`                                                                                 | Port for split-style S3 connection (e.g. `8333` for MinIO)                                                                                                                                        |
| `S3_PROTOCOL`        | for uploads          | `lib/s3.ts`                                                                                 | Protocol for split-style S3 connection: `"http"` or `"https"`                                                                                                                                     |
| `S3_IMAGE_BUCKET`    | for uploads          | `lib/s3.ts`                                                                                 | Bucket name used for uploaded images via the image editor. Defaults to `"images"`                                                                                                                 |
| `S3_REGION`          | optional             | `lib/s3.ts`                                                                                 | Region string; some providers require it. Also accepts `AWS_REGION`                                                                                                                               |
| `SERVER_NAME`        | yes                  | `server.ts`                                                                                 | The hostname of this server (e.g. `localhost`). Used for generating absolute URLs and links                                                                                                       |
| `SITE_URL`           | yes                  | `server.ts`                                                                                 | The full base URL of the site (e.g. `http://localhost:2338`). Used wherever an absolute URL is needed                                                                                             |
| `STORAGE`            | no                   | `lib/storage.ts`                                                                            | Storage backend: `"local"` to write files to disk, `"s3"` to use S3-compatible object storage. Defaults to `"local"`                                                                              |
| `LOCAL_STORAGE_DIR`  | when `STORAGE=local` | `lib/storage.ts`                                                                            | Directory path for local file storage (e.g. `"../storage"`). Sub-directories mirror S3 bucket names                                                                                               |
| `TRANSLATED_ROUTES`  | no                   | `lib/middleware/set_lang.ts`                                                                | Set to `true` to enable language-prefixed URL routing (e.g. `/sl/about`). Defaults to `false`                                                                                                     |

`is_s3_configured()` in `lib/s3.ts` checks for the access key, secret, and endpoint together — leave any of the three blank and uploads fall back to writing under `static/avatars/` instead. See [File Uploads](/forms/file-uploads) for the full flow.

For environment-specific overrides — different SMTP credentials in staging vs production, a different `PORT` for a second instance — keep multiple `.env` files (`.env.staging`, `.env.production`) and copy the right one to `.env` during deploy. Bun reads only `.env` at startup.

<a name="dev-vs-prod-flag"></a>

## The --dev / --prod Flag

The mode the application runs in is determined by a command-line flag, not an environment variable. The render layer reads:

```ts
const is_dev = Bun.argv.includes("--dev");
```

That single line drives every dev-vs-prod branch in the codebase:

- Template caching (off in dev, on in prod)
- The dev-only `props.toJSON` and `props.toPrettyJSON` injections
- The choice of `static/app-dev.css` vs `static/app.css` in the layout
- The live-reload WebSocket endpoint (only in dev)
- The static-file `Cache-Control` header (`no-store` in dev, one-year immutable in prod)

The two scripts in `package.json` choose the flag:

```json
{
	"scripts": {
		"dev": "conc -n tw,dev -c yellow,blue \"bun css:watch\" \"bun development\"",
		"development": "bun --watch server.ts --dev",
		"start": "bun server.ts --prod"
	}
}
```

`bun dev` runs the CSS watcher and `bun development` (`bun --watch server.ts --dev`) side by side. `bun start` runs the production server (`bun server.ts --prod`); in production the systemd unit runs the same `server.ts --prod` entry with `--hot` added so a `git pull` auto-reloads — see [systemd](/deployment/systemd). `NODE_ENV=production` from the systemd unit is included for any third-party library that reads it but isn't load-bearing for Reepolee itself.

<a name="the-config-folder"></a>

## The config/ Folder

Compile-time configuration lives in `config/` as TypeScript files. Each one exports the values the rest of the codebase imports.

```
config/
├── db.ts                     ← database connection; creates a Bun SQL instance from CONNECTION_STRING
├── db_structure.ts           ← generator preferences
├── rate_limit.ts             ← per-scope rate limit rules (see Rate Limiting)
└── supported_languages.ts    ← language list and metadata
```

Every file in `config/` is in version control — there's nothing to copy or generate. Which database you connect to is decided at startup by reading `CONNECTION_STRING` from `.env`.

<a name="config-supported-languages"></a>

### config/supported_languages.ts

Declares the languages your application supports and the metadata used by the picker and the formatters:

```ts
export const languages = ["en", "sl"] as const;
export const active_languages = ["sl", "en"] as const;
export const default_language = "sl";

export const language_names: Record<string, string> = {
	en: "English",
	sl: "Slovenian",
};

export const language_locales: Record<string, string> = {
	en: "en-US",
	sl: "sl-SI",
};
```

Adding a new language is a one-file change here plus the matching `<lang>.json` translation files in `routes/`. Walked through in detail in [Adding a New Language](/recipes/new-language). The full mechanics are in [Languages & Locales](/i18n/languages-and-locales).

<a name="config-db-structure"></a>

### config/db_structure.ts

Generator preferences — which tables to skip, which columns to ignore, how to recognise booleans and dates:

```ts
export const IGNORE_TABLES = ["modules", "sessions", "email", "images", "users", "translations"] as const;
export const MAINTENANCE_FIELDS = ["created_at", "updated_at"] as const;
export const DATE_SUFIXES = ["_on", "_by"] as const;
export const DATETIME_SUFIXES = ["_at"] as const;
export const IGNORE_INDEX_FIELDS = [
	"option_title",
	"search_text",
	"hashed_password",
	"previous_hashed_password",
] as const;
export const IGNORE_ORDER_FIELDS = ["search_text", "hashed_password", "previous_hashed_password"] as const;
export const BOOLEAN_PREFIXES = ["is_", "has_", "can_"] as const;
export const MIN_PASSWORD_LENGTH = Bun.argv.includes("--dev") ? 1 : 8;
```

What each constant does — and how to extend the lists for your project's conventions — is on the [Generators](/database/generators#configuration) page.

<a name="config-db"></a>

### config/db.ts

Creates a Bun `SQL` connection from `CONNECTION_STRING` and exports it as `db`. It also reads `TIME_ZONE` unconditionally to configure timezone constants used by both SQLite and MySQL. The prefix of `CONNECTION_STRING` determines which timezone configuration is applied:

- `sqlite:` → uses UTC for dates, `TIME_ZONE` for timestamps. Logs `Using DB SQLITE`.
- `mysql:` → uses `TIME_ZONE` for all column types. Logs `Using DB MYSQL`.
- Anything else → exits with an error.

You don't edit this file to switch databases — change `CONNECTION_STRING` in `.env`. The TUI's "Set database type" option (`bun tui`) flips it for you. See [Database — Getting Started](/database/getting-started).

<a name="package-json-scripts"></a>

## package.json Scripts

The scripts in `package.json` are the day-to-day commands. The ones you'll use most:

| Script                    | Runs                                                                       | Purpose                                                                              |
| ------------------------- | -------------------------------------------------------------------------- | ------------------------------------------------------------------------------------ |
| `bun dev`                 | `conc "bun css:watch" "bun development"` (→ `bun --watch server.ts --dev`) | Local development with file-watch reload                                             |
| `bun start`               | `bun server.ts --prod`                                                     | Production server (also called by systemd)                                           |
| `bun css:build`           | `tailwindcss -i ./css/app.css -o ./static/app.css --minify`                | Production CSS build                                                                 |
| `bun css:watch`           | `tailwindcss -i ./css/app.css -o ./static/app-dev.css --watch`             | Watching CSS build for development                                                   |
| `bun run release`         | `bun css:build && bun rel`                                                 | Builds production CSS and bumps the patch version                                    |
| `bun run production`      | `git push origin main:production --force`                                  | Force-pushes `main` to the `production` branch (your server pulls from `production`) |
| `bun run service:install` | `sudo cp ./reepolee.service /etc/systemd/system/reepolee.service`          | One-time systemd install on the server                                               |
| `bun run service:logs`    | `journalctl -u reepolee -f`                                                | Follow the production log live                                                       |

Add your own scripts as you need them. There's no Reepolee-specific structure — `package.json` scripts are plain shell commands.

<a name="bun-loading-env"></a>

## How Bun Loads .env

Bun reads `.env` files automatically at startup, with this priority:

1. `.env.local` (if present, gitignored — for personal-machine overrides)
2. `.env.{NODE_ENV}` (if `NODE_ENV` is set — `.env.production` etc.)
3. `.env`

Later files override earlier ones. The variables become available on `Bun.env.NAME` and `process.env.NAME` (for compatibility with code that expects the Node-style API).

There's no `dotenv` package to import and no `dotenv.config()` call to make. If you've used Node's ecosystem before, this is one fewer initialisation step than you're used to.

<a name="secrets-in-production"></a>

## Secrets in Production

`.env` files in production should be readable only by the deploy user — `chmod 600 .env` makes it `rw-------`. The systemd unit runs as the deploy user, which can read its own `.env`; nobody else on the system can.

For deployments that source secrets from a vault (HashiCorp Vault, AWS Secrets Manager, GCP Secret Manager), the integration is whatever fetches the secrets and writes them to `.env` before the systemd service starts. Reepolee reads `Bun.env.NAME` regardless of how the variable got there.

For the simplest production setup — a single VPS with `.env` on disk and `chmod 600` — that's enough. Vaults add value when you have multiple servers, an audit requirement for who-accessed-what, or rotation that has to happen without redeployment. For most projects, the simpler approach is fine until it isn't.
