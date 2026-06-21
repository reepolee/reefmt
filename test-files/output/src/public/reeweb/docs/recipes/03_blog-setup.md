---
title: "Setting Up a Blog"
layout: "reeweb/docs/docs.layout"
---

# Setting Up a Blog

<a name="introduction"></a>

## Introduction

Reeweb supports blog content through markdown files. Each post is a `.md` file in `src/public/blog/`, and the build process renders them to HTML, generates an RSS feed, and includes them in the sitemap. This recipe walks through the complete setup.

<a name="directory-structure"></a>

## Directory Structure

```
src/public/blog/
├── en.json                ← Blog section translations
├── index.ree              ← Blog index page
├── index.ts               ← Blog index data loader
├── 01_hello-world.md      ← Blog post (markdown)
├── 02_second-post.md
└── images/                ← Blog-specific images
```

<a name="step-1-create-the-blog-index-page"></a>

## Step 1: Create the Blog Index Page

Create `src/public/blog/index.ree`:

```ree
{#layout("layout") }

<h1>{= props.ui.blog_title || "Blog" }</h1>

{#each props.posts as post }
    <article>
        <h2><a href="{~ localized_path(post.canonical_path) }">{= post.title }</a></h2>
        <time datetime="{= post.date_iso }">{= post.date_display }</time>
        {#if post.excerpt }
            <p>{= post.excerpt }</p>
        {/if }
        <a href="{~ localized_path(post.canonical_path) }">Read more →</a>
    </article>
{/each }
```

<a name="step-2-create-the-data-loader"></a>

## Step 2: Create the Data Loader

Create `src/public/blog/index.ts` that collects posts from markdown files:

```ts
import { readFileSync, readdirSync } from "fs";
import { join } from "path";
import { parse_frontmatter, template_to_canonical } from "$lib/static_site";

export async function load_template_data(): Promise<Record<string, any>> {
	const blog_dir = join(import.meta.dir, "..", "..", "public", "blog");
	let entries = readdirSync(blog_dir)
		.filter((f) => f.endsWith(".md"))
		.sort();

	const posts = entries
		.map((file) => {
			const content = readFileSync(join(blog_dir, file), "utf-8");
			const { data } = parse_frontmatter(content);
			const canonical_path = template_to_canonical(`blog/${file}`);
			return {
				title: data.title || "Untitled",
				date_iso: data.date || "",
				date_display: data.date
					? new Date(data.date).toLocaleDateString("en-US", {
							year: "numeric",
							month: "long",
							day: "numeric",
						})
					: "",
				excerpt: props.excerpt || "",
				canonical_path,
			};
		})
		.sort((a, b) => b.date_iso.localeCompare(a.date_iso)); // newest first

	return { posts };
}
```

<a name="step-3-create-a-blog-post"></a>

## Step 3: Create a Blog Post

Create `src/public/blog/01_hello-world.md`:

````markdown
---
title: "Hello, World!"
date: 2026-06-01
excerpt: "Welcome to the blog! This is our first post."
layout: "layout"
---

Welcome to the Reeweb blog! This is our first post, written in markdown.

## Features

- Syntax highlighting for code blocks
- Auto-generated heading IDs
- Tailwind classes on all elements
- External links open in new tabs

```typescript
const greeting = "Hello from Reeweb!";
console.log(greeting);
```
````

````

The ordering prefix `01_` is stripped from the URL — the canonical path becomes `/blog/hello-world`.

<a name="step-4-generate-the-rss-feed"></a>
## Step 4: Generate the RSS Feed

Run the RSS generator:

```bash
bun run rss
````

This runs `scripts/generate_rss.ts`, which scans `src/public/blog/` for markdown files, extracts frontmatter, and writes a `feed.xml` to the output directory. The sitemap generator can also be run separately:

```bash
bun run sitemap
```

Both are included in a full build automatically via `bun run build:dist` (if you've defined that script) and can be run independently.

<a name="step-5-add-translations"></a>

## Step 5: Add Translations

Create `src/public/blog/en.json`:

```json
{
	"blog_title": "Blog"
}
```

And `src/public/blog/sl.json`:

```json
{
	"blog_title": "Blog"
}
```

<a name="full-build-with-rss-and-sitemap"></a>

## Full Build with RSS and Sitemap

For production builds that include everything:

```bash
bun run build
bun run rss
bun run sitemap
```

Or combine into a single script in your `package.json`:

```json
"build:full": "bun run build && bun run rss && bun run sitemap"
```
