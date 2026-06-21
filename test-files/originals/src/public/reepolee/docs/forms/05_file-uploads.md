---
title: "File Uploads"
---

# File Uploads

<a name="introduction"></a>

## Introduction

File uploads in Reepolee are plain HTML multipart forms. The route handler reads the body with `req.formData()`, pulls the file out by name, and writes it either to S3-compatible storage (when configured) or to the local filesystem with `Bun.write()`. There's no upload library and no streaming-middleware layer — `req.formData()`, `Bun.write()`, and `Bun.Image` are all built into the runtime and they're enough.

The pattern Reepolee uses for user avatars is the reference implementation. It covers the four things every file upload needs: a multipart form, a server handler that reads the file, a target to write to (S3 or disk), and a way to serve the file back to the browser. Both ends — write and serve — are gated on `is_s3_configured()` so the same code path works in production (with S3) and in local development (without it).

<a name="the-form"></a>

## The Form

<media-frame label="Screenshot — file upload field in a form" ratio="16/9"></media-frame>

Multipart forms need `enctype="multipart/form-data"`. Without it, the browser sends only the filename, not the file contents:

```html
<form method="POST" action="/profile" enctype="multipart/form-data">
	<label for="avatar">Avatar</label>
	<input type="file" id="avatar" name="avatar" accept="image/*" />

	<field-wrapper class="grid">
		<label class="px-3" for="name">Name</label>
		<input type="text" id="name" name="name" value="{= props.record.name }" />
		<validation-error class="mt-1" id="error-name"></validation-error>
	</field-wrapper>

	<button type="submit" class="primary">Save</button>
</form>
```

`accept="image/*"` tells the browser to filter the file picker to image types. It's a hint, not a guarantee — always validate the file type on the server too.

For multiple files, add `multiple` to the input and read `form_data.getAll("name")` instead of `form_data.get("name")` in the handler.

<a name="the-handler"></a>

## The Handler

Read the body with `req.formData()`. That returns a `FormData` object — the same one the browser builds for `fetch()` requests — where text fields come back as strings and file inputs come back as `File` objects:

```ts
import { save_avatar_upload } from "./avatar";

export async function post_auth_profile(req: BunRequest): Promise<Response> {
	const ctx = await create_ctx(req);
	const form_data = await req.formData();

	const data = {
		name: (form_data.get("name") as string)?.trim() || "",
		nickname: (form_data.get("nickname") as string)?.trim() || "",
	};

	const avatar_file = form_data.get("avatar") as File | null;

	// ... validate the text fields ...

	let avatar_filename: string | undefined;
	if (avatar_file && avatar_file.size > 0) {
		try {
			avatar_filename = await save_avatar_upload(avatar_file);
		} catch {
			return render("auth/profile/form", {
				data: { ...data, form_error: "avatar_upload_failed" },
				ctx,
			});
		}
	}

	await update_user_profile(user_id, { ...data, avatar_filename });
	return Response.redirect("/profile", 303);
}
```

A few things worth highlighting:

- **`avatar_file.size > 0`** is the check for "did the user actually upload a file?" When the file input is left empty, `form_data.get("avatar")` is still a `File` object — just with zero size.
- **The text fields and the file field share the same form**. Validate the text fields with your usual Zod schema; the file is handled separately because it isn't a string and doesn't belong in the schema.
- **The handler returns a redirect on success**, just like any other form. The avatar filename is saved to the database and the next request picks it up.

<a name="writing-to-storage"></a>

## Writing to Storage

The reference avatar helper picks a backend at runtime — S3 when configured, the local filesystem otherwise — and uses the same generated filename for both:

```ts
// routes/system/auth/avatar.ts
import { mkdir } from "fs/promises";
import { join } from "path";
import { is_s3_configured, save_to_s3 } from "$lib/s3";

const AVATAR_BUCKET = "users";
const AVATAR_PREFIX = "avatars";
export const AVATAR_DIR = join(process.cwd(), "static", "avatars");

export async function save_avatar_upload(file: File): Promise<string> {
	const ext = file.name.includes(".") ? "." + file.name.split(".").pop()!.toLowerCase() : "";
	const stored_name = `${crypto.randomUUID()}${ext}`;

	if (is_s3_configured()) {
		await save_to_s3(AVATAR_BUCKET, `${AVATAR_PREFIX}/${stored_name}`, file, {
			type: file.type || "application/octet-stream",
			acl: "public-read",
		});
	} else {
		await mkdir(AVATAR_DIR, { recursive: true });
		await Bun.write(join(AVATAR_DIR, stored_name), file);
	}

	return resize_avatar(file, stored_name);
}
```

The pattern, in five steps:

1. **Generate a unique filename** with `crypto.randomUUID()` so two users can't collide and existing files are hard to guess.
2. **Branch on backend.** `is_s3_configured()` (from `$lib/s3`) returns `true` when access key, secret, and endpoint are all set.
3. **On S3**, `save_to_s3(bucket, key, data, options)` auto-creates the bucket on first upload and writes the file. `acl: "public-read"` makes the resulting object reachable through the proxy described below; drop it if you'd rather generate a presigned URL for every fetch.
4. **On disk**, `Bun.write(dest, file)` streams the file without loading it into memory.
5. **Return the canonical filename.** The example calls `resize_avatar(file, stored_name)` next — see [Resizing](#resizing-and-processing). For uploads that don't need post-processing, return `stored_name` directly.

`Bun.write()` and `save_to_s3()` both accept `Blob`, `BunFile`, `Response`, strings, and `Uint8Array`, so the same helper shape works for any source that can produce bytes — not just `File` objects.

<a name="serving-the-file"></a>

## Serving the File Back

There are two paths a stored file can travel back to the browser, depending on whether you went through S3 or wrote to disk. The `<img>` tag in your template doesn't change either way:

```html
<img src="/avatars/{= props.user.avatar_filename }" alt="{= props.user.name }" />
```

**From disk.** Anything under `static/` is served automatically by the fetch fallback in `server.ts`. In production it gets a one-year immutable cache header (safe because the UUID filename never changes); in development it's served with `Cache-Control: no-store` so edits are visible immediately.

**From S3.** `server.ts` calls `register_s3_mount({ url_prefix: "/avatars/", bucket: "users", immutable: true })` at startup. On every request the fallback first runs `handle_s3_request(url)` — when the URL matches a registered mount, the helper fetches the object via a presigned URL, applies the configured cache header (`public, max-age=31536000, immutable` when `immutable: true`), and returns the bytes. The same `/avatars/<filename>` URL works for both backends, which keeps templates portable across environments.

Register additional mounts the same way for other resource types — `register_s3_mount({ url_prefix: "/uploads/", bucket: "files" })` and so on. Mounts are matched longest-prefix-first so a more specific path always wins.

<a name="validation"></a>

## Validating Uploads

`req.formData()` doesn't validate anything. Always check the file before writing it:

- **Size**: `file.size` is the byte count. Reject anything above your limit before calling `Bun.write()`.
- **Type**: `file.type` is the MIME type the browser claims. Don't trust it blindly — verify by reading the file's leading bytes if it matters (PDF starts with `%PDF`, PNG starts with `\x89PNG`, JPEG starts with `\xFF\xD8\xFF`).
- **Extension**: derive from `file.name` after splitting on the last `.`. Reject extensions you don't expect rather than allowing everything.

A reasonable guard for image uploads:

```ts
const MAX_BYTES = 5 * 1024 * 1024; // 5 MB
const ALLOWED_TYPES = ["image/jpeg", "image/png", "image/webp", "image/gif"];

if (avatar_file.size > MAX_BYTES) {
	return form_error("file_too_large");
}
if (!ALLOWED_TYPES.includes(avatar_file.type)) {
	return form_error("file_type_not_allowed");
}
```

<a name="resizing-and-processing"></a>

## Resizing and Processing

Bun's built-in `Bun.Image` runs the decode/resize/encode chain off-thread without any npm dependency. The avatar helper uses it to produce a 128×128 WebP variant of every uploaded image:

```ts
export async function resize_avatar(file: File, stored_name: string): Promise<string> {
	const base_name = stored_name.replace(/\.[^.]+$/, "");
	const resized_name = `${base_name}_128_128.webp`;

	const file_bytes = await file.bytes();
	const img = new Bun.Image(file_bytes);
	const pipeline = img.resize(128, 128, { fit: "inside" }).webp({ quality: 80 });

	if (is_s3_configured()) {
		const webp_bytes = await pipeline.bytes();
		await save_to_s3(AVATAR_BUCKET, `${AVATAR_PREFIX}/${resized_name}`, webp_bytes, {
			type: "image/webp",
			acl: "public-read",
		});
	} else {
		await mkdir(AVATAR_DIR, { recursive: true });
		await pipeline.write(join(AVATAR_DIR, resized_name));
	}

	return resized_name;
}
```

A few things worth highlighting:

- **The resized filename derives from the original.** `<uuid>.jpg` becomes `<uuid>_128_128.webp`, so you can correlate the two and clean both up together if you ever need to.
- **`fit: "inside"` preserves aspect ratio** and pads no pixels — a portrait shot becomes 128×something rather than being squashed.
- **`.write(path)` or `.bytes()`** picks the output sink. Local writes can stream straight to disk; S3 needs the bytes in hand so they can be uploaded as the request body.
- **The original is kept.** The example uploads both the original and the resized version. Drop the original-upload call in `save_avatar_upload` if you only ever want the resized file on storage.

If `Bun.Image` doesn't cover your use case (PDF rendering, EXIF stripping, multi-page TIFFs), reach for a native binary via `Bun.spawn` (ImageMagick, ffmpeg, vips-tools) or an npm library like `sharp`. The shape of the helper doesn't change — read the file, transform, write to the same backend, return the new filename.

<a name="storage-considerations"></a>

## Storage in Production

The two backends — local disk and S3 — each have their place. Choose with deployment shape, not file size.

**Local disk** is the right choice for a single VPS where uploads are part of the same backup story as the database. The notes worth honouring:

- **Don't commit uploads to git.** Add `static/avatars/` (and any other upload directories) to `.gitignore`. Otherwise every deploy overwrites real user uploads with whatever was in the repo.
- **Mount a persistent disk.** A simple deployment on a VPS works fine — the upload directory persists across restarts. For containers and ephemeral hosts, bind-mount a real volume at `static/avatars/`.
- **Back up the upload directory.** It's user data, not artifacts. Treat it the same way you treat the database.

**S3-compatible storage** is the right choice once you outgrow a single box, want a CDN in front of uploads, or run on ephemeral hosts. Set `S3_ACCESSKEYID`, `S3_SECRETACCESSKEY`, and `S3_ENDPOINT` (and optionally `S3_REGION`) in `.env` — `is_s3_configured()` flips the helper over automatically. Anything S3 API-compatible works: AWS S3, Cloudflare R2, Backblaze B2, MinIO for a self-hosted setup.

For local development against S3, `podman run` (or `docker run`) a MinIO container — the README in the project root has a copy-pasteable command. The same `.env` keys point at MinIO locally and at production object storage in deploy, so the application code never has to know which one it's talking to.

<a name="without-javascript"></a>

## Without JavaScript

The whole upload flow above works with JavaScript disabled — it's just a regular multipart form. There's no `FormData` JavaScript construction, no `XMLHttpRequest`, no progress bar. The browser shows its native file picker, submits the form, and reloads the page.

If you want a progress indicator or a drag-and-drop dropzone, layer that on top with a small script using signals or a custom element — but the underlying form keeps working for users without it.
