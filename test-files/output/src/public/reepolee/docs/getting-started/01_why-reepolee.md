---
title: "Why Reepolee"
---

# Why Reepolee

<a name="a-letter-from-the-founders"></a>

## A Letter from the Founders

We've been building web applications professionally for a long time. For most of that time, we used the same tools everyone else used — Node.js, a framework of the month, a bundler that needed its own configuration file, and a `node_modules` folder that weighed more than the application it served.

It worked. Until it didn't.

<a name="the-breaking-change-problem"></a>

## The Breaking-Change Problem

Every few months, something would break. Not because we wrote bad code — but because a dependency we didn't directly control decided to change its API, drop support for something, or simply vanish from the registry. We'd open a project that was running fine six months ago and spend a day untangling peer dependency conflicts before writing a single line of business logic.

We got tired of it.

The real cost isn't the hour you spend fixing the breakage. It's the anxiety that builds up every time you `npm install` on a project you haven't touched in a while. It's the mental overhead of tracking which version of which package is compatible with which other package. It's the pull requests sitting open for months in upstream repos while your production app waits.

We wanted to build something where opening a project after six months felt like opening it the day you left it.

<a name="building-with-llms"></a>

## Building with LLMs

AI coding assistants have become a real part of how we work — but the JavaScript ecosystem was not built with them in mind, and the friction shows.

LLMs are trained on snapshots of the world. They don't have real-time access to changelog entries or release notes, which means they confidently suggest patterns that were best practice two years ago. This wouldn't be such a problem if we wrote vanilla JavaScript, which LLMs understand extremely well. The trouble is that we spent a decade abstracting everything into frameworks — and those abstractions have their own APIs, their own conventions, and their own version histories that drift faster than training data can track.

The deeper problem is what happens when something goes wrong in a dependency. Sometimes the fix is genuinely simple — a one-line change in a library file. But an LLM trained on open-source norms knows not to touch `node_modules`. So it codes around the issue instead: a workaround on top of a workaround, none of which address the root cause. You can report the bug upstream, but you have no say in when the maintainer reviews it, when the PR gets merged, or when a release ships. You are accountable for your application's stability, but you are not in control of all the variables.

Dependencies are one layer of that. External APIs are another. CDN outages, database provider incidents, certificate renewals — the surface area of things that can break your application without you writing a single bad line of code is enormous once you start counting.

We can't eliminate all of that. But we can shrink the part we own. Reepolee has zero runtime dependencies. Every file in your project is a file you can read, change, and reason about — including with an AI assistant. When you hand your codebase over, it sees your routes, your templates, your queries, your business logic. Nothing else.

<a name="database-first"></a>

## Database First

Most frameworks make you describe your data twice: once in the database and once in the application layer. You write a migration, then a model, then a schema, then validation. They drift apart over time, and you spend energy keeping them in sync.

We believed the database should be the single source of truth. Reepolee's generator introspects your actual tables and produces everything from that — typed column definitions, CRUD handlers, forms, validation, translations. Change a column in the database, regenerate, and the rest follows. The schema lives where it belongs.

<a name="simplicity-as-a-feature"></a>

## Simplicity as a Feature

We also wanted something simple to deploy. Not "simple if you set up a container orchestrator" simple. Actually simple.

A Reepolee application is a single Bun process. You copy the files to a server, create a systemd service, and run `bun start`. There is no build step for the server, no transpilation pipeline to configure, no separate process to manage for CSS in production. Bun handles it all. We've included the service files in the repository because there should be nothing to figure out.

<a name="runs-on-anything-deploys-anywhere"></a>

## Runs on Anything, Deploys Anywhere

Because Reepolee is just a Bun process, it runs wherever Bun runs — Linux, macOS, Windows, x86, ARM. We've shipped Reepolee apps to virtual private servers, to a Raspberry Pi sitting on a desk for staging, and to a Mac Mini under a desk doing double duty as the office's internal back-office server. Same codebase, same `bun start`. No vendor-specific build step, no platform adapter, no managed-runtime quirks to learn before your code can run.

That last setup — the Mac Mini on the LAN — matters more than it sounds. When the back-office tool lives on a machine on the same network as the people using it, an outage of the office's external internet connection doesn't take the business down with it. Orders still get entered, invoices still print, the warehouse still ships. The internet is convenience; the LAN is plumbing.

You can mix targets too. Staging on a Pi, production on a VPS, a local mirror on a Mac Mini for when the line goes down — the infrastructure decision can follow the constraints of the project instead of dictating them.

<a name="built-to-pivot"></a>

## Built to Pivot

Clients change direction. A back-office tool we built six months ago turns into a request for a customer-facing eCommerce site on top of it. A simple static page, originally optimized for CDN delivery without a CMS, suddenly needs login, a contact form, and an admin panel to edit the copy. The use case shifts, and the work that came before is expected to come along for the ride.

Time and time again, we'd have to tell clients that a lot of our work would have to be trashed — not because the code was wrong, but because the stack we'd picked for the original use case couldn't bend to the new one. Static-site generators can't grow an application layer without a rewrite. Frameworks built for serverless can't let users write to a local filesystem — there isn't one — so file uploads become another service to wire up (and likely one outside of your control). Every pivot meant a new framework, a new vocabulary, and a new bill.

With Reepolee, this is no longer the case. The same project can render static marketing pages with aggressive caching, host an internal admin panel, run authenticated multi-tenant features, and sit behind a CDN as a mostly-static site that occasionally talks to a database. You add what you need; you don't pay for what you don't. One project, one process, one mental model — bent to whatever the next conversation with the client demands.

<a name="the-stack-we-settled-on"></a>

## The Stack We Settled On

<media-frame label="Diagram — the Reepolee stack (Bun · Ree · SQL)" ratio="16/9"></media-frame>

We made deliberate choices at every layer and we've tried hard not to accumulate tools. Server-side rendering with `.ree` templates keeps the rendering model simple and the HTML in your hands. Fine-grained reactivity with alien-deepsignals adds the interactivity you need on the client without a build pipeline. Tailwind v4 handles styling without asking you to make decisions. Bun's native `SQL` API connects to SQLite or MySQL without an ORM in between.

None of these are novel choices. That's the point. Proven tools with stable APIs, assembled into something that feels coherent rather than bolted together.

<a name="who-reepolee-is-for"></a>

## Who Reepolee Is For

Reepolee is for developers who want to build and ship products, not maintain infrastructure. It's for teams that have been burned by the churn of the JavaScript ecosystem and want something that will still work the same way in two years. It's for anyone who has stared at a blank `node_modules` folder and thought — there has to be a better way.

We built Reepolee because we needed it. We hope you find it useful too.

— The Reepolee Team
