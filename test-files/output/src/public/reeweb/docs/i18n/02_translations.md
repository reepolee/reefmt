---
title: "Translations"
layout: "reeweb/docs/docs.layout"
---

# Translations

<a name="introduction"></a>

## Introduction

Reeweb loads translations from `{lang}.json` files placed next to your templates. Every page has access to the merged translations for its language — both the top-level global strings and the page-specific overrides. Missing keys inherit from other languages automatically via a cross-language fallback, so you never see a blank label because a translation file was incomplete.

<a name="translation-file-layout"></a>

## Translation File Layout

Translation files are JSON files named after a language code. They live alongside the templates they translate:

```
src/public/
├── en.json                    ← English, global scope
├── sl.json                    ← Slovenian, global scope
├── index.ree
├── about/
│   ├── en.json                ← English, /about/ scope
│   ├── sl.json                ← Slovenian, /about/ scope
│   ├── index.ree
│   └── ...
├── blog/
│   ├── en.json                ← English, /blog/ scope
│   └── ...
└── contact/
    ├── en.json
    └── index.ree
```

The loader (in `lib/i18n.ts`) walks the entire directory tree, discovers every `{lang}.json` file, and builds a nested structure per language keyed by directory path.

<a name="key-organization"></a>

## Key Organization

Keys are namespaced by the directory structure. A global `en.json` at the root looks like:

```json
{
	"ui": {
		"site_name": "My Site",
		"language_names": {
			"en": "English",
			"sl": "Slovenian"
		}
	},
	"nav": {
		"home": "Home",
		"about": "About",
		"blog": "Blog",
		"contact": "Contact"
	}
}
```

Keys are accessed in templates via their namespace path — `props.ui.site_name`, `props.nav.home`, and so on. The loader merges each directory's translation file on top of the global scope, so a more specific key overrides a general one.

<a name="page-specific-keys"></a>

## Page-Specific Keys

A page-level translation file overrides global keys for that section. For example, `src/public/blog/en.json`:

```json
{
	"nav": {
		"blog": "All posts"
	},
	"blog": {
		"title": "Blog",
		"read_more": "Read more →",
		"published_on": "Published on"
	}
}
```

In the blog section, `props.nav.blog` resolves to `"All posts"` instead of `"Blog"`. Keys not overridden — `props.nav.home`, `props.nav.about` — continue to inherit from the global scope.

<a name="cross-language-fallback"></a>

## Cross-Language Fallback

When a key is missing in one language but present in another, the `fill_missing()` function in `lib/i18n.ts` copies it from an available source. The rule: **all languages fill from all other languages**, so a key added to any one `{lang}.json` file flows to the others automatically.

This means you can build a site in one language first and add translations incrementally — missing keys display the value from whatever language has them, rather than showing nothing.

**Exception:** The `route_name` key is never inherited across languages (see [Localized Routes](/reeweb/docs/i18n/localized-routes)). Each language must define its own route names.

<a name="template-access"></a>

## Template Access

Translations are merged into the render context and accessed through `props`:

```html
<h1>{= props.ui.site_name }</h1>

<nav>
	<a href="/">{= props.nav.home }</a>
	<a href="/about">{= props.nav.about }</a>
</nav>

{#if props.blog }
<p>{= props.blog.read_more }</p>
{/if }
```

For nested keys, use dot notation or bracket access. The helpers object also provides `nav_label()` for accessing nav keys with a fallback:

```html
<a href="/about">{= nav_label("about") }</a>
```

<a name="lazy-loading-pattern"></a>

## Lazy-Loading Pattern

Not every language needs every key from the start. Because of the cross-language fallback, you can ship a new language with only the keys that differ from an existing language. The loader will fill the rest from the other language's translations automatically.

This is useful during development — add the language to `supported_languages.ts`, create a minimal `{lang}.json` with just the navigation labels, and see the site working immediately. Add more translated text progressively.
