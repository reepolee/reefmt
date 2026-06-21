---
title: "Your Database Is Probably Permanent. Act Like It."
layout: "blog"
track: "Engineering"
published_at: 2026-06-12
author: "Aleš Vaupotič"
image: "/images/responsive/hero-1.png"
excerpt: "Sorting in memory, aggregating in code, splitting one query into five — all to preserve a database migration that never happened. A case for committing to your database fully and using everything it offers."
description: "Why most teams never replace their primary database engine, and why designing for portability costs more than it returns. Covers stored procedures, views, triggers, DDL tooling, codegen as an ORM alternative, and the generator pattern behind Reepolee."
---

There is a piece of architectural advice that circulates through engineering teams like received wisdom:

> "Design your application so you can swap databases at any time."

It sounds responsible and forward-thinking. For the vast majority of software projects it is a comfortable fiction — one that quietly consumes more engineering effort than it ever returns. And here is the part that gets missed whenever the advice comes up: **a system nobody can change without three layers of indirection isn't well-designed, it's just well-defended.** Early abstraction adds friction to the work that has to ship today, and that friction compounds long before any theoretical future benefit arrives. You pay for the option every day, in complexity, in slower delivery, and in features your database would have given you for free if you had let it.

The decisions you make this quarter shape your life as a maintainer years from now, so the long game is worth understanding first.

---

> ### 💡 Optimize for confidence, not optionality
>
> The worst database decisions I've seen came in two flavors: teams who picked something weak because they assumed migration was always a fallback, and teams who hobbled themselves avoiding powerful features out of a lock-in fear that never materialized. Evaluate seriously up front, write down the reasoning, then commit and use the thing fully. A few days of honest evaluation at the start is worth more than months of remediation years later.

---

## How rarely do companies actually switch?

What follows is drawn from experience across many organizations rather than controlled study — and I want to be transparent about that, because the honest framing matters here. These are patterns, not statistics.

| Scenario                                                                         | How often it happens      |
| -------------------------------------------------------------------------------- | ------------------------- |
| Greenfield project stays on its original database indefinitely                   | The overwhelming majority |
| Major version upgrades (e.g., PostgreSQL 14 → 17)                                | Very common               |
| Moving from self-hosted to a managed cloud service                               | Common                    |
| Migrating within the same database family (e.g., MySQL → MariaDB)                | Occasional                |
| Replacing the relational engine entirely (e.g., Oracle → PostgreSQL)             | Uncommon                  |
| Application genuinely designed so the database is swappable with no code changes | Extremely rare            |

When large organizations do replace their database engine, it is almost never because the team found something more attractive. It happens because of licensing costs, acquisitions, end-of-life products, or platform decisions made at the executive level — and it unfolds over months or years, with dedicated migration teams, parallel production runs, and staged cutovers. The application code is usually the easiest part.

There's an obvious counter-argument: maybe companies don't switch because the abstraction worked so well nobody dared try. Sometimes, perhaps. But it doesn't change the math, because the teams carrying that abstraction overhead pay for it every single day whether or not the migration ever comes.

---

## Why the database becomes load-bearing infrastructure

Here is how it tends to go. On one project I worked on, the team committed to multi-database support from day one, so every query was held to the lowest common denominator: basic selects, simple joins, nothing the ORM couldn't generate identically for any backend. Anything harder got pushed up into the application layer. Sorting happened in memory. Aggregations were assembled in code. Queries the database could have resolved in a single round trip became five simple ones stitched together in the service layer, because a single expressive query felt like too much of a commitment. The database was never switched. The code that grew up around those constraints outlived their rationale by years, and every new engineer had to be told why things were done the hard way.

This happens because the database does not stay a data store. Over time it becomes embedded in almost every layer of the system:

- SQL queries and the optimizations built around their actual execution plans in production
- Index design that emerged from real query patterns observed after launch, not from the schema diagram
- Stored procedures and native functions that encapsulate business logic added under time pressure
- Transaction and locking semantics that application behavior has quietly come to depend on
- Extensions that are genuinely difficult to replicate — PostgreSQL's `JSONB`, `GIN` indexes, `pg_trgm` for fuzzy search
- ORM dialect assumptions baked into hundreds of queries across years of releases — and this is before you question whether the ORM was the right call in the first place
- Migration tooling and accumulated schema history that documents decisions nobody fully remembers

Even teams that start with clean ORM abstractions get pulled toward database-specific features over time — not out of carelessness, but because those features solve real problems and reduce complexity elsewhere. Using a capable tool well is not a failure of discipline.

---

## When portability actually matters

Before going further, this argument deserves a real boundary — because there are situations where supporting multiple databases is a genuine engineering requirement and not an architectural indulgence.

If you are building an **open-source library or framework**, your users run whatever database their organization already has. Supporting PostgreSQL and MySQL is not over-engineering; it is the product.

If you are building **commercial software distributed to thousands of enterprise customers**, those customers may be contractually tied to Oracle or SQL Server before you ever speak to them. Portability is a sales requirement.

If you are building a **developer tool or ORM itself**, then obviously the point is abstraction.

Outside these specific cases — and for the vast majority of product teams building software for their own users on infrastructure they control — the portability argument tends to collapse under the weight of its actual costs.

---

## Do your due diligence before you commit

The only moment when thinking carefully about database portability pays meaningful dividends is **before you make the initial choice** — not after you have built half a system on assumptions you are now second-guessing.

That initial evaluation deserves genuine investment. Rather than generic advice about "assessing ecosystem maturity," here are the specific questions worth answering before you commit:

**Can your team debug a slow query plan without looking it up?** Operational familiarity under pressure matters more than benchmark numbers. A database your team knows well will outperform a theoretically superior one that nobody can tune.

**Does the database have a managed cloud offering from your preferred provider?** Self-hosting a database well is a full-time operational responsibility. Knowing whether you can hand that off — and to whom — affects your total cost of ownership for years.

**What does an emergency failover look like?** Walk through the actual procedure before you need it. If the answer is unclear or requires the vendor's professional services team, that is useful information.

**Has the technology absorbed your expected scale in comparable production environments?** Case studies from organizations at roughly your order of magnitude are worth more than synthetic benchmarks.

Notice that most of these questions are about your team, not the database. A technically superior engine is the wrong choice if nobody can operate, tune, or troubleshoot it at two in the morning. The best database for you is the intersection of what fits the use case and what your team can actually run.

---

## Once a system is battle-tested, change becomes a business risk — not a technical one

Architects learn this through experience rather than textbooks: once a system has run reliably long enough to accumulate real operational history, very few people anywhere in the organization are eager to replace one of its core components — and their reluctance is well-founded, not timid. By that point the database is not a line in an architecture document. It is part of a system that has survived real traffic, strange edge cases, incidents, compliance audits, and years of accumulated knowledge about how it behaves under load. It has earned institutional trust that no benchmark can replicate.

Replacing it means spreading risk across the whole organization at once. Engineering managers worry about downtime windows, operations about data integrity during migration, product owners about feature delivery stalling for months, executives about business continuity and regulatory exposure. Even when migration promises real long-term benefits, it competes directly with shipping customer value, and the case for disrupting a system that is doing its job rarely becomes compelling enough to act on. A database that has quietly carried the business for five or eight years is an asset in its own right — and that reliability is easy to underestimate right up until it's gone.

---

## The database is not just storage — use all of it

The conventional advice says business logic belongs in the application layer and the database should be a dumb store. That advice is worth questioning seriously, because some of the most reliable and performant systems I have worked on treat that boundary very differently.

Stored procedures, views, triggers, and functions are not legacy artifacts or architectural compromises from a less enlightened era. They are first-class tools that the database has been optimizing for decades, and they offer things the application layer cannot replicate without significant cost:

**Stored procedures** execute logic where the data already lives, eliminating round trips and running inside the transaction boundary automatically — no extra infrastructure, no coordination overhead.

**Views** create a stable abstraction layer within the database itself — one that can be versioned, permissioned, and composed without changing application code or redeploying anything.

**Triggers** enforce data integrity and reactive behavior at the storage layer, where it cannot be bypassed by application bugs, direct database access, migration scripts run by a junior engineer at midnight, or any other path that skips the application entirely.

**Functions** allow complex logic to be composed, reused, and even tested directly against the database without touching a deployment pipeline.

The usual objection is that these live inside the database, invisible to anyone reading the application code, unreviewable in pull requests, undiscoverable unless you already know to look. That's valid, and it's a solved problem. DDL migration tools let you define every procedure, view, trigger, and function as a plain SQL file committed to the repository, applied in order, and tracked in history. Your database objects get code review, git blame, and a change log like everything else, and the database state at any point becomes reproducible from source — which quietly fixes onboarding, local development, and environment parity too.

So if your database is permanent, treating it as a passive key-value store leaves performance, consistency, and reliability on the table. Data locality alone eliminates whole classes of latency problems that application-layer architectures then spend months solving with caching layers, background jobs, and eventual-consistency models that bring their own failure modes. And once your business logic lives in procedures and functions, the ORM portability question answers itself — you probably didn't need the ORM at all.

---

> ### 💡 Push all your logic into the app layer, and you've made an architectural choice — usually without noticing
>
> Stored procedures, views, triggers, and functions carry decades of optimization. Logic that runs where the data lives gets atomicity, performance, and consistency with no extra infrastructure. Decide consciously what the database should own, and it tends to earn its keep far more than the conventional wisdom suggests.

---

## Generate for the database you chose — not for every database

Once you have genuinely committed to a database and started using its native capabilities, a fair question emerges: how do you keep the application code that talks to it clean and maintainable without reaching for an ORM?

The generator pattern is the answer I keep coming back to. Instead of writing a single data access layer that tries to work across multiple databases at runtime, you generate application code that is specifically and deliberately written for the database you chose. That code lives in your repository, reads like normal application code, and goes through code review like everything else — but it is shaped around your actual database rather than a generic interface designed to paper over the differences between five of them.

The abstraction is real. It just happens at generation time instead of at runtime. You write a definition of what your application needs; the generator produces typed, database-specific code that does exactly that — without the overhead, the magic methods, or the opaque query generation that makes debugging an ORM a special kind of miserable. You can read every line of what runs against your database, because you generated it.

This is the architectural idea behind [Reepolee](/reepolee) — a codegen starter that takes your schema and generates the application-side data access layer tailored to your database of choice. No ORM, no runtime abstraction, no portability theater. The generated code knows exactly which database it is talking to and is written to use that knowledge, not hide it.

---

## Use the full capability of the tool you chose

Choosing a mature, capable database and then deliberately avoiding its strengths to preserve a theoretical portability option is like buying a precision instrument and restricting yourself to what a cheaper general-purpose tool could have done. It is not caution. It is waste dressed up as discipline.

Good engineering means using the capabilities of your tools in ways that are intentional, understood, and justified — while avoiding coupling that delivers no practical benefit. There is a meaningful difference between over-specialization, where application logic becomes entangled with database internals unnecessarily, and simply using native features that your database handles better than any abstraction layer can simulate.

The goal is not a system that pretends to have no dependencies. The goal is a system whose dependencies are chosen deliberately, documented honestly, and used to their full potential.

---

## The question worth asking

The question is not whether your application _can_ switch databases.

The better question is whether the cost of preserving that flexibility today — in complexity, in avoided features, in slower delivery, in daily friction — is genuinely justified by the realistic probability that you will actually need to use it.

In most product teams, that question answers itself over time. The system matures. The database earns its place by surviving real conditions. The migration that was always theoretically possible becomes something nobody with actual context in the organization has any real interest in attempting — because the business case for it never becomes specific enough to act on.

Choose your database carefully. Ask the hard questions since kick-off. Use it fully once you decide. And stop optimizing for a migration that, in all likelihood, will never happen.

Commit.
