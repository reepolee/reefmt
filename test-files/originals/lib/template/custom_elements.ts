/**
 * Custom element pre-processor — extracted from TemplateEngine.compile().
 *
 * Handles three pre-processing steps that run before the main compiler pass:
 * 1. HTML comment stripping:  <!-- ... --> is removed so directives inside comments are NOT compiled
 * 2. Custom HTML element shorthand:  <tag-name attr="val">SLOT</tag-name> → \u0000R\u0000 marker (resolved by compile_to_code)
 * 3. Spread shorthand:  ...identifier → {~ key_values(identifier)}
 */

import { existsSync } from "node:fs";
import { dirname, join } from "node:path";

import type { CompiledFn } from "./types";

/**
 * Parse HTML attributes string into a JS object literal fragment.
 * Extracted from the inner function in TemplateEngine.compile().
 */
export function parse_attributes(attrStr: string): string {
	if (!attrStr?.trim()) return "";
	const parts: string[] = [];
	const attrRegex = /([a-zA-Z_][a-zA-Z0-9_-]*)(?:\s*=\s*(?:"([^"]*)"|'([^']*)'))?/g;
	let m: RegExpExecArray | null;
	while ((m = attrRegex.exec(attrStr)) !== null) {
		const name = m[1];
		const value = m[2] !== undefined ? m[2] : m[3] !== undefined ? m[3] : true;
		if (value === true) {
			parts.push(`"${name}": true`);
		} else if (typeof value === "string" && /^\{[=~]\s+/.test(value) && value.endsWith("}")) {
			// Template expression inside attr — strip {= } / {~ } and emit as raw JS expression
			// e.g. title="{= ui.reset_btn }" → "title": ui.reset_btn (evaluated at render time)
			const expr = value.slice(2, -1).trim();
			parts.push(`"${name}": ${expr}`);
		} else {
			parts.push(`"${name}": ${JSON.stringify(value)}`);
		}
	}
	return parts.join(", ");
}

/**
 * Result of the custom element preprocessing step.
 */
export type PreprocessResult = {
	/** The template string with custom elements converted and comments stripped. */
	template: string;
	/** Compiled slot functions for each custom element found. */
	slotFns: CompiledFn[];
};

/**
 * Pre-process a template string:
 * 1. Strip HTML comments
 * 2. Convert custom HTML elements (<tag-name>) to NUL-bounded \u0000R\u0000 markers (resolved in compile_to_code)
 * 3. Convert spread shorthand (...identifier → {~ key_values(identifier)})
 *
 * @param template    - Raw template string
 * @param viewsDir    - Absolute path to the views directory
 * @param ext         - Template file extension (e.g. ".ree")
 * @param compileSlot - Function to recursively compile slot content
 * @returns The preprocessed template and any compiled slot functions
 */
export function preprocess_template(
	template: string,
	viewsDir: string,
	ext: string,
	compileSlot: (content: string) => CompiledFn,
): PreprocessResult {
	// ── Step 1: Strip HTML comments ──
	// Remove <!-- ... --> before any directive processing, so that
	// {= }, {~ }, {#if}, etc. inside comments are NOT evaluated.
	// This allows generators to emit commented-out CU fields without
	// crashing on missing columns/fields at render time.
	template = template.replace(/<!--[\s\S]*?-->/g, "");

	const slotFns: CompiledFn[] = [];

	// ── Step 2: Process custom HTML elements ──
	// <tag-name attr1="val1">SLOT</tag-name>
	//     → \u0000R\u0000<tag-name>\u0000<slotId>\u0000<propsObj>\u0000
	//         (resolved by compile_to_code into a __rtInclude call)
	//
	// If the tag has a matching component file under components/, it becomes a
	// component call. If not, it's passed through as a native HTML element.
	const custElemRegex = /<([a-zA-Z][a-zA-Z0-9]*-[a-zA-Z0-9-]*)(?:\s([^>]*?))?\s*>([\s\S]*?)<\/\1>/g;
	let processedTemplate = template;

	while (true) {
		custElemRegex.lastIndex = 0;
		const match = custElemRegex.exec(processedTemplate);
		if (!match) break;

		const tagName = match[1];
		const attrStr = match[2] ?? "";
		const slotContent = match[3];

		// Check if a matching component file exists under components/
		const projectRoot = dirname(viewsDir);
		const componentFilePath = join(projectRoot, "components", tagName + ext);
		const componentExists = existsSync(componentFilePath);

		if (componentExists) {
			// Component found → emit a NUL-bounded ReeTag marker that
			// compile_to_code resolves to a direct __rtInclude call. We use a
			// NUL marker instead of {#include(...)} because the directive
			// regex can't parse balanced-brace data expressions.
			// Format: \u0000R\u0000<tagName>\u0000<slotId>\u0000<propsObj>\u0000
			const slotId = slotFns.length;

			// Recursively compile the slot content as a standalone template
			const slotCompiledFn = compileSlot(slotContent);
			slotFns.push(slotCompiledFn);

			const attrs = parse_attributes(attrStr);
			const childrenExpr = `children: await __run_slot(${slotId}, props, __escape, __include, __rtInclude, __currentName)`;
			const propsObj = attrs ? `{${childrenExpr}, attributes: {${attrs}}}` : `{${childrenExpr}}`;

			const marker = `\u0000R\u0000${tagName}\u0000${slotId}\u0000${propsObj}\u0000`;
			processedTemplate =
				processedTemplate.slice(0, match.index) +
				marker +
				processedTemplate.slice(match.index + match[0].length);
		} else {
			// No component file — pass through as a native HTML element.
			const tagNameJson = JSON.stringify(tagName);
			const attrStrJson = attrStr ? JSON.stringify(` ${attrStr}`) : JSON.stringify("");
			const replacement = `{{ __output += "<" + ${tagNameJson} + ${attrStrJson} + ">"; }}${slotContent}{{ __output += "</" + ${tagNameJson} + ">"; }}`;
			processedTemplate =
				processedTemplate.slice(0, match.index) +
				replacement +
				processedTemplate.slice(match.index + match[0].length);
		}
	}

	// ── Step 3: Pre-process spread shorthand ──
	// Convert bare ...identifier → {~ key_values(identifier)}
	// Must NOT affect {{ }} (raw JS) blocks
	processedTemplate = processedTemplate.replace(/\{\{[\s\S]*?\}\}|\.\.\.([A-Za-z_$][\w$]*)/g, (match, identifier) => {
		if (identifier !== undefined) {
			return `{~ key_values(${identifier})}`;
		}
		return match; // Preserve {{ }} blocks unchanged
	});

	return { template: processedTemplate, slotFns };
}
