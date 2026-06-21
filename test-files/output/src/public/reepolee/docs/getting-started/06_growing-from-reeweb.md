---
title: "Growing from Reeweb"
---

# Growing from Reeweb

<a name="introduction"></a>

## Introduction

Reeweb and Reepolee are two sides of the same coin. They share the same `.ree` template engine, the same multi-language i18n system, the same Tailwind CSS pipeline, and the same Bun-native philosophy — zero runtime dependencies, plain TypeScript, no framework lock-in.

The difference is scope. Reeweb is a static site generator. It produces flat HTML files that you deploy to a CDN. Reepolee is a full-stack application framework. It runs as a Bun process with a database, authentication, form handling, email, and an admin panel.

Many of our clients start with Reeweb for a marketing site, a documentation hub, or a multi-language landing page. And that is the right call — static files are faster, simpler, and cheaper to serve than a server process. But as the project grows, the limitations of static content become unavoidable. This page is about that progression and how the two tools fit together.

<a name="the-continuum"></a>

## The Continuum

<media-frame label="Diagram — the Reeweb → Reepolee continuum" ratio="16/9"></media-frame>

Reeweb and Reepolee are not separate products with separate ecosystems. They sit at different points on the same continuum:

```
Reeweb                          Reepolee
─────────────────────────────────────────────────────────>
Static HTML     ← shared →     Server-rendered dynamic
Flat files      ← shared →     Database-backed
CDN deploy      ← shared →     Server process
No auth         ← shared →     Auth + sessions
No forms        ← shared →     Form handling + validation
No admin        ← shared →     Generated admin panels
No email        ← shared →     SMTP + email modules
```

Everything to the left of the arrow is what Reeweb handles. Everything to the right is what Reepolee adds. The shared foundation — templates, i18n, styling, helpers — is identical.

A project that starts on the left can move to the right without throwing away work. The templates render the same way. The translation files load the same way. The components work the same way. What changes is what happens _around_ the rendering — a database, a request pipeline, session management, background jobs.

<a name="when-to-make-the-move"></a>

## When to Make the Move

Reeweb is the right choice when:

- Your content is mostly static (documentation, blog, marketing pages)
- You don't need user accounts or personalised content
- You can handle forms through a third-party service (or don't need them)
- You want the simplest possible deployment (copy files to a CDN)

Reepolee becomes the right choice when:

- You need to **store data** — user profiles, blog posts in a database, customer records
- You need **authentication** — login, registration, password resets, role-based access
- You need **forms that do something** — contact forms that send email, order forms that create records
- You need an **admin panel** — a place for staff to manage content without editing markdown files
- You need **file uploads** — avatars, images, documents that users submit
- You need **dynamic content** — pages that change based on who's viewing them or what's in the database

These are not either/or decisions. You can have a Reepolee application where 90% of the pages are static — rendered server-side, aggressively cached, served as fast as flat files — and 10% are dynamic. The static pages stay static until you choose to make them dynamic.

<a name="what-you-keep"></a>

## What You Keep

When a Reeweb project moves to Reepolee, the following transfers directly:

| Asset             | Reeweb path                     | Reepolee path                               |
| ----------------- | ------------------------------- | ------------------------------------------- |
| `.ree` templates  | `src/public/`                   | `routes/{name}/`                            |
| Components        | `src/components/`               | `components/`                               |
| Translation JSON  | `{lang}.json` per directory     | `{lang}.json` per route folder              |
| Tailwind theme    | `src/css/style.css` `@theme`    | `css/app.css` `@theme`                      |
| Static assets     | `src/public/images/` etc.       | `static/`                                   |
| Config: languages | `config/supported_languages.ts` | `config/supported_languages.ts` (identical) |
| Client JS         | `src/public/*.js`               | `static/*.js`                               |

The template engine is the same code in both projects. A `.ree` file that renders correctly in Reeweb will render correctly in Reepolee — the data prefix is the same (`props.` in both); what Reepolee adds is extra context like the current user, the session, and toast notifications.

<a name="what-you-gain"></a>

## What You Gain

Moving from Reeweb to Reepolee adds capabilities without removing any you already had:

| Capability          | How Reepolee provides it                                                                                                    |
| ------------------- | --------------------------------------------------------------------------------------------------------------------------- |
| **Database**        | SQLite or MySQL. Bun's native `SQL` API. No ORM, no migrations framework.                                                   |
| **Auth**            | Session-based. Login, register, password reset, invitation codes. Role tags (`user`, `admin`, etc.) for access control.     |
| **Forms**           | Zod-validated, server-side with live client feedback. Toast notifications on success.                                       |
| **Admin panel**     | Generated from your database schema. List, search, create, edit, delete — with navigation, module gating, and translations. |
| **Email**           | SMTP-based. Transactional emails (welcome, password reset) and a full email module with templates and resend.               |
| **File uploads**    | S3-compatible storage (MinIO, R2, AWS S3) with local-filesystem fallback.                                                   |
| **Background work** | Temporal.io integration for scheduled tasks, email queues, and long-running processes.                                      |

You add these piece by piece, in whatever order your project needs them. The static pages that don't need auth don't change — they render the same way they always did.

<a name="how-the-templates-differ"></a>

## How the Templates Differ

The `.ree` syntax is identical, and so is the data prefix — both reference render data through `props.`:

```html
<!-- Reeweb template -->
<h1>{= props.title }</h1>
<p>{~ props.body }</p>

<!-- Reepolee template — same prefix -->
<h1>{= props.title }</h1>
<p>{~ props.body }</p>
```

The only difference is _what's available_ in the render context. Reepolee templates have access to additional globals that Reeweb doesn't provide — the current user, toast notifications, flash messages, the authenticated session, language-detection metadata:

```html
{#if props.user }
<p>Welcome back, {= props.user.display_name }.</p>
{/if } {#if props.toasts }
<toasts-area>
	{#each props.toasts as toast }
	<div class="toast">{= toast.message }</div>
	{/each }
</toasts-area>
{/if }
```

These are values that only make sense in a server-side context. In Reeweb, there is no user session to read from, no toast to flash after a redirect — so those keys simply don't exist in the render data.

Components are invoked the **same way** in both environments — they share one engine. A component is a custom element whose attributes arrive under `props.attributes` and whose slot content arrives as `props.children`:

```html
<my-h1 class="text-2xl">Title</my-h1>
```

Pass dynamic, evaluated data through interpolated attributes (`label="{= props.fields.email.label }"`), which the component reads from `props.attributes`. (The older `{@componentName({...})}` function shorthand has been removed from the engine — see [Components](/ree-templates/components).) The only thing that differs between Reeweb and Reepolee here is the **data** available to render, not the syntax: Reepolee injects server-side values like `props.user` and `props.toasts` that simply don't exist in Reeweb's static render.

<a name="not-migration-growth"></a>

## Not Migration — Growth

The word "migration" suggests a one-time event with downtime and risk. That is not the model here.

Reeweb and Reepolee are part of the same ecosystem. You can start a project in Reeweb, launch it, get traffic, learn what your users actually need, and extend the project into a Reepolee application when the requirements justify it. The code you wrote for the static site — templates, components, translations, styling — continues to work. You add layers on top of the foundation you already built.

The relationship between the two tools is not "start with this one, then switch to that one." It is "start with the one that fits today, and grow into the one that fits tomorrow."

For a direct comparison of the two tools and guidance on which fits your current project, see the [Reeweb & Reepolee](/reeweb/docs/getting-started/reeweb-and-reepolee) page.
