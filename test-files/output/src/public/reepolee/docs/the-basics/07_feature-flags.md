---
title: "Feature Flags"
---

# Feature Flags

<a name="introduction"></a>

## Introduction

Reepolee's feature flag system (`lib/feature_flags.ts`) lets you gate functionality behind per-flag conditions — enabled globally, rolled out to a percentage of users, or enabled for specific users via an allowlist or user override. Flags are backed by Redis and evaluated synchronously per-request.

The system is designed for **operational control**: you can ship code to production that is inactive by default, then enable it gradually as you gain confidence. It is not designed for A/B testing or analytics — those are separate concerns with different data requirements.

```ts
import { is_enabled, get_flags } from "$lib/feature_flags";

// Check a single flag for the current user
if (await is_enabled("new_checkout_flow", user.id)) {
	return render("checkout/v2", { data, ctx });
} else {
	return render("checkout/v1", { data, ctx });
}
```

<a name="setup"></a>

## Setup

Feature flags require Redis. Set `REDIS_URL` in your `.env`:

```env
REDIS_URL=redis://localhost:6379
```

When Redis is not available, all flags evaluate to `false` — your code compiles and runs, but every gated feature is hidden. This is safe for local development where you usually want the stable path anyway.

<a name="managing-flags"></a>

## Managing Flags

Flags are managed programmatically via admin functions. There's no YAML file, no UI panel — you call these from your admin routes or a seed script:

### Create or update a flag

```ts
import { set_flag } from "$lib/feature_flags";

await set_flag("new_checkout_flow", {
	enabled: true,
	rollout_pct: 25, // 25% of users
	description: "New one-page checkout with inline validation",
});
```

### Delete a flag

```ts
import { delete_flag } from "$lib/feature_flags";

await delete_flag("new_checkout_flow");
```

### List all flags

```ts
import { list_flags } from "$lib/feature_flags";

const flags = await list_flags();
// {
//   "new_checkout_flow": { enabled: true, rollout_pct: 25, description: "..." },
//   "dark_mode":         { enabled: false, rollout_pct: 100, description: "..." },
// }
```

<a name="evaluation-strategies"></a>

## Evaluation Strategies

<media-frame label="Diagram — flag evaluation (override → allowlist → percentage)" ratio="16/9"></media-frame>

Flags resolve in the following order:

1. **Per-user override** — force on or off for a specific user.
2. **Allowlist** — specific users always see the feature.
3. **Global enabled + rollout percentage** — percentage of users get the feature.

### Per-user override

Force a flag on or off for a single user, optionally with a TTL:

```ts
import { set_user_override, clear_user_override } from "$lib/feature_flags";

// Force on for 1 hour (e.g., support enabling a feature for a specific user request)
await set_user_override("new_checkout_flow", user_id, true, 3600);

// Force off permanently
await set_user_override("new_checkout_flow", user_id, false);

// Clear the override — user goes back to the normal evaluation chain
await clear_user_override("new_checkout_flow", user_id);
```

### Allowlist

Add users who should always see the feature, regardless of rollout percentage:

```ts
import { add_to_allowlist, remove_from_allowlist } from "$lib/feature_flags";

await add_to_allowlist("new_checkout_flow", user_id_1, user_id_2);
await remove_from_allowlist("new_checkout_flow", user_id_1);
```

Allowlisted users are a superset of the rollout — even at 0% rollout, allowlisted users see the feature. This is useful for internal testing (add your team to the allowlist) before opening the flag to a percentage.

### Percentage rollout

The `rollout_pct` field (0–100) determines what fraction of users see the feature. Users are assigned to buckets deterministically via a hash of `{flag_name}:{user_id}`, so the same user always gets the same result — no flip-flopping between requests:

```ts
await set_flag("new_checkout_flow", {
	enabled: true,
	rollout_pct: 10, // 10% of users
	description: "",
});
```

<a name="checking-flags-in-code"></a>

## Checking Flags in Code

### Single flag

```ts
const enabled = await is_enabled("new_checkout_flow", current_user.id);
```

The `user_id` argument is used for allowlist and percentage-rollout bucketing. If the flag is globally enabled with 100% rollout, the user_id is irrelevant — the flag returns `true` regardless.

### Multiple flags (bulk evaluation)

When you need to check several flags for the same user on the same request, use `get_flags()` to batch the Redis calls:

```ts
const flags = await get_flags(["new_checkout_flow", "dark_mode", "beta_search"], current_user.id);
// { "new_checkout_flow": true, "dark_mode": false, "beta_search": true }
```

This issues a single `Promise.all` under the hood — each flag still hits Redis individually but the queries run concurrently.

<a name="flag-naming-conventions"></a>

## Flag Naming Conventions

- Use `snake_case` for flag names.
- Names should describe the feature, not the outcome: `new_checkout_flow`, `dark_mode`, `beta_search`, `improved_search_relevancy`.
- Avoid flag names that describe the work (e.g., `refactor_step_1`) — flags control user-facing behaviour, not code paths.
- Once a flag is fully rolled out (100% of users for a stable period), remove the flag check from the code and call `delete_flag()` to clean up Redis.

<a name="from-flag-to-permanent"></a>

## The Lifecycle of a Flag

A healthy flag goes through these stages:

1. **Development** — flag doesn't exist yet. Feature is built behind `is_enabled()` with a fallback to the old behaviour. Since the flag doesn't exist in Redis, `is_enabled()` returns `false`, and old code runs.
2. **Internal testing** — ship to production. Create the flag in Redis, add your team to the allowlist. Only you see the new feature in production.
3. **Canary** — set `rollout_pct: 5`, then `25`, then `50`. Monitor error rates and user feedback at each step.
4. **Full rollout** — set `enabled: true, rollout_pct: 100`. Everyone sees the feature.
5. **Cleanup** — remove the flag check from the code and call `delete_flag()`. The old code path no longer exists in the codebase.

Most flags take days to weeks to go from 1 to 5. Some never reach 5 — if the feature doesn't work out, set `enabled: false` and leave the dead code path in place until the next cleanup cycle.

<a name="when-to-use-flags-vs-if-else"></a>

## When to Use Flags vs Plain If/Else

Use feature flags when:

- You need to make a runtime decision about which code path to execute.
- You want to change the behaviour without deploying new code.
- You want to gradually expose a feature to more users.
- You need to quickly disable a feature without rolling back a deploy.

Use plain `if/else` when:

- The decision is based on user permissions or subscription tier (use the user object directly).
- The decision never changes after deploy (remove the dead code instead of gating it).
- The gating is temporary and you plan to clean it up in the same PR.

<a name="security-notes"></a>

## Security Notes

- Flag names and states are stored in Redis **without encryption**. Anyone with Redis access can read or modify flags. Protect Redis with a strong password and network isolation.
- Flag evaluation does not log access. If you need an audit trail of who saw which feature, add logging around your `is_enabled()` call site.
- Per-user overrides with a TTL are appropriate for temporary support fixes. For permanent user-level overrides (e.g., "this enterprise customer always gets the beta"), use the allowlist instead.
