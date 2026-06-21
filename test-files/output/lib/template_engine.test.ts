import { describe, expect, test } from "bun:test";
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";

const TE = (await import("./template_engine")).default;

function make_engine(views: string) {
	return new TE({ views, cache: false, ext: ".ree" });
}

function with_temp_dir(fn: (dir: string, engine: any) => Promise<void>) {
	return async () => {
		const dir = mkdtempSync(join(tmpdir(), "reepolee-test-"));
		try {
			await fn(dir, make_engine(dir));
		} finally {
			rmSync(dir, { recursive: true, force: true });
		}
	};
}

describe("TemplateEngine", () => {
	describe("renderString — inline compilation", () => {
		test("interpolates escaped output", async () => {
			const engine = make_engine("/tmp");
			const result = await engine.renderString("Hello {= props.name }!", { name: "World" });
			expect(result).toBe("Hello World!");
		});

		test("escapes HTML in expressions", async () => {
			const engine = make_engine("/tmp");
			const result = await engine.renderString("{= '<script>alert(1)</script>' }");
			expect(result).toBe("&lt;script&gt;alert(1)&lt;/script&gt;");
		});

		test("unescaped output with {~ ... }", async () => {
			const engine = make_engine("/tmp");
			const result = await engine.renderString("{~ '<b>bold</b>' }");
			expect(result).toBe("<b>bold</b>");
		});

		test("raw JS with {{ ... }}", async () => {
			const engine = make_engine("/tmp");
			const result = await engine.renderString("{{ const x = 10; }}val: {= x }");
			expect(result).toBe("val: 10");
		});
	});

	describe("#if / {:else} / {/if}", () => {
		test("renders truthy branch", async () => {
			const engine = make_engine("/tmp");
			const tmpl = "{#if props.show }shown{:else}hidden{/if}";
			expect(await engine.renderString(tmpl, { show: true })).toBe("shown");
		});

		test("renders else branch", async () => {
			const engine = make_engine("/tmp");
			const tmpl = "{#if props.show }shown{:else}hidden{/if}";
			expect(await engine.renderString(tmpl, { show: false })).toBe("hidden");
		});

		test("renders if without else", async () => {
			const engine = make_engine("/tmp");
			const tmpl = "{#if props.show }visible{/if}";
			expect(await engine.renderString(tmpl, { show: false })).toBe("");
		});
	});

	describe("#with / {/with}", () => {
		test("resolves property access on context object", async () => {
			const engine = make_engine("/tmp");
			const tmpl = "{#with props.obj}{= name }{/with}";
			const result = await engine.renderString(tmpl, { obj: { name: "Alice" } });
			expect(result).toBe("Alice");
		});

		test("resolves nested property access", async () => {
			const engine = make_engine("/tmp");
			const tmpl = "{#with props.obj}{= user.email }{/with}";
			const result = await engine.renderString(tmpl, { obj: { user: { email: "a@b.com" } } });
			expect(result).toBe("a@b.com");
		});

		test("works with unescaped output", async () => {
			const engine = make_engine("/tmp");
			const tmpl = "{#with props.obj}{~ html }{/with}";
			const result = await engine.renderString(tmpl, { obj: { html: "<b>bold</b>" } });
			expect(result).toBe("<b>bold</b>");
		});

		test("nested #with blocks", async () => {
			const engine = make_engine("/tmp");
			// Inside the outer `with`, `b` resolves to props.a.b via the with scope chain
			const tmpl = "{#with props.a}{#with b}{= name }{/with}{/with}";
			const result = await engine.renderString(tmpl, { a: { b: { name: "Nested" } } });
			expect(result).toBe("Nested");
		});

		test("each inside with", async () => {
			const engine = make_engine("/tmp");
			const tmpl = "{#with props.obj}{#each items as item}{= item }{/each}{/with}";
			const result = await engine.renderString(tmpl, { obj: { items: ["x", "y"] } });
			expect(result).toBe("xy");
		});

		test("throws for empty with expression", async () => {
			const engine = make_engine("/tmp");
			await expect(engine.renderString("{#with }x{/with}")).rejects.toThrow("Invalid #with syntax");
		});

		test("throws for unclosed with", async () => {
			const engine = make_engine("/tmp");
			await expect(engine.renderString("{#with props.x}a")).rejects.toThrow("Unclosed");
		});

		test("throws for unmatched /with", async () => {
			const engine = make_engine("/tmp");
			await expect(engine.renderString("{/with}")).rejects.toThrow("Unexpected {/with}");
		});

		test("throws else inside with", async () => {
			const engine = make_engine("/tmp");
			await expect(engine.renderString("{#with props.x}a{:else}b{/with}")).rejects.toThrow(
				"{:else} is not allowed",
			);
		});

		test("with inside an each loop", async () => {
			const engine = make_engine("/tmp");
			const tmpl = "{#each props.items as item}{#with item}{= value }{/with}{/each}";
			const result = await engine.renderString(tmpl, { items: [{ value: 1 }, { value: 2 }] });
			expect(result).toBe("12");
		});
	});

	describe("#each / {:else} / {/each}", () => {
		test("iterates over array", async () => {
			const engine = make_engine("/tmp");
			const tmpl = "{#each props.items as item}{= item }{/each}";
			expect(await engine.renderString(tmpl, { items: ["a", "b", "c"] })).toBe("abc");
		});

		test("provides index variable", async () => {
			const engine = make_engine("/tmp");
			const tmpl = "{#each props.items as item, idx}{= idx }:{= item },{/each}";
			expect(await engine.renderString(tmpl, { items: ["x", "y"] })).toBe("0:x,1:y,");
		});

		test("provides key variable for objects", async () => {
			const engine = make_engine("/tmp");
			const tmpl = "{#each props.items as val, idx, key}{= key }={= val },{/each}";
			const result = await engine.renderString(tmpl, { items: { a: 1, b: 2 } });
			expect(result).toBe("a=1,b=2,");
		});

		test("renders else when empty", async () => {
			const engine = make_engine("/tmp");
			const tmpl = "{#each props.items as item}{= item }{:else}empty{/each}";
			expect(await engine.renderString(tmpl, { items: [] })).toBe("empty");
		});
	});

	describe("file-based render", () => {
		test(
			"loads and renders a template file",
			with_temp_dir(async (dir, engine) => {
				writeFileSync(join(dir, "hello.ree"), "Hello {= props.name }!");
				const result = await engine.render("hello", { name: "reepolee" });
				expect(result).toBe("Hello reepolee!");
			}),
		);

		test(
			"throws for missing template",
			with_temp_dir(async (dir, engine) => {
				await expect(engine.render("nonexistent")).rejects.toThrow("not found");
			}),
		);
	});

	describe("includes", () => {
		test(
			"{#include} renders partial template (Props passed via props.* reference)",
			with_temp_dir(async (dir, engine) => {
				writeFileSync(join(dir, "_header.ree"), "<header>{= props.title }</header>");
				// Use props.incData so the expression resolves via the props object
				const tmpl = '<main>{#include("_header", props.incData )}</main>';
				writeFileSync(join(dir, "page.ree"), tmpl);
				const result = await engine.render("page", { incData: { title: "Hi" } });
				expect(result).toBe("<main><header>Hi</header></main>");
			}),
		);

		test(
			"nested includes work (include within include)",
			with_temp_dir(async (dir, engine) => {
				writeFileSync(join(dir, "_shared.ree"), "<small>{= props.text }</small>");
				writeFileSync(join(dir, "_section.ree"), '<section>{#include("_shared", props.inner )}</section>');
				writeFileSync(join(dir, "page.ree"), '<div>{#include("_section", props.sec )}</div>');
				const result = await engine.render("page", {
					sec: { inner: { text: "nested" } },
				});
				expect(result).toBe("<div><section><small>nested</small></section></div>");
			}),
		);

		test(
			"relative include path ./ works",
			with_temp_dir(async (dir, engine) => {
				mkdirSync(join(dir, "partials"), { recursive: true });
				writeFileSync(join(dir, "partials", "_footer.ree"), "<footer>{= props.text }</footer>");
				// The include path is relative to the template's directory
				writeFileSync(
					join(dir, "page.ree"),
					'<main>{#include("./partials/_footer", props.footerData )}</main>',
				);
				const result = await engine.render("page", { footerData: { text: "Footer" } });
				expect(result).toBe("<main><footer>Footer</footer></main>");
			}),
		);
		test(
			"localized include fallback works",
			with_temp_dir(async (dir, engine) => {
				// Create language-specific templates and a default.
				// NOTE: the specific-variant language ("fr") is deliberately NOT any project's
				// default_language, so the fallback assertion below holds regardless of the
				// configured default (en here, sl upstream). Worth upstreaming.
				writeFileSync(join(dir, "_header.fr.ree"), "<header>Hello</header>");
				writeFileSync(join(dir, "_header.ree"), "<header>Default</header>");
				// Use props.emptyData to reference empty data instead of {} which confuses the parser
				writeFileSync(join(dir, "page.ree"), '{#include("_header", props.emptyData )}');

				// French should use the fr variant
				const fr = await engine.render("page", { lang: "fr", emptyData: {} });
				expect(fr).toBe("<header>Hello</header>");

				// German (no variant, and not the default) should fall back to the bare default
				const de = await engine.render("page", { lang: "de", emptyData: {} });
				expect(de).toBe("<header>Default</header>");
			}),
		);
	});

	describe("layouts", () => {
		test(
			"{#layout} wraps content — body must use {~ ... } (unescaped) since it contains HTML",
			with_temp_dir(async (dir, engine) => {
				writeFileSync(join(dir, "_layout.ree"), "<html><body>{~ props.body }</body></html>");
				writeFileSync(join(dir, "child.ree"), '{#layout("_layout")}<h1>Content</h1>');
				const result = await engine.render("child");
				expect(result).toBe("<html><body><h1>Content</h1></body></html>");
			}),
		);
	});

	describe("components", () => {
		test(
			"ReeTag <tag-name> includes from components/ dir",
			with_temp_dir(async (dir, engine) => {
				const componentsDir = join(dirname(dir), "components");
				mkdirSync(componentsDir, { recursive: true });
				writeFileSync(
					join(componentsDir, "my-badge.ree"),
					"<span class='badge'>{= props.attributes.label }</span>",
				);
				writeFileSync(join(dir, "page.ree"), "<div><my-badge label='New'></my-badge></div>");
				const result = await engine.render("page");
				expect(result).toBe("<div><span class='badge'>New</span></div>");
				rmSync(componentsDir, { recursive: true, force: true });
			}),
		);

		test(
			"{#include} explicitly resolves a component from components/ dir",
			with_temp_dir(async (dir, engine) => {
				const componentsDir = join(dirname(dir), "components");
				mkdirSync(componentsDir, { recursive: true });
				writeFileSync(join(componentsDir, "my-badge.ree"), "<span class='badge'>{= props.label }</span>");
				writeFileSync(join(dir, "page.ree"), '{#include("$components/my-badge")}');
				const result = await engine.render("page", { label: "New" });
				expect(result).toBe("<span class='badge'>New</span>");
				rmSync(componentsDir, { recursive: true, force: true });
			}),
		);
	});

	describe("custom HTML elements", () => {
		test(
			"unknown custom element <tag-name> is passed through as literal HTML",
			with_temp_dir(async (_dir, engine) => {
				const result = await engine.renderString("<toasts-area>content</toasts-area>");
				expect(result).toBe("<toasts-area>content</toasts-area>");
			}),
		);

		test(
			"unknown custom element preserves HTML attributes",
			with_temp_dir(async (_dir, engine) => {
				const result = await engine.renderString('<toasts-area class="foo" id="bar">content</toasts-area>');
				expect(result).toBe('<toasts-area class="foo" id="bar">content</toasts-area>');
			}),
		);

		test("deeply nested unknown custom elements do not OOM (regression test)", async () => {
			const engine = make_engine("/tmp");
			const nesting = 20;
			// Build deeply nested structure with hyphenated tags so custElemRegex matches them:
			//   <x-one><x-two>...<x-twenty>text</x-twenty>...</x-two></x-one>
			let tmpl = "text";
			for (let i = 0; i < nesting; i++) {
				// e.g. x-one, x-two, x-three, ...
				const suffixes = [
					"one",
					"two",
					"three",
					"four",
					"five",
					"six",
					"seven",
					"eight",
					"nine",
					"ten",
					"eleven",
					"twelve",
					"thirteen",
					"fourteen",
					"fifteen",
					"sixteen",
					"seventeen",
					"eighteen",
					"nineteen",
					"twenty",
				];
				const tag = `x-${suffixes[i]}`;
				tmpl = `<${tag}>${tmpl}</${tag}>`;
			}
			const start = performance.now();
			const result = await engine.renderString(tmpl);
			const elapsed = performance.now() - start;
			// Should complete quickly (< 2s) — if it takes longer, it likely infinite-looped
			expect(elapsed).toBeLessThan(2000);
			// Verify the full nesting was preserved
			expect(result).toBe(tmpl);
		});

		test(
			"template directives inside unknown custom element slots still work",
			with_temp_dir(async (_dir, engine) => {
				const result = await engine.renderString("<wrapper>{= props.message }</wrapper>", {
					message: "Hello from inside!",
				});
				expect(result).toBe("<wrapper>Hello from inside!</wrapper>");
			}),
		);
		test(
			"known component file takes priority over unknown element passthrough",
			with_temp_dir(async (dir, engine) => {
				// Create a matching component file
				const componentsDir = join(dirname(dir), "components");
				mkdirSync(componentsDir, { recursive: true });
				// HTML attributes on the tag are passed as props.attributes.*
				writeFileSync(
					join(componentsDir, "my-badge.ree"),
					"<span class='badge'>{= props.attributes.label }</span>",
				);
				writeFileSync(join(dir, "page.ree"), "<my-badge label='New'>slot</my-badge>");
				const result = await engine.render("page");
				expect(result).toBe("<span class='badge'>New</span>");
				rmSync(componentsDir, { recursive: true, force: true });
			}),
		);

		test(
			"<auto-complete> renders component with HTML attributes passed as props.attributes",
			with_temp_dir(async (dir, engine) => {
				const componentsDir = join(dirname(dir), "components");
				mkdirSync(componentsDir, { recursive: true });
				// Component echoes its attributes as data-* attrs on a wrapper div
				writeFileSync(
					join(componentsDir, "auto-complete.ree"),
					'<div class="ac" data-field="{= props.attributes["field-name"] }" data-fk-table="{= props.attributes["fk-table"] }" data-fk-column="{= props.attributes["fk-column"] }" data-base-url="{= props.attributes["base-url"] }" data-rows="{= props.attributes["rows"] }"></div>',
				);
				writeFileSync(
					join(dir, "page.ree"),
					'<auto-complete field-name="legal_entity_registration_number" fk-table="legal_entities" fk-column="registration_number" base-url="/partners" rows="15"></auto-complete>',
				);
				const result = await engine.render("page");
				expect(result).toContain('data-field="legal_entity_registration_number"');
				expect(result).toContain('data-fk-table="legal_entities"');
				expect(result).toContain('data-fk-column="registration_number"');
				expect(result).toContain('data-base-url="/partners"');
				expect(result).toContain('data-rows="15"');
				rmSync(componentsDir, { recursive: true, force: true });
			}),
		);

		test(
			"<auto-complete> receives parent props (fields, record) via Object.assign",
			with_temp_dir(async (dir, engine) => {
				const componentsDir = join(dirname(dir), "components");
				mkdirSync(componentsDir, { recursive: true });
				// Component accesses props.fields and props.record from parent scope
				writeFileSync(
					join(componentsDir, "auto-complete.ree"),
					'<div class="ac" data-field="{= props.attributes["field-name"] }">' +
						'<span class="field-label">{= props.fields?.[props.attributes["field-name"]]?.label }</span>' +
						'<span class="field-value">{= props.record?.[props.attributes["field-name"]] }</span>' +
						"</div>",
				);
				writeFileSync(join(dir, "page.ree"), '<auto-complete field-name="company_id"></auto-complete>');

				const fields = {
					company_id: { label: "Company", type: "autocomplete" },
				};
				const record = { company_id: 42 };
				const result = await engine.render("page", { fields, record });

				expect(result).toContain('data-field="company_id"');
				expect(result).toContain('<span class="field-label">Company</span>');
				expect(result).toContain('<span class="field-value">42</span>');
				rmSync(componentsDir, { recursive: true, force: true });
			}),
		);

		test(
			"multiple <auto-complete> elements on page each render independently",
			with_temp_dir(async (dir, engine) => {
				const componentsDir = join(dirname(dir), "components");
				mkdirSync(componentsDir, { recursive: true });
				// Component renders a wrapper with field-name as data-field
				writeFileSync(
					join(componentsDir, "auto-complete.ree"),
					'<div class="ac" data-field="{= props.attributes["field-name"] }"></div>',
				);
				writeFileSync(
					join(dir, "page.ree"),
					'<auto-complete field-name="company_id"></auto-complete>' +
						"<br>" +
						'<auto-complete field-name="partner_id" rows="10"></auto-complete>',
				);
				const result = await engine.render("page");
				expect(result).toBe(
					'<div class="ac" data-field="company_id"></div>' +
						"<br>" +
						'<div class="ac" data-field="partner_id"></div>',
				);
				rmSync(componentsDir, { recursive: true, force: true });
			}),
		);

		test(
			"<auto-complete> slot content is rendered as children prop",
			with_temp_dir(async (dir, engine) => {
				const componentsDir = join(dirname(dir), "components");
				mkdirSync(componentsDir, { recursive: true });
				// Component renders children slot content wrapped in a div
				writeFileSync(
					join(componentsDir, "auto-complete.ree"),
					'<div class="ac-wrapper">{~ props.children }</div>',
				);
				writeFileSync(
					join(dir, "page.ree"),
					'<auto-complete><span class="hint">Type to search</span></auto-complete>',
				);
				const result = await engine.render("page");
				expect(result).toBe('<div class="ac-wrapper"><span class="hint">Type to search</span></div>');
				rmSync(componentsDir, { recursive: true, force: true });
			}),
		);
	});

	describe("HTML comments", () => {
		test("expressions inside HTML comments are NOT evaluated", async () => {
			const engine = make_engine("/tmp");
			// {= props.x } inside <!-- --> should be stripped before compilation
			const result = await engine.renderString("before<!-- {= props.x } -->after", { x: "CRASH" });
			expect(result).toBe("beforeafter");
		});

		test("expressions outside HTML comments are still evaluated", async () => {
			const engine = make_engine("/tmp");
			const result = await engine.renderString("{= props.greeting }<!-- comment -->{= props.name }", {
				greeting: "Hello",
				name: "World",
			});
			expect(result).toBe("HelloWorld");
		});

		test("HTML comments with multi-line content are stripped", async () => {
			const engine = make_engine("/tmp");
			const result = await engine.renderString(
				'<!--\n\t<div class="{= props.cls }">{= props.val }</div>\n-->shown',
				{ cls: "foo", val: "bar" },
			);
			// The expression inside the comment should NOT be evaluated
			expect(result).toBe("shown");
		});

		test("directives inside HTML comments are not processed", async () => {
			const engine = make_engine("/tmp");
			const result = await engine.renderString("start<!-- {#if true}SHOULD_NOT_APPEAR{/if} -->end");
			expect(result).toBe("startend");
		});
	});

	describe("HTML escaping", () => {
		test("escape() handles null/undefined", () => {
			const engine = make_engine("/tmp");
			expect(engine.escape(null)).toBe("");
			expect(engine.escape(undefined)).toBe("");
		});

		test("escape() converts special characters", () => {
			const engine = make_engine("/tmp");
			expect(engine.escape("&<>\"'")).toBe("&amp;&lt;&gt;&quot;&#39;");
		});

		test("escape() converts numbers", () => {
			const engine = make_engine("/tmp");
			expect(engine.escape(42)).toBe("42");
		});
	});

	describe("language-aware loading", () => {
		test(
			"loadLocalized falls back through lang chain",
			with_temp_dir(async (dir, engine) => {
				writeFileSync(join(dir, "page.sl.ree"), "Pozdravljeni!");
				writeFileSync(join(dir, "page.en.ree"), "Hello!");
				const sl = await engine.loadLocalized("page", "sl");
				expect(sl.content).toContain("Pozdravljeni");
				const en = await engine.loadLocalized("page", "en");
				expect(en.content).toContain("Hello");
			}),
		);

		test(
			"loadLocalized falls back to default lang then bare file",
			with_temp_dir(async (dir, engine) => {
				writeFileSync(join(dir, "page.ree"), "Default fallback");
				const result = await engine.loadLocalized("page", "de");
				expect(result.content).toContain("Default fallback");
			}),
		);

		test(
			"render uses language-specific template when available",
			with_temp_dir(async (dir, engine) => {
				writeFileSync(join(dir, "greet.en.ree"), "Hello");
				writeFileSync(join(dir, "greet.sl.ree"), "Zdravo");
				writeFileSync(join(dir, "greet.ree"), "Hi");
				const en = await engine.render("greet", { lang: "en" });
				expect(en).toBe("Hello");
				const sl = await engine.render("greet", { lang: "sl" });
				expect(sl).toBe("Zdravo");
			}),
		);
	});

	describe("helpers — helper_vars eval path", () => {
		test("custom helpers are accessible as template variables", async () => {
			const engine = make_engine("/tmp");
			// Pass a custom helper function via props.helpers
			const helpers = {
				custom_upper: (s: string) => s.toUpperCase(),
			};
			const result = await engine.renderString("{= custom_upper('hello') }", { helpers });
			expect(result).toBe("HELLO");
		});

		test("non-function helpers are ignored in eval", async () => {
			const engine = make_engine("/tmp");
			const helpers = {
				custom_data: { foo: "bar" },
			};
			const result = await engine.renderString("Hello", { helpers });
			expect(result).toBe("Hello");
		});
	});

	describe("cache", () => {
		test(
			"caches compiled file templates when enabled",
			with_temp_dir(async (dir, engine) => {
				const cachedEngine = new TE({ views: dir, cache: true, ext: ".ree" });
				writeFileSync(join(dir, "t.ree"), "a");
				await cachedEngine.render("t");
				expect(Object.keys(cachedEngine.compiledCache).length).toBe(1);
			}),
		);

		test("clearCache empties compiled cache", async () => {
			const engine = new TE({ views: "/tmp", cache: true, ext: ".ree" });
			await engine.renderString("x");
			engine.clearCache();
			expect(Object.keys(engine.compiledCache).length).toBe(0);
		});
	});
});
