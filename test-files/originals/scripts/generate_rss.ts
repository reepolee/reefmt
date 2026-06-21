#!/usr/bin/env bun

/**
 * scripts/generate_rss.ts
 *
 * Emits per-language RSS 2.0 (feed.xml) and JSON Feed 1.1 (feed.json) for
 * the blog directory under public/. Reads `.md` files only — no database or
 * runtime dependency, so it works on statically generated sites.
 *
 * Output (default --blog-dir blog):
 *   dist/blog/feed.xml          ← default language
 *   dist/blog/feed.json         ← default language
 *   dist/<lang>/blog/feed.xml   ← other active languages
 *   dist/<lang>/blog/feed.json
 *
 * Usage:
 *   bun scripts/generate_rss.ts --public ./public --dist ./dist --site-url https://example.com
 *   bun scripts/generate_rss.ts --help
 *
 * CLI options (every option also reads its env fallback):
 *   --public <dir>             Source dir (default: ./public)
 *   --dist <dir>               Output dir (default: ./dist)
 *   --site-url <url>           REQUIRED. Absolute origin used for <link>/url fields.
 *   --blog-dir <name>          Sub-directory under --public to scan (default: blog)
 *   --formats <list>           Comma list: xml, json, or "xml,json" (default: xml,json)
 *   --max-items <n>            Limit items per feed (default: 50)
 *   --feed-title <text>        Override the feed title (default: "<site_name> — <blog>")
 *   --feed-description <text>  Override the feed description (default: per-lang)
 *   --help                     Print this usage and exit
 *
 * Per-post frontmatter is honored:
 *   title:           string                     — falls back to first H1
 *   description:     string                     — falls back to first paragraph
 *   summary:         string                     — alias for description
 *   abstract:        string                     — alias (academic layout)
 *   published_at:    YYYY-MM-DD or ISO datetime — falls back to file mtime
 *   date:            alias for published_at
 *   author:          string OR { name, email }
 *   authors:         array — first entry is used for RSS, all for JSON Feed
 *   rss:             false                      — opt this post out
 *   noindex:         true                       — also opts out
 */

import { existsSync, readFileSync, statSync } from "fs";
import { join, resolve } from "path";

import { active_languages, default_language, language_locales } from "$config/supported_languages";
import { load_all_translations } from "$lib/i18n";
import {
	build_static_route_map,
	collect_page_files,
	parse_frontmatter,
	template_to_canonical,
	type FrontMatter,
} from "$lib/static_site";

// ---------------------------------------------------------------------------
// CLI
// ---------------------------------------------------------------------------

function print_usage() {
	console.error("Usage: bun scripts/generate_rss.ts [options]");
	console.error("");
	console.error("Options:");
	console.error("  --public <dir>             Source directory (default: ./src/public)");
	console.error("  --dist <dir>               Output directory (default: ./dist)");
	console.error("  --site-url <url>           REQUIRED. Absolute origin (or SITE_URL env)");
	console.error("  --blog-dir <name>          Sub-directory under --public (default: blog)");
	console.error("  --formats <list>           Comma list: xml,json (default: xml,json)");
	console.error("  --max-items <n>            Limit items per feed (default: 50)");
	console.error("  --feed-title <text>        Override feed title");
	console.error("  --feed-description <text>  Override feed description");
	console.error("  --help                     Print this usage and exit");
}

type Args = {
	public_dir: string;
	dist_dir: string;
	site_url: string;
	blog_dir: string;
	formats: { xml: boolean; json: boolean };
	max_items: number;
	feed_title: string | null;
	feed_description: string | null;
};

function parse_args(): Args {
	const args = process.argv.slice(2);

	if (args.includes("--help")) {
		print_usage();
		process.exit(0);
	}

	let public_dir = "./src/public";
	let dist_dir = "./dist";
	let site_url = process.env.SITE_URL ?? "";
	let blog_dir = "blog";
	let formats_raw = "xml,json";
	let max_items = 50;
	let feed_title: string | null = null;
	let feed_description: string | null = null;

	for (let i = 0; i < args.length; i++) {
		const arg = args[i];
		if (!arg) continue;

		if (arg === "--public") {
			public_dir = args[++i] ?? public_dir;
		} else if (arg === "--dist") {
			dist_dir = args[++i] ?? dist_dir;
		} else if (arg === "--site-url") {
			site_url = args[++i] ?? site_url;
		} else if (arg === "--blog-dir") {
			blog_dir = args[++i] ?? blog_dir;
		} else if (arg === "--formats") {
			formats_raw = args[++i] ?? formats_raw;
		} else if (arg === "--max-items") {
			const raw = args[++i];
			const parsed = Number(raw);
			if (Number.isFinite(parsed) && parsed > 0) max_items = parsed;
		} else if (arg === "--feed-title") {
			feed_title = args[++i] ?? feed_title;
		} else if (arg === "--feed-description") {
			feed_description = args[++i] ?? feed_description;
		}
	}

	if (!site_url) {
		console.error("✗ --site-url is required (or set SITE_URL in .env)");
		print_usage();
		process.exit(1);
	}

	const tokens = formats_raw
		.split(",")
		.map((s) => s.trim().toLowerCase())
		.filter(Boolean);
	const formats = {
		xml: tokens.includes("xml"),
		json: tokens.includes("json"),
	};

	if (!formats.xml && !formats.json) {
		console.error(`✗ --formats must include at least one of: xml, json (got "${formats_raw}")`);
		process.exit(1);
	}

	return {
		public_dir: resolve(public_dir),
		dist_dir: resolve(dist_dir),
		site_url: site_url.replace(/\/+$/, ""),
		blog_dir: blog_dir.replace(/^\/+|\/+$/g, ""),
		formats,
		max_items,
		feed_title,
		feed_description,
	};
}

// ---------------------------------------------------------------------------
// String helpers
// ---------------------------------------------------------------------------

function xml_escape(s: string): string {
	return s
		.replace(/&/g, "&amp;")
		.replace(/</g, "&lt;")
		.replace(/>/g, "&gt;")
		.replace(/"/g, "&quot;")
		.replace(/'/g, "&apos;");
}

function cdata_wrap(s: string): string {
	const safe = s.replace(/]]>/g, "]]]]><![CDATA[>");
	return `<![CDATA[${safe}]]>`;
}

function strip_markdown(s: string): string {
	let out = s;
	out = out.replace(/!\[[^\]]*\]\([^)]+\)/g, "");
	out = out.replace(/\[([^\]]+)\]\([^)]+\)/g, "$1");
	out = out.replace(/`([^`]+)`/g, "$1");
	out = out.replace(/\*\*([^*]+)\*\*/g, "$1");
	out = out.replace(/__([^_]+)__/g, "$1");
	out = out.replace(/\*([^*]+)\*/g, "$1");
	out = out.replace(/_([^_]+)_/g, "$1");
	out = out.replace(/~~([^~]+)~~/g, "$1");
	out = out.replace(/^>\s+/gm, "");
	return out.trim();
}

function first_paragraph(body: string): string {
	const blocks = body.split(/\r?\n\s*\r?\n/);
	for (const block of blocks) {
		const trimmed = block.trim();
		if (!trimmed) continue;
		if (trimmed.startsWith("#")) continue;
		if (trimmed.startsWith("---")) continue;
		if (trimmed.startsWith("```")) continue;
		return trimmed.replace(/\r?\n/g, " ").replace(/\s+/g, " ");
	}
	return "";
}

function truncate(s: string, limit: number): string {
	if (s.length <= limit) return s;
	const cut = s.slice(0, limit);
	const last_space = cut.lastIndexOf(" ");
	const trimmed = last_space > limit * 0.6 ? cut.slice(0, last_space) : cut;
	return trimmed.replace(/[.,;:!?\s]+$/, "") + "…";
}

function rfc822(date: Date): string {
	return date.toUTCString();
}

function to_iso(date: Date): string {
	return date.toISOString();
}

function parse_date(value: unknown, fallback: Date): Date {
	if (value instanceof Date && !Number.isNaN(value.getTime())) return value;
	if (typeof value === "string" && value.trim()) {
		const parsed = new Date(value.trim());
		if (!Number.isNaN(parsed.getTime())) return parsed;
	}
	return fallback;
}

// ---------------------------------------------------------------------------
// Author normalization
// ---------------------------------------------------------------------------

type Author = { name: string; email?: string; url?: string };

function normalize_authors(fm: FrontMatter): Author[] {
	const out: Author[] = [];

	const raw_authors = fm.authors;
	if (Array.isArray(raw_authors)) {
		for (const entry of raw_authors) {
			if (typeof entry === "string" && entry.trim()) {
				out.push({ name: entry.trim() });
			} else if (entry && typeof entry === "object") {
				const obj = entry as Record<string, unknown>;
				const name = typeof obj.name === "string" ? obj.name.trim() : "";
				if (!name) continue;
				const author: Author = { name };
				if (typeof obj.email === "string" && obj.email.trim()) author.email = obj.email.trim();
				if (typeof obj.url === "string" && obj.url.trim()) author.url = obj.url.trim();
				out.push(author);
			}
		}
	}

	if (out.length === 0) {
		const single = fm.author;
		if (typeof single === "string" && single.trim()) {
			out.push({ name: single.trim() });
		} else if (single && typeof single === "object") {
			const obj = single as Record<string, unknown>;
			const name = typeof obj.name === "string" ? obj.name.trim() : "";
			if (name) {
				const author: Author = { name };
				if (typeof obj.email === "string" && obj.email.trim()) author.email = obj.email.trim();
				if (typeof obj.url === "string" && obj.url.trim()) author.url = obj.url.trim();
				out.push(author);
			}
		}
	}

	return out;
}

// ---------------------------------------------------------------------------
// Post collection
// ---------------------------------------------------------------------------

type Post = {
	rel_path: string;
	canonical_path: string;
	title: string;
	description: string;
	content_html: string;
	published_at: Date;
	authors: Author[];
};

function extract_md_title(fm: FrontMatter, body: string, fallback: string): string {
	const fm_title = fm.title;
	if (typeof fm_title === "string" && fm_title.trim()) return fm_title.trim();

	const h1 = body.match(/^#\s+(.+?)\s*$/m);
	if (h1) return strip_markdown(h1[1]);

	return fallback;
}

function extract_description(fm: FrontMatter, body: string): string {
	const candidates = [fm.description, fm.summary, fm.abstract];
	for (const candidate of candidates) {
		if (typeof candidate === "string" && candidate.trim()) {
			return truncate(strip_markdown(candidate), 320);
		}
	}

	const para = first_paragraph(body);
	return truncate(strip_markdown(para), 320);
}

/** Returns true only for the blog listing index (blog/index.md — the listing page itself).
 *  Nested index.md files (e.g. blog/post-name/index.md) are actual posts. */
function is_blog_listing(rel_path: string, blog_dir: string): boolean {
	return rel_path === `${blog_dir}/index.md`;
}

function resolve_md_for_lang(
	public_dir: string,
	rel_path: string,
	lang: string,
): { content: string; mtime_ms: number } | null {
	const without_ext = rel_path.replace(/\.md$/, "");
	const candidates = [`${without_ext}.${lang}.md`, `${without_ext}.${default_language}.md`, rel_path];

	for (const candidate of candidates) {
		const full = join(public_dir, candidate);
		if (!existsSync(full)) continue;
		const content = readFileSync(full, "utf-8");
		const mtime_ms = statSync(full).mtimeMs;
		return { content, mtime_ms };
	}

	return null;
}

function collect_posts(public_dir: string, blog_dir: string, lang: string, page_files: string[]): Post[] {
	const blog_prefix = blog_dir + "/";
	const blog_md_files = page_files.filter(
		(rel) => rel.endsWith(".md") && rel.startsWith(blog_prefix) && !is_blog_listing(rel, blog_dir),
	);

	const posts: Post[] = [];

	for (const rel_path of blog_md_files) {
		const resolved = resolve_md_for_lang(public_dir, rel_path, lang);
		if (!resolved) continue;

		const { data: fm, body } = parse_frontmatter(resolved.content);

		if (fm.rss === false || fm.noindex === true) continue;

		const canonical = template_to_canonical(rel_path);
		const fallback_title = canonical.split("/").pop() ?? "Untitled";

		const title = extract_md_title(fm, body, fallback_title);
		const description = extract_description(fm, body);

		const raw_html = Bun.markdown.html(body, {
			tables: true,
			strikethrough: true,
			tasklists: true,
			autolinks: { url: true, www: true, email: true },
			headings: { ids: true },
		});

		const published_at = parse_date(fm.published_at ?? fm.date, new Date(resolved.mtime_ms));
		const authors = normalize_authors(fm);

		posts.push({
			rel_path,
			canonical_path: canonical,
			title,
			description,
			content_html: raw_html,
			published_at,
			authors,
		});
	}

	posts.sort((a, b) => b.published_at.getTime() - a.published_at.getTime());

	return posts;
}

// ---------------------------------------------------------------------------
// Feed builders
// ---------------------------------------------------------------------------

type FeedMeta = {
	title: string;
	description: string;
	home_url: string;
	feed_url_xml: string;
	feed_url_json: string;
	lang: string;
	locale: string;
	build_date: Date;
};

function build_rss_xml(meta: FeedMeta, items: Post[], site_url: string): string {
	const item_xml = items
		.map((post) => {
			const url = site_url + post.canonical_path + "/";
			const author = post.authors[0];
			const author_tag = author
				? `      <author>${xml_escape(author.email ?? "no-reply@example.com")} (${xml_escape(author.name)})</author>\n`
				: "";

			return [
				`    <item>`,
				`      <title>${xml_escape(post.title)}</title>`,
				`      <link>${xml_escape(url)}</link>`,
				`      <guid isPermaLink="true">${xml_escape(url)}</guid>`,
				`      <pubDate>${rfc822(post.published_at)}</pubDate>`,
				`      <description>${cdata_wrap(post.description)}</description>`,
				`      <content:encoded>${cdata_wrap(post.content_html)}</content:encoded>`,
				author_tag.trimEnd(),
				`    </item>`,
			]
				.filter(Boolean)
				.join("\n");
		})
		.join("\n");

	return [
		`<?xml version="1.0" encoding="UTF-8"?>`,
		`<rss version="2.0"`,
		`     xmlns:atom="http://www.w3.org/2005/Atom"`,
		`     xmlns:content="http://purl.org/rss/1.0/modules/content/"`,
		`     xmlns:dc="http://purl.org/dc/elements/1.1/">`,
		`  <channel>`,
		`    <title>${xml_escape(meta.title)}</title>`,
		`    <link>${xml_escape(meta.home_url)}</link>`,
		`    <description>${xml_escape(meta.description)}</description>`,
		`    <language>${xml_escape(meta.locale)}</language>`,
		`    <lastBuildDate>${rfc822(meta.build_date)}</lastBuildDate>`,
		`    <atom:link href="${xml_escape(meta.feed_url_xml)}" rel="self" type="application/rss+xml" />`,
		item_xml,
		`  </channel>`,
		`</rss>`,
		``,
	].join("\n");
}

function build_json_feed(meta: FeedMeta, items: Post[], site_url: string): string {
	const json = {
		version: "https://jsonfeed.org/version/1.1",
		title: meta.title,
		description: meta.description,
		home_page_url: meta.home_url,
		feed_url: meta.feed_url_json,
		language: meta.locale,
		items: items.map((post) => {
			const url = site_url + post.canonical_path + "/";
			const item: Record<string, unknown> = {
				id: url,
				url,
				title: post.title,
				summary: post.description,
				content_html: post.content_html,
				date_published: to_iso(post.published_at),
			};
			if (post.authors.length > 0) {
				item.authors = post.authors.map((a) => {
					const author: Record<string, unknown> = { name: a.name };
					if (a.url) author.url = a.url;
					return author;
				});
			}
			return item;
		}),
	};

	return JSON.stringify(json, null, 2) + "\n";
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

async function main() {
	const args = parse_args();

	console.log(`📂 Source:    ${args.public_dir}`);
	console.log(`📦 Output:    ${args.dist_dir}`);
	console.log(`📰 Blog dir:  ${args.blog_dir}/`);
	console.log(`🔗 Site URL:  ${args.site_url}`);
	console.log(`🧾 Formats:   ${[args.formats.xml && "xml", args.formats.json && "json"].filter(Boolean).join(", ")}`);
	console.log("");

	if (!existsSync(args.public_dir)) {
		console.error(`✗ Source directory does not exist: ${args.public_dir}`);
		process.exit(1);
	}
	if (!existsSync(args.dist_dir)) {
		console.error(`✗ Dist directory does not exist: ${args.dist_dir} — run static_build first`);
		process.exit(1);
	}

	const blog_root = join(args.public_dir, args.blog_dir);
	if (!existsSync(blog_root)) {
		console.error(`✗ Blog directory does not exist: ${blog_root}`);
		process.exit(1);
	}

	const translations = await load_all_translations(args.public_dir, active_languages);
	const page_files = collect_page_files(args.public_dir, active_languages);

	// Route map is used so we render localized URLs for the home link consistently
	// with the rest of the static build (even though the default install keeps
	// /blog identical across languages).
	const route_map = build_static_route_map(translations, page_files, active_languages);

	function localized_url(canonical: string, lang: string): string {
		const per_lang = route_map.get(canonical);
		const localized = per_lang?.get(lang) ?? canonical;
		const prefix = lang === default_language ? "" : `/${lang}`;
		const trimmed = (prefix + localized).replace(/\/+$/, "");
		return args.site_url + trimmed + "/";
	}

	const build_date = new Date();
	const blog_canonical = "/" + args.blog_dir;
	let total_items = 0;

	for (const lang of active_languages) {
		const posts = collect_posts(args.public_dir, args.blog_dir, lang, page_files);
		const limited = posts.slice(0, args.max_items);

		const lang_strings = translations[lang]?.routes ?? {};
		const site_name = typeof lang_strings.site_name === "string" ? lang_strings.site_name : "Site";
		const blog_label = typeof lang_strings?.nav?.blog === "string" ? lang_strings.nav.blog : "Blog";
		const locale = language_locales[lang] ?? lang;

		const home_url = localized_url(blog_canonical, lang);
		const lang_path_prefix = lang === default_language ? "" : `/${lang}`;
		const feed_url_xml = args.site_url + lang_path_prefix + "/" + args.blog_dir + "/feed.xml";
		const feed_url_json = args.site_url + lang_path_prefix + "/" + args.blog_dir + "/feed.json";

		const meta: FeedMeta = {
			title: args.feed_title ?? `${site_name} — ${blog_label}`,
			description: args.feed_description ?? `${blog_label} — ${site_name}`,
			home_url,
			feed_url_xml,
			feed_url_json,
			lang,
			locale,
			build_date,
		};

		const out_dir_rel = lang === default_language ? args.blog_dir : `${lang}/${args.blog_dir}`;
		const out_dir = join(args.dist_dir, out_dir_rel);

		if (args.formats.xml) {
			const xml = build_rss_xml(meta, limited, args.site_url);
			const path_xml = join(out_dir, "feed.xml");
			await Bun.write(path_xml, xml);
			console.log(`   ✓ ${path_xml}  (${limited.length} item${limited.length === 1 ? "" : "s"})`);
		}

		if (args.formats.json) {
			const json = build_json_feed(meta, limited, args.site_url);
			const path_json = join(out_dir, "feed.json");
			await Bun.write(path_json, json);
			console.log(`   ✓ ${path_json}  (${limited.length} item${limited.length === 1 ? "" : "s"})`);
		}

		total_items += limited.length;
	}

	console.log("");
	console.log("═".repeat(50));
	console.log(`✓ RSS generation complete`);
	console.log(`  Languages:    ${active_languages.length}`);
	console.log(`  Total items:  ${total_items}`);
	console.log("═".repeat(50));
}

await main();
