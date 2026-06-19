import { default_language, language_locales } from "$config/supported_languages";
import { cache } from "$lib/cache";
import { make_toast } from "$lib/cookies";
import { format_bulk_delete_message, get_lang_from_request, translated_from_request } from "$lib/helpers";
import { delete_from_local, delete_from_s3 } from "$lib/s3";
import type { BunRequest } from "bun";

import { validate_touched } from "./schema/validation_server";
import { delete_record, get_record_by_id } from "./sql";

const TABLE_NAME = "images";
const feature = "images";

// ---------------------------------------------------------------------------
// POST /images/validate
// ---------------------------------------------------------------------------

export async function post_images_validate(req: BunRequest): Promise<Response> {
	const body = await req.json();
	const translated = await translated_from_request(req, import.meta.dir);
	const touched: string[] = body.touched || [];

	const data = {
		folder: body.folder || "",
		filename: body.filename || "",
		s3_key: body.s3_key || "",
		original_filename: body.original_filename || "",
		title: body.title || "",
		description: body.description || "",
		tags: body.tags || "",
		mime_type: body.mime_type || "",
		width: body.width || "",
		height: body.height || "",
		file_size: body.file_size || "",
	};

	const [errors] = validate_touched(data, touched, translated.errors);
	const success = Object.keys(errors).length === 0;

	return new Response(JSON.stringify({ success, errors }), {
		status: 200,
		headers: { "Content-Type": "application/json" },
	});
}

// ---------------------------------------------------------------------------
// POST /images/bulk-delete
// ---------------------------------------------------------------------------

export async function post_images_bulk_delete(req: BunRequest): Promise<Response> {
	const translated = await translated_from_request(req, import.meta.dir);
	const msg = translated.messages ?? {};
	const lang = get_lang_from_request(req) || default_language;
	const locale = language_locales[lang] || "en-US";

	try {
		const body = await req.json();
		const ids: number[] = body.ids || [];

		if (!Array.isArray(ids) || ids.length === 0) {
			return new Response(JSON.stringify({ error: msg.bulk_delete_no_ids || "No images selected." }), {
				status: 400,
				headers: { "Content-Type": "application/json" },
			});
		}

		const bucket = Bun.env.S3_IMAGE_BUCKET || "images";

		const results = await Promise.allSettled(
			ids.map(async (id) => {
				const record = await get_record_by_id(Number(id));
				if (!record) return { deleted: false };

				if (record.s3_key) {
					try {
						await delete_from_s3(bucket, record.s3_key);
						await delete_from_local(bucket, record.s3_key);
						const thumb_key = record.s3_key.replace(/[^/]+$/, (match) => `tn_${match}`);
						await delete_from_s3(bucket, thumb_key);
						await delete_from_local(bucket, thumb_key);
					} catch (err) {
						console.error("⚠️  Failed to delete image files:", err);
					}
				}

				const deleted = await delete_record(Number(id));
				return { deleted: !!deleted };
			}),
		);

		let deleted_count = 0;
		let error_count = 0;

		for (const result of results) {
			if (result.status === "fulfilled") {
				if (result.value.deleted) deleted_count++;
				else error_count++;
			} else {
				console.error("⚠️  Bulk delete error:", result.reason);
				error_count++;
			}
		}

		await cache.invalidate(TABLE_NAME);

		const message = format_bulk_delete_message(msg, deleted_count, error_count, "image", locale);

		const toast_type = error_count > 0 && deleted_count === 0 ? "red" : "green";
		const toast_cookie = make_toast("toast-bulk-delete", { message, type: toast_type, duration: 4000 });

		return new Response(JSON.stringify({ deleted: deleted_count, errors: error_count, message }), {
			status: 200,
			headers: { "Content-Type": "application/json", "Set-Cookie": toast_cookie.toString() },
		});
	} catch (err) {
		console.error("⚠️  Bulk delete failed:", err);
		return new Response(JSON.stringify({ error: msg.bulk_delete_failed || "Bulk delete failed." }), {
			status: 500,
			headers: { "Content-Type": "application/json" },
		});
	}
}
