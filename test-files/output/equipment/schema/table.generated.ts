// This file is auto-generated. Do not modify manually.
import { type FormFieldDef } from "$root/generator/schema/types";

export type equipment_type = { id?: number; code?: string; name?: string; };

export const fields: Record<string, FormFieldDef> = {
	"code": {
		"name": "code",
		"type": "text",
		"required": true,
		"is_nullable": false,
		"max": 10,
		"attributes": {
			"column_type": "varchar(10)",
			"domain_type": "code_medium",
			"domain_compliant": true,
			"initial_width": "10ch",
			"initial_class": "",
		},
	},
	"name": {
		"name": "name",
		"type": "text",
		"required": true,
		"is_nullable": false,
		"max": 50,
		"attributes": {
			"column_type": "varchar(50)",
			"domain_type": "phone",
			"domain_compliant": true,
			"initial_width": "50ch",
			"initial_class": "",
		},
	},
};
export const indexed_columns: string[] = ["id", "code"];


export const v_fields: Record<string, FormFieldDef> | null = null;
