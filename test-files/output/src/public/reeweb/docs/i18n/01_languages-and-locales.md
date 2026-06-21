---
title: "Languages & Locales"
layout: "reeweb/docs/docs.layout"
---

# Languages & Locales

<a name="introduction"></a>

## Introduction

Reeweb has first-class support for multi-language sites. Every page renders in every configured language, the default language is served at the root of your site, and other languages get a language-prefixed URL path. The configuration lives in a single file.

<a name="configuration"></a>

## Configuration

Languages are declared in `config/supported_languages.ts`:

```ts
// All languages with translation files
export const languages = ["en", "sl"] as const;

// Languages shown in the language switcher
export const active_languages = ["sl", "en"] as const;

// First in list — served at root (no /{lang} prefix)
export const default_language = "sl";

// Human-readable names for the language switcher
export const language_names: Record<string, string> = {
	en: "English",
	sl: "Slovenian",
};

// BCP-47 locale strings for date/time formatting
export const language_locales: Record<string, string> = {
	en: "en-US",
	sl: "sl-SI",
};
```

The distinction between `languages` and `active_languages` matters:

- **`languages`** — all languages that have translation files. The build loads translations for these.
- **`active_languages`** — languages that appear in the UI language switcher. A subset of `languages`.

Both are typed as `readonly string[]` (using `as const`), which gives you full TypeScript type safety when referencing them elsewhere.

<a name="how-languages-affect-urls"></a>

## How Languages Affect URLs

The default language is served at the root of your site — no URL prefix. All other languages are served under `/{lang}/`:

```
/                  → Slovenian (default)
/about/            → Slovenian
/english-only/     → Slovenian (English-only page renders at root)
/en/               → English index page
/en/about/         → English about page
```

This is handled automatically by both the build script (`scripts/build.ts`) and the dev server (`scripts/dev.ts`). The URL structure is:

```
/{lang}/{localized-path}/
```

Where `{localized-path}` may differ per language if you use `route_name` in translations (see [Localized Routes](/reeweb/docs/i18n/localized-routes)).

<a name="adding-a-new-language"></a>

## Adding a New Language

To add a language, say German (`de`):

1. Add `"de"` to `languages`, `active_languages`, `language_names`, and `language_locales` in `config/supported_languages.ts`
2. Create a `de.json` translation file in each directory that has an `en.json` or `sl.json`

That's it. The build script automatically discovers the new translation files and renders every page in German. No other configuration is needed.

See the [Adding a Language](/reeweb/docs/recipes/adding-a-language) recipe for a step-by-step walkthrough.

<a name="language-variant-templates"></a>

## Language-Variant Templates

Most pages share the same template across languages — only the translation strings differ. For pages that need different markup per language (a landing page with language-specific hero content, for example), you can create language-specific template variants:

- `about/index.en.ree` — English version
- `about/index.sl.ree` — Slovenian version

The resolution chain is:

1. `{name}.{requestedLang}.ree` — exact match for the current language
2. `{name}.{default_language}.ree` — fallback to the default language
3. `{name}.ree` — generic fallback

This applies to every template load — pages, layouts, includes, and components.

<a name="locale-strings"></a>

## Locale Strings for Date Formatting

The `language_locales` map feeds into template helpers like `js_date_to_locale_string()`, which use `Intl.DateTimeFormat` under the hood. Each entry should be a valid BCP-47 locale tag:

| Code | Locale  | Example Date |
| ---- | ------- | ------------ |
| `en` | `en-US` | 06/15/26     |
| `sl` | `sl-SI` | 15. 06. 26   |
| `de` | `de-DE` | 15.06.26     |
| `fr` | `fr-FR` | 15/06/26     |

Using the right locale means dates, times, currencies, and percentages are automatically formatted according to regional conventions.
