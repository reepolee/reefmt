/**
 * Reeweb docs sidebar: ordered list of top-level folders that should appear
 * in the navigation. Array order = display order.
 *
 * `slug` must match the folder name under `public/`.
 * `title` is the human label shown in the sidebar.
 */

export const reeweb_docs_sidebar: { slug: string; title: string; prepend?: { title: string; url: string }[] }[] = [
	{ slug: "prologue", title: "Releases" },
	{
		slug: "getting-started",
		title: "Getting Started",
		prepend: [{ title: "Introduction", url: "/reeweb/docs" }],
	},
	{ slug: "ree-templates", title: "Ree Templates" },
	{ slug: "styling", title: "Styling" },
	{ slug: "i18n", title: "Internationalisation" },
	{ slug: "deployment", title: "Deployment" },
	{ slug: "reference", title: "Reference" },
	{ slug: "recipes", title: "Recipes" },
	{ slug: "community", title: "Community" },
];
