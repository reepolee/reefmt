/**
 * Registry of documentation sites hosted on reepolee.com.
 *
 * Each entry maps a URL prefix to the docs folder under `src/public/` and the
 * sidebar config used to build that site's navigation. Both the static builder
 * (`scripts/build.ts`) and the dev server (`scripts/dev.ts`) look a page's
 * canonical path up here to decide whether it is a docs page and, if so, which
 * sidebar to render.
 *
 * Adding a new docs site (e.g. a future LSP) is a one-line entry here plus the
 * matching `src/public/<tool>/docs/` folder and sidebar config.
 */

import { ree_templates_docs_sidebar } from "$root/src/lib/_ree_templates_docs_sidebar";
import { reefmt_docs_sidebar } from "$root/src/lib/_reefmt_docs_sidebar";
import { reemerge_docs_sidebar } from "$root/src/lib/_reemerge_docs_sidebar";
import { reepolee_docs_sidebar } from "$root/src/lib/_reepolee_docs_sidebar";
import { reeweb_docs_sidebar } from "$root/src/lib/_reeweb_docs_sidebar";
import { sqlfmt_docs_sidebar } from "$root/src/lib/_sqlfmt_docs_sidebar";

export type DocsSite = {
	/** URL prefix that identifies pages belonging to this docs site. */
	prefix: string;
	/** Folder under `src/public/` holding the docs markdown + layout. */
	root: string;
	/** Ordered sidebar sections for this site. */
	sidebar: { slug: string; title: string; prepend?: { title: string; url: string }[] }[];
};

export const docs_sites: DocsSite[] = [
	{ prefix: "/reepolee/docs", root: "reepolee/docs", sidebar: reepolee_docs_sidebar },
	{ prefix: "/reeweb/docs", root: "reeweb/docs", sidebar: reeweb_docs_sidebar },
	{ prefix: "/reefmt/docs", root: "reefmt/docs", sidebar: reefmt_docs_sidebar },
	{ prefix: "/reemerge/docs", root: "reemerge/docs", sidebar: reemerge_docs_sidebar },
	{ prefix: "/sqlfmt/docs", root: "sqlfmt/docs", sidebar: sqlfmt_docs_sidebar },
	{ prefix: "/ree-templates-vscode/docs", root: "ree-templates-vscode/docs", sidebar: ree_templates_docs_sidebar },
];

/** Find the docs site (if any) that a canonical path belongs to. */
export function find_docs_site(canonical_path: string): DocsSite | undefined {
	return docs_sites.find((site) => canonical_path.startsWith(site.prefix));
}
