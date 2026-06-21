#!/usr/bin/env bun

/**
 * scripts/static_build.ts
 *
 * Static site generator: renders .ree templates from a source directory
 * to static HTML in an output directory, with full multi-language support.
 *
 * Usage:
 *   bun scripts/static_build.ts [--public ./src/public] [--dist ./dist] [--base-url /] [--site-url https://example.com]
 *   bun scripts/static_build.ts --help
 *
 * The source directory should contain:
 *   - .ree template files (any nesting depth)
 *   - <lang>.json translation files (same structure as routes/)
 *   - Static assets (CSS, images, JS) — copied verbatim to dist/ root
 *
 * Output structure (using localized route names):
 *   /dist/
 *     index.html              ← default language (SL) at root
 *     /o-nas/index.html       ← SL localized /about → "o-nas"
 *     /kontakt/index.html     ← SL localized /contact → "kontakt"
 *     /blog/index.html        ← SL blog (route_name same as canonical)
 *     /blog/blog-post1/       ← SL nested localized paths
 *     /css/
 *       style.css
 *     /en/                    ← other languages with /{lang} prefix
 *       index.html
 *       /about/index.html
 *       /contact/index.html
 *       /blog/index.html
 *       /blog/blog-post1/
 *
 * Language-variant templates (about.sl.ree → about.en.ree → about.ree)
 * are resolved automatically by the template engine.
 */

import { existsSync, mkdirSync, copyFileSync, rmSync, readFileSync, readdirSync } from "fs";
import { join, dirname, resolve } from "path";
import { pathToFileURL } from "url";

import { pagination as pagination_config } from "$config/pagination";
import { redirects as raw_redirects } from "$config/redirects";
import {
	languages,
	active_languages,
	default_language,
	language_names,
	language_locales,
} from "$config/supported_languages";
import { resolve_route_records } from "$lib/collect_records";
import { load_all_translations } from "$lib/i18n";
import { process_docs_markdown } from "$lib/markdown_docs";
import { chunk_count, pagination_labels, paginate, read_per_page_override } from "$lib/pagination";
import { check_collisions_and_validate_targets, emit_redirects, load_and_validate_redirects } from "$lib/redirects";
import {
	build_static_route_map,
	collect_page_files,
	page_is_localized,
	parse_frontmatter,
	path_to_namespace,
	template_to_canonical,
	walk_dir,
} from "$lib/static_site";
import TemplateEngine from "$lib/template_engine";
import { create_template_helpers } from "$lib/template_helpers";
import { expand_doc_figures } from "$root/src/lib/docs_figures";
import { find_docs_site } from "$root/src/lib/docs_sites";
import { markdown_styles } from "$root/src/lib/markdown_styles";
import {
	project_helper_functions,
	read_project_version_info,
	resolve_md_layout,
	build_docs_sidebar_groups,
	coming_soon_body,
} from "$root/src/lib/project_helpers";
import { path_is_localized } from "$root/src/lib/seo_localization";

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function print_usage() {
	console.error("Usage: bun scripts/static_build.ts [options]");
	console.error("");
	console.error("Options:");
	console.error("  --public <dir>   Source directory with .ree templates (default: ./src/public)");
	console.error("  --dist <dir>     Output directory for static HTML (default: ./dist)");
	console.error("  --base-url <url> Base URL for the site (default: /)");
	console.error("  --site-url <url> Full site URL for hreflang links (default: empty)");
	console.error("  --verbose        Log each rendered file");
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
	let base_url = "/";
	let site_url = process.env.SITE_URL ?? "";
	let verbose = false;

	for (let i = 0; i < args.length; i++) {
		const arg = args[i];
		if (!arg) continue;

		if (arg === "--public") {
			public_dir = args[++i] ?? public_dir;
		} else if (arg === "--dist") {
			dist_dir = args[++i] ?? dist_dir;
		} else if (arg === "--base-url") {
			base_url = args[++i] ?? base_url;
		} else if (arg === "--site-url") {
			site_url = args[++i] ?? site_url;
		} else if (arg === "--verbose") {
			verbose = true;
		}
	}

	return {
		public_dir: resolve(public_dir),
		dist_dir: resolve(dist_dir),
		base_url,
		site_url: site_url.replace(/\/+$/, ""),
		verbose,
	};
}

/** Traverse a nested translation object by dot-separated path. */
function get_nested(obj: any, path: string): any {
	if (!path || !obj) return {};

	const parts = path.split(".");
	let current = obj;

	for (const part of parts) {
		if (!current || typeof current !== "object") return {};
		current = current[part];
	}

	return current ?? {};
}

/** Deep-merge source into target (mutates target). */
function deep_merge(target: any, source: any): any {
	for (const key of Object.keys(source ?? {})) {
		const sv = source[key];
		const tv = target[key];

		if (sv && typeof sv === "object" && !Array.isArray(sv) && tv && typeof tv === "object" && !Array.isArray(tv)) {
			deep_merge(tv, sv);
		} else {
			target[key] = sv;
		}
	}

	return target;
}

/**
 * Resolve the most specific .md file for a given language with fallback chain:
 *   {name}.{lang}.md → {name}.{default_lang}.md → {name}.md
 * Returns the file content and resolved relative path, or null if none found.
 */
async function resolve_md_file(
	base_rel_path: string,
	lang: string,
	default_language: string,
	public_dir: string,
): Promise<{ content: string; resolved_path: string } | null> {
	const name_without_ext = base_rel_path.replace(/\.md$/, "");
	const candidates = [`${name_without_ext}.${lang}.md`, `${name_without_ext}.${default_language}.md`, base_rel_path];

	for (const candidate of candidates) {
		const full_path = join(public_dir, candidate);
		if (existsSync(full_path)) {
			const content = await Bun.file(full_path).text();
			return { content, resolved_path: candidate };
		}
	}

	return null;
}

/**
 * Extract a page title from markdown content.
 * Checks frontmatter.title first, then first # Heading.
 */
function extract_md_title(content: string): string {
	const { data: frontmatter, body } = parse_frontmatter(content);
	if (frontmatter.title) return String(frontmatter.title);

	const h1_match = body.match(/^#\s+(.+)$/m);
	if (h1_match) return h1_match[1].trim();

	return "Untitled";
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

async function main() {
	const { public_dir, dist_dir, base_url, site_url, verbose } = parse_args();

	console.log(`📂 Source:  ${public_dir}`);
	console.log(`📦 Output:  ${dist_dir}`);
	console.log(`🌐 Base:    ${base_url || "/"}`);

	if (site_url) {
		console.log(`🔗 Site URL: ${site_url}`);
	} else {
		console.log(`   ⚠  No --site-url provided — skipping hreflang links (Google requires absolute URLs)`);
	}
	console.log("");

	if (!existsSync(public_dir)) {
		console.error(`✗ Source directory does not exist: ${public_dir}`);
		process.exit(1);
	}

	// Phase 1: schema-validate redirects before doing any work.
	// Collision checks and target validation happen in Phase 2, after dist/ is built.
	const redirects = load_and_validate_redirects(raw_redirects);
	if (redirects.length > 0) {
		console.log(`🔗 Redirects: ${redirects.length} declared`);
		console.log("");
	}

	// Track what the build produces, so Phase 2 can detect collisions.
	const generated_routes = new Set<string>();
	const static_asset_paths = new Set<string>();

	// Clear dist directory before build (remove stale files from previous builds)
	if (existsSync(dist_dir)) {
		rmSync(dist_dir, { recursive: true, force: true });
	}

	// Ensure dist directory exists
	mkdirSync(dist_dir, { recursive: true });

	// 1. Load translations from public/ (same loader as lib/i18n.ts)
	console.log("📖 Loading translations...");
	const translations = await load_all_translations(public_dir, languages);
	console.log(`   ✓ ${languages.length} language(s) loaded`);

	// Derive language self-names for the switcher from each language's own translation file.
	// (en.json says "English", sl.json says "Slovenščina") — auto-adapts when languages are added.
	const language_self_names: Record<string, string> = {};
	for (const lang of languages) {
		language_self_names[lang] = translations[lang]?.routes?.ui?.language_names?.[lang] ?? lang;
	}

	// Language URL prefixes: default language at root (""), others at "/{lang}"
	const language_urls: Record<string, string> = {};
	for (const lang of languages) {
		language_urls[lang] = lang === default_language ? "" : `/${lang}`;
	}

	// 2. Create template engine pointed at public/
	const engine = new TemplateEngine({
		views: public_dir,
		ext: ".ree",
		cache: false, // never cache during build
		autoEscape: true,
	});

	// 3. Walk public/ and split into renderable templates vs static assets
	const all_files = walk_dir(public_dir);
	const all_page_files = collect_page_files(public_dir, languages);
	const ree_files = all_page_files.filter((f) => f.endsWith(".ree"));
	const md_files = all_page_files.filter((f) => f.endsWith(".md"));

	const static_files: string[] = [];
	for (const file of all_files) {
		if (!file.endsWith(".ree") && !file.endsWith(".json") && !file.endsWith(".ts") && !file.endsWith(".md")) {
			static_files.push(file);
		}
	}

	const raw_ree_count = all_files.filter((f) => f.endsWith(".ree")).length;
	const raw_md_count = all_files.filter((f) => f.endsWith(".md")).length;

	console.log(`   📄 ${ree_files.length} template(s) found (from ${raw_ree_count} file(s))`);
	console.log(`   📝 ${md_files.length} markdown file(s) found (from ${raw_md_count} file(s))`);
	console.log(`   🎨 ${static_files.length} static file(s) found`);
	console.log("");

	// Index templates of paginated routes are rendered by the dedicated pagination
	// phase (once per page-number, with pagination props), not by the normal .ree
	// loop. Collect their rel paths so both the data-file loop and the render loop
	// can skip them. They stay in `ree_files` so the route map still knows /<route>.
	const paginated_index_rels = new Set<string>();
	if (pagination_config.enabled) {
		for (const route of pagination_config.routes) {
			const route_dir = route.route.replace(/^\/+|\/+$/g, "");
			paginated_index_rels.add(`${route_dir}/index.ree`);
		}
	}

	// 3c. Load dynamic data for templates that have a .ts file
	// Convention: for "index.ree", look for "index.ts" in the same directory.
	// The .ts file must export an async function `load_template_data()` returning a Record.
	const template_data_map = new Map<string, Record<string, any>>();

	console.log("   📊 Loading data files...");

	for (const rel_path of ree_files) {
		// Paginated route indexes resolve their own records in the pagination phase.
		if (paginated_index_rels.has(rel_path)) continue;

		// Data file path: same base name but .ts instead of .ree
		const data_rel_path = rel_path.replace(/\.ree$/, ".ts");
		const data_full_path = join(public_dir, data_rel_path);

		if (existsSync(data_full_path)) {
			console.log(`     Trying ${data_rel_path}...`);

			if (verbose) {
				console.log(`   📊 Loading data from ${data_rel_path}`);
			}

			try {
				// Dynamic import on Windows needs file:// URL or it can hang
				const file_url = pathToFileURL(data_full_path).href;
				const data_module = await import(file_url);

				console.log("data_module:", data_module);

				if (typeof data_module.load_template_data === "function") {
					console.log(`     Calling load_template_data() in ${data_rel_path}...`);
					const result = await data_module.load_template_data();

					console.log("result:", result);

					template_data_map.set(rel_path, result ?? {});
					console.log(`   ✓ Data loaded for ${rel_path}`);
				} else {
					console.warn(`   ⚠  ${data_rel_path} does not export load_template_data()`);
				}
			} catch (err) {
				const msg = err instanceof Error ? err.message : String(err);
				console.warn(`   ⚠  Could not load ${data_rel_path}: ${msg}`);
			}
		}
	}

	if (template_data_map.size > 0) {
		console.log(`   ✓ ${template_data_map.size} data file(s) loaded`);
	} else {
		console.log("   No data files to load");
	}

	// Build route map for localized path resolution
	// (must be after ree_files and md_files is populated)
	console.log("  🗺️  Building route map...");
	const all_template_files = [...ree_files, ...md_files];
	const route_map = build_static_route_map(translations, all_template_files, languages);
	console.log(`     ✓ ${route_map.size} template(s) mapped`);

	/** Resolve a canonical path to its localized version for a given language. */
	function resolve_localized_path(canonical_path: string, lang: string): string {
		const per_lang = route_map.get(canonical_path);
		if (!per_lang) return canonical_path;
		return per_lang.get(lang) ?? canonical_path;
	}

	/**
	 * Get the full URL for a canonical path in the current language.
	 * Includes the language prefix for non-default languages.
	 */
	function localized_url_for_lang(canonical_path: string, target_lang: string): string {
		const localized = resolve_localized_path(canonical_path, target_lang);
		const prefix = target_lang === default_language ? "" : `/${target_lang}`;
		return prefix + localized;
	}

	console.log("   🖨️  Rendering templates...");

	// 4. Render each template per language
	let rendered_count = 0;
	let error_count = 0;

	const year = new Date().getFullYear();

	// Read version info for the docs sidebar version pill
	const { reepolee_version, reeweb_version, tailwind_version } = await read_project_version_info();

	for (const rel_path of ree_files) {
		// Paginated route indexes are rendered by the pagination phase below.
		if (paginated_index_rels.has(rel_path)) continue;

		// Template name without extension (for engine.render)
		const template_name = rel_path.replace(/\.ree$/, "");

		// Canonical path for this template
		const canonical_path = template_to_canonical(rel_path);

		console.log(`     Rendering ${rel_path}...`);

		for (const lang of languages) {
			// Merge translations: global "routes" + route-specific
			const global_strings = translations[lang]?.routes ?? {};
			const namespace = path_to_namespace(rel_path);
			const route_strings = namespace ? get_nested(translations[lang], namespace) : {};

			const merged = deep_merge(structuredClone(global_strings), route_strings);

			const locale = language_locales[lang];

			// Resolve localized path for this language
			const localized_path = resolve_localized_path(canonical_path, lang);

			// Default language is served at root (no /{lang} prefix)
			const is_default = lang === default_language;

			// The output file is at the localized path (directory-style: /o-nas/index.html)
			// Root path "/" is special — it goes to just index.html
			let output_rel: string;
			let verbose_label: string;
			if (localized_path === "/") {
				// Home page: root goes to index.html, other langs go to {lang}/index.html
				output_rel = is_default ? "index.html" : `${lang}/index.html`;
				verbose_label = is_default ? "(root)/index.html" : `${lang}/index.html`;
			} else {
				const localized_no_lead = localized_path.replace(/^\//, "");
				output_rel = is_default ? `${localized_no_lead}/index.html` : `${lang}/${localized_no_lead}/index.html`;
				verbose_label = `${is_default ? "(root)" : lang}/${localized_no_lead}/index.html`;
			}

			const lang_url_prefix = is_default ? "" : `/${lang}`;
			const request_url = localized_path === "/" ? lang_url_prefix + "/" : lang_url_prefix + localized_path + "/";

			generated_routes.add(request_url);

			// English-only subtrees (blog, product docs) aren't localized: skip
			// the hreflang cluster and point every non-default variant at the
			// default-language URL via rel=canonical below. .ree pages carry no
			// frontmatter, so the decision is purely path-based here.
			const localized = path_is_localized(canonical_path);

			// Strip trailing slashes before adding one back (handles home page "/" and "/en/" safely).
			const abs_url = (path: string) => site_url + path.replace(/\/+$/, "") + "/";

			// Build hreflang alternate links for this canonical page
			// (only when --site-url is provided; hreflang requires absolute URLs for Google)
			const hreflang_links: { lang: string; href: string }[] = [];
			if (site_url && localized) {
				for (const alt_lang of languages) {
					hreflang_links.push({
						lang: alt_lang,
						href: abs_url(localized_url_for_lang(canonical_path, alt_lang)),
					});
				}
				// x-default points to the default language variant
				hreflang_links.push({
					lang: "x-default",
					href: abs_url(localized_url_for_lang(canonical_path, default_language)),
				});
			}

			// Canonical URL: a page is its own canonical, except a non-default
			// variant of a non-localized page, which canonicalizes to the
			// default-language URL.
			const canonical_url =
				site_url && (is_default || localized)
					? abs_url(request_url)
					: site_url
						? abs_url(localized_url_for_lang(canonical_path, default_language))
						: "";

			const template_data = template_data_map.get(rel_path) ?? {};

			const data: Record<string, any> = {
				lang,
				lang_url_prefix,
				locale,
				active_languages,
				language_names,
				language_self_names,
				default_language,
				base_url,
				site_url,
				hreflang_links,
				site_name: "Static Site",
				year,
				is_dev: false,
				rendered_at: new Date().toISOString(),
				request_url,
				canonical_path,
				canonical_url,
				language_urls,
				reepolee_version,
				reeweb_version,
				tailwind_version,
				// Spread template data first so merged translations can override if needed
				...template_data,
				...merged,
			};

			// Add template helpers (localized_path, locale_date, etc.)
			data.helpers = create_template_helpers(data, project_helper_functions);

			// Override localized_path with static-build version that includes lang prefix
			data.helpers.localized_path = (path: string) => {
				return localized_url_for_lang(path, lang);
			};

			// Helper for language switcher: get the URL for the same canonical page in another language
			data.helpers.localized_path_for_lang = (target_lang: string, path: string) => {
				return localized_url_for_lang(path, target_lang);
			};

			try {
				const html = await engine.render(template_name, data);

				const output_path = join(dist_dir, output_rel);
				const output_dir = dirname(output_path);

				mkdirSync(output_dir, { recursive: true });
				await Bun.write(output_path, html);

				rendered_count++;

				if (verbose) {
					console.log(`   ✓ ${verbose_label}`);
				}
			} catch (err) {
				error_count++;
				const msg = err instanceof Error ? err.message : String(err);
				console.error(`   ✗ ${lang}/${rel_path}: ${msg}`);
			}
		}
	}

	// ── Build generic sidebar navigation ─────────────────────────
	// For every folder whose index.md has `has_sidebar: true` in frontmatter,
	// build a per-language list of sidebar links from all .md files in that folder.
	const sidebar_map = new Map<string, Map<string, { title: string; canonical_path: string }[]>>();

	for (const base_rel_path of md_files) {
		const base_name = base_rel_path.split("/").pop() ?? "";
		if (base_name !== "index.md" && !base_name.match(/^\d+_index\.md$/)) continue;

		const resolved = await resolve_md_file(base_rel_path, default_language, default_language, public_dir);
		if (!resolved) continue;

		const { data: frontmatter } = parse_frontmatter(resolved.content);
		const show_sidebar = frontmatter.has_sidebar;
		if (show_sidebar !== true && show_sidebar !== "true" && show_sidebar !== "yes") continue;

		const folder_path = base_rel_path.replace(/\/?(?:\d+_)?index\.md$/, "");

		const folder_md_files = md_files
			.filter((f) => f.startsWith(folder_path + "/") && f !== base_rel_path)
			.sort((a, b) => a.localeCompare(b));

		if (folder_md_files.length === 0) continue;

		const per_lang_sidebar = new Map<string, { title: string; canonical_path: string }[]>();

		for (const lang of languages) {
			const links: { title: string; canonical_path: string }[] = [];

			for (const page_rel_path of folder_md_files) {
				const resolved_page = await resolve_md_file(page_rel_path, lang, default_language, public_dir);
				if (!resolved_page) continue;

				const { data: page_frontmatter } = parse_frontmatter(resolved_page.content);
				const skip_nav = page_frontmatter["skip-navigation"];
				if (skip_nav === true || skip_nav === "true" || skip_nav === "yes") continue;

				const title = extract_md_title(resolved_page.content);
				const canonical_path_link = template_to_canonical(page_rel_path);

				links.push({ title, canonical_path: canonical_path_link });
			}

			per_lang_sidebar.set(lang, links);
		}

		sidebar_map.set(folder_path, per_lang_sidebar);

		console.log(`   📑 Sidebar enabled for /${folder_path} (${folder_md_files.length} page(s))`);
	}

	// ── Render markdown files ──────────────────────────────────
	if (md_files.length > 0) {
		console.log("   📝 Rendering markdown files...");
	}

	for (const base_rel_path of md_files) {
		const canonical_path = template_to_canonical(base_rel_path);

		if (verbose) {
			console.log(`     Rendering markdown ${base_rel_path}...`);
		}

		for (const lang of languages) {
			const resolved = await resolve_md_file(base_rel_path, lang, default_language, public_dir);

			if (!resolved) {
				error_count++;
				console.error(`   ✗ ${lang}/${base_rel_path}: markdown file not found`);
				continue;
			}

			const { content: md_content } = resolved;
			const { data: frontmatter, body: markdown_body } = parse_frontmatter(md_content);

			const raw_html = Bun.markdown.html(markdown_body, {
				tables: true,
				strikethrough: true,
				tasklists: true,
				autolinks: { url: true, www: true, email: true },
				headings: { ids: true },
			});

			const { html: processed_html, headings: all_headings } = process_docs_markdown(raw_html, markdown_styles);
			// Expand <media-frame> placeholder tokens into the homepage-style card.
			const html_body = expand_doc_figures(processed_html);
			const toc_headings = all_headings.filter((h) => h.level > 1);
			const first_section = toc_headings[0]?.id ?? "";

			// Auto-detect docs pages via the docs-site registry (see config/docs_sites.ts)
			const docs_site = find_docs_site(canonical_path);
			const layout = resolve_md_layout(base_rel_path, frontmatter, public_dir);
			let docs_sidebar_groups: any[] | undefined;
			if (docs_site) {
				docs_sidebar_groups = build_docs_sidebar_groups(
					docs_site.sidebar,
					docs_site.root,
					canonical_path,
					public_dir,
					md_files,
					languages,
					default_language,
					lang,
				);
			}

			const locale = language_locales[lang];

			const localized_path = resolve_localized_path(canonical_path, lang);

			const is_default = lang === default_language;

			let output_rel: string;
			let verbose_label: string;
			if (localized_path === "/") {
				output_rel = is_default ? "index.html" : `${lang}/index.html`;
				verbose_label = is_default ? "(root)/index.html" : `${lang}/index.html`;
			} else {
				const localized_no_lead = localized_path.replace(/^\//, "");
				output_rel = is_default ? `${localized_no_lead}/index.html` : `${lang}/${localized_no_lead}/index.html`;
				verbose_label = `${is_default ? "(root)" : lang}/${localized_no_lead}/index.html`;
			}

			const lang_url_prefix = is_default ? "" : `/${lang}`;
			const request_url = localized_path === "/" ? lang_url_prefix + "/" : lang_url_prefix + localized_path + "/";

			generated_routes.add(request_url);

			// A page is localized unless its frontmatter opts out (`localize: false`)
			// or it lives in an English-only subtree (blog, product docs). Non-
			// localized pages skip the hreflang cluster (a byte-identical page is
			// not a real language alternate) and point every other-language URL at
			// the default via rel=canonical below.
			const localized = page_is_localized(frontmatter) && path_is_localized(canonical_path);

			const abs_url = (path: string) => site_url + path.replace(/\/+$/, "") + "/";

			const hreflang_links: { lang: string; href: string }[] = [];
			if (site_url && localized) {
				for (const alt_lang of languages) {
					hreflang_links.push({
						lang: alt_lang,
						href: abs_url(localized_url_for_lang(canonical_path, alt_lang)),
					});
				}
				hreflang_links.push({
					lang: "x-default",
					href: abs_url(localized_url_for_lang(canonical_path, default_language)),
				});
			}

			// Canonical URL: a page is its own canonical, except a non-default
			// variant of a non-localized page, which canonicalizes to the
			// default-language URL.
			const canonical_url =
				site_url && (is_default || localized)
					? abs_url(request_url)
					: site_url
						? abs_url(localized_url_for_lang(canonical_path, default_language))
						: "";

			const global_strings = translations[lang]?.routes ?? {};

			let sidebar_links: { title: string; url: string; active: boolean }[] | undefined;
			for (const [folder_path, per_lang] of sidebar_map) {
				const folder_canonical = "/" + folder_path;
				if (canonical_path === folder_canonical || canonical_path.startsWith(folder_canonical + "/")) {
					sidebar_links = (per_lang.get(lang) ?? []).map((link) => ({
						title: link.title,
						url: localized_url_for_lang(link.canonical_path, lang),
						active: link.canonical_path === canonical_path,
					}));
					break;
				}
			}

			const page_title = (frontmatter.title as string) || frontmatter.site_name || "Static Site";
			const heading = (frontmatter.heading as string) || page_title;
			const body_markup =
				frontmatter.layout === "coming-soon"
					? coming_soon_body(heading)
					: `<article class="article-body mx-auto max-w-3xl">${html_body}</article>`;

			const data: Record<string, any> = {
				lang,
				lang_url_prefix,
				locale,
				active_languages,
				language_names,
				language_self_names,
				default_language,
				base_url,
				site_url,
				hreflang_links,
				site_name: frontmatter.site_name || "Static Site",
				page_title,
				year,
				is_dev: false,
				rendered_at: new Date().toISOString(),
				request_url,
				canonical_path,
				canonical_url,
				language_urls,
				reepolee_version,
				reeweb_version,
				tailwind_version,
				...global_strings,
				sidebar: sidebar_links,
				docs_sidebar_groups,
				toc_headings,
				active_section: first_section,
				body: body_markup,
				...frontmatter,
			};

			data.helpers = create_template_helpers(data, project_helper_functions);
			data.helpers.localized_path = (path: string) => {
				return localized_url_for_lang(path, lang);
			};
			data.helpers.localized_path_for_lang = (target_lang: string, path: string) => {
				return localized_url_for_lang(path, target_lang);
			};

			try {
				const html = await engine.render(layout, data);

				const output_path = join(dist_dir, output_rel);
				const output_dir = dirname(output_path);

				mkdirSync(output_dir, { recursive: true });
				await Bun.write(output_path, html);

				rendered_count++;

				if (verbose) {
					console.log(`   ✓ (md) ${verbose_label}`);
				}
			} catch (err) {
				error_count++;
				const msg = err instanceof Error ? err.message : String(err);
				console.error(`   ✗ ${lang}/${base_rel_path}: ${msg}`);
			}
		}
	}

	// ── 4b. Render paginated index pages for registered routes ───────
	// For each enabled route we resolve its records (markdown by default, or an
	// external `load_records(lang)` loader), chunk them, and render the route's
	// index.ree once per page-number, per language:
	//   page 1   → /<route>/            (the normal index location)
	//   page ≥ 2 → /<route>/<segment>/<n>/
	if (pagination_config.enabled && pagination_config.routes.length > 0) {
		console.log("");
		console.log("   📄 Rendering paginated routes...");

		const seg = pagination_config.path_segment;

		for (const route of pagination_config.routes) {
			const route_dir = route.route.replace(/^\/+|\/+$/g, "");
			const index_rel = `${route_dir}/index.ree`;
			const template_name = `${route_dir}/index`;
			const index_full = join(public_dir, index_rel);

			if (!existsSync(index_full)) {
				console.warn(`   ⚠  Pagination route "${route_dir}" has no ${index_rel} — skipping`);
				continue;
			}

			const canonical_path = template_to_canonical(index_rel); // e.g. "/blog"
			const namespace = path_to_namespace(index_rel); // e.g. "blog"

			// per_page precedence: a literal `per-page="N"` on the component in the
			// template source > route config > global default.
			const index_source = readFileSync(index_full, "utf-8");
			const per_page = read_per_page_override(index_source) ?? route.per_page ?? pagination_config.per_page;

			for (const lang of languages) {
				// Markdown by default, or an external load_records(lang) loader.
				const records = await resolve_route_records(public_dir, route, lang, all_page_files);

				const last_page = chunk_count(records.length, per_page);

				const is_default = lang === default_language;
				const lang_url_prefix = is_default ? "" : `/${lang}`;
				const locale = language_locales[lang];

				// href for page `n` of this route, in `target_lang`.
				// `seg` empty → /blog/2/ ; otherwise → /blog/<seg>/2/.
				const page_url_for_lang = (target_lang: string, n: number) => {
					const base = localized_url_for_lang(canonical_path, target_lang).replace(/\/+$/, "");
					if (n <= 1) return `${base}/`;
					return seg ? `${base}/${seg}/${n}/` : `${base}/${n}/`;
				};
				const page_url = (n: number) => page_url_for_lang(lang, n);

				// Translations: global "routes" + route namespace (same as the .ree loop).
				const global_strings = translations[lang]?.routes ?? {};
				const route_strings = namespace ? get_nested(translations[lang], namespace) : {};
				const merged = deep_merge(structuredClone(global_strings), route_strings);

				// Labels live under ui.pagination to avoid colliding with the injected
				// `pagination` (PaginationData) prop when `...merged` is spread below.
				const labels = pagination_labels((global_strings as any).ui?.pagination);

				for (let page = 1; page <= last_page; page++) {
					const request_url = page_url(page);
					const output_rel = request_url.replace(/^\//, "").replace(/\/$/, "") + "/index.html";

					generated_routes.add(request_url);

					const pagination_data = paginate(
						records.length,
						page,
						per_page,
						{
							show_when_single_page: pagination_config.show_when_single_page,
							always_show_prev_next: pagination_config.always_show_prev_next,
							labels,
						},
						page_url,
					);

					const page_records = records.slice((page - 1) * per_page, page * per_page);

					// hreflang alternates: same page number in every language.
					const hreflang_links: { lang: string; href: string }[] = [];
					if (site_url) {
						for (const alt_lang of languages) {
							const alt_href = site_url + page_url_for_lang(alt_lang, page).replace(/\/+$/, "") + "/";
							hreflang_links.push({ lang: alt_lang, href: alt_href });
						}
						const default_href =
							site_url + page_url_for_lang(default_language, page).replace(/\/+$/, "") + "/";
						hreflang_links.push({ lang: "x-default", href: default_href });
					}

					// Each numbered page is its own canonical (index all pages).
					const canonical_url = site_url ? site_url + request_url.replace(/\/+$/, "") + "/" : "";

					const data: Record<string, any> = {
						lang,
						lang_url_prefix,
						locale,
						active_languages,
						language_names,
						language_self_names,
						default_language,
						base_url,
						site_url,
						hreflang_links,
						site_name: "Static Site",
						year,
						is_dev: false,
						rendered_at: new Date().toISOString(),
						request_url,
						canonical_path,
						canonical_url,
						language_urls,
						records: page_records,
						pagination: pagination_data,
						pagination_variant: pagination_config.variant,
						...merged,
					};

					data.helpers = create_template_helpers(data, project_helper_functions);
					data.helpers.localized_path = (path: string) => localized_url_for_lang(path, lang);
					data.helpers.localized_path_for_lang = (target_lang: string, path: string) =>
						localized_url_for_lang(path, target_lang);

					try {
						const html = await engine.render(template_name, data);

						const output_path = join(dist_dir, output_rel);
						mkdirSync(dirname(output_path), { recursive: true });
						await Bun.write(output_path, html);

						rendered_count++;
						if (verbose) console.log(`   ✓ (page ${page}/${last_page}) ${output_rel}`);
					} catch (err) {
						error_count++;
						const msg = err instanceof Error ? err.message : String(err);
						console.error(`   ✗ ${lang}/${index_rel} page ${page}: ${msg}`);
					}
				}

				console.log(`   📄 /${route_dir} [${lang}]: ${records.length} record(s) → ${last_page} page(s)`);
			}
		}
	}

	// 5. Copy static files to dist root
	for (const rel_path of static_files) {
		const src_path = join(public_dir, rel_path);
		const dest_path = join(dist_dir, rel_path);

		mkdirSync(dirname(dest_path), { recursive: true });
		copyFileSync(src_path, dest_path);

		static_asset_paths.add("/" + rel_path.split(/[\\/]/).join("/"));
	}

	// 6. Phase 2: collision-check and emit redirects.
	// Runs after pages are rendered and static assets are copied, so dist/ is
	// in its final state for target-existence and collision checks.
	if (redirects.length > 0) {
		check_collisions_and_validate_targets(redirects, dist_dir, generated_routes, static_asset_paths);
		await emit_redirects(redirects, dist_dir);
		console.log(`   🔗 ${redirects.length} redirect(s) emitted (dist/_redirects + HTML stubs)`);
	}

	// 7. Summary
	const total = (ree_files.length + md_files.length) * languages.length;

	console.log("");
	console.log("═".repeat(50));
	console.log(`✓ Build complete`);
	console.log(`  Templates rendered:  ${rendered_count}/${total}`);

	if (error_count > 0) {
		console.log(`  Errors:             ${error_count}`);
	}

	console.log(`  Static files copied: ${static_files.length}`);
	console.log(`  Output directory:    ${dist_dir}`);
	console.log("═".repeat(50));

	process.exit(error_count > 0 ? 1 : 0);
}

await main();
