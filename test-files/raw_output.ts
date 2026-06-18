import { randomInt } from "node:crypto";

import { is_enabled } from "$lib/feature_flags";
import { translated_from_request } from "$lib/helpers";
import { render } from "$lib/render";
import { create_ctx } from "$lib/request_context";
import type { BunRequest } from "bun";

export async function about_page(req: BunRequest): Promise<Response> {
				const [ctx, translated] = await Promise.all([create_ctx(req, import.meta.dir), translated_from_request(req, import.meta.dir)]);
				
				// Use feature flag to show demo text to ~33% of users (deterministic per user)
				// Seed the flag via server.ts startup
				let user_id = ctx.user?.id ? String(ctx.user.id) : req.headers.get("x-forwarded-for") || req.headers.get("x-real-ip") || "anonymous";
				
				// override it for test
				user_id = randomInt(99999).toString();
				const show_demo_text = await is_enabled("show_about_demo_text", user_id);
				
				const toasts = [
								{
												key: "1",
												value: {
																id: "1",
																message: "Saved successfully",
																type: "green",
																duration: 1000
												}
								},
								{
												key: "2",
												value: {
																id: "2",
																message: "Low <b>disk</b> space",
																type: "yellow",
																duration: 8000
												}
								},
								{
												key: "3",
												value: {
																id: "3",
																message: "Upload failed",
																type: "red",
																duration: 3000
												}
								}
				];
				
				const slovenski_html = html`
		<p>
			<strong>reepolee.com</strong> je spletna stran, ki se predstavlja kot platforma za
			<em>IT svetovanje in digitalno transformacijo</em>.
		</p>

		<p>
			Reepolee ponuja storitve na področju razvoja programske opreme, optimizacije tehnoloških sistemov in pomoči
			podjetjem pri digitalizaciji.
		</p>

		<h2>Osnovne značilnosti</h2>

		<ul>
			<li>Fokus na digitalno transformacijo podjetij</li>
			<li>IT svetovanje in razvoj programske opreme</li>
			<li>Uporaba sodobnih spletnih tehnologij</li>
		</ul>
	`;
				
				return render("about", {
								ctx,
								data: {
												...translated,
												toasts,
												slovenski_html,
												show_demo_text
								}
				});
}
