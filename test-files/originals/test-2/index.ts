import { default_language } from "$config/supported_languages";
import { cache } from "$lib/cache";
import {
	extract_params_from_url,
	get_cookies_by_prefix,
	get_lang_from_request,
	get_table_name_from_dir,
	localized_url,
	translated_from_request,
} from "$lib/helpers";
import { sql_log } from "$lib/logger";
import { build_pagination_urls, get_limit_numeric, get_limit_options, parse_pagination_params } from "$lib/pagination";
import { render } from "$lib/render";
import { create_ctx } from "$lib/request_context";
import { enrich_filter_defs, get_filter_defs, resolve_filters } from "$lib/table_filters";
import type { BunRequest } from "bun";

import { post_images_process, post_images_save } from "./editor_server";
import { post_images_bulk_delete, post_images_validate } from "./handlers";
import { columns, enable_delete, fields } from "./schema/table";
import { delete_record, get_record_by_id, search_records } from "./sql";
// ---------------------------------------------------------------------------
// Config
// ---------------------------------------------------------------------------

const TABLE_NAME = "images";
const feature = get_table_name_from_dir(import.meta.dir);
const route_prefix = "/system";
const prefix_segments = route_prefix ? route_prefix.split("/").filter(Boolean).length : 0;
const SORT_OPTIONS = [
	{ value: "id::asc", label: "ID (Ascending)" },
	{ value: "id::desc", label: "ID (Descending)" },
	{ value: "title::asc", label: "Title (Ascending)" },
	{ value: "title::desc", label: "Title (Descending)" },
	{ value: "folder::asc", label: "Folder (Ascending)" },
	{ value: "folder::desc", label: "Folder (Descending)" },
];

function base_path(): string {
	return route_prefix ? `${route_prefix}/${feature}` : `/${feature}`;
}

function entity_path(id?: number): string {
	return id ? `${base_path()}/${id}/edit` : base_path();
}

function get_redirect_from_referer(req: BunRequest): string | null {
	const referer = req.headers.get("referer");
	if (!referer) return null;
	try {
		const url = new URL(referer);
		const path = url.pathname + url.search;
		if (path.includes(base_path()) && !path.includes("/edit")) return path;
	} catch {
		// Invalid referer — ignore
	}
	return null;
}

// ---------------------------------------------------------------------------
// Route map
// ---------------------------------------------------------------------------

export const system_images_crud = {
	"/images": { GET: get_images_index },
	"/images/new": get_images_new,
	"/images/validate": { POST: post_images_validate },
	"/images/process": { POST: post_images_process },
	"/images/save": { POST: post_images_save },
	"/images/:id/edit": { GET: get_images_edit, POST: post_images_edit },
	"/images/bulk-delete": { POST: post_images_bulk_delete },
};

// ---------------------------------------------------------------------------
// GET /images — List page (CRUD table)
// ---------------------------------------------------------------------------

export async function get_images_index(req: BunRequest): Promise<Response> {
	const ctx = await create_ctx(req, import.meta.dir);
	ctx.toasts = get_cookies_by_prefix(req, "toast-");

	const { query, offset, limit, order_by, scope, filters, filter_not } = parse_pagination_params(req.url, 20, ["scope"]);
	const limit_numeric = get_limit_numeric(limit);

	const raw_filter_defs = get_filter_defs(columns, fields);
	const filter_clauses = resolve_filters(raw_filter_defs, filters, filter_not);

	const [result, translated] = await Promise.all([
		search_records(query, offset, limit_numeric, order_by, "", filter_clauses),
		translated_from_request(req, import.meta.dir),
	]);

	// Enrich filter_defs with translated labels, option lists, and URL param state
	const { labels } = translated;
	const filter_defs = enrich_filter_defs(raw_filter_defs, labels, filters, filter_not, {});

	const limit_options = get_limit_options(limit === "all" ? "all" : (limit as number));

	const { prev_url, next_url, first_url, last_url } = build_pagination_urls(
		base_path(),
		offset,
		limit_numeric,
		result.total,
		query,
		order_by,
		scope,
		filters,
	);

	const grid_cols = `${
		Object.entries(columns)
			.filter(([_, v]: [string, any]) => v.grid !== false)
			.map(([_, v]: [string, any]) => (typeof v === "string" ? v : v.width))
			.join(" ")
	} auto`;

	return render("index", {
		data: {
			title: "Images",
			records: result.records,
			query: query || "",
			limit,
			offset,
			order_by,
			total: result.total,
			limit_options,
			sort_options: SORT_OPTIONS,
			prev_url,
			next_url,
			first_url,
			last_url,
			columns,
			grid_cols,
			filter_defs,
			filter_clauses,
			filter_params: filters,
			filter_not_params: filter_not,
			active_filter_count: filter_clauses.length,
			enable_delete,
			...translated,
			v_labels: translated.v_labels || {},
			images_basepath: Bun.env.S3_IMAGE_BUCKET || "images",
		},
		ctx,
	});
}

// ---------------------------------------------------------------------------
// GET /images/new — New image form
// ---------------------------------------------------------------------------

export async function get_images_new(req: BunRequest): Promise<Response> {
	const ctx = await create_ctx(req, import.meta.dir);
	const lang = get_lang_from_request(req) || default_language;
	const translated = await translated_from_request(req, import.meta.dir);

	const url = new URL(req.url);
	const folder = url.searchParams.get("folder") || "";

	const localized_base = localized_url(base_path(), lang);

	const record = {
		id: 0,
		folder,
		title: "",
		description: "",
		tags: "",
		s3_key: "",
		created_at: "",
	};

	const editor_config = {
		process_url: "/system/images/process",
		save_url: "/system/images/save",
		return_url: localized_base,
		edit_mode: false,
	};

	const image_basepath = `/${Bun.env.S3_IMAGE_BUCKET || "images"}`;

	return render("form", {
		data: {
			title: translated.ui?.new_image || "New Image",
			record,
			editor_config,
			editor_config_json: JSON.stringify(
				{
					csrfToken: req.headers.get("X-CSRF-Token") || "",
					processUrl: editor_config.process_url,
					saveUrl: editor_config.save_url,
					returnUrl: editor_config.return_url,
					imageId: "0",
					s3Key: "",
					title: "",
					description: "",
					tags: "",
					folder: record.folder,
					editMode: false,
					image_basepath: image_basepath,
					app_basepath: localized_base,
				},
				null,
				"\t",
			),
			...translated,
		},
		ctx,
	});
}

// ---------------------------------------------------------------------------
// GET /images/:id/edit — Edit image form
// ---------------------------------------------------------------------------

export async function get_images_edit(req: BunRequest): Promise<Response> {
	const ctx = await create_ctx(req, import.meta.dir);
	const lang = get_lang_from_request(req) || default_language;
	const [id = 0] = extract_params_from_url(req, 1, prefix_segments);
	const [record, translated] = await Promise.all([
		get_record_by_id(id),
		translated_from_request(req, import.meta.dir),
	]);

	if (!record) {
		return render("notfound", {
			data: { title: "404 Not Found", ...translated },
			status: 404,
			ctx,
		});
	}
	const localized_base = localized_url(base_path(), lang);

	const editor_config = {
		process_url: "/system/images/process",
		save_url: "/system/images/save",
		return_url: localized_base,
		edit_mode: true,
	};

	const image_basepath = `/${Bun.env.S3_IMAGE_BUCKET || "images"}`;

	return render("form", {
		data: {
			title: `Edit ${record.original_filename || record.filename || "image"}`,
			record,
			action: entity_path(record.id),
			editor_config,
			editor_config_json: JSON.stringify(
				{
					csrfToken: req.headers.get("X-CSRF-Token") || "",
					processUrl: editor_config.process_url,
					saveUrl: editor_config.save_url,
					returnUrl: editor_config.return_url,
					imageId: String(record.id || 0),
					s3Key: record.s3_key || "",
					title: record.title || "",
					description: record.description || "",
					tags: record.tags || "",
					folder: record.folder || "",
					editMode: true,
					image_basepath: image_basepath,
					app_basepath: localized_base,
				},
				null,
				"\t",
			),
			...translated,
		},
		ctx,
	});
}

// ---------------------------------------------------------------------------
// POST /images/:id/edit — Edit form handler (delete action only)
// ---------------------------------------------------------------------------

export async function post_images_edit(req: BunRequest): Promise<Response> {
	const ctx = await create_ctx(req, import.meta.dir);
	const [id = 0] = extract_params_from_url(req, 1, prefix_segments);
	const body = await req.text();
	const _lang = get_lang_from_request(req) || default_language;
	const translated = await translated_from_request(req, import.meta.dir);
	const params = new URLSearchParams(body);
	const action = params.get("_action");

	const bp = base_path();
	const redirect_url = localized_url(bp, _lang);

	if (action === "delete") {
		try {
			// Fetch record to get the S3 key before deleting
			const record = await get_record_by_id(id);

			if (!record) {
				return render("notfound", {
					data: { title: "404 Not Found", ...translated },
					status: 404,
					ctx,
				});
			}

			// Delete files from storage
			if (record?.s3_key) {
				try {
					const bucket = Bun.env.S3_IMAGE_BUCKET || "images";
					// Delete from S3 (skips gracefully if S3 not configured)
					await delete_from_s3(bucket, record.s3_key);
					// Also delete from local storage (skips gracefully if not available)
					await delete_from_local(bucket, record.s3_key);

					const thumb_key = record.s3_key.replace(/[^/]+$/, (match) => `tn_${match}`);
					await delete_from_s3(bucket, thumb_key);
					await delete_from_local(bucket, thumb_key);
				} catch (err) {
					console.error("⚠️  Failed to delete image files:", err);
				}
			}

			const deleted = await delete_record(id);

			if (deleted) {
				sql_log({ s: "Delete", t: `${feature}`, id }, req);
				await cache.invalidate(TABLE_NAME);
				return Response.redirect(redirect_url, 303);
			}

			return render("notfound", {
				data: { title: "404 Not Found", ...translated },
				status: 404,
				ctx,
			});
		} catch (error) {
			const existing_record = await get_record_by_id(id);

			if (!existing_record) {
				return render("notfound", {
					data: { title: "404 Not Found", ...translated },
					status: 404,
					ctx,
				});
			}

			const error_message = error instanceof Error && error.message.includes("foreign key")
				? "Cannot delete this image because it's referenced by other records."
				: "Error deleting image.";

			return render("form", {
				data: {
					title: `Edit ${existing_record.original_filename || existing_record.filename || "image"}`,
					record: existing_record,
					action: entity_path(id),
					editor_config: {
						process_url: "/system/images/process",
						save_url: "/system/images/save",
						return_url: base_path(),
						edit_mode: true,
					},
					form_errors: error_message,
					...translated,
				},
				ctx,
			});
		}
	}

	return Response.redirect(entity_path(id), 303);
}
