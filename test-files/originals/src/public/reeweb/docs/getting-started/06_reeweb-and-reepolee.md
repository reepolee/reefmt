---
title: "Reeweb & Reepolee"
layout: "reeweb/docs/docs.layout"
---

# Reeweb & Reepolee

<a name="introduction"></a>

## Introduction

Reeweb and Reepolee are two products from the same team, built on the same foundation. They share the template engine, the i18n system, the Tailwind pipeline, and the philosophy of zero runtime dependencies. The difference is scope: Reeweb generates static sites; Reepolee runs full-stack applications.

This page explains how they relate, when to use which, and how a project can move between them as requirements change.

<a name="the-ecosystem"></a>

## The Ecosystem

Both tools are built on the same shared layer:

| Area       | Shared Foundation                            |
| ---------- | -------------------------------------------- |
| Templates  | `.ree` template engine                       |
| i18n       | Translations + `route_name` URL localisation |
| Styling    | Tailwind CSS v4 pipeline                     |
| Routing    | `slugify()` / `route_aliases`                |
| Components | File-based component system                  |
| Philosophy | Zero runtime dependencies                    |

|            | Reeweb (Static generator) | Reepolee (Full-stack) |
| ---------- | ------------------------- | --------------------- |
| Output     | Flat HTML                 | Bun server process    |
| Deploy     | CDN                       | VPS / container       |
| Rendering  | Build-time                | Request-time          |
| Server     | None to operate           | Bun process           |
| Database   | None                      | SQLite or MySQL       |
| Auth       | None                      | Sessions + roles      |
| Forms      | Third-party               | Built-in Zod + SMTP   |
| Admin      | Edit files                | Generated CRUD panels |
| Uploads    | N/A                       | S3 / local filesystem |
| Background | N/A                       | Temporal.io           |

The shared foundation means skills, templates, and translations transfer directly between the two. Learning one is learning most of the other.

<a name="when-to-use-which"></a>

## When to Use Which

### Choose Reeweb when:

| Situation                   | Why                                                                             |
| --------------------------- | ------------------------------------------------------------------------------- |
| Documentation site          | Markdown files render to fast, cacheable HTML. No server needed.                |
| Marketing / landing page    | Static files are the fastest thing you can serve. Deploy anywhere.              |
| Multi-language content site | Built-in i18n with localized URLs, hreflang links, and cross-language fallback. |
| Blog with RSS               | Markdown posts + data loader + RSS generator = complete blog pipeline.          |
| Client brochure site        | No maintenance. No server to patch. No database to migrate.                     |
| Prototype or MVP            | Ship in minutes. Iterate on content, not infrastructure.                        |

### Choose Reepolee when:

| Situation                  | Why                                                                                     |
| -------------------------- | --------------------------------------------------------------------------------------- |
| You need to store data     | Database-backed CRUD with generated admin panels.                                       |
| You need user accounts     | Login, registration, password resets, role-based access control.                        |
| You need forms that submit | Contact forms, order forms, any POST handler — with validation and email notifications. |
| You need an admin panel    | Generated from your schema. Staff manage content without touching files.                |
| You need file uploads      | Avatars, images, documents — stored on S3 or the local filesystem.                      |
| You need dynamic content   | Pages that change based on the user, the time of day, or what's in the database.        |

<a name="can-you-use-both"></a>

## Can You Use Both?

Not on the same domain at the same time — they're different deployment models (static files vs server process). However, the practical distinction is small: a Reepolee page with no database queries serves an HTTP response indistinguishable from a static HTML file. Your CDN caches it the same way and your users load it at the same speed.

Reeweb and Reepolee project files can coexist in the same repository under different directories. The shared foundation files (`lib/template_engine.ts` plus the `lib/template/` engine modules, `lib/i18n.ts`, etc.) are identical between the two projects, so a monorepo with both shares the core library — the template engine is maintained as one codebase upstream.

The practical model: **start with Reeweb, scale into Reepolee.** Launch a marketing site as a Reeweb project, get traffic, learn what users need, then extend the project into a Reepolee application when the requirements justify it.

<a name="faq"></a>

## Frequently Asked Questions

<a name="faq-downgrade"></a>

### Can I use Reeweb for a site that needs a database?

No. Reeweb has no database connection, no server process, and no write path. If you need to store and retrieve data dynamically, you need Reepolee or a third-party backend. Third-party options (Headless CMS, BaaS) add network latency and external dependencies — Reepolee keeps everything in one process.

<a name="faq-rewrite"></a>

### Will moving from Reeweb to Reepolee require a rewrite?

No. The template engine, component system, translation files, and Tailwind configuration are the same. Templates need a prefix change (`props.` → `data.`) because Reepolee injects additional server-side context, but the syntax and structure are identical. Many teams migrate a site of 50+ pages in a day.

<a name="faq-harder"></a>

### Is Reepolee harder to learn than Reeweb?

It has more features, so there's more to learn. But the foundation — templates, i18n, styling — is identical. If you know Reeweb, you already know the hardest part of Reepolee. The rest (database queries, auth flows, form handlers) follows standard patterns documented in the [Reepolee docs](/reepolee/docs).

<a name="faq-share-templates"></a>

### Can I share templates between a Reeweb site and a Reepolee app?

The `.ree` files are interchangeable with the data-prefix change (`props.` → `data.`). Components, translation files, and stylesheets transfer directly. You can keep a shared component library in a separate repository and use it in both projects.

<a name="faq-reasons-to-stay"></a>

### What reasons would I have to stay on Reeweb?

Reeweb is the better choice when you want the simplest possible deployment (copy files to a CDN), zero operational overhead (no server to maintain, no database to back up), and the fastest possible page loads (static files from a global CDN are faster than any server-rendered page). If none of your requirements need a database, authentication, or server-side processing, Reeweb is the right tool and there is no reason to leave.

<a name="faq-start-with-reepolee"></a>

### Should I start with Reepolee even if I only need static pages?

That depends on your confidence that the project will need dynamic features. If you're certain the project will stay static, Reeweb is simpler and cheaper to deploy. If you suspect you'll need auth, a database, or forms within the first few months, starting with Reepolee saves the transition later — you can serve static content from Reepolee just as easily as from Reeweb.

<a name="faq-hybrid"></a>

### Can I have static pages and dynamic pages in the same Reepolee project?

Yes — this is one of Reepolee's core design goals. A Reepolee route handler that queries no database and returns a static template is indistinguishable from a flat HTML file in terms of the HTTP response. You can have marketing pages rendered server-side alongside admin panels, user profiles, and database-backed content, all from the same route table, same templates, same deployment.

<a name="ecosystem-summary"></a>

## Summary

|                 | Reeweb                                   | Reepolee                             |
| --------------- | ---------------------------------------- | ------------------------------------ |
| **Deployment**  | Static files → CDN                       | Bun process → VPS / container        |
| **Content**     | Flat files (`.ree`, `.md`) at build time | Database records at request time     |
| **Auth**        | None                                     | Session-based, role-gated            |
| **Forms**       | Third-party service required             | Built-in Zod validation + SMTP       |
| **Admin**       | Edit files in a text editor              | Generated CRUD panels                |
| **Best for**    | Docs, blogs, marketing sites             | Applications, SaaS, internal tools   |
| **Shared with** | —                                        | Same template engine, i18n, Tailwind |

Reeweb and Reepolee are not competitors. They are two tiers of the same ecosystem, designed so that a project can start at the tier that fits today and grow into the tier that fits tomorrow.
