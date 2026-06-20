// This file is auto-generated. Do not modify manually.
import { type FormFieldDef } from "$root/generator/schema/types";

export type frameworks_type = {
	id?: number;
	name?: string;
	tagline?: string;
	author_id?: number;
	reviewer_id?: number;
	language_id?: number;
	first_commit_at?: string;
	is_javascript?: number;
	created_at?: string;
	updated_at?: string;
};

export const fields: Record<string, FormFieldDef> = {
	"name": {
		"name": "name",
		"type": "text",
		"required": true,
		"is_nullable": false,
		"max": 15,
		"attributes": {
			"column_type": "varchar(15)",
			"domain_type": null,
			"domain_compliant": false,
			"initial_width": "15ch",
			"initial_class": "",
		},
	},
	"tagline": {
		"name": "tagline",
		"type": "text",
		"required": true,
		"is_nullable": false,
		"max": 100,
		"attributes": {
			"column_type": "varchar(100)",
			"domain_type": "first_name",
			"domain_compliant": true,
			"initial_width": "80ch",
			"initial_class": "",
		},
	},
	"author_id": {
		"name": "author_id",
		"type": "select",
		"required": true,
		"is_nullable": false,
		"attributes": {
			"column_type": "int(10) unsigned",
			"filter": true,
			"foreign_key": { "table": "authors", "column": "id" },
			"fk_type": "number",
			"domain_type": null,
			"domain_compliant": false,
			"initial_width": "auto",
			"initial_class": "",
		},
	},
	"reviewer_id": {
		"name": "reviewer_id",
		"type": "select",
		"required": true,
		"is_nullable": false,
		"attributes": {
			"column_type": "int(10) unsigned",
			"filter": true,
			"foreign_key": { "table": "authors", "column": "id" },
			"fk_type": "number",
			"domain_type": null,
			"domain_compliant": false,
			"initial_width": "auto",
			"initial_class": "",
		},
	},
	"language_id": {
		"name": "language_id",
		"type": "select",
		"required": true,
		"is_nullable": false,
		"attributes": {
			"column_type": "int(10) unsigned",
			"filter": true,
			"foreign_key": { "table": "languages", "column": "id" },
			"fk_type": "number",
			"domain_type": null,
			"domain_compliant": false,
			"initial_width": "auto",
			"initial_class": "",
		},
	},
	"first_commit_at": {
		"name": "first_commit_at",
		"type": "date",
		"required": true,
		"is_nullable": false,
		"attributes": {
			"column_type": "date",
			"domain_type": "timestamp",
			"domain_compliant": false,
			"initial_width": "20ch",
			"initial_class": "",
		},
	},
	"is_javascript": {
		"name": "is_javascript",
		"type": "number",
		"required": true,
		"is_nullable": false,
		"attributes": {
			"column_type": "tinyint(1)",
			"domain_type": "boolean",
			"domain_compliant": true,
			"initial_width": "10ch",
			"initial_class": "text-center",
		},
	},
};
export const indexed_columns: string[] = [
	"id",
	"name",
	"author_id",
	"reviewer_id",
	"language_id",
];


export const v_fields: Record<string, FormFieldDef> = {
	"name": {
		"name": "name",
		"type": "text",
		"required": true,
		"is_nullable": false,
		"max": 15,
		"attributes": {
			"column_type": "varchar(15)",
			"domain_type": null,
			"domain_compliant": false,
			"initial_width": "15ch",
			"initial_class": "",
		},
	},
	"tagline": {
		"name": "tagline",
		"type": "text",
		"required": true,
		"is_nullable": false,
		"max": 100,
		"attributes": {
			"column_type": "varchar(100)",
			"domain_type": "first_name",
			"domain_compliant": true,
			"initial_width": "80ch",
			"initial_class": "",
		},
	},
	"first_commit_at": {
		"name": "first_commit_at",
		"type": "date",
		"required": true,
		"is_nullable": false,
		"attributes": {
			"column_type": "date",
			"domain_type": "timestamp",
			"domain_compliant": false,
			"initial_width": "20ch",
			"initial_class": "",
		},
	},
	"is_javascript": {
		"name": "is_javascript",
		"type": "number",
		"required": true,
		"is_nullable": false,
		"attributes": {
			"column_type": "tinyint(1)",
			"domain_type": "boolean",
			"domain_compliant": true,
			"initial_width": "10ch",
			"initial_class": "text-center",
		},
	},
	"author_name": {
		"name": "author_name",
		"type": "select",
		"required": false,
		"is_nullable": true,
		"attributes": {
			"column_type": "varchar(15)",
			"foreign_key": { "table": "authors", "column": "name" },
			"fk_type": "string",
			"domain_type": null,
			"domain_compliant": false,
			"initial_width": "15ch",
			"initial_class": "",
		},
	},
	"language_name": {
		"name": "language_name",
		"type": "select",
		"required": false,
		"is_nullable": true,
		"attributes": {
			"column_type": "varchar(15)",
			"foreign_key": { "table": "languages", "column": "name" },
			"fk_type": "string",
			"domain_type": null,
			"domain_compliant": false,
			"initial_width": "15ch",
			"initial_class": "",
		},
	},
	"reviewer_name": {
		"name": "reviewer_name",
		"type": "text",
		"required": false,
		"is_nullable": true,
		"max": 15,
		"attributes": {
			"column_type": "varchar(15)",
			"domain_type": null,
			"domain_compliant": false,
			"initial_width": "15ch",
			"initial_class": "",
		},
	},
	"author_id": {
		"name": "author_id",
		"type": "select",
		"required": true,
		"is_nullable": false,
		"attributes": {
			"column_type": "int(10) unsigned",
			"filter": true,
			"foreign_key": { "table": "authors", "column": "id" },
			"fk_type": "number",
			"domain_type": null,
			"domain_compliant": false,
			"initial_width": "auto",
			"initial_class": "",
		},
	},
	"language_id": {
		"name": "language_id",
		"type": "select",
		"required": true,
		"is_nullable": false,
		"attributes": {
			"column_type": "int(10) unsigned",
			"filter": true,
			"foreign_key": { "table": "languages", "column": "id" },
			"fk_type": "number",
			"domain_type": null,
			"domain_compliant": false,
			"initial_width": "auto",
			"initial_class": "",
		},
	},
	"reviewer_id": {
		"name": "reviewer_id",
		"type": "select",
		"required": true,
		"is_nullable": false,
		"attributes": {
			"column_type": "int(10) unsigned",
			"filter": true,
			"foreign_key": { "table": "reviewers", "column": "id" },
			"fk_type": "number",
			"domain_type": null,
			"domain_compliant": false,
			"initial_width": "auto",
			"initial_class": "",
		},
	},
};
