/**
 * ── Project markdown styling ─────────────────────────────────
 *
 * Tailwind class strings injected onto rendered markdown elements by
 * `process_docs_markdown()`. This is the project-specific presentation layer —
 * edit it freely to restyle docs/blog markdown.
 *
 * The generic pipeline (heading/TOC scan, syntax highlighting, external-link
 * handling) lives in `lib/markdown_docs.ts` and should NOT be modified, so it
 * stays upstream-upgradeable. Only the classes below are yours to change.
 */

import type { MarkdownStyles } from "$lib/markdown_docs";

export const markdown_styles: MarkdownStyles = {
	heading: (level) => {
		if (level === 1) return "font-display text-4xl italic mb-6 scroll-mt-30";
		if (level === 2) return "font-display text-3xl italic mt-12 mb-6 scroll-mt-30";
		return "font-semibold text-lg mt-8 mb-3 scroll-mt-30";
	},
	pre: "code-block relative rounded-xl overflow-hidden bg-code-bg border border-white/5 p-5 mb-6",
	anchor: "text-accent underline underline-offset-2 decoration-accent/40 hover:decoration-accent transition-colors",
	paragraph: "text-muted leading-relaxed mb-6",
	inline_code: "font-mono text-xs bg-warm px-1.5 py-0.5 rounded",
	blockquote: "border-l-4 border-accent pl-4 py-2 italic text-muted mb-6",
	ul: "list-disc list-inside space-y-2 mb-6 text-muted",
	ol: "list-decimal list-inside space-y-2 mb-6 text-muted",
	li: "leading-relaxed",
	table: "w-full text-sm text-left border-collapse",
	table_wrapper: "mb-6 rounded-xl border border-divider overflow-hidden",
	thead: "bg-warm",
	tbody: "divide-y divide-divider",
	th: "px-4 py-3 font-semibold text-ink align-bottom",
	td: "px-4 py-3 align-top text-muted leading-relaxed wrap-anywhere",
};
