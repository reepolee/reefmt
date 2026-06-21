const PID_FILE = ".server.pid";
const port = Bun.env.PORT ?? 3000;

let child1: ReturnType<typeof Bun.spawn> | null = null;
let child2: ReturnType<typeof Bun.spawn> | null = null;
let is_restarting = false;
let last_restart = 0;
const RESTART_COOLDOWN = 2000;

function parse_env_file(content: string): Record<string, string> {
	const env: Record<string, string> = {};
	for (const line of content.split("\n")) {
		const trimmed = line.trim();
		if (!trimmed || trimmed.startsWith("#")) continue;
		const eq_idx = trimmed.indexOf("=");
		if (eq_idx <= 0) continue;
		const key = trimmed.slice(0, eq_idx).trim();
		let value = trimmed.slice(eq_idx + 1).trim();
		if ((value.startsWith('"') && value.endsWith('"')) || (value.startsWith("'") && value.endsWith("'"))) {
			value = value.slice(1, -1);
		}
		if (key) env[key] = value;
	}
	return env;
}

export async function kill_previous() {
	const file = Bun.file(PID_FILE);

	if (!(await file.exists())) return;

	const pid = Number(await file.text());

	if (Number.isFinite(pid)) {
		try {
			process.kill(pid, "SIGKILL");
			console.log(`💀 Killed previous server PID ${pid}`);
		} catch {}
	}
}

async function restart(trigger_file?: string) {
	if (is_restarting) return;

	await kill_previous();

	if (last_restart && Date.now() - last_restart < RESTART_COOLDOWN) return;
	is_restarting = true;

	if (trigger_file) {
		console.log(`\n🔄 Change detected in ${trigger_file}, restarting...`);
	} else {
		console.log("\n🔄 Starting...");
	}

	child1?.kill();
	child2?.kill();
	child1 = null;
	child2 = null;

	last_restart = Date.now();

	await Bun.sleep(600);

	child1 = Bun.spawn(
		["tailwindcss", "-i", "./src/css/style.css", "-o", "./src/public/css/style.min.css", "--watch", "--verbose"],
		{
			stdio: ["ignore", "inherit", "inherit"],
		},
	);

	let child_env: Record<string, string> = {};
	// Start with process.env as baseline (PATH, SystemRoot, etc.)
	for (const key of Object.keys(process.env)) {
		const val = process.env[key];
		if (val !== undefined) child_env[key] = val;
	}
	try {
		// Overlay .env — takes priority over system env
		const fresh_env = parse_env_file(await Bun.file("./.env").text());
		Object.assign(child_env, fresh_env);
	} catch {}

	child2 = Bun.spawn(["bun", "scripts/dev.ts"], {
		stdio: ["ignore", "inherit", "inherit"],
		env: child_env as Record<string, string>,
	});

	await Bun.write(PID_FILE, child2.pid.toString());

	is_restarting = false;
}

function open_browser(url: string) {
	if (process.platform === "win32") {
		Bun.spawn(["cmd", "/c", "start", "", url], {
			detached: true,
			stdio: ["ignore", "ignore", "ignore"],
		});
		return;
	}

	const cmd = process.platform === "darwin" ? "open" : "xdg-open";

	Bun.spawn([cmd, url], {
		detached: true,
		stdio: ["ignore", "ignore", "ignore"],
	});
}

if (process.stdin.isTTY) {
	process.stdin.setRawMode(true);
	process.stdin.resume();
	process.stdin.setEncoding("utf8");
	process.stdin.on("data", (key) => {
		if (key === "\u0003") {
			console.log("\nExiting...");
			child1?.kill();
			child2?.kill();
			process.exit();
		}
		if (key === "o" || key === "O") open_browser(`http://localhost:${port}`);
	});
	console.log(`Press "o" to open http://localhost:${port} in browser`);
	console.log("Press Ctrl+C to exit\n");
}

import { existsSync, watch } from "node:fs";

const watch_files = ["./scripts/dev.ts", "./.env", "./bunfig.toml"];
const watched: string[] = [];
for (const file of watch_files) {
	if (!existsSync(file)) continue;
	try {
		watch(file, (_, filename) => {
			if (!filename) {
				console.log(`📁 ${file} — no filename (Windows directory event)`);
				return;
			}
			restart(file);
		});
		watched.push(file);
	} catch (e) {
		console.error(`Could not watch ${file}:`, e);
	}
}
console.log(`👀 Watching files: ${watched.join(", ")}`);

restart();
