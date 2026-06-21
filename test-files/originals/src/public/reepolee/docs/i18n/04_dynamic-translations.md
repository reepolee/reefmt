---
title: "Dynamic Translations"
---

# Dynamic Translations

<a name="introduction"></a>

## Introduction

Translations live in the `translations` database table — the source of truth (see [Translations](/i18n/translations)). Translating by hand through the `/system/translations` admin UI is the most reliable path: you see exactly what each language looks like. For projects with many languages or frequent content updates, the AI translation pass fills the gaps automatically, leaving you to review the result rather than type every string.

The automated path is **`sync:languages`** — it scans the database for keys present in one language but missing (or still untranslated) in another, translates them via an LLM, and writes the results back to the database. It is non-destructive: already-translated values are left untouched.

<a name="sync-translations"></a>

## Synchronising Translation Keys

When a key exists for one language but not another — you added an English string, or just added a new language — `sync:languages` closes the gap. It scans the `translations` table across every namespace, finds keys missing from a language, AI-translates them, and writes them straight to the DB:

```bash
bun run sync:languages
```

This is `bun generator/sync_translations.ts --translate` under the hood. Because the database is authoritative, there are no JSON files to edit and no placeholder markers to find and replace — the missing rows are simply created with translated values. Run it whenever you've added strings or a language and want the other languages filled in.

To find or scaffold keys that templates reference but the DB doesn't have yet, use the **Sync missing translations** TUI tool, and to remove DB keys no longer referenced by any template, use **Prune unused translations** — both emit reviewable `.sql` files. See [Translations → Maintenance](/i18n/translations#maintenance-prune-sync).

<a name="ai-translation-pass"></a>

## AI Translation Pass

<media-frame label="TUI — add language with AI translation" ratio="4/3"></media-frame>

`sync:languages` is the standalone translation pass. The same `--translate` flag is also accepted by the resource generators, so a freshly generated module can have its labels translated in the same step:

```bash
bun generator/resource crud users --translate
```

The pass works across all languages the project supports (from `config/supported_languages.ts`). It reads the English rows from the database, sends them to the LLM, and writes the returned translations back into the `translations` table for each language. Only keys that are missing or still hold the English placeholder are sent — already-translated rows are left untouched.

<a name="setup"></a>

### Setup

Set `OPENROUTER_KEY` in your `.env`:

```env
OPENROUTER_KEY=YOUR_OPENROUTER_KEY
```

The translation pass uses Claude Haiku by default — fast and cost-effective for bulk translation. To use a different OpenRouter model, set `OPENROUTER_MODEL`:

```env
OPENROUTER_MODEL=anthropic/claude-sonnet-20241022
```

<a name="providers"></a>

### Choosing a Provider

OpenRouter is one of three backends `generator/ai-provider.ts` can use. It resolves the provider in priority order:

1. **Ollama** — if `OLLAMA_URL` is set, a local model is used (set `OLLAMA_MODEL` too). Takes priority over everything else, so a developer with Ollama running translates offline and for free.
2. **OpenRouter** — if `OPENROUTER_KEY` is set (and no `OLLAMA_URL`).
3. **Hugging Face Inference** — if `HF_TOKEN` is set and `OPENROUTER_KEY` is not. Defaults to `Helsinki-NLP/opus-mt`; override with `HF_URL` / `HF_MODEL`. The language pair (e.g. `en-sl`) is derived automatically.

```env
# Local-first (highest priority)
# OLLAMA_URL=http://localhost:11434
# OLLAMA_MODEL=gemma2

# Hugging Face fallback
# HF_TOKEN=hf_xxxxxxxx
```

All three providers feed the same translation pipeline, so switching backends is purely an environment change — the commands don't differ.

<a name="what-gets-translated"></a>

### What Gets Translated

The AI translates every key in the namespaces it processes, including:

- UI labels, headings, and button text
- Validation error messages
- Field labels and placeholders
- Action names and selectors

The one exception is `route_name` — it is **never** auto-translated. Localised URL segments are too sensitive to get wrong; they should always be reviewed by a human before deploying.

<a name="translate-workflow"></a>

### Typical Workflow

The flow for adding a batch of new strings across all languages:

```bash
# 1. Add the new keys to the DB (e.g. via the Sync missing translations TUI,
#    which scans templates for refs and emits INSERT SQL), then apply the SQL.

# 2. Auto-translate everything still untranslated across all languages.
bun run sync:languages
```

`sync:languages` sends only the keys that are missing or still hold their English placeholder — anything already translated is untouched. After the LLM returns, review the result in the `/system/translations` admin UI.

<a name="manual-review-after-translation"></a>

## Manual Review After Translation

AI-translated strings are usually correct for the common cases — button labels, field names, short messages. For domain-specific terms, brand names, or context-dependent phrasing, a human review pass is essential.

The `/system/translations` admin UI is the place to do it: browse a namespace, compare the languages side by side, and correct anything the model got wrong. Because edits write directly to the `translations` table, corrections take effect on the next request (the admin module reloads translations after each save) — no redeploy.

For terms the AI consistently gets wrong, the simplest fix is to translate those keys by hand once and leave them — `sync:languages` won't overwrite a key that already has a non-placeholder value.

<a name="when-not-to-automate"></a>

## When Not to Automate

AI translation is appropriate for:

- Bulk-translating a newly added language
- Filling in 50+ strings after adding a feature
- Keeping translations in sync during early development

AI translation is **not** appropriate for:

- Marketing copy, landing pages, or any text where tone and voice matter
- Legal disclaimers, terms of service, or anything with legal implications
- `route_name` values (URL segments are never auto-translated)
- Any string that, if translated wrong, would confuse or mislead users

For those cases, translate by hand through the admin UI — the AI pass skips keys that already have a translated value.
