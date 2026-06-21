---
title: "Why Reeweb"
layout: "reeweb/docs/docs.layout"
---

# Why Reeweb

<a name="a-letter-from-the-founders"></a>

## A Letter from the Founders

We've been building web applications professionally for a long time — and somewhere along the way, we noticed that the simplest projects often had the most complicated setups. A marketing site that should take a day to build ended up needing a bundler configuration, a content strategy, and a dependency graph that looked like a map of the London Underground. We'd open a project that was running fine six months ago and spend a day untangling peer dependency conflicts before writing a single line of content.

We got tired of it.

We wanted something that got out of the way.

<a name="the-node-modules-problem"></a>

## The node_modules Problem

Every dependency is a promise. It promises to work the same way tomorrow, to have no breaking changes, to be maintained by someone who cares, and to never introduce a security vulnerability that sends you scrambling for an upgrade. And for a while, most of them keep that promise. But the ones that don't — the abandoned package, the silent API break, the license change that your legal team flags six months after you shipped — those cost more than the sum of all the ones that worked.

For a static site, the calculus is worse. You are generating HTML files. There is no user session to protect, no database connection to pool, no real-time state to manage. And yet the tooling around static sites has grown to resemble the tooling around real-time applications — the same bundler configuration, the same 2,000-line `node_modules` tree, the same fragile dependency graph that collapses the moment one transitive dependency decides to publish a breaking change.

We wanted to build a static site generator that didn't make that trade-off. Reeweb has zero runtime dependencies. Every line of the template engine, the build system, and the dev server is written in plain TypeScript using Bun's native APIs. When you open a Reeweb project six months from now, it will build and render exactly the same way it did the day you left it. There is no `npm audit`, no package you forgot you depended on, no `node_modules` folder that takes minutes to install.

<a name="static-by-default-dynamic-when-needed"></a>

## Static by Default, Dynamic When Needed

Most projects don't need a database, a login system, or a real-time backend. They need fast HTML, well-organised templates, multi-language support, and a build step that doesn't take longer than writing the content itself. We've built documentation sites, marketing pages, and multi-language landing pages for clients who had tried the full-stack approach and found it overkill — a server process they didn't need, a database they had to back up, a login screen their visitors never saw.

Reeweb starts with those assumptions. You write `.ree` templates and markdown, get a fully rendered static site, and deploy it anywhere — a CDN, a VPS, a Raspberry Pi, or a simple static file host. There's no server process to keep running, no database connection to maintain, and no runtime to monitor. The output is flat HTML, served as fast as your CDN can deliver it.

But when your project outgrows static, Reeweb doesn't force you to start over. The same template system, the same component library, and the same build pipeline work whether you're generating 10 pages or 10,000. And when you need the features that a static site can't provide — authentication, a database, form handling, an admin panel — the project can grow into Reepolee without a rewrite. The templates, translations, and components you built for the static site continue to work. You add what you need; you don't pay for what you don't.

<a name="bun-native"></a>

## Bun Native

Reeweb runs on Bun. Not because Bun is trendy — because it solves real problems that mattered to us when we were building this.

Bun replaces Node.js, the package manager, the bundler, and the test runner all in one binary. It understands TypeScript natively, so there is no `tsconfig.json` to configure and no build step for your build scripts — they run as-is. `bun install` takes seconds, not minutes. The same runtime works on macOS, Linux, Windows, x86, and ARM. The same `bun run build` command on a developer's laptop produces the same output on a CI runner, which produces the same output on a Cloudflare Pages build server.

We made a deliberate choice to build on Bun rather than on top of the Node.js ecosystem. Not because Node.js can't run a static site generator — it obviously can — but because building on Bun meant we could ship a tool that has zero npm dependencies at runtime. When you install Reeweb, you get Reeweb. Not Reeweb plus a hundred packages you didn't choose.

<a name="multi-language-by-design"></a>

## Multi-Language by Design

We've lost track of how many clients came to us with a bilingual site where the "other language" was hidden behind a Google Translate widget in the corner, or served from a separate subdomain with a different design system, or simply marked as "coming soon" for months. Multi-language support is almost always an afterthought — a layer you add on top of a tool that was designed for one language, one market, one audience.

In Reeweb, it's the other way around. Multi-language is built into the template engine, the build pipeline, and the URL structure from day one. We've built bilingual sites for clients in markets where serving content in the wrong language is not an SEO problem — it's a credibility problem. A Slovenian visitor landing on an English-only page is not a user who will browse around until they find what they need. They leave.

Reeweb handles this at the foundation. You write translation `.json` files next to your templates, and the build system does the rest: localized URLs (`/about/` in English, `/o-nas/` in Slovenian), hreflang links for Google, locale-aware date formatting, and fallback chains so a missing translation never leaves a blank on the page. You don't need a third-party translation service or a plugin. It's just how the tool works.

<a name="familiar-templating"></a>

## Familiar Templating

Ree templates look like HTML with a few extra tags — `{= }` for output, `{#if}` and `{#each}` for control flow, `{#layout}` and `{#include}` for composition. The template engine compiles to an async JavaScript function on first use and caches the result. In production, rendering is fast and predictable. In development, caching is off so changes appear immediately.

There is nothing novel about the syntax. That is the point. Drawing from Eta.js for output tags and from Svelte for control-flow blocks, we landed on something that feels familiar from the first time you open a `.ree` file. If you know HTML and a little JavaScript, you already know Ree. The implementation lives in `lib/template_engine.ts` and a handful of small modules under `lib/template/` — plain TypeScript, readable, auditable, and yours to modify if you need it to behave differently.

<a name="the-stack-we-settled-on"></a>

## The Stack We Settled On

We made deliberate choices at every layer. Custom `.ree` templates keep the rendering model simple and the HTML in your hands. Multi-language support is built in, not bolted on. Tailwind v4 handles styling without asking you to make architectural decisions. Bun's native APIs power the build, the dev server, and the preview server — one runtime for the whole pipeline.

None of these are novel choices. That is the point. Proven tools with stable APIs, assembled into something that feels coherent rather than bolted together.

<a name="who-reeweb-is-for"></a>

## Who Reeweb Is For

Reeweb is for developers who want to ship a fast, well-structured website without the overhead of a full application framework. It's for teams that need multi-language support without the complexity. It's for anyone who has ever opened their `node_modules` folder on a static site project and wondered why generating HTML files requires two thousand packages.

We built Reeweb because we needed a static site generator that felt as well-considered as the application framework we use every day. We hope you find it useful too.

— The Reepolee Team
