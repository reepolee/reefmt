---
title: "Adding a Language"
layout: "reeweb/docs/docs.layout"
---

# Adding a Language

<a name="introduction"></a>

## Introduction

Reeweb ships with English and Slovenian. This recipe walks through adding German (`de`) as a third language, end-to-end.

<a name="step-1-configure-the-language"></a>

## Step 1: Configure the Language

Add the new language to `config/supported_languages.ts`:

```ts
export const languages = ["en", "sl", "de"] as const;
export const active_languages = ["sl", "en", "de"] as const;
export const default_language = "sl";

export const language_names: Record<string, string> = {
	en: "English",
	sl: "Slovenian",
	de: "German",
};

export const language_locales: Record<string, string> = {
	en: "en-US",
	sl: "sl-SI",
	de: "de-DE",
};
```

If German should appear in the language switcher, add it to `active_languages`. If you're building the site in German but don't want it visible yet, add it only to `languages` — it will render but won't appear in the switcher.

<a name="step-2-add-translation-files"></a>

## Step 2: Add Translation Files

Create a `de.json` in the same directories as your existing translation files. Start with the global file at `src/public/de.json`:

```json
{
	"ui": {
		"site_name": "Meine Seite",
		"language_names": {
			"en": "Englisch",
			"sl": "Slowenisch",
			"de": "Deutsch"
		}
	},
	"nav": {
		"home": "Startseite",
		"about": "Über uns",
		"blog": "Blog",
		"contact": "Kontakt"
	}
}
```

Because of the cross-language fallback in `lib/i18n.ts`, you only need to provide the keys that differ from other languages. Any key you leave out will inherit from whatever language has it defined.

<a name="step-3-add-localized-route-names-optional"></a>

## Step 3: Add Localized Route Names (Optional)

To translate URL paths for German:

```json
{
	"about": { "route_name": "ueber-uns" },
	"contact": { "route_name": "kontakt" },
	"blog": { "route_name": "blog" }
}
```

The `slugify()` function in `lib/route_aliases.ts` handles the transliteration — it uses NFKD normalization to decompose characters and then strips combining diacritics, so `"ü"` becomes `"u"` (the diaeresis is stripped). Explicit exceptions apply: `"ß"` becomes `"ss"`, `"æ"` becomes `"ae"`, `"œ"` becomes `"oe"`.

<a name="step-4-rebuild"></a>

## Step 4: Rebuild

```bash
bun run build
```

The build script automatically discovers `de.json` files and renders every page in German. The German version is served at `/de/`:

```
/de/              → German homepage
/de/ueber-uns/    → German about page
/de/kontakt/      → German contact page
/de/blog/         → German blog index
```

<a name="step-5-verify"></a>

## Step 5: Verify

Check that:

- The language switcher shows "German"
- Navigation labels display in German
- All pages are reachable at `/de/...` paths
- Dates format correctly for the `de-DE` locale
- The `hreflang` links (if `SITE_URL` is set) include `de`

<a name="language-specific-templates"></a>

## Language-Specific Templates

If a page needs different markup in German (not just different text), create a language-variant template:

- `about/index.de.ree` — German-specific version
- `about/index.ree` — fallback for other languages

The resolution chain: `index.de.ree` → `index.sl.ree` (default) → `index.ree`.
