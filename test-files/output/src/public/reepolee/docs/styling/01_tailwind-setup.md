---
title: "Tailwind Setup"
---

# Tailwind Setup

<a name="introduction"></a>

## Introduction

Reepolee uses Tailwind v4 for styling. There is no PostCSS pipeline, no plugin registry, and no JavaScript-side configuration file — Tailwind v4 reads its configuration from CSS itself. The CLI scans your source files, picks up the class names you use, and produces a CSS file with only those utilities included.

The entry point is `css/app.css`. The CLI compiles it to `static/app.css` for production and `static/app-dev.css` during development. The layout loads whichever one matches the current mode.

This page covers the setup, the build pipeline, and how to extend it. Theming, form defaults, and page transitions each have their own page in this section.

<a name="the-entry-point"></a>

## The Entry Point

`css/app.css` starts with the Tailwind import and the source-scanning directives. Everything else — design tokens, base styles, components, utilities — is layered on top in the same file:

```css
@import "tailwindcss";

@source not ".git/**/*";
@source not ".vscode/**/*";
@source not ".claude/**/*";
@source not "node_modules/**/*";

@source "routes/**/*";
@source "static/**/*";
@source "components/**/*";
@source "generator/**/*";
@source "lib/**/*";

@theme {
	/* design tokens — see Theming & Tokens */
}

@layer base {
	/* element defaults */
}

@layer components {
	/* reusable patterns */
}

@import "./forms.css";
@import "./toasts.css";

@utility primary {
	/* ... */
}
```

The `@source` directives tell Tailwind where to look for class names. Anything in `routes/`, `components/`, `static/`, and the rest is scanned; everything in `.git`, `node_modules`, and your IDE folders is skipped. New directories you add (an `emails/` folder, a `mobile/` partial set) need an explicit `@source` line so their classes make it into the output.

<a name="building-css"></a>

## Building CSS

Two scripts cover the lifecycle:

```bash
bun run css:watch    # development — rebuilds on every change
bun run css:build    # production — writes a minified bundle
```

The watcher writes to `static/app-dev.css` and stays running. The production build writes to `static/app.css` and exits.

Both call `tailwindcss` (the v4 CLI installed globally via `bun add -g @tailwindcss/cli`). The full commands behind the scripts:

```bash
tailwindcss -i ./css/app.css -o ./static/app-dev.css --watch
tailwindcss -i ./css/app.css -o ./static/app.css --minify
```

`bun dev` runs the watcher alongside the server through `concurrently`, so editing a template and seeing the new utility show up in the browser is a single change away.

<a name="dev-vs-prod-css"></a>

## How the Layout Picks the Right File

The layout template references the stylesheet conditionally based on whether the server is in dev mode:

```html
<link rel="stylesheet" href='/app{~ props.is_dev ? "-dev" : "" }.css?v={= props.version }' />
```

In development, `props.is_dev` is `true` and the layout loads `/app-dev.css`. In production it's `false` and the layout loads `/app.css`. The `?v={= props.version }` query parameter is a cache-buster — in production it's the version from `package.json` (so cached stylesheets are invalidated on deploy), in development it's a timestamp (so every reload picks up the freshest CSS).

<a name="source-scanning"></a>

## Source Scanning

Tailwind v4 scans files for class names by reading the actual text — it doesn't parse JSX or HTML in any structured way, so anything that looks like a class name is picked up. A useful side effect: TypeScript files can declare class names as string literals and they're included in the output:

```ts
const status_colors = {
	pending: "bg-yellow-100 text-yellow-800",
	active: "bg-green-100 text-green-800",
	failed: "bg-red-100 text-red-800",
};
```

Tailwind sees `bg-yellow-100`, `text-yellow-800`, etc. and includes them.

The flip side: class names assembled at runtime won't work. `` `bg-${color}` `` produces a string Tailwind can't see, so the class isn't in the output and the browser ignores it. The fix is to enumerate the possible values explicitly — a lookup object like the one above, or a list comment somewhere that Tailwind picks up.

<a name="excluded-directories"></a>

## Excluded Directories

The `@source not` directives at the top exclude noise. The defaults are tuned for a typical Reepolee project:

| Excluded            | Why                                                      |
| ------------------- | -------------------------------------------------------- |
| `.git/**/*`         | Commit messages and config aren't sources of class names |
| `.vscode/**/*`      | Editor settings                                          |
| `.claude/**/*`      | Editor settings for Claude Code, if present              |
| `node_modules/**/*` | Third-party dependencies — their classes aren't ours     |

If you add a directory that contains source-like files (`vendor/`, `docs/`, `archive/`) and don't want it scanned, add another `@source not` line. The exclusions are evaluated before the inclusions, so a `not` always wins over an explicit `@source` for the same path.

<a name="the-static-directory"></a>

## What Lives in static/

Everything served directly to the browser sits in `static/`. The server's fetch fallback streams these files with aggressive caching in production (a one-year immutable cache header) and `no-store` in development.

A typical `static/` layout:

```
static/
├── app.css                # compiled Tailwind (production)
├── app-dev.css            # compiled Tailwind (development)
├── signals-ui.js          # tiny self-contained reactive DOM bindings
├── form-controller.js     # live form validation
├── spa-loader.js          # SPA-style navigation
├── helpers-client.js      # $/$$ globals, navigate_to()
├── dialog-confirm.js      # confirm-button handler for <dialog>
├── checkbox-group.js      # bulk-select tables
├── image-editor.js        # canvas-based image editing
├── web-components/        # custom elements
│   ├── validation-error.js
│   ├── toasts-area.js
│   ├── title-display.js
│   └── image-editor.js
├── avatars/               # user-uploaded profile images
├── favicon.svg
└── logo.svg
```

Each script's job is documented in the [Client-Side](/client-side/signals-ui) section.

<a name="when-not-to-touch-the-css"></a>

## When Not to Touch app.css

Most styling happens directly in templates with Tailwind utilities — `class="text-lg font-semibold mb-4"` and so on. `css/app.css` is for project-wide concerns that don't belong inline:

- **Design tokens** (`@theme` block) — see [Theming & Tokens](/styling/theming-and-tokens).
- **Element defaults** (`@layer base`) — global resets, base typography, native control overrides.
- **Reusable patterns** (`@layer components`) — class names you use across many templates (`.nav-item`, `.button`).
- **Custom utilities** (`@utility ...`) — compositional shortcuts that compose with other Tailwind utilities (`primary`, `secondary`).
- **Cross-page CSS APIs** — `@view-transition`, `@page` (print styles), `@property` (CSS custom properties with types).

If you're tempted to write a long sequence of utilities in a template and reuse it three times, that's a candidate for a `@layer components` class. If you're tempted to write the same component class twice, that's a candidate for one of the input components in `components/`.
