---
title: "Stop Using Rust for Your CRUD App"
layout: "blog"
track: "Engineering"
published_at: 2026-05-28
author: "Aleš Vaupotič"
image: "/images/responsive/hero-contact.png"
excerpt: "By a Solutions Architect with four decades in the industry — on why Bun and TypeScript beat Rust for web application work, and what LLMs have to do with it."
description: "By a Solutions Architect with four decades in the industry — on why Bun and TypeScript beat Rust for web application work, and what LLMs have to do with it."
---

I've watched a lot of technology waves come and go: Basic to Pascal, DOS Pascal to Windows Delphi, desktop to web. The current one is teams reaching for Rust to build internal dashboards and CRUD apps that serve a few hundred users.

That's the wrong call. Not because Rust is bad — it's extraordinary. But an extraordinary tool used in the wrong context produces ordinary outcomes, and sometimes catastrophic ones. For ordinary web application work, Bun and TypeScript are the better bet, and the reasons compound over the life of the project.

---

## One Language, Everywhere

With Bun and TypeScript you write one language across the whole stack: server logic, database queries, the API layer, frontend components, build scripts, test runners, CI pipelines. You get strong typing, modern tooling, and isomorphic code without asking anyone to carry two or three separate mental models at once.

Rust on the backend means your frontend engineers can't read your server code. That gap is invisible on the org chart and very visible the moment something breaks at the boundary between them.

## The Knowledge Base Is Enormous — and That Matters

When someone on your team hits a confusing async boundary in TypeScript, an answer is minutes away: a StackOverflow thread, a Bun issue, an MDN page. The equivalent wall in Rust — lifetime errors, pin projections, async trait objects — sends them into RFCs and forum threads that assume expertise they don't yet have. The language isn't harder for the sake of being harder; it's solving problems your CRUD app doesn't have, and your team pays the learning cost anyway.

## Bun Eliminates the Build Step

You run TypeScript directly. No `tsc` compile step, no Webpack, no Vite config to debug. You write a `.ts` file, run it with `bun run`, and it works. On a project that several developers will touch over several years, zero build tooling is not a convenience — it's a maintenance advantage that keeps paying out long after launch.

---

## A Word on LLMs

LLMs are very good at generating TypeScript and JavaScript. The training data is vast, the patterns are well established, and a prompt for a Bun HTTP handler usually comes back as coherent, idiomatic, immediately runnable code.

The catch is review. Generated code still has to be read, understood, and owned by whoever ships it, and that means knowing the language it's written in. Excellent LLM-generated Rust is still a liability when nobody on the team can confidently own it. TypeScript keeps the developer in the seat: the output is readable by everyone, fast to correct, and fast to reason about under pressure — which is exactly when you need to.

---

## The Takeaway

Cognitive overhead kills more projects than slow runtimes do. Choose the language your whole team can read at midnight, on a stack with four decades of accumulated knowledge and the largest developer community in the world behind it. One language, no build step, shared types end to end, and LLM output your team can actually audit.

In the long run, the boring choice wins.
