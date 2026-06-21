---
title: "Blog Setup"
---

# Blog Setup

<a name="introduction"></a>

## Introduction

A blog on Reepolee is a database-backed CRUD route with markdown content and an RSS feed. The generator creates the admin CRUD for you; the remaining work is adding a rich text field for the body, rendering markdown on the public side, and generating the feed.

This page walks through the complete setup — database table, generated admin, public display, and RSS feed — in about twenty minutes.

<a name="step-1-create-the-table"></a>

## Step 1 — Create the Database Table

Add a `posts` table to your schema:

```sql
CREATE TABLE posts (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    slug TEXT NOT NULL UNIQUE,
    body TEXT NOT NULL DEFAULT '',
    excerpt TEXT DEFAULT '' NULL,
    author TEXT DEFAULT '' NULL,
    published_at DATETIME DEFAULT NULL,
    is_published BOOLEAN NOT NULL DEFAULT 0,
    created_at DATETIME DEFAULT current_timestamp,
    updated_at DATETIME DEFAULT NULL
);
```

The key columns:

- **`slug`** — URL-safe identifier for the post, used in the public route instead of the numeric ID. `UNIQUE` constraint prevents duplicate slugs.
- **`body`** — The post content in markdown format. Stored as text, rendered to HTML at request time.
- **`excerpt`** — Short summary for list pages and RSS feeds. Optional — if empty, you can generate one from the body.
- **`published_at`** — When the post was published. `NULL` means not yet published. Used for sorting and RSS feed dates.
- **`is_published`** — Draft/published flag. The public route only shows posts where `is_published = 1`.

Add this to your `sql/sqlite/01-init-sqlite.sql` or `sql/mysql/01-init-mysql.sql` and reset the database or run the migration manually.

Now add `"posts"` to the `IGNORE_TABLES` list in `config/db_structure.ts` — you'll be customising the generated files and don't want them overwritten by the next generator run. (Or skip this, generate first, and add it after.)

<a name="step-2-generate-the-crud"></a>

## Step 2 — Generate the CRUD

```bash
bun generator/resource crud posts
```

This creates `routes/posts/` with the standard CRUD files. After generation, you'll want to customise:

1. **`routes/posts/schema/table.ts`** — change the `body` field type to `textarea` and add a `comment` hint so the generator picks it up on re-runs:

```ts
export const posts_columns: ColumnDef[] = [
	{ name: "title", type: "text", required: true },
	{ name: "slug", type: "text", required: true },
	{ name: "body", type: "textarea" }, // changed from "text"
	{ name: "excerpt", type: "textarea" },
	{ name: "author", type: "text" },
	{ name: "published_at", type: "date" },
	{ name: "is_published", type: "boolean" },
];
```

2. **`routes/posts/form.ree`** — the body field is now a `<textarea>` which the admin can use for markdown editing. For a richer editing experience, you can add a toolbar or integrate Pell as shown in the [Email Module](/email/email-module#composing-with-pell).

<a name="step-3-create-the-public-route"></a>

## Step 3 — Create the Public Route

<media-frame label="Screenshot — public blog index (list of posts)" ratio="16/9"></media-frame>

The admin CRUD lives at `/posts/` (or `/admin/posts/` if you used `--prefix`). The public view needs two routes: a list page showing published posts and a detail page for a single post.

Create `routes/blog/` with these files:

**`routes/blog/index.ts`**:

```ts
import { create_ctx } from "$lib/request_context";
import { timed_query } from "$lib/timed_sql";
import { render } from "$lib/render";
import { translated_from_request } from "$lib/helpers";
import { db } from "$config/db";

export async function get_blog_index(req: BunRequest): Promise<Response> {
	const ctx = await create_ctx(req);
	const translated = await translated_from_request(req, import.meta.dir);

	const posts = await timed_query(
		"blog",
		"list_published",
		() =>
			db`
            SELECT id, title, slug, excerpt, author, published_at
            FROM posts
            WHERE is_published = 1 AND published_at IS NOT NULL
            ORDER BY published_at DESC
        `,
		(r) => ({ count: r.length }),
	);

	return render("blog/index", {
		data: { posts, ...translated },
		ctx,
	});
}

export async function get_blog_post(req: BunRequest): Promise<Response> {
	const ctx = await create_ctx(req);
	const translated = await translated_from_request(req, import.meta.dir);
	const slug = req.params.slug;

	const [post] = await db`
        SELECT * FROM posts
        WHERE slug = ${slug} AND is_published = 1
        LIMIT 1
    `;

	if (!post) {
		return render("notfound", { status: 404, ctx });
	}

	// Render markdown body to HTML
	post.body_html = Bun.markdown.html(post.body);

	return render("blog/post", {
		data: { post, ...translated },
		ctx,
	});
}
```

**`routes/blog/index.ree`** — list template:

```html
{#layout("layout")}

<h1>{= props.ui.title }</h1>

{#each props.posts as post }
<article class="mb-8 border-b pb-4">
	<h2>
		<a href="/blog/{= post.slug }">{= post.title }</a>
	</h2>
	{#if post.excerpt }
	<p>{= post.excerpt }</p>
	{/if }
	<time datetime="{= js_date_to_iso_string(post.published_at) }">
		{= js_date_to_locale_string(post.published_at) }
	</time>
	{#if post.author }
	<span class="ml-2 text-muted">{= post.author }</span>
	{/if }
</article>
{:else }
<p>{= props.ui.no_posts }</p>
{/each }
```

**`routes/blog/post.ree`** — detail template:

```html
{#layout("layout")}

<article>
	<h1>{= props.post.title }</h1>
	<time datetime="{= js_date_to_iso_string(props.post.published_at) }">
		{= js_date_to_locale_string(props.post.published_at) }
	</time>
	<div class="prose max-w-none mt-6">{~ props.post.body_html }</div>
</article>
```

**`routes/blog/en.json`**:

```json
{
	"route_name": "blog",
	"ui": {
		"title": "Blog",
		"no_posts": "No posts yet."
	}
}
```

**Register the routes** in `routes/routes.ts`:

```ts
import { get_blog_index, get_blog_post } from "$routes/blog";

export const routes = {
	"/": home_page,
	"/blog": get_blog_index,
	"/blog/:slug": get_blog_post,
	// ... existing routes
};
```

<a name="step-4-add-an-rss-feed"></a>

## Step 4 — Add an RSS Feed

The RSS feed endpoint reuses the same query as the blog index but returns XML instead of HTML:

**`routes/blog/rss.ts`**:

```ts
import { db } from "$config/db";
import { now_iso_str } from "$lib/temporal";

export async function get_blog_rss(req: BunRequest): Promise<Response> {
	const url = new URL(req.url);
	const site_url = `${url.protocol}//${url.host}`;

	const posts = await db`
        SELECT id, title, slug, excerpt, author, published_at
        FROM posts
        WHERE is_published = 1 AND published_at IS NOT NULL
        ORDER BY published_at DESC
        LIMIT 20
    `;

	const items = posts
		.map(
			(p) => `
        <item>
            <title>${escape_xml(p.title)}</title>
            <link>${site_url}/blog/${escape_xml(p.slug)}</link>
            <description>${escape_xml(p.excerpt || "")}</description>
            <guid>${site_url}/blog/${escape_xml(p.slug)}</guid>
            <pubDate>${new Date(p.published_at).toUTCString()}</pubDate>
        </item>
    `,
		)
		.join("\n");

	const xml = `<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0" xmlns:atom="http://www.w3.org/2005/Atom">
    <channel>
        <title>Blog</title>
        <link>${site_url}/blog</link>
        <atom:link href="${site_url}/blog/rss" rel="self" type="application/rss+xml"/>
        ${items}
    </channel>
</rss>`;

	return new Response(xml, {
		headers: { "Content-Type": "application/rss+xml; charset=utf-8" },
	});
}

function escape_xml(s: string): string {
	return s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;");
}
```

Register the feed in `routes/routes.ts`:

```ts
import { get_blog_rss } from "$routes/blog/rss";

export const routes = {
	// ...
	"/blog": get_blog_index,
	"/blog/:slug": get_blog_post,
	"/blog/rss": get_blog_rss,
};
```

<a name="step-5-style-the-public-pages"></a>

## Step 5 — Style the Public Pages

<media-frame label="Screenshot — styled blog post (prose typography)" ratio="4/3"></media-frame>

The blog templates use Tailwind classes directly. Add a `prose` utility for the post body if you want typographic styling:

```css
/* In css/app.css */
@layer components {
	.prose {
		max-width: 65ch;
		line-height: 1.75;
	}
	.prose h2 {
		font-size: 1.5rem;
		font-weight: 600;
		margin-top: 2rem;
	}
	.prose p {
		margin-top: 1.25rem;
	}
	.prose pre {
		background: #1a1a2e;
		color: #e0e0e0;
		padding: 1rem;
		border-radius: 0.5rem;
		overflow-x: auto;
	}
	.prose code {
		font-size: 0.875rem;
	}
	.prose img {
		border-radius: 0.5rem;
		margin: 2rem 0;
	}
}
```

For post-by-post customisation, add a `css` field to the posts table and inject it into the layout conditionally.
