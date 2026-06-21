---
title: "Localized Routes"
layout: "reeweb/docs/docs.layout"
---

# Localized Routes

<a name="introduction"></a>

## Introduction

By default, every page in Reeweb has the same URL path in every language — `/about/` is `/about/` in English, Slovenian, and German. Localized routes let you translate the path itself: `/about/` in English becomes `/o-nas/` in Slovenian and `/ueber-uns/` in German. The translation files declare a `route_name` key, and the build system generates the appropriate per-language URLs automatically.

<a name="setting-a-route-name"></a>

## Setting a Route Name

Add a `route_name` key to any section's translation object. The key name is the canonical URL segment, and the value is the translated path segment:

```json
// src/public/en.json
{
	"about": {
		"route_name": "about",
		"title": "About us"
	},
	"contact": {
		"route_name": "contact",
		"title": "Contact"
	}
}
```

```json
// src/public/sl.json
{
	"about": {
		"route_name": "o nas",
		"title": "O nas"
	},
	"contact": {
		"route_name": "kontakt",
		"title": "Kontakt"
	}
}
```

With these in place, the build generates:

- `/about/` (English)
- `/o-nas/` (Slovenian)
- `/contact/` (English)
- `/kontakt/` (Slovenian)

The `route_name` value is processed through `slugify()` in `lib/route_aliases.ts`, which transliterates Unicode characters to ASCII and normalises the result to a URL-safe slug:

```ts
slugify("o nas"); // → "o-nas"
slugify("übersetzen"); // → "ubersetzen"  (diacritics removed)
slugify("straße"); // → "strasse"     (ß → ss)
```

<a name="route_name-isolation"></a>

## Route Name Isolation

A critical rule: **`route_name` is never inherited across languages.** If English defines `about.route_name` but Slovenian doesn't, the Slovenian URL stays as `/about/` — it does not inherit the English "about". This is enforced in `lib/i18n.ts`:

```ts
// Never inherit route_name from other languages.
if (key === "route_name") continue;
```

This means you must define a `route_name` in every language where you want a localized path. This is intentional — a Slovenian URL like `/about/` using an English word is fine, but a Slovenian URL like `/ueber-uns/` with a German word would be confusing. Each language controls its own paths.

<a name="nested-localized-routes"></a>

## Nested Localized Routes

Route names work at any nesting depth. For a blog post at `blog/my-post-slug`:

```json
// sl.json
{
	"blog": {
		"route_name": "novice"
	}
}
```

This produces `/novice/my-post-slug/` in Slovenian while staying at `/blog/my-post-slug/` in English. The route map in `lib/static_site.ts` (`build_static_route_map()`) walks the translation tree segment by segment, substituting `route_name` where present and keeping the canonical segment where not.

<a name="how-it-affects-links"></a>

## How It Affects Links

When you link between pages, always use the canonical path — not the localized URL. The `localized_path()` helper resolves the correct URL for the active language at render time:

```html
<!-- Correct: use canonical path, helper resolves the right URL -->
<a href="{~ localized_path('/about') }">About</a>

<!-- Wrong: hard-coding a localized path breaks in other languages -->
<a href="/o-nas/">About</a>
```

For links to the same page in a different language, use the `localized_path_for_lang()` helper:

```html
{#each props.active_languages as target_lang}
<a href="{~ localized_path_for_lang(target_lang, '/about') }"> {= props.language_names[target_lang] } </a>
{/each}
```

This resolves `/about/` to the correct URL in any language — `/about/` for English, `/o-nas/` for Slovenian.

<a name="hreflang-links"></a>

## Hreflang Alternate Links

When `--site-url` is provided to the build script, every page gets `<link rel="alternate" hreflang="...">` tags pointing to each language variant — including an `x-default` entry pointing to the default language. These are required by Google for multi-language SEO.

```html
<link rel="alternate" hreflang="en" href="https://example.com/about/" />
<link rel="alternate" hreflang="sl" href="https://example.com/o-nas/" />
<link rel="alternate" hreflang="x-default" href="https://example.com/o-nas/" />
```

The x-default points to the default language (Slovenian in the example above), which is what unauthenticated users see when they land on the root URL.
