#!/usr/bin/env bun

/**
 * scripts/generate_sitemap.ts
 *
 * Emits dist/sitemap.xml — one <url> per (canonical page × active language),
 * each carrying <xhtml:link rel="alternate"> entries for every active language
 * plus x-default. Mirrors the hreflang block that static_build.ts inlines into
 * rendered HTML so crawlers see a consistent view.
 *
 * Usage:
 *   bun scripts/generate_sitemap.ts --public ./public --dist ./dist --site-url https://example.com
 *   bun scripts/generate_sitemap.ts --help
 *
 * Per-page frontmatter (on either the base file or any language variant) can
 * override defaults:
 *
 *   ---
 *   lastmod: 2026-05-01      # explicit <lastmod> (highest priority)
 *   published_at: 2026-05-01 # (or `date:`) fallback for dated content (blog posts)
 *   sitemap: false   # opt this canonical page out of the sitemap
 *   noindex: true    # same effect — exclude from sitemap
 *   ---
 */

import { existsSync, statSync } from "fs";
import { join, resolve } from "path";

import { active_languages, default_language } from "$config/supported_languages";
import { load_all_translations } from "$lib/i18n";
import {
	build_static_route_map,
	collect_page_files,
	page_is_localized,
	read_frontmatter,
	template_to_canonical,
	type FrontMatter,
} from "$lib/static_site";
import { write_sitemap_files } from "$root/src/lib/project_helpers";
import { path_is_localized } from "$root/src/lib/seo_localization";

// ---------------------------------------------------------------------------
// CLI
// ---------------------------------------------------------------------------

function print_usage() {
	console.error("Usage: bun scripts/generate_sitemap.ts [options]");
	console.error("");
	console.error("Options:");
	console.error("  --public <dir>   Source directory with page templates (default: ./src/public)");
	console.error("  --dist <dir>     Output directory; sitemap.xml is written here (default: ./dist)");
	console.error("  --site-url <url> REQUIRED. Absolute origin used for <loc> and hreflang.");
	console.error("  --help           Print this usage and exit");
}

function parse_args() {
	const args = process.argv.slice(2);

	if (args.includes("--help")) {
		print_usage();
		process.exit(0);
	}

	let public_dir = "./src/public";
	let dist_dir = "./dist";
	let site_url = process.env.SITE_URL ?? "";

	for (let i = 0; i < args.length; i++) {
		const arg = args[i];
		if (!arg) continue;

		if (arg === "--public") {
			public_dir = args[++i] ?? public_dir;
		} else if (arg === "--dist") {
			dist_dir = args[++i] ?? dist_dir;
		} else if (arg === "--site-url") {
			site_url = args[++i] ?? site_url;
		}
	}

	if (!site_url) {
		console.error("✗ --site-url is required (or set SITE_URL in .env)");
		print_usage();
		process.exit(1);
	}

	return {
		public_dir: resolve(public_dir),
		dist_dir: resolve(dist_dir),
		site_url: site_url.replace(/\/+$/, ""),
	};
}

// ---------------------------------------------------------------------------
// XML helpers
// ---------------------------------------------------------------------------

function xml_escape(s: string): string {
	return s
		.replace(/&/g, "&amp;")
		.replace(/</g, "&lt;")
		.replace(/>/g, "&gt;")
		.replace(/"/g, "&quot;")
		.replace(/'/g, "&apos;");
}

// ---------------------------------------------------------------------------
// Page metadata collection
// ---------------------------------------------------------------------------

interface PageMeta {
	frontmatter: FrontMatter;
	mtime_ms: number;
}

/**
 * Read frontmatter and mtime across the base file and every language variant
 * that exists on disk. `sitemap: false` / `noindex: true` are taken from any
 * variant. mtime is the newest across variants.
 */
function read_page_meta(public_dir: string, rel_path: string, languages: readonly string[]): PageMeta {
	const ext_match = rel_path.match(/\.(ree|md)$/);
	if (!ext_match) return { frontmatter: {}, mtime_ms: 0 };

	const ext = ext_match[1]!;
	const base_without_ext = rel_path.slice(0, -(ext.length + 1));

	const candidates = [
		join(public_dir, rel_path),
		...languages.map((lang) => join(public_dir, `${base_without_ext}.${lang}.${ext}`)),
	];

	const merged: FrontMatter = {};
	let mtime_ms = 0;
	let any_exists = false;

	for (const path of candidates) {
		if (!existsSync(path)) continue;
		any_exists = true;

		const fm = read_frontmatter(path);
		for (const [k, v] of Object.entries(fm)) {
			if (merged[k] === undefined) merged[k] = v;
		}
		if (fm.sitemap === false) merged.sitemap = false;
		if (fm.noindex === true) merged.noindex = true;
		if (fm.localize === false) merged.localize = false;

		const file_mtime = statSync(path).mtimeMs;
		if (file_mtime > mtime_ms) mtime_ms = file_mtime;
	}

	if (!any_exists) return { frontmatter: {}, mtime_ms: 0 };
	return { frontmatter: merged, mtime_ms };
}

function lastmod_for(meta: PageMeta): string | null {
	const fm_lastmod = meta.frontmatter.lastmod;
	if (typeof fm_lastmod === "string" && fm_lastmod.trim()) {
		return fm_lastmod.trim();
	}
	// Use published_at from frontmatter for blog posts and other dated content
	const fm_published = meta.frontmatter.published_at ?? meta.frontmatter.date;
	if (typeof fm_published === "string" && fm_published.trim()) {
		const d = new Date(fm_published.trim());
		if (!Number.isNaN(d.getTime())) {
			return d.toISOString().slice(0, 10);
		}
	}
	if (meta.mtime_ms > 0) {
		return new Date(meta.mtime_ms).toISOString().slice(0, 10);
	}
	return null;
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

async function main() {
	const { public_dir, dist_dir, site_url } = parse_args();

	console.log(`📂 Source:   ${public_dir}`);
	console.log(`📦 Output:   ${dist_dir}/sitemap.xml`);
	console.log(`🔗 Site URL: ${site_url}`);
	console.log("");

	if (!existsSync(public_dir)) {
		console.error(`✗ Source directory does not exist: ${public_dir}`);
		process.exit(1);
	}
	if (!existsSync(dist_dir)) {
		console.error(`✗ Dist directory does not exist: ${dist_dir} — run static_build first`);
		process.exit(1);
	}

	const translations = await load_all_translations(public_dir, active_languages);
	const page_files = collect_page_files(public_dir, active_languages);
	const route_map = build_static_route_map(translations, page_files, active_languages);

	function localized_url_for_lang(canonical_path: string, lang: string): string {
		const per_lang = route_map.get(canonical_path);
		const localized = per_lang?.get(lang) ?? canonical_path;
		const prefix = lang === default_language ? "" : `/${lang}`;
		const trimmed = (prefix + localized).replace(/\/+$/, "");
		return site_url + trimmed + "/";
	}

	const url_entries: string[] = [];
	let included = 0;
	let skipped = 0;

	for (const rel_path of page_files) {
		const meta = read_page_meta(public_dir, rel_path, active_languages);

		if (meta.frontmatter.sitemap === false || meta.frontmatter.noindex === true) {
			skipped++;
			continue;
		}

		const canonical = template_to_canonical(rel_path);
		const lastmod = lastmod_for(meta);

		// A page is localized unless its frontmatter opts out (`localize: false`,
		// upstream) or it lives in an English-only subtree (blog, product docs —
		// project policy in src/lib/seo_localization). Non-localized pages emit
		// only the default-language <loc> and drop the hreflang cluster; the
		// other-language URLs are duplicates that canonicalize back to the
		// default in the rendered HTML.
		const localized = page_is_localized(meta.frontmatter) && path_is_localized(canonical);
		const languages_to_emit = localized ? active_languages : [default_language];

		const alternates: string[] = [];
		if (languages_to_emit.length > 1) {
			for (const lang of languages_to_emit) {
				alternates.push(
					`    <xhtml:link rel="alternate" hreflang="${lang}" href="${xml_escape(localized_url_for_lang(canonical, lang))}"/>`,
				);
			}
			alternates.push(
				`    <xhtml:link rel="alternate" hreflang="x-default" href="${xml_escape(localized_url_for_lang(canonical, default_language))}"/>`,
			);
		}

		for (const lang of languages_to_emit) {
			const loc = localized_url_for_lang(canonical, lang);
			const lines = [`  <url>`, `    <loc>${xml_escape(loc)}</loc>`];
			if (lastmod) lines.push(`    <lastmod>${lastmod}</lastmod>`);
			lines.push(...alternates);
			lines.push(`  </url>`);
			url_entries.push(lines.join("\n"));
			included++;
		}
	}

	const xml = [
		`<?xml version="1.0" encoding="UTF-8"?>`,
		`<?xml-stylesheet type="text/xsl" href="/sitemap.xsl"?>`,
		`<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9"`,
		`        xmlns:xhtml="http://www.w3.org/1999/xhtml">`,
		...url_entries,
		`</urlset>`,
		``,
	].join("\n");

	const robots_txt = `User-agent: *\nAllow: /\n\nSitemap: ${site_url}/sitemap.xml\n`;
	await write_sitemap_files(xml, robots_txt, dist_dir, public_dir);

	console.log("═".repeat(50));
	console.log(`✓ Sitemap written`);
	console.log(`  URLs included:    ${included}`);
	if (skipped > 0) console.log(`  Pages skipped:    ${skipped} (sitemap:false / noindex:true)`);
	console.log(`  Output files:     ${join(dist_dir, "sitemap.xml")}`);
	console.log(`                    ${join(dist_dir, "robots.txt")}`);
	console.log("═".repeat(50));
}

await main();
