/**
 * sqlfmt docs sidebar: ordered list of top-level folders that should appear
 * in the navigation. Array order = display order.
 *
 * `slug` must match the folder name under `public/sqlfmt/docs/`.
 * `title` is the human label shown in the sidebar.
 */

export const sqlfmt_docs_sidebar: { slug: string; title: string; prepend?: { title: string; url: string }[] }[] = [
	{
		slug: "getting-started",
		title: "Getting Started",
		prepend: [{ title: "Introduction", url: "/sqlfmt/docs" }],
	},
	{ slug: "usage", title: "Usage" },
	{ slug: "formatting", title: "Formatting" },
	{ slug: "contributing", title: "Contributing" },
];
