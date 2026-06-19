import { unlink } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";

import { db } from "$config/db";
import { cache } from "$lib/cache";
import { generate_thumbnail, get_image_dims, process_and_save_to_s3, process_image } from "$lib/image_processor";
import { duration_ms, log_error, log_info, log_warn } from "$lib/logger";
import { delete_from_local, delete_from_s3, save_to_s3 } from "$lib/s3";
import { uuid_v7 } from "$lib/uuid";
import { require_auth, resolve_session } from "$root/routes/system/auth/middleware";
import type { BunRequest } from "bun";

import {
	cleanup,
	ext_of,
	form_int,
	hard_clone,
	mime_to_ext,
	parse_crop,
	parse_resize,
	save_upload_to_temp,
	type Upload,
} from "./editor_helpers";

const IMAGE_BUCKET = Bun.env.S3_IMAGE_BUCKET || "images";

// ---------------------------------------------------------------------------
// DB helpers
// ---------------------------------------------------------------------------

async function db_insert_image(
	folder: string,
	filename: string,
	s3_key: string,
	original_name: string,
	title: string,
	description: string,
	tags: string,
	mime: string,
	width: number,
	height: number,
	file_size: number,
): Promise<number> {
	log_warn("editor", "db_insert_image:input", {
		folder,
		filename,
		s3_key,
		original_name,
		typeof_original_name: typeof original_name,
		length: original_name?.length,
		json: JSON.stringify(original_name),
	});

	const rows = await db`
		INSERT INTO images (
			folder,
			filename,
			s3_key,
			original_filename,
			title,
			description,
			tags,
			mime_type,
			width,
			height,
			file_size
		)
		VALUES (
			${folder},
			${filename},
			${s3_key},
			${original_name},
			${title},
			${description},
			${tags},
			${mime},
			${width},
			${height},
			${file_size}
		)
		RETURNING id
	`;

	log_warn("editor", "db_insert_image:complete", {
		rows,
		original_name_after_query: original_name,
	});

	return rows[0]?.id ?? 0;
}

// ---------------------------------------------------------------------------
// POST /images/process — Preview
// ---------------------------------------------------------------------------

export async function post_images_process(req: BunRequest): Promise<Response> {
	const start = process.hrtime.bigint();

	log_info("editor", "POST /images/process:start");

	const auth_ctx = await resolve_session(req);
	const guard = require_auth(auth_ctx, req);

	if (guard) {
		return guard;
	}

	let upload: Upload | undefined;
	let result: Awaited<ReturnType<typeof process_image>> | undefined;

	try {
		const form_data = await req.formData();

		upload = await save_upload_to_temp(form_data);

		const format = (form_data.get("format") as string) || "webp";
		const quality = form_int(form_data, "quality") || 85;

		result = await process_image(upload.temp_path, {
			crop: parse_crop(form_data),
			resize: parse_resize(form_data),
			format,
			quality,
		});

		const bytes = await Bun.file(result.output_path).bytes();

		log_info("editor", "POST /images/process:done", {
			width: result.width,
			height: result.height,
			duration: duration_ms(start),
		});

		return new Response(bytes, {
			status: 200,
			headers: {
				"Content-Type": result.mime,
				"X-Image-Width": String(result.width),
				"X-Image-Height": String(result.height),
				"X-Image-Format": format,
			},
		});
	} catch (err) {
		log_error("editor", "POST /images/process:failed", err instanceof Error ? err : new Error(String(err)));

		return new Response(err instanceof Error ? err.message : "Processing failed", { status: 500 });
	} finally {
		await cleanup(upload?.temp_path, result?.output_path);
	}
}

// ---------------------------------------------------------------------------
// POST /images/save — Process + S3 + DB
// ---------------------------------------------------------------------------

export async function post_images_save(req: BunRequest): Promise<Response> {
	const start = process.hrtime.bigint();

	log_info("editor", "POST /images/save:start");

	const auth_ctx = await resolve_session(req);
	const guard = require_auth(auth_ctx, req);

	if (guard) {
		return guard;
	}

	let upload: Upload | undefined;
	let result: Awaited<ReturnType<typeof process_and_save_to_s3>> | undefined;

	try {
		const form_data = await req.formData();

		log_warn("editor", "POST /images/save:form parsed");

		upload = await save_upload_to_temp(form_data);

		log_warn("editor", "POST /images/save:after upload", {
			upload_original_name: upload.original_name,
			upload_temp_path: upload.temp_path,
		});

		// HARD CLONE — avoid Bun native string mutation
		const original_name = hard_clone(upload.original_name);
		const temp_path = hard_clone(upload.temp_path);

		log_warn("editor", "POST /images/save:primitives frozen", {
			original_name,
			temp_path,
			same_ref_upload: Object.is(original_name, upload.original_name),
		});

		const original_s3_key = hard_clone(`${(form_data.get("s3_key") as string) || ""}`);

		const save_as_copy = form_data.get("save_as_copy") === "1";

		const keep_original = form_data.get("keep_original") === "1";

		const format = hard_clone(`${(form_data.get("format") as string) || "webp"}`);

		const quality = form_int(form_data, "quality") || 85;

		const folder = hard_clone(
			`${((form_data.get("folder") as string) || "").trim().replace(/^\/+|\/+$/g, "") || ""}`,
		);

		const title = hard_clone(`${(form_data.get("title") as string) || ""}`);

		const description = hard_clone(`${(form_data.get("description") as string) || ""}`);

		const tags = hard_clone(`${(form_data.get("tags") as string) || ""}`);

		// ── Compute S3 key for edit-mode saves, respecting folder ──
		let target_s3_key = "";

		if (!save_as_copy && original_s3_key) {
			const base_filename = original_s3_key.split("/").pop() || original_s3_key;
			const key_with_format = base_filename.replace(/\.[^.]+$/, `.${format}`);

			target_s3_key = folder ? `${folder}/${key_with_format}` : key_with_format;
			target_s3_key = hard_clone(target_s3_key);
		}

		log_warn("editor", "POST /images/save:before process_and_save_to_s3", {
			original_name,
			temp_path,
			target_s3_key,
		});

		result = await process_and_save_to_s3(temp_path, {
			crop: parse_crop(form_data),
			resize: parse_resize(form_data),
			format,
			quality,
			s3_key: target_s3_key || undefined,
			folder: target_s3_key ? undefined : folder,
		});

		log_warn("editor", "POST /images/save:after process_and_save_to_s3", {
			original_name,
			temp_path,
			result_filename: result.filename,
			result_s3_key: result.s3_key,
		});

		const final_s3_key = hard_clone(`${result.s3_key || result.filename}`);

		// Derive filename from the actual S3 key so it always matches (extension + UUID)
		const result_filename = hard_clone(final_s3_key.split("/").pop() || final_s3_key);

		const result_mime = hard_clone(`${result.mime}`);

		const result_width = result.width;
		const result_height = result.height;
		const result_file_size = result.file_size;

		const result_s3_url = hard_clone(`${result.s3_url || ""}`);

		let db_id = 0;

		if (!save_as_copy && original_s3_key) {
			await db`
				UPDATE images SET
					mime_type = ${result_mime},
					width = ${result_width},
					height = ${result_height},
					file_size = ${result_file_size},
					filename = ${result_filename},
					title = ${title},
					description = ${description},
					tags = ${tags},
					folder = ${folder},
					s3_key = ${final_s3_key},
					updated_at = CURRENT_TIMESTAMP
				WHERE s3_key = ${original_s3_key}
			`;

			const rows = await db`
				SELECT id FROM images
				WHERE s3_key = ${final_s3_key}
			`;

			db_id = rows[0]?.id ?? 0; // ── Delete old files if the key changed (e.g. folder or format change) ──
			if (original_s3_key !== final_s3_key) {
				try {
					// Delete the old image from S3 (skips gracefully if S3 not configured)
					log_info("editor", "POST /images/save: deleting old image", {
						old_key: original_s3_key,
						new_key: final_s3_key,
					});
					await delete_from_s3(IMAGE_BUCKET, original_s3_key);
					// Also delete from local storage (skips gracefully if not available)
					await delete_from_local(IMAGE_BUCKET, original_s3_key);

					// Also delete the old thumbnail (tn_ prefix)
					const old_thumb_key = original_s3_key.replace(/[^/]+$/, (match) => `tn_${match}`);
					if (old_thumb_key !== original_s3_key) {
						log_info("editor", "POST /images/save: deleting old thumbnail", {
							old_thumb_key,
						});
						await delete_from_s3(IMAGE_BUCKET, old_thumb_key);
						await delete_from_local(IMAGE_BUCKET, old_thumb_key);
					}

					log_info("editor", "POST /images/save: old files deleted");
				} catch (err) {
					// Non-fatal — log and continue
					log_error(
						"editor",
						"POST /images/save: failed to delete old files",
						err instanceof Error ? err : new Error(String(err)),
					);
				}
			}
		} else {
			db_id = await db_insert_image(
				folder,
				result_filename,
				final_s3_key,
				hard_clone(original_name),
				title,
				description,
				tags,
				result_mime,
				result_width,
				result_height,
				result_file_size,
			);
		}

		await cache.invalidate("images");

		log_warn("editor", "POST /images/save:db complete", {
			db_id,
			original_name_after_db: original_name,
		});

		let original_db_id = 0;
		let original_s3_url = "";

		if (!original_s3_key && keep_original) {
			try {
				const orig_file = Bun.file(temp_path);

				const orig_size = orig_file.size;

				const orig_mime = hard_clone(`${orig_file.type || "application/octet-stream"}`);

				const orig_ext = ext_of(original_name) || mime_to_ext(orig_mime);

				const orig_filename = hard_clone(`${uuid_v7()}${orig_ext}`);

				const orig_s3_key = !folder ? orig_filename : hard_clone(`${folder}/${orig_filename}`);

				const orig_original_name = hard_clone(original_name);

				log_warn("editor", "POST /images/save:before save_to_s3", {
					orig_ext,
					orig_filename,
					orig_s3_key,
					orig_original_name,
				});

				await save_to_s3(IMAGE_BUCKET, orig_s3_key, orig_file, { type: orig_mime });

				log_warn("editor", "POST /images/save:after save_to_s3", {
					orig_s3_key,
					orig_original_name,
				});

				// ── Generate 100×100 thumbnail for the original ──
				const thumb_ext = mime_to_ext(orig_mime);

				const thumb_s3_key = hard_clone(orig_s3_key.replace(/[^/]+$/, (match) => `tn_${match}`));

				const thumb_output = join(tmpdir(), "reepolee-editor", `tn_${uuid_v7()}${thumb_ext}`);

				try {
					await generate_thumbnail(temp_path, thumb_output, 100);

					await save_to_s3(IMAGE_BUCKET, thumb_s3_key, Bun.file(thumb_output), { type: orig_mime });

					log_warn("editor", "POST /images/save:original thumbnail saved", {
						thumb_s3_key,
					});
				} catch (err) {
					log_error(
						"editor",
						"POST /images/save:original thumbnail failed",
						err instanceof Error ? err : new Error(String(err)),
					);
				} finally {
					try {
						await unlink(thumb_output);
					} catch {
						// ignore cleanup errors
					}
				}

				const { width: orig_width, height: orig_height } = await get_image_dims(temp_path);

				original_db_id = await db_insert_image(
					folder,
					orig_filename,
					orig_s3_key,
					orig_original_name,
					title,
					description,
					tags,
					orig_mime,
					orig_width,
					orig_height,
					orig_size,
				);

				original_s3_url = hard_clone(`/${IMAGE_BUCKET}/${orig_s3_key}`);

				log_warn("editor", "POST /images/save:original saved", {
					orig_s3_key,
					original_db_id,
					orig_original_name,
				});
			} catch (err) {
				log_error(
					"editor",
					"POST /images/save:original save failed",
					err instanceof Error ? err : new Error(String(err)),
				);
			}
		}

		log_info("editor", "POST /images/save:done", {
			db_id,
			original_db_id,
			duration: duration_ms(start),
		});

		const json = JSON.stringify({
			db_id,
			s3_url: result_s3_url,
			width: result_width,
			height: result_height,
			format,
			size_kb: result_file_size / 1024,
			filename: result_filename,
			s3_key: final_s3_key,
		});

		return new Response(json, {
			status: 200,
			headers: { "Content-Type": "application/json" },
		});
	} catch (err) {
		log_error("editor", "POST /images/save:failed", err instanceof Error ? err : new Error(String(err)));

		return new Response(err instanceof Error ? err.message : "Save failed", { status: 500 });
	} finally {
		await cleanup(upload?.temp_path, result?.output_path);
	}
}
