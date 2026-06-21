import { existsSync, readdirSync, readFileSync } from "fs";
import { join } from "path";

import { parse_frontmatter, template_to_canonical } from "$lib/static_site";
/**
 * Data loader for the /reepolee/docs index page.
 * Computes sidebar groups by discovering markdown files in each section folder,
 * and provides the docs homepage content.
 */
import { reepolee_docs_sidebar } from "$root/src/lib/_reepolee_docs_sidebar";

export async function load_template_data(): Promise<Record<string, any>> {
	const public_dir = join(import.meta.dir, "..", "..", "..", "public");
	const docs_dir = join(public_dir, "reepolee", "docs");

	const sidebar_groups = build_sidebar_groups(docs_dir);
	const canonical_path = "/reepolee/docs";

	return {
		title: "Reepolee documentation",
		docs_sidebar_groups: sidebar_groups,
		canonical_path,
		active_page: canonical_path,
	};
}

/**
 * Build sidebar groups by scanning each section folder for .md files.
 */

type SidebarLink = { title: string; url: string };

type SidebarGroup = {
	slug: string;
	title: string;
	is_active: boolean;
	links: SidebarLink[];
};

function build_sidebar_groups(docs_dir: string): SidebarGroup[] {
	const groups: SidebarGroup[] = [];

	for (const section of reepolee_docs_sidebar) {
		const folder_path = join(docs_dir, section.slug);
		const links: SidebarLink[] = [];

		// Prepend links first
		if (section.prepend) {
			for (const item of section.prepend) {
				links.push({ title: item.title, url: item.url });
			}
		}

		// Discover .md files in this section folder (respecting ordering prefix)
		if (existsSync(folder_path)) {
			const entries = readdirSync(folder_path)
				.filter((f) => f.endsWith(".md"))
				.sort();

			for (const entry of entries) {
				const full_path = join(folder_path, entry);
				const content = readFileSync(full_path, "utf-8");
				const { data } = parse_frontmatter(content);
				const title =
					(data.title as string) || entry.replace(/^\d+_/, "").replace(/\.md$/, "").replace(/-/g, " ");

				const canonical = template_to_canonical(`reepolee/docs/${section.slug}/${entry}`);
				links.push({ title, url: canonical });
			}
		}

		// Start "Getting Started" expanded on the index page so users immediately
		// see the sidebar is interactive (it includes the active "Introduction" link).
		const is_active = section.slug === "getting-started";

		groups.push({
			slug: section.slug,
			title: section.title,
			is_active,
			links,
		});
	}

	return groups;
}
