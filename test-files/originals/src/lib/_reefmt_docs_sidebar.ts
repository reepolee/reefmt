/**
 * reefmt docs sidebar: ordered list of top-level folders that should appear
 * in the navigation. Array order = display order.
 *
 * `slug` must match the folder name under `public/reefmt/docs/`.
 * `title` is the human label shown in the sidebar.
 */

export const reefmt_docs_sidebar: { slug: string; title: string; prepend?: { title: string; url: string }[] }[] = [
	{
		slug: "getting-started",
		title: "Getting Started",
		prepend: [{ title: "Introduction", url: "/reefmt/docs" }],
	},
	{ slug: "usage", title: "Usage" },
	{ slug: "configuration", title: "Configuration" },
	{ slug: "editor-integration", title: "Editor Integration" },
	{ slug: "contributing", title: "Contributing" },
];
