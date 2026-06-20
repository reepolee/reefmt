// This file is auto-generated. Do not modify manually.
import { type FormFieldDef } from "$root/generator/schema/types";

export type equipment_items_type = {
	id?: number;
	title?: string;
	equipment_code?: string;
	cena?: number | null | undefined;
	kratica?: string | null | undefined;
	created_at?: string | null | undefined;
	updated_at?: string | null | undefined;
};

export const fields: Record<string, FormFieldDef> = {
	"title": {
		"name": "title",
		"type": "text",
		"required": true,
		"is_nullable": false,
		"max": 30,
		"attributes": {
			"column_type": "varchar(30)",
			"domain_type": "street_extra",
			"domain_compliant": true,
			"initial_width": "30ch",
			"initial_class": "",
		},
	},
	"equipment_code": {
		"name": "equipment_code",
		"type": "select",
		"required": true,
		"is_nullable": false,
		"attributes": {
			"column_type": "varchar(10)",
			"foreign_key": { "table": "equipment", "column": "code" },
			"fk_type": "string",
			"domain_type": "code_medium",
			"domain_compliant": true,
			"initial_width": "10ch",
			"initial_class": "",
		},
	},
	"cena": {
		"name": "cena",
		"type": "number",
		"required": false,
		"is_nullable": true,
		"attributes": {
			"column_type": "decimal(18,2)",
			"domain_type": "currency",
			"domain_compliant": true,
			"initial_width": "20ch",
			"initial_class": "text-right",
		},
	},
	"kratica": {
		"name": "kratica",
		"type": "text",
		"required": false,
		"is_nullable": true,
		"max": 10,
		"attributes": {
			"column_type": "varchar(10)",
			"domain_type": "code_medium",
			"domain_compliant": true,
			"initial_width": "10ch",
			"initial_class": "",
		},
	},
};
export const indexed_columns: string[] = ["id", "equipment_code"];


export const v_fields: Record<string, FormFieldDef> | null = null;

export const parent = {
	"table": "equipment",
	"fk_column": "equipment_code",
	"route_param": "code",
	"label": "Equipment",
};
