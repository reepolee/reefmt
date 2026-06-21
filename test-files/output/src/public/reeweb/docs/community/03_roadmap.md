---
title: "Roadmap"
layout: "reeweb/docs/docs.layout"
---

# Roadmap

<a name="introduction"></a>

## Introduction

This page tracks the larger initiatives in flight or on the horizon. Smaller fixes and incremental improvements happen continuously and don't get individual entries here.

Reeweb ships when things are ready, not on a fixed schedule. Items below are organised by horizon (next release, this year, longer term) rather than by date.

<a name="next-release"></a>

## Next Release (0.2.x)

Items being worked on for the next minor version.

### Full-text search for documentation

The current search modal is a placeholder. The next release will include client-side search powered by a pre-built index.

### Plugin system for template extensions

Allow third-party `.ree` component libraries to be installed via npm and auto-discovered by the template engine.

<a name="this-year"></a>

## This Year (2026)

### Expanded documentation

Cookbook recipes, migration guides, and a broader set of examples covering real-world static site patterns.

### Starter templates

Ready-to-use project templates for common site types: blog, documentation, landing page, multi-language corporate site.

<a name="longer-term"></a>

## Longer Term

Items that are real ambitions but not actively in flight.

### Reeweb 1.0

The 1.0 release is when the public surface is stable enough that breaking changes become rare and explicit.

### Visual editor

A drag-and-drop page builder that outputs `.ree` templates — making Reeweb accessible to non-developers while keeping the output developer-friendly.

<a name="not-on-the-roadmap"></a>

## Not on the Roadmap

A few things we don't currently plan to add:

- **A database or authentication layer.** That's what Reepolee is for — Reeweb is deliberately static.
- **A client-side framework.** The output is HTML; add your own JS as needed.
- **Plugin system / extensions.** Reeweb's extension model is "edit the file."

<a name="influencing-the-roadmap"></a>

## Influencing the Roadmap

The single most useful input we get is "I tried to do X and it was awkward in this specific way." Concrete pain is what moves items up the priority list.

The forum for this is [GitHub Discussions](https://github.com/reepolee/reeweb/discussions).
