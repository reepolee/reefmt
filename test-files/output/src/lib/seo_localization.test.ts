import { describe, expect, test } from "bun:test";

import { path_is_localized } from "./seo_localization";

describe("path_is_localized", () => {
	test("marketing pages are localized", () => {
		for (const p of ["/", "/about", "/projects", "/enterprise", "/o-nas"]) {
			expect(path_is_localized(p)).toBe(true);
		}
	});

	test("the blog tree is not localized", () => {
		expect(path_is_localized("/blog")).toBe(false);
		expect(path_is_localized("/blog/fail-loudly-env-vars")).toBe(false);
	});

	test("every product's docs tree is not localized", () => {
		for (const p of ["/reepolee/docs", "/reeweb/docs/getting-started", "/sqlfmt/docs", "/reefmt/docs/x/y"]) {
			expect(path_is_localized(p)).toBe(false);
		}
	});

	test("does not over-match: /docs only counts under a product segment", () => {
		// A top-level "/docs" has no product segment, so it stays localized;
		// "/blogging" must not be caught by the /blog rule.
		expect(path_is_localized("/docs")).toBe(true);
		expect(path_is_localized("/blogging")).toBe(true);
	});
});
