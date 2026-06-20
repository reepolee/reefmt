import {
	get_table_name_from_dir,
	translated_from_request,
	get_cookie,
	get_lang_from_request,
	localized_url,
	get_cookies_by_prefix,
} from "$lib/helpers";
import { get_global_scopes, get_scope_clause } from "$lib/global_scopes";
import { enrich_filter_defs, get_filter_defs, resolve_filters } from "$lib/table_filters";
import { render, render_to_string } from "$lib/render";
import { create_ctx } from "$lib/request_context";

import {
	get_record_by_id,
	create_record,
	update_record,
	delete_record,
	search_records,
	get_record_by_id_and_parent,
	TABLE_NAME,
} from "./sql";
import { cache } from "$lib/cache";
;
import { default_language, language_locales } from "$config/supported_languages";
import { sql_log } from "$lib/logger";
import { type BunRequest } from "bun";

import { validate, validate_touched } from "./schema/validation_server";
import { columns, enable_delete, fields } from "./schema/table";

import { get_equipment_options_by_code } from "./sql";

import { get_equipment_options_by_id } from "./sql";

;


export const user_equipment_items_crud = {
	"/equipment/:code/equipment_items": { POST: post_equipment_items_index },
	"/equipment/:code/equipment_items/:id/edit-data": get_equipment_items_edit,
	"/equipment/:code/equipment_items/:id/edit": { POST: post_equipment_items_edit },
	"/equipment_items/validate": { POST: post_equipment_items_validate },
	"/equipment/:code/equipment_items/validate": { POST: post_equipment_items_validate },
};


const feature = get_table_name_from_dir(import.meta.dir);
const route_prefix = "/user";
const PARENT_FK_COLUMN = "equipment_code";
const prefix_segments = route_prefix ? route_prefix.split("/").filter(Boolean).length : 0;
const DEFAULT_LIMIT = 20;
const ALLOWED_LIMITS = [
	5,
	10,
	20,
	30,
	50,
	100,
];
const DEFAULT_ORDER_BY = "id::asc";
const SORT_OPTIONS = [
	{ "value": "id::asc", "label": "ID (Ascending)" },
	{ "value": "id::desc", "label": "ID (Descending)" },
	{ "value": "equipment_code::asc", "label": "Equipment code (Ascending)" },
	{ "value": "equipment_code::desc", "label": "Equipment code (Descending)" },
];

function parse_pagination_params(url: string): {
	query: string;
	offset: number;
	limit: number | "all";
	order_by: string;
} {
	const urlObj = new URL(url, "http://localhost");
	const query = urlObj.searchParams.get("query") || "";
	const offset = Math.max(0, parseInt(urlObj.searchParams.get("offset") || "0", 10));
	const limit_param = urlObj.searchParams.get("limit") || String(DEFAULT_LIMIT);
	const order_by = urlObj.searchParams.get("order_by") || DEFAULT_ORDER_BY;

	let limit: number | "all" = DEFAULT_LIMIT;
	if (limit_param === "all") { limit = "all"; } else {
		const parsed = parseInt(limit_param, 10);
		limit = !Number.isNaN(parsed) ? parsed : DEFAULT_LIMIT;
	}

	return {
		query,
		offset,
		limit,
		order_by,
	};
}

function get_limit_options(current_limit: number | "all"): (number | "all")[] {
	const options = [...ALLOWED_LIMITS];
	if (current_limit !== "all" && !options.includes(current_limit as number)) {
		options.push(current_limit as number);
		options.sort((a, b) => (a as number) - (b as number));
	}
	options.push("all");

	return options;
}

function build_pagination_urls(current_offset: number, limit_numeric: number, total: number, query: string, order_by: string): {
	prev_url: string | null;
	next_url: string | null;
	first_url: string | null;
	last_url: string | null;
} {
	const bp = base_path();
	const query_param = query ? `&query=${encodeURIComponent(query)}` : "";
	const limit_param = `limit=${limit_numeric}`;
	const order_param = `&order_by=${encodeURIComponent(order_by)}`;

	const prev_offset = Math.max(0, current_offset - limit_numeric);
	const prev_url = current_offset > 0 ? `${bp}?offset=${prev_offset}&${limit_param}${query_param}${order_param}` : null;

	const next_offset = current_offset + limit_numeric;
	const next_url = next_offset < total ? `${bp}?offset=${next_offset}&${limit_param}${query_param}${order_param}` : null;

	const last_offset = total > 0 ? Math.max(0, Math.ceil(total / limit_numeric) * limit_numeric - limit_numeric) : 0;
	const first_url = current_offset > 0 ? `${bp}?offset=0&${limit_param}${query_param}${order_param}` : null;
	const last_url = total > 0 && current_offset < last_offset ? `${bp}?offset=${last_offset}&${limit_param}${query_param}${order_param}` : null;

	return {
		prev_url,
		next_url,
		first_url,
		last_url,
	};
}

function base_path(parent_id: number | string): string { return `${route_prefix}/equipment/${String(parent_id)}/equipment_items`; }

function entity_path(parent_id: number | string, child_id?: number | string): string { return child_id ? `${base_path(parent_id)}/${child_id}/edit` : base_path(parent_id); }

function child_data_path(parent_id: number | string, child_id: number | string): string { return `${base_path(parent_id)}/${child_id}/edit-data`; }


export async function post_equipment_items_validate(req: BunRequest): Promise<Response> {
	const body = await req.json();
	const translated = await translated_from_request(req, import.meta.dir);
	const touched: string[] = body.touched || [];

	const data = {
		title: body.title || "",
		equipment_code: body.equipment_code || "",
		cena: body.cena || "",
		kratica: body.kratica || "",
	};

	const [errors, valid_data] = validate_touched(data, touched, translated.errors);
	const success = Object.keys(errors).length === 0;

	return new Response(JSON.stringify({ success, errors }), { status: 200, headers: { "Content-Type": "application/json" } });
}


export async function post_equipment_items_index(req: BunRequest): Promise<Response> {
	const body = await req.text();
	const translated = await translated_from_request(req, import.meta.dir);
	const params = new URLSearchParams(body);

	const data = {
		title: params.get(`title`)?.trim() || "",
		equipment_code: params.get(`equipment_code`)?.trim() || "",
		cena: params.get(`cena`)?.trim() || "",
		kratica: params.get(`kratica`)?.trim() || "",
	};

	// Preserve parent FK before validation (required by Zod schema)
	data.equipment_code = req.params.code;

	const [errors, valid_data] = validate(data, translated.errors);

	if (Object.keys(errors).length > 0) { return new Response(JSON.stringify({ success: false, errors }), { status: 422, headers: { "Content-Type": "application/json" } }); }

	try {
		const created_record = await create_record(valid_data);
		sql_log({ s: "Create", t: `${feature}`, r: { ...created_record } }, req);

		return new Response(JSON.stringify({ success: true, record: created_record }), { headers: { "Content-Type": "application/json" } });

	} catch (error) {
		const error_key = error instanceof Error && error.message.toLowerCase().includes("duplicate entry") ? "duplicate_key" : "error_creating_record";

		const error_message = translated.errors[error_key];

		return new Response(JSON.stringify({ success: false, form_errors: error_message, errors }), { status: 422, headers: { "Content-Type": "application/json" } });
	}

}


export async function get_equipment_items_edit(req: BunRequest): Promise<Response> {
	const parent_id = req.params.code;
	const child_id = req.params.id || "";
	const record = await get_record_by_id_and_parent(child_id, parent_id);

	if (!record) { return new Response(JSON.stringify({ error: "Not found" }), { status: 404, headers: { "Content-Type": "application/json" } }); }

	return new Response(JSON.stringify({ record }), { headers: { "Content-Type": "application/json" } });
}


export async function post_equipment_items_edit(req: BunRequest): Promise<Response> {
	const parent_id = req.params.code;
	const child_id = req.params.id || "";
	const lookup_record = await get_record_by_id_and_parent(child_id, parent_id);
	const id = lookup_record?.id || "";
	if (!lookup_record) { return new Response(JSON.stringify({ error: "Not found" }), { status: 404, headers: { "Content-Type": "application/json" } }); }

	const body = await req.text();
	const translated = await translated_from_request(req, import.meta.dir);
	const params = new URLSearchParams(body);
	const action = params.get("_action");

	if (action === "delete") {
		try {
			const deleted = await await delete_record(child_id);

			if (deleted) {
				await cache.invalidate(TABLE_NAME);
				await cache.invalidate("equipment");
				sql_log({ s: "Delete", t: `${feature}`, id: child_id }, req);
				return new Response(JSON.stringify({ success: true }), { headers: { "Content-Type": "application/json" } });
			}

			return new Response(JSON.stringify({ error: "Not found" }), { status: 404, headers: { "Content-Type": "application/json" } });
		} catch (error) {
			const error_message = error instanceof Error && error.message.includes("foreign key") ? "Cannot delete this record because it's referenced by other records." : "Error deleting record.";

			return new Response(JSON.stringify({ error: error_message }), { status: 400, headers: { "Content-Type": "application/json" } });
		}
	}

	const data = {
		title: params.get(`title`)?.trim() || "",
		equipment_code: params.get(`equipment_code`)?.trim() || "",
		cena: params.get(`cena`)?.trim() || "",
		kratica: params.get(`kratica`)?.trim() || "",
	};

	// Preserve parent FK before validation (required by Zod schema)
	data.equipment_code = parent_id;

	const [errors, valid_data] = validate(data, translated.errors);

	if (Object.keys(errors).length > 0) { return new Response(JSON.stringify({ success: false, errors }), { status: 422, headers: { "Content-Type": "application/json" } }); }

	let record;
	try {
		record = await update_record(id, valid_data);
		await cache.invalidate(TABLE_NAME);
		await cache.invalidate("equipment");
		sql_log({ s: "Update", t: `${feature}`, r: { ...record } }, req);
	} catch (error) {
		const error_key = error instanceof Error && error.message.toLowerCase().includes("duplicate entry") ? "duplicate_key" : "error_creating_record";

		const error_message = translated.errors[error_key];

		return new Response(JSON.stringify({ success: false, form_errors: error_message, errors }), { status: 422, headers: { "Content-Type": "application/json" } });
	}

	if (!record) { return new Response(JSON.stringify({ error: "Not found" }), { status: 404, headers: { "Content-Type": "application/json" } }); }

	return new Response(JSON.stringify({ success: true, record }), { headers: { "Content-Type": "application/json" } });
}
