/**
 * ── Docs figure placeholders ─────────────────────────────────
 *
 * Expands `<media-frame label="…" ratio="…">` tokens authored inside markdown
 * docs into the same labelled placeholder card the homepage uses
 * (`src/components/media-frame.ree`). The card is a styled aspect-ratio box with
 * an offset brand-accent frame and a centred caption marking which asset belongs
 * there — a visible "screenshot goes here" slot we can capture against later.
 *
 * Why a build-time expansion rather than the `.ree` component? Markdown bodies
 * are rendered with `Bun.markdown.html()` and injected raw into the layout — the
 * template engine only renders the surrounding layout, never the markdown body,
 * so a `<media-frame>` written in a `.md` file would never be expanded. This
 * runs in `scripts/build.ts` right after `process_docs_markdown()`.
 *
 * Authoring (in any `.md` doc):
 *
 *   <media-frame label="Screenshot — generated list view" ratio="16/9"></media-frame>
 *
 * `label` is the caption shown inside the frame; `ratio` is any CSS aspect-ratio
 * (default "16/9"). Every class used below is also used by `media-frame.ree`, so
 * Tailwind already emits them — markdown files are not scanned for classes.
 */

/** The placeholder-card markup, mirroring `src/components/media-frame.ree`. */
export function media_frame_markup(label: string, ratio = "16/9"): string {
	const caption = label ? `<div class="text-muted font-mono text-xs tracking-wide uppercase">${label}</div>` : "";
	return (
		`<div class="relative mt-2 mb-8">` +
		`<div class="absolute -right-4 -bottom-4 hidden h-2/3 w-2/3 rounded-2xl border border-brand/40 lg:block"></div>` +
		`<div class="bg-warm border-divider relative flex items-center justify-center overflow-hidden rounded-2xl border shadow-sm" style="aspect-ratio: ${ratio};">` +
		`<div class="flex flex-col items-center gap-3 p-6 text-center">` +
		`<div class="text-brand/60 h-10 w-10"><img src="/ree-file.svg" alt="" class="h-full w-full opacity-50" /></div>` +
		caption +
		`</div></div></div>`
	);
}

const ATTR_RE = (name: string) => new RegExp(`${name}="([^"]*)"`);

/**
 * Replace every `<media-frame …></media-frame>` token (optionally wrapped in the
 * `<p>` that the markdown renderer puts around a lone inline element) with the
 * placeholder card.
 */
export function expand_doc_figures(html: string): string {
	const FRAME_RE = /(?:<p[^>]*>\s*)?<media-frame\b([^>]*)><\/media-frame>(?:\s*<\/p>)?/g;
	return html.replace(FRAME_RE, (_match, attrs: string) => {
		const label = (attrs.match(ATTR_RE("label")) || [, ""])[1] ?? "";
		const ratio = (attrs.match(ATTR_RE("ratio")) || [, "16/9"])[1] || "16/9";
		return media_frame_markup(label, ratio);
	});
}
