---
title: "Contributing"
---

# Contributing

<a name="introduction"></a>

## Introduction

Reepolee is open source, but it is not open contribution. The source is published so you can read it, audit it, learn from it, and fork it — not so that the maintainers' time is spent triaging external pull requests.

The reason is simple. We want to spend as much time as humanly possible building Reepolee itself, and the most reliable way to do that is to keep the surface area we owe to the outside world small. A framework that drifts under the weight of community patches becomes unstable; a framework with a small, deliberate group of authors stays coherent. We are optimising for the second outcome.

The repository is at [github.com/reepolee/reepolee](https://github.com/reepolee/reepolee). The documentation site is at [github.com/reepolee/docs.reepolee.com](https://github.com/reepolee/docs.reepolee.com).

<a name="no-issue-tracker"></a>

## There Is No Issue Tracker

Reepolee does not have a public issue tracker you can freely post to. Bug reports, feature requests, and "this should be different" tickets do not have a queue to land in.

We know this is unusual. The trade-off is honest about what an issue tracker actually demands of the people running it — constant triage, repeated context, and a backlog that grows faster than it shrinks. Removing it gives us the room to build.

<a name="discussions"></a>

## Open a Discussion First

The minimum requirement for putting something in front of us is a [GitHub Discussion](https://github.com/reepolee/reepolee/discussions). That is the only channel.

A useful discussion looks like:

- **A concrete problem you've actually run into**, not a hypothetical. "I tried to do X, the result was Y, I expected Z" beats "Reepolee should support X."
- **The smallest reproduction you can manage** when the topic is a bug — ideally a single route or template, not a full project.
- **Your Reepolee and Bun versions** — the `version` field in `package.json` and the output of `bun --version`.
- **What you've already tried**, so the thread doesn't retread ground you've covered.

If we find your input valuable, we will promote the discussion into an item on our roadmap. That is how something becomes work we commit to. A discussion that doesn't get promoted is not a rejection of you — it is a statement that the issue, as framed, isn't something we plan to take on. You are welcome to keep using the discussion thread to talk it through with other readers.

<a name="fork-and-modify"></a>

## Fork and Modify Freely

Reepolee is structured so that every line of code lives in your project's user land. There is no `node_modules/reepolee` to monkey-patch around. The `lib/`, `routes/`, `components/`, and `config/` directories are yours from the moment you clone the starter.

This is the part of the design that makes the closed-contribution model workable. If you need behaviour Reepolee doesn't ship — a different session encoder, a custom queue driver, a route convention that suits your team — you do not need our permission, our review cycle, or our roadmap. You change the file. The change is part of your application from then on, and it stays under your control on your timeline, not ours.

The implication is that "Reepolee doesn't do X" is rarely a blocker for you. It is a blocker for the next person who wants X to be there by default, which is what discussions and the roadmap are for.

<a name="security-issues"></a>

## Security Issues

Security reports are the one exception to the no-public-channel rule. Email security@reepolee.com directly rather than opening a discussion. We will acknowledge within a few days and coordinate disclosure privately.

<a name="building-and-running-locally"></a>

## Building and Running Locally

If you want to read, audit, or fork the framework itself:

```bash
git clone https://github.com/reepolee/reepolee.git
cd reepolee
bun install
cp config/db_sqlite.ts config/db.ts
cp .env.example .env
bun dev
```

The dev server runs on `http://localhost:2338`. Edit any file under `lib/`, `routes/`, or `components/` and hot-reload picks up the change.

For the documentation site:

```bash
git clone https://github.com/reepolee/docs.reepolee.com.git
cd docs.reepolee.com
bun install
bun run index.ts
```

The docs site uses the same default port — run them on different ports if you have both open at once.

<a name="license"></a>

## License

The repository's BSD-style license covers what you can do with the code: fork it, modify it, ship it inside your own product. There is no CLA, no copyright assignment, and no obligation to send anything back upstream. Your fork is your fork.
