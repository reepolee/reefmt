export type { frameworks_type } from "./table.generated";
export { v_fields, fields, indexed_columns } from "./table.generated";

// domain - canonical domain type from DOMAIN_TYPES taxonomy. Null when no match.
// Add compliant column to flag SQL mismatches against the canonical type.
// grid - set to false to hide from index grid while keeping for filtering.
const columns: Record<string, {
	width: string;
	class: string;
	domain?: string;
	filter?: boolean;
	grid?: boolean;
}> = {
	"checkbox": { width: "10ch", class: "text-center" },
	"id": { width: "10ch", class: "" },
	"name": { width: "15ch", class: "" },
	"tagline": { width: "80ch", class: "", domain: "first_name" },
	"first_commit_at": { width: "20ch", class: "", domain: "timestamp" },
	"is_javascript": { width: "10ch", class: "text-center", domain: "boolean" },
	"author_name": { width: "15ch", class: "" },
	"language_name": { width: "15ch", class: "" },
	"reviewer_name": { width: "15ch", class: "" },
	"author_id": {
		width: "auto",
		class: "",
		filter: true,
		grid: false,
	},
	"language_id": {
		width: "auto",
		class: "",
		filter: true,
		grid: false,
	},
	"reviewer_id": {
		width: "auto",
		class: "",
		filter: true,
		grid: false,
	},
};

// Route param for URL paths - change to a different column for URL obscurity.
const route_param = "id";

// Enable/disable delete functionality (bulk delete + record delete).
// Set to true to enable delete for this table. Children in nested CRUD always have delete enabled.
const enable_delete = false;

// Pagination strategy: "cursor" (keyset-based) or "offset" (LIMIT/OFFSET).
// Cursor is best for real-time tables, offset for numbered navigation.
// Set at schema generation time via TUI or --pagination flag.
const pagination_strategy: "cursor" | "offset" = "offset";

// Render strategy: "load" (synchronous, full page after DB query) or "stream" (progressive via DPU).
// Streaming sends the page shell immediately, then streams records and pagination
// as <template for> chunks after DB queries resolve.
const render_strategy: "stream" | "load" = "load";
export { columns, route_param, enable_delete, pagination_strategy, render_strategy };
