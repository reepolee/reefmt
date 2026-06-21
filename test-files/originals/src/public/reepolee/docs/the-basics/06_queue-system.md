---
title: "Queue / Job System"
---

# Queue / Job System

<a name="introduction"></a>

## Introduction

Reepolee ships with a Redis-backed job queue for background processing — sending email, generating reports, processing uploaded images, or any other work that shouldn't happen during a web request. The queue lives in `queue/index.ts` and is self-contained: there's no external library, no worker framework, and no configuration beyond a Redis URL.

The queue model is simple: **enqueue a job**, **a worker picks it up**, **the job runs**. Failed jobs are retried up to a configurable limit, then moved to a dead-letter set for inspection.

```ts
import { enqueue } from "$queue/index";

// In a route handler — returns instantly
const job_id = await enqueue({
	type: "send_email",
	payload: { to: "user@example.com", subject: "Welcome", body: "..." },
});
```

A worker elsewhere in the process — or in a separate process — picks up the job and runs your handler:

```ts
import { start_worker, init_queue } from "$queue/index";

await init_queue(); // connect to Redis

start_worker("send_email", async (job) => {
	await send_mail(job.payload);
});
```

<a name="setup"></a>

## Setup

Set `REDIS_URL` in your `.env`:

```env
REDIS_URL=redis://localhost:6379
```

Initialise the queue once at startup, before starting workers or enqueuing jobs:

```ts
import { init_queue } from "$queue/index";

await init_queue();
// Queue is ready — workers can start, jobs can be enqueued
```

If `REDIS_URL` is not set, `init_queue()` logs a warning and the queue operates in degraded mode: `enqueue()` throws, `start_worker()` is a no-op, and all queue inspection functions return sensible defaults (empty arrays, zero counts). This lets you develop and test without Redis running, as long as your code paths handle enqueue failures gracefully.

<a name="enqueuing-jobs"></a>

## Enqueuing Jobs

```ts
const job_id = await enqueue({
    type: "send_email",           // required — identifies the job handler
    payload: { to, subject },     // required — JSON-serialisable data
    queue?: "emails",             // optional — queue name, defaults to `type`
    max_attempts?: 5,             // optional — retries before dead letter (default 3)
    scheduled_for?: Temporal.Instant, // optional — run at a specific time
});
```

- **`type`** — the job type identifier. Workers subscribe by type, so `type` determines which handler runs. Convention is snake_case: `send_email`, `generate_report`, `process_image`.
- **`payload`** — the data the handler needs. Must be JSON-serialisable (plain objects, arrays, strings, numbers, booleans, null). Functions, symbols, and circular references will fail.
- **`queue`** — the named queue to use. Multiple job types can share a queue (a worker can process `send_email`, `send_sms`, and `send_push` from the same queue). Defaults to the job `type`.
- **`max_attempts`** — how many times to retry before the job lands in the dead-letter set.
- **`scheduled_for`** — a `Temporal.Instant` for delayed execution. Jobs scheduled for the future are stored in a sorted set and picked up by workers when their time comes.

The returned `job_id` is a UUID v4 string. Save it if you need to inspect the job's status later.

<a name="starting-workers"></a>

## Starting Workers

```ts
start_worker("send_email", async (job) => {
    await send_mail(job.payload);
}, {
    queue?: "emails",          // optional — queue to consume from (defaults to type)
    concurrency?: 3,           // optional — parallel workers (default 1)
});
```

Workers block on `BRPOP` waiting for jobs. Each concurrency slot opens a separate Redis connection (necessary because `BRPOP` blocks the connection until data arrives).

The handler receives the full `Job` object:

```ts
type Job = {
	id: string; // UUID v4
	type: string; // job type identifier
	queue: string; // queue name
	payload: any; // the data you enqueued
	status: "pending" | "running" | "completed" | "failed";
	attempts: number; // how many times it's been tried
	max_attempts: number; // retry limit
	error_message: string | null;
	created_at: number; // epoch ms
	last_run_at: number; // epoch ms
	scheduled_for: number; // epoch ms (0 for immediate)
};
```

If the handler throws, the job is retried. After `max_attempts` failures, it goes to the dead-letter set.

<a name="worker-lifecycle"></a>

### Worker Lifecycle

1. Worker connects to Redis and issues `BRPOP` on the queue.
2. A job arrives; the worker sets its status to `"running"` and adds it to the `queue:running` set.
3. The handler executes. If it resolves, the job is marked `"completed"` and removed from the running set.
4. If the handler throws, the job is either retried (pushed back to the queue) or sent to the dead-letter set after exhausting retries.
5. If the crash is unrecoverable (process dies), the orphan reaper will re-enqueue the job.

<a name="the-orphan-reaper"></a>

## The Orphan Reaper

When a worker crashes mid-job, the job stays in `"running"` status forever — no one will retry it. The orphan reaper handles this by scanning for jobs that have been running longer than a timeout:

```ts
import { reap_orphans } from "$queue/index";

// Run once at startup to recover jobs from a previous crash
await reap_orphans(300_000); // 5 minute timeout (default)
```

The reaper checks the `queue:running` set, finds jobs whose `last_run_at` is older than the timeout, increments their attempt count, and re-enqueues them. Call it once at worker startup to recover from a previous crash.

<a name="inspecting-queues"></a>

## Inspecting and Managing Queues

<media-frame label="Diagram — job lifecycle: enqueue → worker → retry → dead-letter" ratio="16/9"></media-frame>

The queue module exposes inspection functions for building admin panels or debugging:

```ts
import {
	get_job, // Job | null — fetch one job by id
	get_failed_job_ids, // string[] — failed job ids for a queue
	get_pending_job_ids, // string[] — pending job ids for a queue
	queue_length, // number — count of pending jobs
	scan_queue_names, // string[] — discover active queue names
	retry_job, // boolean — reset a failed job and re-enqueue it
	is_worker_alive, // boolean — is a worker process currently running?
} from "$queue/index";
```

```ts
// Count pending email jobs
const pending = await queue_length("send_email");

// List failed jobs
const failed_ids = await get_failed_job_ids("send_email", 50);
for (const id of failed_ids) {
	const job = await get_job(id);
	console.log(job?.error_message);
}

// Retry a specific failed job
await retry_job(job_id);

// Check whether the worker process is alive
const alive = await is_worker_alive();
```

The worker writes its PID to Redis via `set_worker_heartbeat()` on startup. `is_worker_alive()` reads the PID and verifies the process is still running via `kill -0`.

<a name="clearing-queues"></a>

## Clearing Queues

For testing and maintenance, you can clear one or all queues:

```ts
import {
	clear_queue_pending, // clear only pending jobs
	clear_queue_failed, // clear only failed jobs
	clear_queue_delayed, // clear only scheduled jobs
	clear_queue_all, // clear pending + failed + delayed + running for one queue
	clear_all_queues, // clear everything across all queues
} from "$queue/index";

// Clear all failed email jobs
const cleared = await clear_queue_failed("send_email");
console.log(`Cleared ${cleared} failed jobs`);

// Nuclear option — clear everything
const result = await clear_all_queues();
console.log(`Cleared ${result.pending} pending, ${result.failed} failed, ${result.delayed} delayed`);
```

<a name="graceful-shutdown"></a>

## Graceful Shutdown

Call `close_queue()` during shutdown to close the Redis connection:

```ts
import { close_queue } from "$queue/index";

process.on("SIGTERM", async () => {
	await close_queue();
	process.exit(0);
});
```

The current in-flight jobs will be picked up by the orphan reaper on the next worker startup.

<a name="redis-data-layout"></a>

## Redis Data Layout

The queue uses the following key patterns in Redis:

| Key                    | Type   | Purpose                                    |
| ---------------------- | ------ | ------------------------------------------ |
| `job:{id}`             | Hash   | Full job metadata (auto-expires after 24h) |
| `queue:{name}`         | List   | Pending job IDs                            |
| `queue:{name}:delayed` | ZSet   | Scheduled job IDs (score = timestamp ms)   |
| `queue:{name}:failed`  | ZSet   | Permanently failed job IDs                 |
| `queue:running`        | Set    | Job IDs currently being processed          |
| `queue:worker:pid`     | String | Worker process PID for heartbeat           |

All job hashes auto-expire 24 hours after creation, so temporary job data doesn't accumulate indefinitely even if you never explicitly clear queues.

<a name="when-not-to-use-the-queue"></a>

## When Not to Use the Queue

The queue is appropriate for:

- Sending transactional email (registration, password reset, notifications)
- Processing uploaded images (resizing, format conversion)
- Generating reports or exports asynchronously
- Webhook delivery with retries

The queue is **not** appropriate for:

- Real-time message delivery (use WebSockets or Server-Sent Events)
- Exactly-once processing (the queue guarantees at-least-once — a crash after the handler runs but before the status is saved to Redis will cause a duplicate)
- Very frequent, lightweight tasks (a `setInterval` in the same process is simpler and cheaper than enqueuing a job for every tick)
