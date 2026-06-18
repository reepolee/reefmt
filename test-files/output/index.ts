import { default_language } from "$config/supported_languages";
import { get_available_tables, get_global_scopes, get_scope_clause } from "$lib/global_scopes";
import { get_cookie, get_cookies_by_prefix, get_lang_from_request, get_table_name_from_dir, localized_url, translated_from_request } from "$lib/helpers";
import { sql_log } from "$lib/logger";
import { get_available_modules } from "$lib/modules";
import { build_pagination_urls, get_limit_numeric, get_limit_options, parse_pagination_params } from "$lib/pagination";
import { render } from "$lib/render";
import { create_ctx } from "$lib/request_context";
import { enrich_filter_defs, get_filter_defs, resolve_filters } from "$lib/table_filters";
import type { BunRequest } from "bun";

import { post_global_scopes_edit } from "./edit_handlers";
import { get_global_scopes_edit, get_global_scopes_new, post_global_scopes_bulk_delete, post_global_scopes_test_scope, post_global_scopes_validate } from "./handlers";
import { columns, enable_delete, fields } from "./schema/table";
import { validate } from "./schema/validation_server";
import { create_record, search_records } from "./sql";

export { enable_delete };

export const system_global_scopes_crud = {
	"/global_scopes": { GET: get_global_scopes_index, POST: post_global_scopes_index },
	"/global_scopes/new": get_global_scopes_new,
	"/global_scopes/validate": { POST: post_global_scopes_validate },
	"/global_scopes/test-scope": { POST: post_global_scopes_test_scope },
	"/global_scopes/:id/edit": { GET: get_global_scopes_edit, POST: post_global_scopes_edit },
	"/global_scopes/bulk-delete": { POST: post_global_scopes_bulk_delete }
};


const TABLE_NAME = "global_scopes";
const feature = get_table_name_from_dir(import.meta.dir);
const route_prefix = "/system";
const SORT_OPTIONS = [
	{ value: "id::asc", label: "ID (Ascending)" },
	{ value: "id::desc", label: "ID (Descending)" },
	{ value: "table_name::asc", label: "Table name (Ascending)" },
	{ value: "table_name::desc", label: "Table name (Descending)" },
	{ value: "scope_key::asc", label: "Scope key (Ascending)" },
	{ value: "scope_key::desc", label: "Scope key (Descending)" }
];


function base_path(): string { return route_prefix ? `${route_prefix}/${feature}` : `/${feature}`; }

// ---------------------------------------------------------------------------
// GET /global_scopes — List index
// ---------------------------------------------------------------------------

export async function get_global_scopes_index(req: BunRequest): Promise<Response> {
const [ctx, translated] = await Promise.all([create_ctx(req, import.meta.dir), translated_from_request(req, import.meta.dir)]);
ctx.toasts = get_cookies_by_prefix(req, "toast-");
const { query, offset, limit, order_by, scope, filters, filter_not } = parse_pagination_params(req.url, 20, ["scope"]);
const limit_numeric = get_limit_numeric(limit);
	
const module_code = route_prefix ? route_prefix.slice(1) : "";
	
const global_scopes = await get_global_scopes(TABLE_NAME, "global_scopes", module_code);
const scope_key = scope || get_cookie(req, "scope_global_scopes") || global_scopes.find(s => s.is_default)?.scope_key || "";
const scope_clause = scope_key ? await get_scope_clause(TABLE_NAME, scope_key, ctx, "global_scopes", module_code) : "";
	
const raw_filter_defs = get_filter_defs(columns, fields);
const filter_clauses = resolve_filters(raw_filter_defs, filters, filter_not);
	
	// Enrich filter_defs with translated labels, option lists, and URL param state
const { labels } = translated;
const filter_defs = enrich_filter_defs(raw_filter_defs, labels, filters, filter_not, {  });
	
const result = await search_records(query, offset, limit_numeric, order_by, scope_clause, filter_clauses);
	
const grid_cols = `${Object.entries(columns).filter(([_, v]) => v.grid !== false).map(([_, v]) => (typeof v === "string" ? v : v.width)).join(" ")} auto`;
	
const limit_options = get_limit_options(limit === "all" ? "all" : (limit as number));
	
const { prev_url, next_url, first_url, last_url } = build_pagination_urls(base_path(), offset, limit_numeric, result.total, query, order_by, scope_key, filters);
	
return render("index", { data: {
		title: "Global Scopes",
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
		enable_delete,
		...translated,
		v_labels: translated.v_labels || {  },
		scopes: translated.scopes || {  }
	}, ctx });
}

// ---------------------------------------------------------------------------
// POST /global_scopes — Create new record
// ---------------------------------------------------------------------------

export async function post_global_scopes_index(req: BunRequest): Promise<Response> {
const ctx = await create_ctx(req, import.meta.dir);
const body = await req.text();
const _lang = get_lang_from_request(req) || default_language;
const translated = await translated_from_request(req, import.meta.dir);
const params = new URLSearchParams(body);
	
const data = {
		module_code: params.get("module_code")?.trim?.() || "",
		feature_name: params.get("feature_name")?.trim?.() || "",
		table_name: params.get("table")?.trim?.() || "",
		scope_key: params.get("scope_key")?.trim?.() || "",
		display_name: params.get("display_name")?.trim?.() || "",
		where_clause: params.get("where_clause")?.trim?.() || "",
		sort_order: params.get("sort_order")?.trim?.() || "",
		is_default: params.get("is_default")?.trim?.() || ""
	};
	
const [errors, valid_data] = validate(data, translated.errors);
	
if (Object.keys(errors).length > 0) {
const [module_options, table_options] = await Promise.all([get_available_modules(), get_available_tables()]);
return render("form", { data: {
			record: { ...data, table: data.table_name },
			errors,
			action: base_path(),
			module_options,
			table_options,
			...translated,
			enable_delete
		}, ctx });
	}
	
try {
const created_record = await create_record(valid_data);
sql_log({ s: "Create", t: `${feature}`, r: { ...created_record } }, req);
return Response.redirect(localized_url(base_path(), _lang), 303);
	}
 catch (error) {
const error_key = error instanceof Error && error.message.toLowerCase().includes("duplicate entry") ? "duplicate_key" : "error_creating_record";
const error_message = translated.errors[error_key];
const [module_options, table_options] = await Promise.all([get_available_modules(), get_available_tables()]);
return render("form", { data: {
			save_label: "Shrani zapis",
			title: "New record",
			record: { ...data, table: data.table_name },
			errors,
			form_errors: error_message,
			action: base_path(),
			module_options,
			table_options,
			...translated,
			enable_delete
		}, ctx });
	}
}
