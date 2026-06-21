/**
 * src/lib/seo_localization.ts
 *
 * Project-specific SEO localization policy. Kept here (not in `lib/`, which
 * mirrors upstream reeweb) and consumed by the build + sitemap scripts.
 *
 * Upstream's `page_is_localized()` (in `$lib/static_site`) lets an individual
 * page opt out of localization with `localize: false` in its frontmatter. This
 * module is the path-based equivalent: whole subtrees that are English-only,
 * declared once so new pages under them are covered automatically without
 * touching every file's frontmatter.
 *
 * For reepolee.com that's:
 *   - the blog — written in English for a developer audience, and
 *   - every product's `/docs` tree — English-only. The `/sl/<product>/docs`
 *     URLs exist only so a Slovenian reader's chosen language (stored in a
 *     cookie) isn't forced to flip when they open the docs; they are not
 *     separate translations and must not compete in search.
 *
 * Effect for a matched path: only the default-language URL is canonical and
 * indexable. Non-default variants carry `<link rel="canonical">` back to the
 * default, drop out of the hreflang cluster, and are excluded from the sitemap
 * — exactly what `localize: false` does, applied by path.
 */

/** Canonical-path prefixes for English-only subtrees that are never localized. */
export const NON_LOCALIZED_PATH_PATTERNS: readonly RegExp[] = [
	/^\/blog(\/|$)/, // /blog and everything under it
	/^\/[^/]+\/docs(\/|$)/, // /<product>/docs and everything under it
];

/** Whether a canonical path falls outside the English-only subtrees. */
export function path_is_localized(canonical_path: string): boolean {
	return !NON_LOCALIZED_PATH_PATTERNS.some((re) => re.test(canonical_path));
}
