import { z } from "$vendor/zod.min.js";
import {
	validate_schema,
	z_date_required,
	z_date_optional,
	z_datetime_required,
	z_datetime_optional,
} from "$lib/validation_helpers";


export const schema = z.object({
	id: z.coerce.number().optional(),
	name: z.string().min(1, "name_required").max(15, "name_max"),
	tagline: z.string().min(1, "tagline_required").max(100, "tagline_max"),
	author_id: z.coerce.number().min(1, "must_be_selected"),
	reviewer_id: z.coerce.number().min(1, "must_be_selected"),
	language_id: z.coerce.number().min(1, "must_be_selected"),
	first_commit_at: z_date_required,
	is_javascript: z.coerce.number().min(0, "is_javascript_required"),
});


export const validate = (data: any, messages?: Record<string, string>) => { return validate_schema(schema, data, undefined, messages); };

export const validate_touched = (data: any, touched: string[], messages?: Record<string, string>) => { return validate_schema(schema, data, touched, messages); };
