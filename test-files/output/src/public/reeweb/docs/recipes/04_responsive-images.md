---
title: "Responsive Images"
layout: "reeweb/docs/docs.layout"
---

# Responsive Images

<a name="introduction"></a>

## Introduction

Reeweb generates responsive image variants **at build time** from committed
originals, so you never commit the resized/re-encoded files. You drop a
full-size image into `assets/images/`, and the build produces width-stepped
WebP + JPEG variants that the `<responsive-image>` component serves through a
`<picture>` element. Only originals live in git; every variant is a build
artifact.

The pieces:

| Piece                           | Role                                                     |
| ------------------------------- | -------------------------------------------------------- |
| `assets/images/`                | Committed originals (outside `src/public`, never served) |
| `scripts/prepare_images.ts`     | Build-time generator (uses `Bun.Image`, zero deps)       |
| `src/public/images/responsive/` | Generated variants (git-ignored, served)                 |
| `config/responsive_images.ts`   | Widths + quality — the single source of truth            |
| `lib/images.ts`                 | `srcset()` / `webp()` / `jpeg()` URL helpers             |
| `<responsive-image>`            | Renders a `<picture>` from those helpers                 |

<a name="adding-an-image"></a>

## Adding an image

1. Put the original in `assets/images/` (e.g. `assets/images/hero.png`).
2. Run a build (or `bun run prepare:images`). Variants land in
   `src/public/images/responsive/`:

```
src/public/images/responsive/
  hero.png  hero.webp                 full-size recode
  300/  hero.png  hero.webp           ┐ width variants — resized by width,
  500/  …                             │ aspect preserved, never upscaled
  800/  …                             ┘
  1440/ …
```

Sub-folders under `assets/images/` are preserved in the output.

<a name="the-component"></a>

## The `<responsive-image>` component

Pass a `src` pointing at the original's served path; the component builds the
`<picture>` srcset for you:

```html
{{ props.src = "/images/responsive/hero.png"; props.alt = "Our team"; }}
<responsive-image
	loading="eager"
	sizes="(max-width: 1024px) 100vw, 640px"
	image_class="h-full w-full object-cover"
></responsive-image>
```

Props: `src`, `alt`, `class` (the `<picture>`), `image_class` (the `<img>`),
`sizes`, and `loading` (defaults to `lazy`). The browser downloads the smallest
variant that satisfies `sizes`.

<a name="configuration"></a>

## Configuration

Widths and quality are defined once in `config/responsive_images.ts`:

```ts
export const responsive_widths = [300, 500, 800, 1440] as const;
export const responsive_quality = { webp: 80, jpeg: 80 } as const;
```

Both the generator and the component's `srcset` read this file, so they never
drift — change a width here and the generated files and the markup update
together. Override per-build with `--widths` / `--quality*` flags.

<a name="why-no-avif"></a>

## Why no AVIF (yet)

`Bun.Image` uses the OS-native codec, which has no AV1 encoder on many platforms
(Intel macOS, typical Linux CI), so AVIF is not generated. WebP (~97% browser
support) plus a JPEG fallback cover it. The component keeps an AVIF `<source>`
**commented out** — `<picture>` does not fall back when a chosen source 404s, so
a live AVIF source with no AVIF files would show broken images. Re-enable both
the component line and AVIF generation together if you adopt an AVIF-capable
encoder.

<a name="build-wiring"></a>

## Build wiring

`prepare:images` runs **before** `build`/`build:dist` (blocking) and
**concurrently** in `dev` (so the server starts instantly). The script is
incremental: warm runs are near-instant, and editing one original rebuilds only
that image. See the [Scripts Reference](/reeweb/docs/reference/scripts-reference)
for the exact commands.
