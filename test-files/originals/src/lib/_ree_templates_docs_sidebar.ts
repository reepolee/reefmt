/**
 * ree-templates (VSCode extension) docs sidebar: ordered list of top-level
 * folders that should appear in the navigation. Array order = display order.
 *
 * `slug` must match the folder name under `public/ree-templates-vscode/docs/`.
 * `title` is the human label shown in the sidebar.
 */

export const ree_templates_docs_sidebar: { slug: string; title: string; prepend?: { title: string; url: string }[] }[] =
	[
		{
			slug: "getting-started",
			title: "Getting Started",
			prepend: [{ title: "Introduction", url: "/ree-templates-vscode/docs" }],
		},
		{ slug: "features", title: "Features" },
		{ slug: "configuration", title: "Configuration" },
	];
