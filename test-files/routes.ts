import { feature_enabled, feature_routes } from "$lib/helpers";
import { build_nav_routes, build_routes, type RouteDef } from "$lib/route_builder";
import { home_page } from "$routes/home";
import { auth_crud } from "$routes/system/auth";
import { cache_routes } from "$routes/system/cache";
import { system_global_scopes_crud } from "$routes/system/global_scopes";
import { system_modules_crud } from "$routes/system/modules";
import { system_queues_page } from "$routes/system/queues";
import { rate_limit_routes } from "$routes/system/rate_limits";
import { system_users_crud } from "$routes/system/users";

import { about_page } from "./examples/about";
import { email_page } from "./examples/email";
import { modern_css_page } from "./examples/modern_css";
import { open_free_map_page } from "./examples/open_free_map";
import { signals_page } from "./examples/signals";
import { system_images_crud } from "./system/images";
import { system_translations_crud } from "./system/translations";

const route_defs: RouteDef[] = [
	// Pages
	{ url: "/", handler: home_page },
	{ url: "/examples/about", handler: about_page, nav_title_key: "examples.about" },
	{ url: "/examples/signals", handler: signals_page, nav_title_key: "examples.signals" },
	{ url: "/examples/modern_css", handler: modern_css_page, nav_title_key: "examples.modern_css" },
	{ url: "/examples/email", resource: email_page, nav_title_key: "examples.email" },
	{ url: "/examples/open_free_map", handler: open_free_map_page, nav_title_key: "examples.open_free_map" },

	// SYSTEM
	...feature_routes(feature_enabled("RATE_LIMITING"), rate_limit_routes),
	...feature_routes(feature_enabled("CACHE_ENABLED"), cache_routes),

	{ url: "/system/queues", crud: system_queues_page, nav_title_key: "system.queues", module: "system" },

	{ url: "/system/users", crud: system_users_crud, nav_title_key: "system.users", module: "system" },
	{ url: "/system/images", crud: system_images_crud, nav_title_key: "system.images", module: "system" },
	{
		url: "/system/global_scopes",
		crud: system_global_scopes_crud,
		nav_title_key: "system.global_scopes",
		module: "system",
	},
	{
		url: "/system/translations",
		crud: system_translations_crud,
		nav_title_key: "system.translations",
		module: "system",
	},
	// GENERATED

	{ url: "/system/modules", crud: system_modules_crud, nav_title_key: "system.modules", module: "system" },
];

export const nav_routes = build_nav_routes(route_defs);

export const routes = {
	...build_routes(route_defs),
	...auth_crud,
	// GENERATED CHILD CRUD:start
	// GENERATED CHILD CRUD:end
};
// [reload 1781625424470,axcz3kquy46]
