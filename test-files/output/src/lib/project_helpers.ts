#!/usr/bin/env bun

/**
 * src/lib/project_helpers.ts
 *
 * Project-specific helpers and shared utilities for the reepolee.com static site.
 * These are custom extensions to the base reeweb static site generator.
 *
 * Contains:
 *   - Reepolee-specific template helpers (md, is_current_route), registered for
 *     templates via `project_helper_functions`. Generic helpers that exist in
 *     upstream reeweb (tw_merge, avif/webp/jpeg/srcset) live in the base
 *     lib/template_helpers.ts instead, to stay aligned with upstream.
 *   - Shared build/dev infra helpers (build_docs_sidebar_groups, resolve_md_layout,
 *     read_project_version_info, coming_soon_body, write_sitemap_files).
 */

import { existsSync, readdirSync, readFileSync } from "fs";
import { join, dirname } from "path";

import { parse_frontmatter, template_to_canonical, type FrontMatter } from "$lib/static_site";

// ===========================================================================
// Template helper functions (moved from lib/template_helpers.ts)
// ===========================================================================

// ---------------------------------------------------------------------------
// Inline markdown
// ---------------------------------------------------------------------------

/**
 * Render a short string of inline markdown to HTML.
 * Converts literal `\n` to actual newlines before rendering since .ree
 * template strings escape backslashes ("\n" in .ree → `\n` as text).
 * Supports **bold**, [text](url), <br>, inline HTML and emoji.
 */
export function md(text: string): string {
	if (!text) return "";
	// .ree template strings use double-escaped \n — convert to real newlines
	const normalized = String(text).replace(/\\\\n/g, "\n");
	return Bun.markdown.html(normalized, {
		tables: true,
		strikethrough: true,
		autolinks: { url: true, www: true, email: true },
	});
}

// ---------------------------------------------------------------------------
// Route matching
// ---------------------------------------------------------------------------

export function is_current_route(checking_url: string, current_url?: string, exact_match = true): boolean {
	if (!current_url) return false;
	return exact_match ? checking_url === current_url : current_url.includes(checking_url);
}

// ---------------------------------------------------------------------------
// Project helpers object for template injection
// ---------------------------------------------------------------------------

/**
 * Object of project-specific helper functions to merge into template helpers
 * via `create_template_helpers(data, project_helper_functions)`.
 */
export const project_helper_functions: Record<string, unknown> = {
	md,
	is_current_route,
};

// ===========================================================================
// Shared build/dev infra helpers
// ===========================================================================

// ---------------------------------------------------------------------------
// Version info
// ---------------------------------------------------------------------------

export type ProjectVersionInfo = {
	reepolee_version: string;
	reeweb_version: string;
	tailwind_version: string;
};

/**
 * Read version info from package.json — shared between dev.ts and build.ts.
 */
export async function read_project_version_info(): Promise<ProjectVersionInfo> {
	const pkg_json = JSON.parse(await Bun.file("package.json").text());
	return {
		reepolee_version: String(pkg_json.version ?? ""),
		reeweb_version: String(pkg_json.version ?? ""),
		tailwind_version: String(pkg_json.devDependencies?.tailwindcss ?? "").replace(/^[^\d]*/, ""),
	};
}

// ---------------------------------------------------------------------------
// Layout resolution for markdown files
// ---------------------------------------------------------------------------

/**
 * Resolve the layout for a markdown file.
 * Priority:
 *   1. frontmatter `layout` key
 *   2. Nearest *.layout.ree walking up the directory tree (stops before root)
 *   3. Default "layout"
 *
 * This is an enhanced version over the base reeweb resolve_layout_for_md.
 */
export function resolve_md_layout(rel_path: string, frontmatter: FrontMatter, public_dir: string): string {
	if (frontmatter.layout) {
		const base = String(frontmatter.layout)
			.replace(/\.ree$/, "")
			.replace(/\.layout$/, "");
		for (const candidate of [`${base}.layout`, base]) {
			if (existsSync(join(public_dir, `${candidate}.ree`))) return candidate;
		}
	}

	let dir = dirname(rel_path).replace(/\\\\/g, "/");
	while (dir && dir !== ".") {
		try {
			for (const entry of readdirSync(join(public_dir, dir))) {
				if (entry.endsWith(".layout.ree")) {
					return `${dir}/${entry.slice(0, -".ree".length)}`;
				}
			}
		} catch {}
		const parent = dirname(dir).replace(/\\\\/g, "/");
		if (parent === dir) break;
		dir = parent;
	}

	return "layout";
}

// ---------------------------------------------------------------------------
// Docs sidebar groups
// ---------------------------------------------------------------------------

/**
 * Build docs sidebar groups from a config array + discovered markdown files
 * in each section folder. Shared between dev.ts and build.ts.
 */
export function build_docs_sidebar_groups(
	sidebar: { slug: string; title: string; prepend?: { title: string; url: string }[] }[],
	base_folder: string,
	canonical_path: string,
	public_dir: string,
	md_files: string[],
	languages: readonly string[],
	default_language: string,
	lang: string,
): { slug: string; title: string; is_active: boolean; links: { title: string; url: string }[] }[] {
	const groups: { slug: string; title: string; is_active: boolean; links: { title: string; url: string }[] }[] = [];

	for (const section of sidebar) {
		const folder_path = `${base_folder}/${section.slug}`;
		const links: { title: string; url: string }[] = [];

		if (section.prepend) {
			for (const item of section.prepend) {
				links.push({ title: item.title, url: item.url });
			}
		}

		const prefix = `${folder_path}/`;
		const section_md_files = md_files.filter((f) => f.startsWith(prefix)).sort();

		for (const rel_path of section_md_files) {
			const full_path = join(public_dir, rel_path);
			try {
				const content = readFileSync(full_path, "utf-8");
				const { data } = parse_frontmatter(content);
				const title =
					(data.title as string) ||
					rel_path.split("/").pop()?.replace(/^\d+_/, "").replace(/\.md$/, "").replace(/-/g, " ") ||
					"";
				const file_canonical = template_to_canonical(rel_path);
				links.push({ title, url: file_canonical });
			} catch {
				// skip unresolvable files
			}
		}

		const is_active = links.some((l) => l.url === canonical_path);

		groups.push({
			slug: section.slug,
			title: section.title,
			is_active,
			links,
		});
	}

	return groups;
}

// ---------------------------------------------------------------------------
// Coming-soon layout body markup
// ---------------------------------------------------------------------------

/**
 * Generate HTML markup for a "coming-soon" page layout.
 */
export function coming_soon_body(heading: string): string {
	return `<div class="flex flex-col items-center justify-center py-24 px-6 text-center max-w-md mx-auto">
	<svg class="w-14 h-14 mb-8 text-accent/40" viewBox="0 0 270 270"><circle cx="135" cy="135" r="135" fill="currentColor"/><path d="M67 101C67 82.2223 82.2223 67 101 67H135V169C135 187.778 119.778 203 101 203H67V101Z" fill="white"/><path d="M135 67H169C187.778 67 203 82.2223 203 101V101C203 119.778 187.778 135 169 135V135C150.222 135 135 119.778 135 101V67Z" fill="white" fill-opacity="0.5"/></svg>
	<h1 class="font-display text-4xl italic mb-4">${heading}</h1>
	<p class="text-muted leading-relaxed">This page is still being written. We're actively working on the documentation — check back soon.</p>
</div>`;
}

// ---------------------------------------------------------------------------
// Dev server file output
// ---------------------------------------------------------------------------

/**
 * Write sitemap.xml and robots.txt to both dist/ and src/public/ (for dev server).
 */
export async function write_sitemap_files(
	xml: string,
	robots_txt: string,
	dist_dir: string,
	public_dir: string,
): Promise<void> {
	await Bun.write(join(dist_dir, "sitemap.xml"), xml);
	await Bun.write(join(public_dir, "sitemap.xml"), xml);
	await Bun.write(join(dist_dir, "robots.txt"), robots_txt);
	await Bun.write(join(public_dir, "robots.txt"), robots_txt);
}
