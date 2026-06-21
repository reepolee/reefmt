---
title: "Quality Doesn't Matter ... Until It Does"
layout: "blog"
track: "For decision-makers"
published_at: 2026-06-16
author: "Aleš Vaupotič"
image: "/images/responsive/hero-2.png"
excerpt: "When a social app's like count is off by a hundred, nobody cares. When your invoice goes out for the wrong amount, everyone cares — loudly and expensively. On a decade of velocity culture, diluted craft standards, and AI as the latest accelerant."
description: "Why quality standards must differ between social apps and enterprise software — and how investor pressure, low barriers to entry, and AI are driving a visible decline in the software that actually runs businesses."
---

There is a conversation happening right now in every engineering team, every startup, every enterprise boardroom, and it goes something like: _"Just use AI to write the code, ship it faster, iterate later."_ I understand the appeal. If you've been around long enough, you've watched silver bullets come and go — CASE tools in the eighties, RAD in the nineties, agile as a magic word in the two-thousands — and every time, the industry convinced itself the new thing would let us move faster without paying for it. We're doing it again, and this time the stakes are higher for a certain class of applications.

Let me be direct about what I'm seeing: the quality of software being shipped has been visibly, measurably, embarrassingly declining for the better part of a decade. AI did not cause that. It is the latest and most powerful accelerant of something already well underway.

The real drivers started earlier. A long run of cheap capital and investor appetite for hypergrowth created enormous pressure to ship fast and grow at any cost, with quality treated as something to worry about after the next funding round. At the same time, the barrier to entering the industry dropped to nearly zero — not a bad thing in itself, but it meant large numbers of people began writing production code without the foundational education or apprenticeship this craft historically required. It became easy and cheap to declare yourself a developer, get hired as one, and ship code into systems real businesses depend on, without ever being seriously exposed to the idea that correctness is a discipline rather than a feeling.

> **Correctness is a discipline, not a feeling — and a surprisingly large share of the people now writing production code have never been asked to develop it.**

Velocity pressure from above and diluted craft standards from within had already been eroding quality for years before any AI tool arrived. AI took those conditions and added rocket fuel, applying the same pressure and the same reduced standards to a process that produces ten times the code in the same time. The volume of unreviewed, under-tested, contextually inappropriate code being merged into production has climbed sharply, and the consequences are starting to show. We handed a genuinely useful tool to an industry that was already cutting too many corners, and told everyone the output was good enough to ship.

---

## The Low-Stakes Majority Nobody Talks About

Here is something the tech press rarely acknowledges: a very large portion of the web runs under a completely different failure model than enterprise and business software, and almost everything we learned to accept about software quality in the last fifteen years came from watching that portion and wrongly generalizing its lessons to domains where they don't belong.

The examples are everywhere once you look. On **social media**, a like count that's wrong for thirty seconds, a dropped notification, a timeline that skips three hours — someone shrugs and refreshes. On a **marketing or landing page**, a slow hero image or a typo in a form's error message gets noticed Monday and fixed Tuesday, and the world continues. On **content and publishing sites**, a missing image or a stale search result sends the reader to another tab; no data is lost, no obligation broken. **Streaming platforms** ship hundreds of A/B variants knowing some won't work perfectly, because shipping fast and fixing fast genuinely beats getting it right the first time — for them. And in **gamification** — leaderboards, badges, streak counters — a badge that shows up an hour late is, at worst, mildly annoying.

What do these categories have in common? They deal in **attention and engagement**, not **transactions and commitments**. The user is browsing, reacting, exploring. Nothing they do creates a binding obligation, transfers real money, alters a legal record, or changes the physical stock in a warehouse. When something is wrong, they ignore it, refresh, or wait, and the wrongness becomes irrelevant.

> **The defining question is not how many users you have. It is whether your system deals in attention and engagement, or in transactions and commitments. That distinction should drive almost every architectural and quality decision you make.**

This is not an accident of culture. These platforms were designed from the start around the idea that approximate correctness is acceptable, because for them it genuinely is. The CAP theorem debates, the BASE-versus-ACID wars, eventual consistency as a philosophy — all of it emerged from large-scale social and e-commerce platforms solving problems at a scale where perfect consistency was impossible and inconsistency was tolerable. They built brilliant systems for their problems. Then the rest of the industry looked at what they built, saw that it was fast and scalable and used cool technology, and decided everyone should build that way.

And now here we are, using AI to generate code that inherits all of those assumptions — "ship it and fix it later," "move fast and break things," "good enough is good enough" — and deploying it into contexts where those assumptions are catastrophically wrong.

---

## What Happens When a Business Application Drops the Ball

Let me tell you what "dropping the ball" looks like in the applications that actually run the economy — because a lot of younger developers have never had to sit across a table from a furious CFO and explain why the software did something wrong.

An invoice goes out for the wrong amount. Somewhere in the payment logic, a rounding function behaved differently in a currency-conversion edge case the generated code never considered, because the model was trained on millions of examples that mostly worked and had no reason to know your business has contractual rounding rules negotiated with specific large clients. The client notices. They call. Now you have a dispute that takes three weeks to resolve, and during those weeks the relationship frays, the account manager is stressed, finance is reconciling by hand, and legal is reading the contract for a penalty clause — all because one line rounded the wrong way.

Or consider inventory. A warehouse system has a subtle concurrency bug in its stock-reservation logic — nothing exotic, a classic race condition. Two orders land at almost the same instant for the last unit. Both succeed. Both customers get a confirmation. You've oversold by one, which means one customer won't receive what they paid for. They get angry, they leave a review, they may never come back. At scale — and these bugs do happen at scale — the financial and reputational damage is real and lasting.

Or healthcare, where I have colleagues working on electronic health records and clinical decision support. I won't detail what "dropping the ball" means there, because the consequences run from regulatory penalties to patient harm, and anyone reasonable understands that a wrong drug-dosage recommendation is not the same category of problem as a wrong movie recommendation.

These aren't theoretical. They're the failures I've watched happen throughout my career, long before AI — but AI has sharply accelerated the conditions that make them likely, because it makes it so easy to produce code that looks correct, passes casual review, and behaves correctly in ninety-nine percent of cases while failing catastrophically in the one percent that matters most.

---

## How AI and LLMs Are Reshaping (Degrading?) Software Quality

I want to be fair here, because I am not one of those people who thinks AI tools are useless or that we should pretend they do not exist. They are genuinely useful. I use them myself. The problem is not the tools — the problem is how we are integrating them into our development culture and what assumptions we are implicitly accepting when we do.

Large language models are, by their nature, pattern-matching systems trained on the aggregate of existing code. That makes them very good at producing code that looks like the code most commonly written for a problem, and much less reliable when the correct solution is uncommon: when the domain has unusual constraints that rarely appear in training data, when the edge cases are subtle and domain-specific, or when correctness depends on business rules that live only in someone's head or in a contract that was never digitized.

The whole value of custom business software over off-the-shelf is that it encodes those specifics — the exceptions, the special cases, the regulatory requirements, the rules accumulated over years of operation. And that is exactly where the model is weakest.

> **AI generates plausible median code — correct for the most common version of your problem. Business software is defined precisely by its exceptions. That is the gap.**

There is also a deeper cultural problem. Because AI makes code so cheap to produce, the pressure — from management, from investors, from the competitive environment — is to use that speed without paying for it with proportional increases in review rigor, testing depth, and architectural thinking. Velocity goes up; the quality gates don't. More code ships faster with less scrutiny, and even if the defect rate per line holds steady, the defect rate per unit of time climbs because so many more lines are shipping.

I watch teams slide from carefully reviewing every pull request to merging on a glance, and it is genuinely alarming when it happens in codebases that process financial transactions or manage medical records. Peer review, for all its friction, is one of the most important quality mechanisms we have, and we are quietly dismantling it in the name of velocity.

> **"AI wrote it, it looks fine, merge it" is not a code review. It is an abdication of one — and in high-stakes software, the consequences eventually become very visible.**

---

## There Is a Right Kind of Code Generation, and It Is Not AI

Here is something that gets lost in the current AI enthusiasm, and it matters enormously for high-stakes software: code generation has existed for decades, it is genuinely useful, and the good kind of it works on completely different principles than what LLMs do. Conflating the two is one of the more consequential category errors that our industry is currently making.

Consider what a proper domain-aware generator does. You give it your database schema — the real structure of your data, with your naming conventions, your column types, your relationships and constraints — and it produces, deterministically, a full stack of consistent artifacts: route handlers, SQL queries, form templates, validation schemas, translation keys, type definitions. Same input, same output, every time. No probability distribution, no sampling, no plausible median — just your schema in and correct, consistent code out, according to rules you wrote, reviewed, and trust because you understand them completely.

The key word is consistent. In a business application, inconsistency is a silent killer. When ten developers over three years each write their own version of the CRUD pattern, you get ten approaches to validation, ten ways of handling database errors, ten conventions for delete confirmation, ten assumptions about what a half-failed form submission does. None of these is catastrophic alone. Together they produce a codebase that is impossible to reason about completely, impossible to audit confidently, and full of edge-case variations that only surface under production conditions nobody tested.

A well-designed generator eliminates that entire class of problem by construction. You review the generator once, as a team — its patterns, its SQL concurrency handling, its validation, its error handling — and every resource it produces inherits those approved patterns identically. Find a bug in the pattern, a rounding issue in a currency field, a missing uniqueness check, a transaction that should be atomic and isn't, and you fix it in one place and regenerate. With AI-generated code, that same logical bug shows up in slightly different forms across dozens of files, each found and fixed by hand.

This is precisely the kind of system I've been building for years: introspect the schema as the single source of truth, and from it flows the route handlers, the SQL layer, the form templates, the server-side validation, the translation files, even the navigation. The generator knows a decimal column in a financial context needs specific handling, that a foreign key implies particular join patterns, that a timestamp has display and filtering requirements. That knowledge is reviewed once and applied everywhere. It's dramatically faster than writing by hand without trading away correctness, because correctness is baked into the generator rather than left to a model's probabilistic judgment at generation time.

That is the distinction I want clearly understood: _automated generation_ is deterministic, auditable, and an executable expression of your architectural decisions; _AI generation_ is a probabilistic best guess at what code usually looks like for a problem that superficially resembles yours. For the boilerplate of business software — and there is a lot of it — automated generation is almost always the better choice. Faster than hand-writing, more consistent than AI, and the review burden sits on the generator instead of being spread across every file it produces.

---

## The False Equivalence of "All Software Is the Same"

One of the most damaging moves of the last decade has been the gradual erasure of the distinctions between categories of software. We drifted into deciding — without anyone explicitly choosing it — that all software should be built, deployed, maintained, and measured the same way.

Five minutes of thought shows how wrong that is. A social media feature and a bank transfer are not the same kind of software. A feed algorithm and a medical dosing calculator are not the same kind of software. A gaming leaderboard and an inventory system are not the same kind of software. They are different artifacts with different correctness requirements, failure consequences, regulatory contexts, and user expectations.

"Move fast and break things" makes sense for an early consumer product chasing product-market fit, where the cost of breaking things is low. It makes roughly zero sense for software that processes payroll, manages supply chains, handles insurance claims, or operates industrial equipment. Taking a philosophy built for one extreme end of the risk spectrum and applying it across the whole spectrum is, in retrospect, one of the more puzzling failures of collective reasoning I've witnessed in my career.

I'm not arguing that we should slow down everywhere or reject AI tools. I'm arguing for a return to contextual judgment — the kind that says: this is a social media feature, so reasonable velocity and tolerable error rates are fine here; this processes financial transactions on behalf of businesses, so it needs different standards, different testing, different review, and a more careful use of AI assistance.

---

## The Enterprise and E-commerce Developer's Responsibility

If you work on software that handles money, manages inventory, stores medical information, calculates taxes, processes legal documents, generates invoices, or does anything else where a mistake has direct, measurable consequences for real people — I'm speaking to you, and I want you to think hard about what the current environment is asking you to accept.

You're being asked to ship faster, lean on generated code more liberally, cut review cycles, and trust that tests will catch the rest. Some of that pressure is legitimate: the competition is real, the need for velocity is real, and these tools genuinely help you do good work faster when used with care.

> **But you need to be the person in the room who remembers what the consequences of failure look like in your domain.**

The one who insists on real test coverage for financial calculations, who demands that inventory logic be reviewed by someone who understands the concurrency model, who requires that any generated code touching money or legal obligations be scrutinized with the rigor we always applied before AI existed. You push back, constructively and professionally, when the velocity being asked for is incompatible with the quality your domain requires. That isn't pessimism or Ludditism; it's professionalism. Surgeons use far better tools than they did forty years ago, and the standard for an acceptable surgical outcome did not drop because the tools improved. Neither should ours.

---

## What Good Actually Looks Like in High-Stakes Software

In the interest of being constructive rather than just critical, let me describe what I believe good practice looks like for enterprise, business, and e-commerce software development in the current AI-augmented environment.

It means drawing a clear line between the parts of your application that are structurally repetitive and the parts that encode real business logic, and treating them differently. The repetitive parts — CRUD, SQL, form templates, validation schemas, API endpoints — belong to a deterministic generator you can audit once and trust everywhere. The business logic — the pricing engine, the inventory reservation, the rules specific to your contracts and regulatory environment — is where human authorship and human review are irreplaceable, and where you should be most skeptical of AI, precisely because that's where it's most likely to be wrong about the details that matter most.

It means keeping — and probably increasing — review rigor on any code that touches financial, medical, legal, or inventory-critical paths. AI generating the code does not mean AI reviewed it. And reviewing generated code is underestimated: it is a different and often more exhausting act than writing it yourself. When you write, the thinking happens before the first line — how the piece fits the architecture, which patterns the codebase uses, which edge cases your domain requires. The code is an expression of a mental model you built in context. When you review generated code, none of that prior thinking is present; you're handed a finished artifact and asked to reconstruct the model behind it and judge whether that model fit your context — harder still because generated code tends to be more verbose than experienced human code, giving subtle incorrectness more places to hide behind reasonable-looking structure.

It means investing seriously in automated tests of business logic at exactly the boundaries and edge cases most likely to be wrong. Tests for financial calculations should cover every rounding rule, every currency combination, every contractual exception. It's tedious, and it's the only reliable way to know your code is correct.

It means building observability around correctness, not just performance. You should know immediately when invoice totals drift, when inventory counts go impossible, when transaction reconciliation fails. The faster you catch a correctness failure, the less it costs you.

And perhaps most importantly, it means organizational cultures in which raising quality concerns is respected and supported rather than treated as an obstacle to velocity. The developer who says "I think we need another week to get this financial logic right" is not being obstructionist — they are being professional. The organization that treats that developer as a problem to be overcome rather than a valuable signal to be listened to is building toward a very expensive incident.

---

## We Have Seen This Before

This is not the first time the industry has done this, and probably not the last. Every major transition produces a wave of enthusiasm in which people overgeneralize the new capability, apply it where it doesn't belong, ship things that fail, and eventually settle into a mature understanding of where it's genuinely useful and where the old disciplines still apply. It happened with object-oriented programming, client-server, the web, agile, cloud. Each time, the early enthusiasm promised the new approach would eliminate problems it could not actually eliminate, followed by painful learning, followed by a more nuanced synthesis.

We are in the middle of that process right now with AI-assisted development. The enthusiasm is high, the generalization is wide, and the painful learning has begun in some domains but has not yet reached the broader consciousness of the industry. The blog posts about AI-caused production incidents in serious business software are starting to appear. The retrospective analyses are beginning. The mature synthesis will come, as it always has.

What I am hoping — and what this post is my small contribution toward — is that the painful learning period is shorter and less expensive this time, particularly for the organizations and users who can least afford to pay the price of it. A social media company that ships a bad feature and rolls it back has lost some user engagement. A healthcare company that ships incorrect medication logic has potentially harmed patients. The asymmetry of consequences should be driving asymmetric standards, and right now, in too many places, it is not.

---

## Closing: The Price of Quality Is Not Optional

There is a tempting argument I hear often: the market will sort this out, organizations that ship bad software in high-stakes domains will lose to ones that ship better, and the incentives will pull quality up on their own. I've watched markets try to sort out software quality for years, and I'm not convinced the mechanism works reliably or quickly enough to protect the people who depend on this software to work correctly.

The quality of software that runs businesses, handles money, manages inventory, and serves patients is not just a competitive differentiator — it is a professional and ethical obligation. The developers and architects who build that software have a responsibility that goes beyond shipping fast and iterating later. We carry the responsibility of knowing what "incorrect" looks like in our domain and refusing to ship it, regardless of the pressure.

AI tools are not going away, and they should not go away — they are genuinely useful. But useful tools in the hands of professionals require the professionals to maintain their professional standards.

> **A faster saw does not lower the standards for carpentry. A faster code generator should not lower the standards for software that real businesses depend on to operate correctly.**

Quality doesn't matter — until it does. And in business software, in enterprise software, in e-commerce, in finance, in healthcare, in logistics, in any domain where incorrect software causes real harm to real people and real organizations — it always matters. It has always mattered. We just temporarily forgot that, and now it is time to remember.

---

_The author has been designing and building software systems since 1986, across domains ranging from embedded systems to enterprise architecture. The opinions expressed are their own and reflect observations accumulated over four decades of watching the industry evolve — sometimes wisely, sometimes not._
