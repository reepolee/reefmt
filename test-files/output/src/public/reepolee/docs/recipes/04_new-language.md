---
title: "Adding a New Language"
---

# Adding a New Language

<a name="introduction"></a>

## Introduction

A Reepolee project ships with whatever languages you need — there's no "framework default" beyond the languages you declare. Adding one means registering the language code, seeding its translations, and optionally localising URLs; the system picks it up automatically. Translations cascade across languages so a partial translation is never broken; missing keys fall back to a language that has them.

> **Translations are DB-first.** The `translations` database table is the source of truth — the **Add language** tool copies the English rows for the new language straight into the DB and (optionally) AI-translates them in place. JSON files in `routes/` are legacy seeds; see [Translations](/i18n/translations) for the full model. The fastest, canonical path is the TUI below. The hand-written JSON steps that follow show the underlying shape and still work as seeds, but the database wins on merge.

This recipe walks through adding French to a project that already has English and Slovenian. The same steps work for any language code your project needs.

<a name="the-fast-path"></a>

## The Fast Path — `bun tui`

<media-frame label="TUI — add language flow (enter code → AI-translate)" ratio="4/3"></media-frame>

The interactive setup tool wraps the whole add-a-language workflow:

```bash
bun tui
# pick "Add language" → enter "fr" → answer "y" to AI-translate
```

What it does in one shot:

- adds the code to `languages`, `active_languages`, `language_names`, and `language_locales` in `config/supported_languages.ts`
- reads every English row from the `translations` table and inserts a copy for the new language across all namespaces, so the new language starts fully populated (with English values as placeholders)
- updates the `ui.language_names` / `ui.language_names_to` entries for every existing language so the new language shows up in their pickers
- when you opt in to AI translation, runs the configured AI provider and translates the new rows **in the database**. The provider is resolved by environment: **Ollama** (`OLLAMA_URL`, local, highest priority), then **OpenRouter** (`OPENROUTER_KEY`), then **Hugging Face** (`HF_TOKEN`) — see [Dynamic Translations → Choosing a Provider](/i18n/dynamic-translations#providers)

The rest of this page walks through the underlying pieces, plus the URL-localisation and language-mismatch dialog details. With the TUI fast path you can skip straight to [Step 4: Localise URLs](#step-4-localise-urls).

<a name="step-1-register-the-language"></a>

## Step 1: Register the Language

Open `config/supported_languages.ts` and add `"fr"` to both `languages` and `active_languages`:

```ts
export const languages = ["en", "sl", "fr"] as const;
export const active_languages = ["sl", "en", "fr"] as const;
export const default_language = "sl";

export const language_names: Record<string, string> = {
	en: "English",
	sl: "Slovenian",
	fr: "French",
};

export const language_locales: Record<string, string> = {
	en: "en-US",
	sl: "sl-SI",
	fr: "fr-FR",
};
```

The two arrays serve different purposes:

- **`languages`** — every code that has translation files. The translation loader walks `routes/` looking for `<code>.json` matching this list.
- **`active_languages`** — the codes the language picker shows users. Usually equal to `languages`, but you can keep a language out of the picker while you're translating it (set `active_languages` to `["sl", "en"]` and `languages` to `["sl", "en", "fr"]`; French strings load but no UI offers French to users).

`language_names` is what the picker displays. `language_locales` is the BCP-47 locale used by date and currency formatters — see [Languages & Locales](/i18n/languages-and-locales#the-config-file).

After this change, the system _expects_ French translation files to exist. If they don't, the fallback merge fills missing keys from English or Slovenian — French users see the page in some language rather than blank fields.

<a name="step-2-create-the-root-translation-file"></a>

## Step 2: Create the Root Translation File

Global strings live at `routes/<code>.json`. Copy `routes/en.json` to `routes/fr.json` as a starting point:

```bash
cp routes/en.json routes/fr.json
```

Open `routes/fr.json` and translate the visible strings. The shape stays the same — same keys, French values:

```json
{
	"labels": {
		"0": "Non",
		"1": "Oui",
		"save": "Enregistrer",
		"cancel": "Annuler",
		"back": "Retour",
		"all": "Tous les enregistrements",
		"per_page": "par page",
		"select": "-- Sélectionner --"
	},
	"lang_mismatch_title": "Langue différente",
	"lang_mismatch_body": "Cette page est en",
	"lang_mismatch_switch": "Garder ma langue",
	"lang_mismatch_dismiss": "Rester ici",
	"language_names": {
		"en": "Anglais",
		"sl": "Slovène",
		"fr": "Français"
	},
	"nav": {
		"home": "Accueil",
		"books": "Livres"
	}
}
```

A few things to notice:

- **Don't add a top-level `route_name` here.** That key is reserved for URL segment localisation and only makes sense per-feature (see [Step 4](#step-4-localise-urls) below).
- **`language_names`** is what _French speakers_ see in the picker. Slovenian users see `"Slovène"` listed as French because the picker reads from each language's own translations, but the dialog in [Step 5](#step-5-the-language-mismatch-dialog) uses the visitor's preferred language.
- **The structure mirrors English.** Keeping the same keys means the fallback merge works cleanly — any keys you don't translate fall back to English (or whichever language has them).

Restart the dev server. Visit `/?lang=fr` and the labels appear in French — the language switcher in the layout now offers French alongside Slovenian and English.

<a name="step-3-translate-route-specific-strings"></a>

## Step 3: Translate Route-Specific Strings

Every namespace that has English and Slovenian rows needs French rows for full coverage. The **Add language** TUI does this for you when you add the code (above): it copies every English row in the `translations` table to the new language so French starts fully populated with English values as placeholders. If you registered the language by editing the config by hand, run the same step explicitly:

```bash
bun generator/add_language.ts fr            # copy EN rows → FR in the DB
bun generator/add_language.ts fr --translate # …and AI-translate them in place
```

At this point French exists for every key, but the values are still English. Fill them in by editing rows through the `/system/translations` admin UI, or let the AI pass do the first draft (next).

Restart the server, visit `/login?lang=fr`, confirm the strings render — they'll read in English until translated, never as blanks or braced key paths.

<a name="auto-translate"></a>

### Auto-Translating Missing Keys

For the first pass on many strings, let an LLM fill in the gaps. `sync:languages` scans the `translations` table for keys present in one language but missing (or still untranslated) in another, translates them, and writes the results **back to the database**:

```bash
bun run sync:languages
```

This uses whichever AI provider your environment selects — Ollama, OpenRouter, or Hugging Face ([Choosing a Provider](/i18n/dynamic-translations#providers)). With OpenRouter (the common default), set `OPENROUTER_KEY` in `.env` (account at [openrouter.ai](https://openrouter.ai)); the run sends each key with the English value as context to Claude Haiku and writes the translation to the DB. Output cost is cheap — translating an entire mid-sized application's strings runs to a few cents.

Always review LLM-translated strings before shipping (the `/system/translations` admin UI is the place to do it). The model gets nuance right most of the time but occasionally produces overly literal phrasing or mistranslates UI conventions ("Submit" as a verb vs as a button label, for example).

<a name="step-4-localise-urls"></a>

## Step 4: Localise URLs

Reepolee translates URL segments the same way it translates strings — through the `route_name` key in translation files. The canonical `/login` URL becomes `/connexion` in French if you set:

```json
// routes/system/auth/login/fr.json
{
    "route_name": "connexion",
    "title": "Connexion",
    "fields": { ... },
    "labels": { "submit": "Se connecter" }
}
```

Now `/login` is also reachable as `/connexion` in French. The handler is the same; the URL is an alias. For URLs mounted under a prefix (e.g. `/system/users`), translate each parent segment via its own `route_name` so the full path localises (`/sistem/uporabniki`).

The route map is built at startup, so restart the server to pick up new `route_name` translations. Confirm by:

- Visiting `/connexion` directly — the login form renders in French (because the URL implies French).
- Switching to French via `?lang=fr` from any page — the browser redirects to the French version of the canonical URL, including the localised path.

`route_name` is the _only_ translation key that doesn't fall back across languages. If you don't translate the URL for a specific feature, the canonical English segment is used. That's intentional — it lets you ship a fully-translated UI before translating any URLs and have everything work.

<a name="step-5-the-language-mismatch-dialog"></a>

## Step 5: The Language-Mismatch Dialog

A French-speaking user (cookie says `lang=fr`) might land on `/login` (the English URL) by clicking a link in an email or sharing. To avoid silently switching languages, the layout includes a dialog that detects the mismatch and offers to switch.

The dialog uses the user's _preferred_ language (French in this case) for its own text — so the offer is comprehensible. The keys it needs:

```json
// routes/fr.json
{
	"lang_mismatch_title": "Langue différente",
	"lang_mismatch_body": "Cette page est en",
	"lang_mismatch_switch": "Garder ma langue",
	"lang_mismatch_dismiss": "Rester ici",
	"lang_mismatch_switch_to": "Passer à",
	"language_names_to": {
		"en": "anglais",
		"sl": "slovène",
		"fr": "français"
	}
}
```

`language_names` is for nominative use ("English"); `language_names_to` is for the "Switch to _X_" prepositional context where French (and many other languages) inflects the noun differently. For English both forms happen to be the same, but for other languages they diverge.

To test: visit any page with `?lang=fr`, then change the URL to a Slovenian-only path (`/o-nas` if your project has it). The dialog should appear, in French, asking whether to stay or switch.

<a name="step-6-locale-aware-formatting"></a>

## Step 6: Locale-Aware Formatting

Date and currency helpers automatically use the active language's locale via `props.locale`:

```html
<p>{= js_date_to_locale_string(record.created_at) }</p>
<!-- en: "1/15/2026"  sl: "15. 1. 2026"  fr: "15/01/2026" -->

<p>{~ display_currency(record.price) }</p>
<!-- en: "€1,234.56"  sl: "1.234,56 €"  fr: "1 234,56 €" -->
```

Because we set `language_locales.fr = "fr-FR"` in [Step 1](#step-1-register-the-language), every helper that takes an optional locale uses French formatting on French pages without per-call configuration.

For locale-specific currency — euro for European languages, dollar for US English, yen for Japanese — pass the symbol explicitly when it varies per language:

```html
{~ display_currency(record.price, props.locale, false, props.lang === 'fr' ? '€' : '$') }
```

A cleaner pattern is a helper map per language, or a translation key that holds the symbol:

```json
// routes/fr.json
{ "currency_symbol": "€" }
```

```html
{~ display_currency(record.price, props.locale, false, props.currency_symbol) }
```

<a name="step-7-html-lang-attribute"></a>

## Step 7: The html lang Attribute

The `<html lang="...">` attribute should reflect the current page's language for screen readers and search engines. The shipped layout already does this:

```html
<html lang="{= props.lang }"></html>
```

`props.lang` is auto-injected by `render()` based on the resolution chain (see [Languages & Locales](/i18n/languages-and-locales#language-resolution)). After Step 1, French pages get `<html lang="fr">` automatically — no further changes needed.

For stricter SEO — separate URLs per language, sitemap entries per locale, hreflang annotations — generate them from the route map:

```html
<!-- in <head> -->
{#each props.active_languages as code } {{ const localized = localized_path_for(props.request_url, code) }} {#if
localized && code !== props.lang }
<link rel="alternate" hreflang="{= code }" href="{= localized }" />
{/if } {/each }
```

`hreflang` tags tell search engines which URL serves each language version. Combined with localised URLs, this gives search engines the full picture of a multilingual site.

<a name="step-8-finding-untranslated-strings"></a>

## Step 8: Finding Untranslated Strings

Because `add_language` seeds the new language with English values, an untranslated key shows up as a row whose French value still equals the English one. Query the `translations` table to list them:

```sql
SELECT fr.namespace, fr.key_path, fr.translation
FROM translations fr
JOIN translations en
  ON en.namespace = fr.namespace AND en.key_path = fr.key_path AND en.lang = 'en'
WHERE fr.lang = 'fr' AND fr.translation = en.translation;
```

Each row is a key that hasn't been translated yet (or that happens to be identical across languages — proper nouns, "OK"). The `/system/translations` admin UI is the convenient place to work through them; `bun run sync:languages` will AI-fill the same gaps in one pass.

**Browse the site in the new language** with the dev server's `props.toJSON` debug. In dev mode, every render injects `props.toJSON` (the full data payload as JSON). Append a `<pre>{~ props.toPrettyJSON }</pre>` to a development-only debug route, navigate the site in French, and any value that's still in English shows up in the JSON dump.

<a name="removing-a-language"></a>

## Removing a Language

Removing a language is a single TUI action — the reverse of adding one:

```bash
bun tui
# pick "Remove language" → choose the code → confirm
```

It strips the code from `config/supported_languages.ts`, deletes every row for that language from the `translations` table, removes the language's JSON seed files, and cleans up cross-language references (the `ui.language_names` / `ui.language_names_to` entries other languages held for it). If you remove the current `default_language`, it picks the first remaining language as the new default. The CLI form is `bun generator/remove_language.ts <code> [--force]` (`--force` skips the confirmation prompt for scripted use).

<a name="whats-next"></a>

## What's Next

You have a third language wired in end-to-end — strings, URLs, the mismatch dialog, locale-aware formatting. Adding a fourth (or removing one) is exactly the same flow.

To go deeper:

- **[Translations](/i18n/translations)** — the full file-loading and namespace model, the fallback merge behaviour.
- **[Localized Routes](/i18n/localized-routes)** — the route-map machinery that powers URL aliasing.
- **[Languages & Locales](/i18n/languages-and-locales)** — language resolution, the X-Lang header, building a custom language picker.
