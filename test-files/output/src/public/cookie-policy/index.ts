/**
 * Renders the Cookie Policy markdown (per language) to HTML at build time.
 */
import { join } from "path";

async function render_md(rel_path: string): Promise<string> {
	const full_path = join(process.cwd(), "src/content", rel_path);
	const text = await Bun.file(full_path).text();
	return Bun.markdown.html(text, {
		tables: true,
		strikethrough: true,
		autolinks: { url: true, www: true, email: true },
		headings: { ids: true },
	});
}

export async function load_template_data(): Promise<Record<string, any>> {
	return {
		policy_html: {
			en: await render_md("en/cookie-policy.md"),
			sl: await render_md("sl/cookie-policy.md"),
		},
	};
}
