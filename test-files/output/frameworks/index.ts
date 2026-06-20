import {
	get_table_name_from_dir,
	translated_from_request,
	get_cookie,
	get_lang_from_request,
	localized_url,
	get_cookies_by_prefix,
	create_toast_cookie,
	format_bulk_delete_message,
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
	TABLE_NAME,
} from "./sql";
import { cache } from "$lib/cache";
import { get_all_records_view } from "./sql_view";
;
import { default_language, language_locales } from "$config/supported_languages";
import { sql_log } from "$lib/logger";
import { Cookie, type BunRequest } from "bun";

import { validate, validate_touched } from "./schema/validation_server";
import { columns, enable_delete, fields } from "./schema/table";

import { get_authors_options_by_id } from "./sql";

import { get_languages_options_by_id } from "./sql";

;


export const user_frameworks_crud = {
	"/frameworks": { GET: get_frameworks_index, POST: post_frameworks_index },
	"/frameworks/new": get_frameworks_new,
	"/frameworks/validate": { POST: post_frameworks_validate },
	"/frameworks/:id/edit": { GET: get_frameworks_edit, POST: post_frameworks_edit },
	"/frameworks/bulk-delete": { POST: post_frameworks_bulk_delete },

};


const feature = get_table_name_from_dir(import.meta.dir);
const route_prefix = "/user";
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
	{ "value": "name::asc", "label": "Name (Ascending)" },
	{ "value": "name::desc", "label": "Name (Descending)" },
	{ "value": "author_id::asc", "label": "Author id (Ascending)" },
	{ "value": "author_id::desc", "label": "Author id (Descending)" },
	{ "value": "reviewer_id::asc", "label": "Reviewer id (Ascending)" },
	{ "value": "reviewer_id::desc", "label": "Reviewer id (Descending)" },
	{ "value": "language_id::asc", "label": "Language id (Ascending)" },
	{ "value": "language_id::desc", "label": "Language id (Descending)" },
];

function base_path(): string { return route_prefix ? `${route_prefix}/${feature}` : `/${feature}`; }

function entity_path(id?: number | string): string { return id ? `${base_path()}/${id}/edit` : base_path(); }

function parse_pagination_params(url: string): {
	query: string;
	offset: number;
	limit: number | "all";
	order_by: string;
	scope: string;
	filters: Record<string, string>;
} {
	const urlObj = new URL(url, "http://localhost");
	const query = urlObj.searchParams.get("query") || "";
	const offset = Math.max(0, parseInt(urlObj.searchParams.get("offset") || "0", 10));
	const limit_param = urlObj.searchParams.get("limit") || String(DEFAULT_LIMIT);
	const order_by = urlObj.searchParams.get("order_by") || DEFAULT_ORDER_BY;
	const scope = urlObj.searchParams.get("scope") || "";

	let limit: number | "all" = DEFAULT_LIMIT;
	if (limit_param === "all") { limit = "all"; } else {
		const parsed = parseInt(limit_param, 10);
		limit = !Number.isNaN(parsed) ? parsed : DEFAULT_LIMIT;
	}

	// Extract filter_ params - accumulate multi-values as comma-separated
	const filters: Record<string, string> = {};
	const filter_not: Record<string, string> = {};
	for (const [key, value] of urlObj.searchParams.entries()) {
		if (key.startsWith("filter_not_")) {
			const fkey = key.slice(11);
			filter_not[fkey] = value;
		} else if (key.startsWith("filter_")) {
			const fkey = key.slice(7);
			if (filters[fkey]) { filters[fkey] += `,${value}`; } else { filters[fkey] = value; }
		}
	}

	return {
		query,
		offset,
		limit,
		order_by,
		scope,
		filters,
		filter_not,
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

function build_filter_param(filters: Record<string, string>): string {
	const parts = Object.entries(filters).map(([key, value]) => `filter_${encodeURIComponent(key)}=${encodeURIComponent(value)}`);
	return parts.length > 0 ? `&${parts.join("&")}` : "";
}

function build_pagination_urls(
	current_offset: number,
	limit_numeric: number,
	total: number,
	query: string,
	order_by: string,
	scope: string = "",
	filters: Record<string, string> = {},
): {
	prev_url: string | null;
	next_url: string | null;
	first_url: string | null;
	last_url: string | null;
} {
	const bp = base_path();
	const query_param = query ? `&query=${encodeURIComponent(query)}` : "";
	const limit_param = `limit=${limit_numeric}`;
	const order_param = `&order_by=${encodeURIComponent(order_by)}`;
	const scope_param = scope ? `&scope=${encodeURIComponent(scope)}` : "";
	const filter_param = build_filter_param(filters);

	const prev_offset = Math.max(0, current_offset - limit_numeric);
	const prev_url = current_offset > 0 ? `${bp}?offset=${prev_offset}&${limit_param}${query_param}${order_param}${scope_param}${filter_param}` : null;

	const next_offset = current_offset + limit_numeric;
	const next_url = next_offset < total ? `${bp}?offset=${next_offset}&${limit_param}${query_param}${order_param}${scope_param}${filter_param}` : null;

	const last_offset = total > 0 ? Math.max(0, Math.ceil(total / limit_numeric) * limit_numeric - limit_numeric) : 0;
	const first_url = current_offset > 0 ? `${bp}?offset=0&${limit_param}${query_param}${order_param}${scope_param}${filter_param}` : null;
	const last_url = total > 0 && current_offset < last_offset ? `${bp}?offset=${last_offset}&${limit_param}${query_param}${order_param}${scope_param}${filter_param}` : null;

	return {
		prev_url,
		next_url,
		first_url,
		last_url,
	};
}

function get_redirect_from_referer(req: BunRequest): string | null {
	const referer = req.headers.get("referer");
	if (!referer) return null;
	try {
		const url = new URL(referer);
		const path = url.pathname + url.search;
		if (path.includes(base_path()) && !path.includes("/edit")) return path;
	} catch (e) {
		console.warn("Invalid referer URL:", referer, e);
	}
	return null;
}


export async function post_frameworks_validate(req: BunRequest): Promise<Response> {
	const body = await req.json();
	const translated = await translated_from_request(req, import.meta.dir);
	const touched: string[] = body.touched || [];

	const data = {
		name: body.name || "",
		tagline: body.tagline || "",
		author_id: body.author_id || "",
		reviewer_id: body.reviewer_id || "",
		language_id: body.language_id || "",
		first_commit_at: body.first_commit_at || "",
		is_javascript: body.is_javascript || "",
	};

	const [errors, valid_data] = validate_touched(data, touched, translated.errors);
	const success = Object.keys(errors).length === 0;

	return new Response(JSON.stringify({ success, errors }), { status: 200, headers: { "Content-Type": "application/json" } });
}


export async function get_frameworks_index(req: BunRequest): Promise<Response> {
	const ctx = await create_ctx(req, import.meta.dir);
	// Read toast cookies so they survive page reload
	ctx.toasts = get_cookies_by_prefix(req, "toast-");
	const { query, offset, limit, order_by, scope, filters, filter_not } = parse_pagination_params(req.url);
	const limit_numeric = limit === "all" ? 999999 : limit;

	// Derive module_code from route_prefix so scopes are filtered by module
	const module_code = route_prefix ? route_prefix.slice(1) : "";

	// Resolve table scopes and translations in parallel
	const [global_scopes, translated] = await Promise.all([get_global_scopes(TABLE_NAME, "frameworks", module_code), translated_from_request(req, import.meta.dir)]);
	const scope_key = scope || get_cookie(req, "scope_frameworks") || global_scopes.find((s) => s.is_default)?.scope_key || "";
	const scope_clause = scope_key ? await get_scope_clause(TABLE_NAME, scope_key, ctx, "frameworks", module_code) : "";

	// Resolve filter definitions and WHERE clauses from URL params
	const raw_filter_defs = get_filter_defs(columns, fields);
	const filter_clauses = resolve_filters(raw_filter_defs, filters, filter_not);

	// Load FK filter options for filter panel checkboxes
	const filter_author_id_options = await get_authors_options_by_id();
	const filter_reviewer_id_options = await get_authors_options_by_id();
	const filter_language_id_options = await get_languages_options_by_id();

	// Enrich filter_defs with translated labels, option lists, and URL param state
	const { labels } = translated;
	const filter_defs = enrich_filter_defs(raw_filter_defs, labels, filters, filter_not, {
		author_id: filter_author_id_options,
		reviewer_id: filter_reviewer_id_options,
		language_id: filter_language_id_options,
	});

	let result: { records: any[]; total: number; };

	try { result = await get_all_records_view(query, offset, limit_numeric, order_by, scope_clause, filter_clauses); } catch (e) {
		console.warn("View v_frameworks not found, using table:", e);
		result = await search_records(query, offset, limit_numeric, order_by, scope_clause, filter_clauses);
	}


	const limit_options = get_limit_options(limit === "all" ? "all" : (limit as number));

	const { prev_url, next_url, first_url, last_url } = build_pagination_urls(offset, limit_numeric, result.total, query, order_by, scope_key, filters);

	// Build dynamic grid cols from the columns map (exclude grid: false columns)
	// Last column gets "auto" so it fills remaining row width
	const grid_cols = `${Object.entries(columns)
		.filter(([_, v]: [string, any]) => v.grid !== false)
		.map(([_, v]: [string, any]) => (typeof v === "string" ? v : v.width))
		.join(" ")} auto`;

	return render("index", {
		data: {
			title: "Frameworks",
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
			global_scopes,
			scope: scope_key,
			columns,
			grid_cols,
			filter_defs,
			filter_clauses,
			filter_params: filters,
			filter_not_params: filter_not,
			active_filter_count: filter_clauses.length,
			...translated,
			v_labels: translated.v_labels || {},
			scopes: translated.scopes || {},
			enable_delete,
		},
		ctx,
	});
}


export async function post_frameworks_index(req: BunRequest): Promise<Response> {
	const ctx = await create_ctx(req, import.meta.dir);
	const body = await req.text();
	const _lang = get_lang_from_request(req) || default_language;
	const translated = await translated_from_request(req, import.meta.dir);
	const params = new URLSearchParams(body);

	const data = {
		name: params.get(`name`)?.trim() || "",
		tagline: params.get(`tagline`)?.trim() || "",
		author_id: params.get(`author_id`)?.trim() || "",
		reviewer_id: params.get(`reviewer_id`)?.trim() || "",
		language_id: params.get(`language_id`)?.trim() || "",
		first_commit_at: params.get(`first_commit_at`)?.trim() || "",
		is_javascript: params.get(`is_javascript`)?.trim() || "",
	};



	const [errors, valid_data] = validate(data, translated.errors);

	if (Object.keys(errors).length > 0) {
		return render("form", {
			data: {
				record: data,
				errors,
				action: base_path(),
				...translated,
				enable_delete,
			},
			ctx,
		});
	}

	try {
		const created_record = await create_record(valid_data);
		await cache.invalidate(TABLE_NAME);
		sql_log({ s: "Create", "t": `${feature}`, r: { ...created_record } }, req);

		const save_action = params.get("_save_action");
		if (save_action === "stay") {
			// Save: go to edit page for new record
			const route_param_value = created_record.id || created_record.id;
			return Response.redirect(localized_url(entity_path(route_param_value), _lang), 303);
		}
		return Response.redirect(localized_url(base_path(), _lang), 303);

	} catch (error) {
		const error_key = error instanceof Error && error.message.toLowerCase().includes("duplicate entry") ? "duplicate_key" : "error_creating_record";

		const error_message = translated.errors[error_key];

		return render("form", {
			data: {
				save_label: "Shrani zapis",
				title: "New record",
				record: data,
				errors,
				form_errors: error_message,
				action: base_path(),
				...translated,
				enable_delete,
			},
			ctx,
		});
	}

}


export async function get_frameworks_new(req: BunRequest): Promise<Response> {
	const ctx = await create_ctx(req, import.meta.dir);
	// Start translation fetch early so it runs in parallel with FK/tags/autocomplete loaders
	const translated_promise = translated_from_request(req, import.meta.dir);

	const authors_options_by_id = await get_authors_options_by_id();
	const languages_options_by_id = await get_languages_options_by_id();



	const translated = await translated_promise;

	return render("form", {
		data: {
			title: "New record",
			record: {
				name: "",
				tagline: "",
				author_id: "",
				reviewer_id: "",
				language_id: "",
				first_commit_at: "",
				is_javascript: -1,
			},
			errors: {
				name: "",
				tagline: "",
				author_id: "",
				reviewer_id: "",
				language_id: "",
				first_commit_at: "",
				is_javascript: "",
			},
			action: base_path(),
			...translated,
			authors_options_by_id,
			languages_options_by_id,


			enable_delete,
		},
		ctx,
	});
}


export async function get_frameworks_edit(req: BunRequest): Promise<Response> {
	const ctx = await create_ctx(req, import.meta.dir);
	// Start translation fetch early so it runs in parallel with the record lookup
	const translated_promise = translated_from_request(req, import.meta.dir);
	const id = req.params.id ? String(req.params.id) : "";
	const record = await get_record_by_id(id);

	if (!record) { return render("notfound", { data: { title: "404 Not Found" }, status: 404, ctx }); }

	const authors_options_by_id = await get_authors_options_by_id();
	const languages_options_by_id = await get_languages_options_by_id();


	// crud:child:fetch:start
	// crud:child:fetch:end

	const translated = await translated_promise;

	const bp = base_path();
	return render("form", {
		data: {
			title: `Edit ${record.name}`,
			record,
			back_route: `${bp}?there_should_be_back_params`,
			errors: {
				name: "",
				tagline: "",
				author_id: "",
				reviewer_id: "",
				language_id: "",
				first_commit_at: "",
				is_javascript: "",
			},
			action: entity_path(record.id),
			...translated,
			authors_options_by_id,
			languages_options_by_id,


			// crud:child:data:start
			// crud:child:data:end
			enable_delete,
		},
		ctx,
	});
}


export async function post_frameworks_edit(req: BunRequest): Promise<Response> {
	const ctx = await create_ctx(req, import.meta.dir);
	const id = req.params.id ? String(req.params.id) : "";
	const body = await req.text();
	const _lang = get_lang_from_request(req) || default_language;
	const translated = await translated_from_request(req, import.meta.dir);
	const params = new URLSearchParams(body);
	const action = params.get("_action");
	const return_url_from_form = params.get("_return_url");
	const save_action = params.get("_save_action");

	const bp = base_path();
	let redirect_url = localized_url(bp, _lang);
	if (save_action === "stay") {
		// Save: stay on edit page - id is always available from the lookup above
		redirect_url = localized_url(entity_path(id), _lang);
	} else if (return_url_from_form?.includes(bp)) { redirect_url = return_url_from_form; } else {
		const redirect_from_referer = get_redirect_from_referer(req);
		if (redirect_from_referer) redirect_url = redirect_from_referer;
	}
	if (action === "delete") {
		if (!enable_delete) { return new Response(JSON.stringify({ error: "Delete is disabled." }), { status: 403, headers: { "Content-Type": "application/json" } }); }
		try {
			const deleted = await delete_record(id);

			if (deleted) {
				await cache.invalidate(TABLE_NAME);
				sql_log({ s: "Delete", "t": `${feature}`, id }, req);
				return Response.redirect(redirect_url, 303);
			}

			return render("notfound", { data: { title: "404 Not Found", ...translated }, status: 404, ctx });
		} catch (error) {
			const existing_record = await get_record_by_id(id);
			if (!existing_record) { return render("notfound", { data: { title: "404 Not Found", ...translated }, status: 404, ctx }); }

			const error_message = error instanceof Error && error.message.includes("foreign key") ? "Cannot delete this record because it's referenced by other records." : "Error deleting record.";

			return render("form", {
				data: {
					title: `Edit ${existing_record.name}`,
					record: existing_record,
					form_errors: error_message,
					errors: {},
					action: entity_path(id),
					...translated,
					enable_delete,
				},
				ctx,
			});
		}
	}


	const data = {
		name: params.get(`name`)?.trim() || "",
		tagline: params.get(`tagline`)?.trim() || "",
		author_id: params.get(`author_id`)?.trim() || "",
		reviewer_id: params.get(`reviewer_id`)?.trim() || "",
		language_id: params.get(`language_id`)?.trim() || "",
		first_commit_at: params.get(`first_commit_at`)?.trim() || "",
		is_javascript: params.get(`is_javascript`)?.trim() || "",
	};

	const [errors, valid_data] = validate(data, translated.errors);

	if (Object.keys(errors).length > 0) {
		const existing_record = await get_record_by_id(id);
		if (!existing_record) { return render("notfound", { data: { title: "404 Not Found", ...translated }, status: 404, ctx }); }
		// crud:child:fetch:start
		// crud:child:fetch:end
		return render("form", {
			data: {
				title: `Edit ${existing_record.name}`,
				record: { ...existing_record, ...data },
				errors,
				action: entity_path(id),
				...translated,
				// crud:child:data:start
				// crud:child:data:end
				enable_delete,
			},
			ctx,
		});
	}


	let record;
	try {
		record = await update_record(id, valid_data);
		await cache.invalidate(TABLE_NAME);
		sql_log({ s: "Update", "t": `${feature}`, r: { ...record } }, req);
	} catch (error) {
		const error_key = error instanceof Error && error.message.toLowerCase().includes("duplicate entry") ? "duplicate_key" : "error_creating_record";

		const error_message = translated.errors[error_key];

		return render("form", {
			data: {
				record: data,
				errors,
				form_errors: error_message,
				action: entity_path(id),
				...translated,
				enable_delete,
			},
			ctx,
		});
	}


	if (!record) { return render("notfound", { data: { title: "404 Not Found", ...translated }, status: 404, ctx }); }

	const cookie = await create_toast_cookie({
		record_id: record.id,
		feature,
		message: translated.messages.record_updated,
		type: "green",
		req,
	});

	const headers = new Headers({ Location: redirect_url });

	headers.append("Set-Cookie", cookie.toString());

	return new Response(null, { status: 303, headers });
}


export async function post_frameworks_bulk_delete(req: BunRequest): Promise<Response> {
	if (!enable_delete) { return new Response(JSON.stringify({ error: "Bulk delete is disabled." }), { status: 403, headers: { "Content-Type": "application/json" } }); }
	const translated = await translated_from_request(req, import.meta.dir);
	const msg = translated.messages ?? {};
	const lang = get_lang_from_request(req) || default_language;
	const locale = language_locales[lang] || "en-US";

	try {
		const body = await req.json();
		const ids: (number | string)[] = body.ids || [];

		if (!Array.isArray(ids) || ids.length === 0) {
			return new Response(JSON.stringify({ error: msg.bulk_delete_no_ids || "No records selected." }), { status: 400, headers: { "Content-Type": "application/json" } });
		}

		let deleted_count = 0;
		let error_count = 0;
		for (const id of ids) {
			try {
				const deleted = await delete_record(String(id));
				if (deleted) {
					sql_log({ s: "Delete", t: `${feature}`, id: String(id) }, req);
					deleted_count++;
				} else { error_count++; }
			} catch (err) {
				console.error(`⚠️  Bulk delete error for ID ${id}:`, err);
				error_count++;
			}
		}

		await cache.invalidate(TABLE_NAME);

		const message = format_bulk_delete_message(msg, deleted_count, error_count, "record", locale);

		// Set toast cookie so the message survives page reload
		const toast_type = error_count > 0 && deleted_count === 0 ? "red" : "green";
		const toast_cookie = new Cookie({
			name: "toast-bulk-delete",
			value: JSON.stringify({
				id: "toast-bulk-delete",
				message,
				type: toast_type,
				duration: 4000,
			}),
			path: "/",
		});

		return new Response(JSON.stringify({ deleted: deleted_count, errors: error_count, message }), {
			status: 200,
			headers: { "Content-Type": "application/json", "Set-Cookie": toast_cookie.toString() },
		});
	} catch (err) {
		console.error("⚠️  Bulk delete failed:", err);
		return new Response(JSON.stringify({ error: msg.bulk_delete_failed || "Bulk delete failed." }), { status: 500, headers: { "Content-Type": "application/json" } });
	}
}
