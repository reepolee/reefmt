#!/usr/bin/env bun

/**
 * scripts/dev.ts
 *
 * Development server for src/public/ pages. Serves .ree templates and .md files
 * directly with live reload — no build step required.
 *
 * Usage:
 *   bun scripts/dev.ts                         # serve ./src/public on :3000
 *   bun scripts/dev.ts --port 8080             # custom port
 *   bun scripts/dev.ts --public ./src          # custom source dir
 *
 * Features:
 *   - .ree template rendering via TemplateEngine (multi-language)
 *   - .md file rendering via Bun.markdown.html + process_docs_markdown
 *   - Multi-language routing: default at /, others at /{lang}/
 *   - Live reload via WebSocket at /__reload
 *   - File watcher on src/public/ triggers auto-reload
 *   - Static file serving (CSS, JS, images) from src/public/ and static/
 */

import { existsSync, watch, readFileSync, readdirSync, statSync } from "fs";
import { join, dirname, extname, resolve, relative } from "path";
import { pathToFileURL } from "url";

import { pagination as pagination_config, type PaginationRoute } from "$config/pagination";
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
import {
	build_static_route_map,
	collect_page_files,
	parse_frontmatter,
	path_to_namespace,
	template_to_canonical,
	walk_dir,
} from "$lib/static_site";
import TemplateEngine from "$lib/template_engine";
import { create_template_helpers } from "$lib/template_helpers";
import { find_docs_site } from "$root/src/lib/docs_sites";
import { markdown_styles } from "$root/src/lib/markdown_styles";
import {
	project_helper_functions,
	read_project_version_info,
	resolve_md_layout,
	build_docs_sidebar_groups,
	coming_soon_body,
} from "$root/src/lib/project_helpers";

// ---------------------------------------------------------------------------
// Live reload state
// ---------------------------------------------------------------------------

const reload_clients = new Set<WebSocket>();

const CLIENT_SCRIPT = `
<script>
(function() {
  var protocol = location.protocol === "https:" ? "wss:" : "ws:";
  var ws = new WebSocket(protocol + "//" + location.host + "/__reload");
  var wasOpen = false;
  ws.onopen = function() { wasOpen = true; };
  ws.onmessage = function(e) {
    try { var d = JSON.parse(e.data); if (d.type === "reload") location.reload(); } catch(e) {}
  };
  ws.onclose = function() {
    if (wasOpen) { setTimeout(function() { location.reload(); }, 500); }
    else { setTimeout(arguments.callee, 1000); }
  };
})();
</script>`.trim();

function notify_reload() {
	for (const ws of reload_clients) {
		if (ws.readyState === WebSocket.OPEN) {
			ws.send(JSON.stringify({ type: "reload" }));
		}
	}
}

function inject_live_reload(html: string): string {
	// CLIENT_SCRIPT already includes <script> tags, so no wrapping needed
	if (/<\/body>/i.test(html)) return html.replace(/<\/body>/i, `${CLIENT_SCRIPT}</body>`);
	return html + CLIENT_SCRIPT;
}

// ---------------------------------------------------------------------------
// MIME types
// ---------------------------------------------------------------------------

const MIME_TYPES: Record<string, string> = {
	".html": "text/html; charset=utf-8",
	".css": "text/css; charset=utf-8",
	".js": "application/javascript; charset=utf-8",
	".json": "application/json; charset=utf-8",
	".svg": "image/svg+xml",
	".png": "image/png",
	".jpg": "image/jpeg",
	".jpeg": "image/jpeg",
	".gif": "image/gif",
	".webp": "image/webp",
	".ico": "image/x-icon",
	".woff2": "font/woff2",
	".woff": "font/woff",
	".ttf": "font/ttf",
	".txt": "text/plain; charset=utf-8",
	".xml": "application/xml; charset=utf-8",
};

function mime_type(path: string): string {
	return MIME_TYPES[extname(path).toLowerCase()] ?? "application/octet-stream";
}

// ---------------------------------------------------------------------------
// Arg parsing
// ---------------------------------------------------------------------------

let public_dir = "./src/public";
let port = 3000;

for (let i = 0; i < process.argv.length; i++) {
	const arg = process.argv[i];
	if (!arg) continue;
	if (arg === "--public" || arg === "--dir") public_dir = process.argv[++i] ?? public_dir;
	else if (arg === "--port" || arg === "-p") port = Number(process.argv[++i]) || port;
	else if (arg === "--help" || arg === "-h") {
		console.log("Usage: bun scripts/dev.ts [--public ./src/public] [--port 3000]");
		process.exit(0);
	}
}

public_dir = resolve(public_dir);
const project_root = resolve(".");
const static_dir = join(project_root, "static");
const dist_dir = join(project_root, "dist");

if (!existsSync(public_dir)) {
	console.error(`✗ Source directory not found: ${public_dir}`);
	process.exit(1);
}

// ---------------------------------------------------------------------------
// Startup: load translations, build route map, collect templates
// ---------------------------------------------------------------------------

const { reepolee_version, reeweb_version, tailwind_version } = await read_project_version_info();

console.log("📖 Loading translations...");
let translations = await load_all_translations(public_dir, languages);
console.log(`   ✓ ${languages.length} language(s) loaded`);

const all_page_files = collect_page_files(public_dir, languages);
const ree_files = all_page_files.filter((f) => f.endsWith(".ree"));
const md_files = all_page_files.filter((f) => f.endsWith(".md"));

console.log(`   📄 ${ree_files.length} template(s), 📝 ${md_files.length} markdown file(s)`);

// Build canonical → template path lookup (reverse of template_to_canonical)
const canonical_to_template = new Map<string, string>();
for (const rel_path of all_page_files) {
	const canonical = template_to_canonical(rel_path);
	if (!canonical_to_template.has(canonical)) {
		canonical_to_template.set(canonical, rel_path);
	}
}

// Build route map for localized URL resolution
let route_map = build_static_route_map(translations, all_page_files, languages);

// Reverse route map: (lang → localized_path) → canonical
// Used to resolve requests to localized routes like /sl/kontakt/ → /contact
let localized_to_canonical = new Map<string, Map<string, string>>();
for (const [canonical, per_lang] of route_map) {
	for (const [lang, localized_path] of per_lang) {
		if (!localized_to_canonical.has(lang)) {
			localized_to_canonical.set(lang, new Map());
		}
		localized_to_canonical.get(lang)!.set(localized_path, canonical);
	}
}

function resolve_localized_path(canonical_path: string, lang: string): string {
	const per_lang = route_map.get(canonical_path);
	if (!per_lang) return canonical_path;
	return per_lang.get(lang) ?? canonical_path;
}

function resolve_canonical_from_localized(localized_path: string, lang: string): string | null {
	const per_lang = localized_to_canonical.get(lang);
	if (!per_lang) return null;
	return per_lang.get(localized_path) ?? null;
}

function localized_url_for_lang(canonical_path: string, target_lang: string): string {
	const localized = resolve_localized_path(canonical_path, target_lang);
	const prefix = target_lang === default_language ? "" : `/${target_lang}`;
	return prefix + localized;
}

// Derive language self-names
let language_self_names: Record<string, string> = {};
for (const lang of languages) {
	language_self_names[lang] = translations[lang]?.routes?.ui?.language_names?.[lang] ?? lang;
}

const language_urls: Record<string, string> = {};
for (const lang of languages) {
	language_urls[lang] = lang === default_language ? "" : `/${lang}`;
}

// Create template engine without cache (dev mode)
const engine = new TemplateEngine({
	views: public_dir,
	ext: ".ree",
	cache: false,
	autoEscape: true,
});

// Keep a map of loaded data modules keyed by template path.
// In dev mode, we reload them on each request since they change often.
const data_module_cache = new Map<string, any>();

async function load_template_data(rel_path: string): Promise<Record<string, any>> {
	const data_rel_path = rel_path.replace(/\.ree$/, ".ts");
	const data_full_path = join(public_dir, data_rel_path);
	if (!existsSync(data_full_path)) return {};

	try {
		// Bun caches ES modules by URL, so a plain re-import would keep serving
		// stale page data after a `.ts` edit. Bust the cache on file mtime so
		// data changes hot-reload in dev (without growing the registry per-request).
		const mtime = statSync(data_full_path).mtimeMs;
		const file_url = `${pathToFileURL(data_full_path).href}?t=${mtime}`;
		const data_module = await import(file_url);
		if (typeof data_module.load_template_data === "function") {
			return (await data_module.load_template_data()) ?? {};
		}
	} catch {}
	return {};
}

// ---------------------------------------------------------------------------
// File watcher
// ---------------------------------------------------------------------------

async function reload_translations() {
	translations = await load_all_translations(public_dir, languages);

	// Rebuild route map (route_name may have changed)
	route_map = build_static_route_map(translations, all_page_files, languages);

	// Rebuild localized_to_canonical
	localized_to_canonical = new Map<string, Map<string, string>>();
	for (const [canonical, per_lang] of route_map) {
		for (const [lang, localized_path] of per_lang) {
			if (!localized_to_canonical.has(lang)) {
				localized_to_canonical.set(lang, new Map());
			}
			localized_to_canonical.get(lang)!.set(localized_path, canonical);
		}
	}

	// Rebuild language self-names
	for (const lang of languages) {
		language_self_names[lang] = translations[lang]?.routes?.ui?.language_names?.[lang] ?? lang;
	}
}

let watch_debounce: ReturnType<typeof setTimeout> | null = null;

try {
	watch(public_dir, { recursive: true }, (_event, filename) => {
		if (!filename) return;
		// Ignore non-source extensions
		const ext = extname(filename).toLowerCase();
		const is_json = ext === ".json";
		if (is_json || ext === ".ts" || ext === ".ree" || ext === ".md") {
			if (watch_debounce) clearTimeout(watch_debounce);
			watch_debounce = setTimeout(async () => {
				console.log(`   🔄 Change detected: ${filename}`);
				if (is_json) {
					try {
						await reload_translations();
					} catch (err) {
						const msg = err instanceof Error ? err.message : String(err);
						console.error(`   ✗ Failed to reload translations: ${msg}`);
						return;
					}
				}
				notify_reload();
			}, 100);
		}
	});
	console.log("   👀 Watching for changes...");
} catch (err) {
	console.warn("   ⚠  File watching not available on this platform");
}

// ---------------------------------------------------------------------------
// URL → template / file resolution
// ---------------------------------------------------------------------------

/**
 * Parse language and canonical path from a request URL.
 *
 *   /              → { lang: "sl", path: "/" }
 *   /en/           → { lang: "en", path: "/" }
 *   /about/        → { lang: "sl", path: "/about" }
 *   /en/about/     → { lang: "en", path: "/about" }
 *   /css/style.css → { lang: null, path: "/css/style.css" }   (static)
 */
function resolve_request(url_path: string): { lang: string | null; path: string } {
	const normalized = url_path.replace(/\/+$/, "") || "/";

	// Check if first segment is a language code
	const segments = normalized.split("/").filter(Boolean);
	const first = segments[0];

	if (first && languages.includes(first)) {
		const rest = segments.slice(1);
		const path = rest.length > 0 ? "/" + rest.join("/") : "/";
		return { lang: first, path };
	}

	// No language prefix → use default language
	return { lang: default_language, path: normalized };
}

/**
 * Try to resolve a canonical path + language to a template or file.
 * Returns the rendering strategy or null.
 */
function resolve_template(
	canonical: string,
	lang: string,
):
	| { kind: "ree"; rel_path: string }
	| { kind: "md"; rel_path: string; layout: string }
	| { kind: "static"; full_path: string }
	| null {
	// 1. Check hash map first (fast path)
	let template = canonical_to_template.get(canonical);

	// 2. If not found, try reverse route map (localized → canonical)
	if (!template) {
		const resolved_canonical = resolve_canonical_from_localized(canonical, lang);
		if (resolved_canonical) {
			template = canonical_to_template.get(resolved_canonical);
		}
	}

	if (template) {
		if (template.endsWith(".ree")) return { kind: "ree", rel_path: template };
		if (template.endsWith(".md")) {
			const full_path = join(public_dir, template);
			const text = existsSync(full_path) ? readFileSync(full_path, "utf-8") : "";
			const { data: fm } = text ? parse_frontmatter(text) : { data: {} };
			const layout = resolve_md_layout(template, fm, public_dir);
			return { kind: "md", rel_path: template, layout };
		}
	}

	// 3. Try direct file paths
	const without_slash = canonical.replace(/^\//, "");
	const ree_path = join(public_dir, without_slash + ".ree");
	const md_path = join(public_dir, without_slash + ".md");

	if (existsSync(ree_path)) {
		const rel = relative(public_dir, ree_path).replace(/\\/g, "/");
		return { kind: "ree", rel_path: rel };
	}

	if (existsSync(md_path)) {
		const rel = relative(public_dir, md_path).replace(/\\/g, "/");
		const text_md = existsSync(md_path) ? readFileSync(md_path, "utf-8") : "";
		const { data: fm_md } = text_md ? parse_frontmatter(text_md) : { data: {} };
		const layout = resolve_md_layout(rel, fm_md, public_dir);
		return { kind: "md", rel_path: rel, layout };
	}

	// 3. Try index files
	const index_ree = join(public_dir, without_slash, "index.ree");
	const index_md = join(public_dir, without_slash, "index.md");

	if (existsSync(index_ree)) {
		const rel = relative(public_dir, index_ree).replace(/\\/g, "/");
		return { kind: "ree", rel_path: rel };
	}

	if (existsSync(index_md)) {
		const rel = relative(public_dir, index_md).replace(/\\/g, "/");
		const text_idx = existsSync(index_md) ? readFileSync(index_md, "utf-8") : "";
		const { data: fm_idx } = text_idx ? parse_frontmatter(text_idx) : { data: {} };
		const layout = resolve_md_layout(rel, fm_idx, public_dir);
		return { kind: "md", rel_path: rel, layout };
	}

	return null;
}

// ---------------------------------------------------------------------------
// Static file resolution
// ---------------------------------------------------------------------------

function find_static_file(url_path: string): string | null {
	const cleaned = url_path.replace(/^\//, "");

	// Try public/ first
	const pub = join(public_dir, cleaned);
	if (
		existsSync(pub) &&
		!pub.endsWith(".ree") &&
		!pub.endsWith(".md") &&
		!pub.endsWith(".json") &&
		!pub.endsWith(".ts")
	) {
		return pub;
	}

	// Try static/
	const stat = join(static_dir, cleaned);
	if (existsSync(stat)) return stat;

	return null;
}

// ---------------------------------------------------------------------------
// Generated build artifacts (sitemap, feeds)
// ---------------------------------------------------------------------------
//
// Sitemap and RSS/JSON feeds are emitted to dist/ by the build scripts, not
// served from src/public/. To avoid 404s on links like /sitemap.xml in dev,
// we serve the last-built copy from dist/ as a convenience. These are stale
// until the next `bun run build:dist` (or `bun run sitemap` / `bun run rss`).
//
// robots.txt is intentionally NOT included: dev keeps serving the source
// src/public/robots.txt (Disallow: /) so the dev server stays unindexable.

function is_generated_artifact(url_path: string): boolean {
	if (url_path === "/sitemap.xml") return true;
	if (url_path.endsWith("/feed.xml") || url_path.endsWith("/feed.json")) return true;
	return false;
}

function find_dist_artifact(url_path: string): string | null {
	if (!is_generated_artifact(url_path)) return null;

	const cleaned = url_path.replace(/^\//, "");
	const candidate = join(dist_dir, cleaned);
	if (existsSync(candidate)) return candidate;

	return null;
}

// ---------------------------------------------------------------------------
// Sidebar map (same logic as build.ts)
// ---------------------------------------------------------------------------

async function build_sidebar_map(): Promise<Map<string, Map<string, { title: string; canonical_path: string }[]>>> {
	const sidebar_map = new Map<string, Map<string, { title: string; canonical_path: string }[]>>();

	for (const base_rel_path of md_files) {
		const base_name = base_rel_path.split("/").pop() ?? "";
		if (base_name !== "index.md" && !/^\d+_index\.md$/.test(base_name)) continue;

		const full = join(public_dir, base_rel_path);
		if (!existsSync(full)) continue;
		const content = readFileSync(full, "utf-8");
		const { data: frontmatter } = parse_frontmatter(content);
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
				// Resolve language-variant
				const without_ext = page_rel_path.replace(/\.md$/, "");
				const candidates = [
					`${without_ext}.${lang}.md`,
					`${without_ext}.${default_language}.md`,
					page_rel_path,
				];
				let resolved_content = "";
				for (const candidate of candidates) {
					const p = join(public_dir, candidate);
					if (existsSync(p)) {
						resolved_content = readFileSync(p, "utf-8");
						break;
					}
				}
				if (!resolved_content) continue;

				const { data: page_fm, body } = parse_frontmatter(resolved_content);
				const skip_nav = page_fm["skip-navigation"];
				if (skip_nav === true || skip_nav === "true" || skip_nav === "yes") continue;

				const title =
					typeof page_fm.title === "string" && page_fm.title.trim()
						? page_fm.title.trim()
						: (body.match(/^#\s+(.+)$/m)?.[1]?.trim() ?? "Untitled");

				const canonical_path_link = template_to_canonical(page_rel_path);
				links.push({ title, canonical_path: canonical_path_link });
			}
			per_lang_sidebar.set(lang, links);
		}
		sidebar_map.set(folder_path, per_lang_sidebar);
	}

	return sidebar_map;
}

const sidebar_map = await build_sidebar_map();

// ---------------------------------------------------------------------------
// Request handler
// ---------------------------------------------------------------------------

function respond_html(body: string, status = 200): Response {
	return new Response(body, {
		status,
		headers: {
			"Content-Type": "text/html; charset=utf-8",
			"Content-Disposition": "inline",
			"Cache-Control": "no-cache, no-store, must-revalidate",
		},
	});
}

function respond_file(full_path: string): Response {
	const file = Bun.file(full_path);
	return new Response(file, {
		headers: {
			"Content-Type": mime_type(full_path),
			"Content-Disposition": "inline",
			"Cache-Control": "no-cache",
		},
	});
}

function respond_not_found(): Response {
	return respond_html("<h1>404 Not Found</h1>", 404);
}

function respond_error(msg: string): Response {
	return respond_html(`<h1>500 Error</h1><pre>${msg}</pre>`, 500);
}

// ---------------------------------------------------------------------------
// Pagination (mirrors the pagination phase in scripts/build.ts)
// ---------------------------------------------------------------------------

/**
 * Match a request path to a registered pagination route + page number.
 *   /blog          → page 1
 *   /blog/2        → page 2
 * `path` is already language-stripped and trailing-slash-normalized.
 */
function match_pagination(path: string, lang: string): { route: PaginationRoute; page: number; canonical_base: string } | null {
	if (!pagination_config.enabled) return null;

	const seg = pagination_config.path_segment;

	for (const route of pagination_config.routes) {
		const route_dir = route.route.replace(/^\/+|\/+$/g, "");
		const canonical_base = "/" + route_dir;
		const localized_base = resolve_localized_path(canonical_base, lang);

		// page 1: the route index itself
		if (path === localized_base) return { route, page: 1, canonical_base };

		// page ≥ 2: <localized_base>/<segment>/<n>, or <localized_base>/<n> when
		// the segment is empty. A non-numeric tail (a real post slug) won't match.
		const prefix = seg ? `${localized_base}/${seg}/` : `${localized_base}/`;
		if (path.startsWith(prefix)) {
			const rest = path.slice(prefix.length);
			if (/^\d+$/.test(rest)) return { route, page: parseInt(rest, 10), canonical_base };
		}
	}

	return null;
}

async function render_pagination(
	match: { route: PaginationRoute; page: number; canonical_base: string },
	lang: string,
	year: number,
): Promise<Response> {
	const { route, page, canonical_base } = match;
	const route_dir = route.route.replace(/^\/+|\/+$/g, "");
	const index_rel = `${route_dir}/index.ree`;

	if (!existsSync(join(public_dir, index_rel))) return respond_not_found();

	// per_page precedence: literal `per-page="N"` on the component > route > global.
	const index_source = readFileSync(join(public_dir, index_rel), "utf-8");
	const per_page = read_per_page_override(index_source) ?? route.per_page ?? pagination_config.per_page;

	const records = await resolve_route_records(public_dir, route, lang, all_page_files);
	const last_page = chunk_count(records.length, per_page);

	if (page < 1 || page > last_page) return respond_not_found();

	const seg = pagination_config.path_segment;
	const page_url = (n: number) => {
		const base = localized_url_for_lang(canonical_base, lang).replace(/\/+$/, "");
		if (n <= 1) return `${base}/`;
		return seg ? `${base}/${seg}/${n}/` : `${base}/${n}/`;
	};

	const namespace = path_to_namespace(index_rel);
	const global_strings = translations[lang]?.routes ?? {};
	const route_strings = namespace ? get_nested(translations[lang], namespace) : {};
	const merged = deep_merge(structuredClone(global_strings), route_strings);
	const labels = pagination_labels((global_strings as any).ui?.pagination);

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

	const is_default = lang === default_language;
	const lang_url_prefix = is_default ? "" : `/${lang}`;

	const data: Record<string, any> = {
		lang,
		lang_url_prefix,
		locale: language_locales[lang],
		active_languages,
		language_names,
		language_self_names,
		default_language,
		base_url: "/",
		site_url: "",
		hreflang_links: [],
		site_name: "Dev",
		year,
		is_dev: true,
		rendered_at: new Date().toISOString(),
		request_url: page_url(page),
		canonical_path: canonical_base,
		language_urls,
		records: page_records,
		pagination: pagination_data,
		pagination_variant: pagination_config.variant,
		...merged,
	};

	data.helpers = create_template_helpers(data, project_helper_functions);
	data.helpers.localized_path = (p: string) => localized_url_for_lang(p, lang);
	data.helpers.localized_path_for_lang = (target: string, p: string) => localized_url_for_lang(p, target);

	try {
		const html = await engine.render(`${route_dir}/index`, data);
		return respond_html(inject_live_reload(html));
	} catch (err) {
		const msg = err instanceof Error ? err.message : String(err);
		console.error(`   ✗ ${lang}/${index_rel} page ${page}: ${msg}`);
		return respond_error(msg);
	}
}

// ---------------------------------------------------------------------------
// Server
// ---------------------------------------------------------------------------

const server = Bun.serve({
	port,
	fetch: async (req: Request): Promise<Response> => {
		const url = new URL(req.url);
		const pathname = url.pathname;

		// ── WebSocket live reload ────────────────────────────
		if (pathname === "/__reload") {
			if (server.upgrade(req)) return;
			return new Response("Expected WebSocket upgrade", { status: 426 });
		}

		// ── Resolve language and canonical path ──────────────
		const { lang, path: canonical } = resolve_request(pathname);

		// ── Pagination routes (page 1 and /<n>) take precedence ──
		if (lang) {
			const pmatch = match_pagination(canonical, lang);
			if (pmatch) {
				return await render_pagination(pmatch, lang, new Date().getFullYear());
			}
		}

		// ── Try template / markdown resolution ───────────────
		if (lang) {
			const resolved = resolve_template(canonical, lang);
			if (resolved) {
				const year = new Date().getFullYear();

				if (resolved.kind === "ree") {
					// ── Render .ree template ────────────────────
					try {
						const template_name = resolved.rel_path.replace(/\.ree$/, "");
						const namespace = path_to_namespace(resolved.rel_path);
						const global_strings = translations[lang]?.routes ?? {};
						const route_strings = namespace ? get_nested(translations[lang], namespace) : {};

						const merged = deep_merge(structuredClone(global_strings), route_strings);
						const canonical_path = template_to_canonical(resolved.rel_path);
						const localized_path = resolve_localized_path(canonical_path, lang);
						const is_default = lang === default_language;
						const lang_url_prefix = is_default ? "" : `/${lang}`;
						const request_url =
							localized_path === "/" ? lang_url_prefix + "/" : lang_url_prefix + localized_path + "/";

						// Load data from .ts file
						const template_data = await load_template_data(resolved.rel_path);

						const data: Record<string, any> = {
							lang,
							lang_url_prefix,
							locale: language_locales[lang],
							active_languages,
							language_names,
							language_self_names,
							default_language,
							base_url: "/",
							site_url: "",
							hreflang_links: [],
							site_name: "Dev",
							year,
							is_dev: true,
							rendered_at: new Date().toISOString(),
							request_url,
							canonical_path,
							language_urls,
							reepolee_version,
							reeweb_version,
							tailwind_version,
							...template_data,
							...merged,
						};

						data.helpers = create_template_helpers(data, project_helper_functions);
						data.helpers.localized_path = (p: string) => localized_url_for_lang(p, lang);
						data.helpers.localized_path_for_lang = (target: string, p: string) =>
							localized_url_for_lang(p, target);

						const html = await engine.render(template_name, data);
						return respond_html(inject_live_reload(html));
					} catch (err) {
						const msg = err instanceof Error ? err.message : String(err);
						console.error(`   ✗ ${lang}/${resolved.rel_path}: ${msg}`);
						return respond_error(msg);
					}
				}

				if (resolved.kind === "md") {
					// ── Render .md file ──────────────────────────
					try {
						// Resolve language-variant markdown file
						const without_ext = resolved.rel_path.replace(/\.md$/, "");
						const candidates = [
							`${without_ext}.${lang}.md`,
							`${without_ext}.${default_language}.md`,
							resolved.rel_path,
						];
						let md_content = "";
						let md_rel_path = "";
						for (const c of candidates) {
							const p = join(public_dir, c);
							if (existsSync(p)) {
								md_content = readFileSync(p, "utf-8");
								md_rel_path = c;
								break;
							}
						}
						if (!md_content) return respond_not_found();

						const { data: frontmatter, body: markdown_body } = parse_frontmatter(md_content);
						const canonical_path = template_to_canonical(md_rel_path);

						const raw_html = Bun.markdown.html(markdown_body, {
							tables: true,
							strikethrough: true,
							tasklists: true,
							autolinks: { url: true, www: true, email: true },
							headings: { ids: true },
						});

						const { html: html_body, headings: all_headings } = process_docs_markdown(
							raw_html,
							markdown_styles,
						);
						const toc_headings = all_headings.filter((h) => h.level > 1);
						const first_section = toc_headings[0]?.id ?? "";

						const localized_path = resolve_localized_path(canonical_path, lang);
						const is_default = lang === default_language;
						const lang_url_prefix = is_default ? "" : `/${lang}`;
						const request_url =
							localized_path === "/" ? lang_url_prefix + "/" : lang_url_prefix + localized_path + "/";

						const global_strings = translations[lang]?.routes ?? {};

						// Build sidebar links
						let sidebar_links: { title: string; url: string; active: boolean }[] | undefined;
						for (const [folder_path, per_lang] of sidebar_map) {
							const folder_canonical = "/" + folder_path;
							if (
								canonical_path === folder_canonical ||
								canonical_path.startsWith(folder_canonical + "/")
							) {
								sidebar_links = (per_lang.get(lang) ?? []).map((link) => ({
									title: link.title,
									url: localized_url_for_lang(link.canonical_path, lang),
									active: link.canonical_path === canonical_path,
								}));
								break;
							}
						}

						// Auto-detect docs pages via the docs-site registry (see config/docs_sites.ts)
						const docs_site = find_docs_site(canonical_path);
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

						const page_title = (frontmatter.title as string) || frontmatter.site_name || "Dev";
						const heading = (frontmatter.heading as string) || page_title;
						const body_markup =
							frontmatter.layout === "coming-soon"
								? coming_soon_body(heading)
								: `<article class="article-body mx-auto max-w-3xl">${html_body}</article>`;

						const data: Record<string, any> = {
							lang,
							lang_url_prefix,
							locale: language_locales[lang],
							active_languages,
							language_names,
							language_self_names,
							default_language,
							base_url: "/",
							site_url: "",
							hreflang_links: [],
							site_name: frontmatter.site_name || "Dev",
							page_title,
							year,
							is_dev: true,
							rendered_at: new Date().toISOString(),
							request_url,
							canonical_path,
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
						data.helpers.localized_path = (p: string) => localized_url_for_lang(p, lang);
						data.helpers.localized_path_for_lang = (target: string, p: string) =>
							localized_url_for_lang(p, target);

						const html = await engine.render(resolved.layout, data);
						return respond_html(inject_live_reload(html));
					} catch (err) {
						const msg = err instanceof Error ? err.message : String(err);
						console.error(`   ✗ ${lang}/${resolved.rel_path}: ${msg}`);
						return respond_error(msg);
					}
				}
			}
		}

		// ── Try static file ──────────────────────────────────
		const static_file = find_static_file(pathname);
		if (static_file) {
			return respond_file(static_file);
		}

		// ── Try generated build artifacts from dist/ (sitemap, feeds) ──
		const dist_artifact = find_dist_artifact(pathname);
		if (dist_artifact) {
			return respond_file(dist_artifact);
		}
		if (is_generated_artifact(pathname)) {
			// Known artifact, but not built yet — guide the developer.
			const hint =
				`<h1>404 Not Found</h1><p><code>${pathname}</code> is a generated artifact. ` +
				`Run <code>bun run build:dist</code> (or <code>bun run sitemap</code> / ` +
				`<code>bun run rss</code>) to produce it in <code>dist/</code>.</p>`;
			return respond_html(hint, 404);
		}

		return respond_not_found();
	},
	websocket: {
		open(ws: WebSocket) {
			reload_clients.add(ws);
		},
		close(ws: WebSocket) {
			reload_clients.delete(ws);
		},
		message() {},
	},
});

// ---------------------------------------------------------------------------
// Helpers (inlined from build.ts)
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Startup complete
// ---------------------------------------------------------------------------

console.log(`\n  🖥️  Dev server ready at \x1b[32mhttp://localhost:${port}/\x1b[0m`);
console.log(`  📂 Source: ${public_dir}`);
console.log(
	`  🌐 Languages: ${active_languages.map((l) => `${l}${l === default_language ? " (default)" : ""}`).join(", ")}`,
);
console.log(`  🔄 Live reload: active`);
console.log("");
