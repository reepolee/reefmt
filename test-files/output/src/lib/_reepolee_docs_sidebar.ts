/**
 * Docs sidebar: ordered list of top-level folders that should appear in the
 * navigation. Array order = display order. Folders not listed here are silently
 * skipped from the sidebar.
 *
 * `slug` must match the folder name under `public/`.
 * `title` is the human label shown in the sidebar.
 */

export type SidebarSection = {
	slug: string;
	title: string;
	/** Fixed links prepended before the section's auto-discovered markdown pages. */
	prepend?: { title: string; url: string }[];
};

export const reepolee_docs_sidebar: SidebarSection[] = [
	{ slug: "prologue", title: "Releases" },
	{
		slug: "getting-started",
		title: "Getting Started",
		prepend: [{ title: "Introduction", url: "/reepolee/docs" }],
	},
	{ slug: "the-basics", title: "The Basics" },
	{ slug: "ree-templates", title: "Ree Templates" },
	{ slug: "forms", title: "Forms" },
	{ slug: "database", title: "Database" },
	{ slug: "security", title: "Security" },
	{ slug: "client-side", title: "Client-Side" },
	{ slug: "styling", title: "Styling" },
	{ slug: "i18n", title: "Internationalization" },
	{ slug: "email", title: "Email" },
	{ slug: "deployment", title: "Deployment" },
	{ slug: "recipes", title: "Recipes" },
	{ slug: "community", title: "Community" },
];
