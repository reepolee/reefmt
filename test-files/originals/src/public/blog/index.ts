/**
 * Build-time records for the Blog listing.
 *
 * Frontmatter is the single source of truth: each post lives at
 * `blog/<slug>/index.md` and carries its own `title`, `excerpt`, `image`,
 * `published_at` and `track`. We collect them with the shared
 * `collect_records()` helper (which already excludes the listing index and
 * keeps nested folder-per-post files) and decorate each record with the
 * presentational fields the card needs.
 *
 * Consumed by the pagination phase in scripts/build.ts / scripts/dev.ts via the
 * `load_records(lang)` contract (see config/pagination.ts). No hardcoded list.
 */

import { existsSync, readdirSync } from "fs";
import { join, resolve } from "path";

import { collect_records } from "$lib/collect_records";
import { read_frontmatter } from "$lib/static_site";

export type BlogCard = {
	slug: string;
	canonical_path: string;
	title: string;
	excerpt: string;
	image: string;
	track: string;
	date: string;
};

export async function load_records(lang: string): Promise<BlogCard[]> {
	const blog_dir = import.meta.dir; // …/src/public/blog
	const public_dir = resolve(blog_dir, ".."); // …/src/public

	// Folder-per-post markdown files, relative to the public dir.
	const page_files = readdirSync(blog_dir, { withFileTypes: true })
		.filter((d) => d.isDirectory() && existsSync(join(blog_dir, d.name, "index.md")))
		.map((d) => `blog/${d.name}/index.md`);

	const records = collect_records(public_dir, "blog", lang, page_files, "date_desc");

	return records.map((r) => {
		const slug = r.canonical_path.replace(/^\/blog\//, "");
		const fm = read_frontmatter(join(public_dir, r.rel_path));
		const excerpt = typeof fm.excerpt === "string" && fm.excerpt.trim() ? fm.excerpt.trim() : r.description;

		return {
			slug,
			canonical_path: r.canonical_path,
			title: r.title,
			excerpt,
			image: typeof fm.image === "string" ? fm.image : "",
			track: typeof fm.track === "string" ? fm.track : "",
			date: r.published_at.toISOString().slice(0, 10),
		};
	});
}
