---
title: "systemd"
---

# systemd

<a name="introduction"></a>

## Introduction

`bun start` runs the application until the terminal session ends. To keep it running after you log out, surviving server reboots, restarting if it crashes, you need a process supervisor. On Linux, that's systemd.

Reepolee ships with a `reepolee.service` file that you copy into `/etc/systemd/system/` and start. Once installed, the application starts on boot, restarts on crash, and writes its log output to `journalctl` where you can read it. The `package.json` includes script aliases for the install, reload, and lifecycle commands so you don't have to remember the `systemctl` flags.

<a name="the-service-file"></a>

## The Service File

The shipped `reepolee.service` is a minimal systemd unit:

```ini
[Unit]
Description=Reepolee App Server
After=network.target

[Service]
Type=simple
User=deploy
WorkingDirectory=/home/deploy/app
ExecStart=/home/deploy/.bun/bin/bun --hot /home/deploy/app/server.ts --prod
Restart=always
RestartSec=5
Environment=NODE_ENV=production

[Install]
WantedBy=multi-user.target
```

Edit the file before installing it. The fields you need to set:

| Field              | What to put                                                                                                    |
| ------------------ | -------------------------------------------------------------------------------------------------------------- |
| `User`             | The deploy user you created in [Preparing the Server](/deployment/preparing-the-server#creating-a-deploy-user) |
| `WorkingDirectory` | The absolute path to your application directory                                                                |
| `ExecStart`        | The absolute path to the deploy user's `bun`, the absolute path to `server.ts`, and the `--prod` flag          |

The `--hot` flag in `ExecStart` is intentional: it tells Bun to watch for file changes and reload automatically. Combined with `Restart=always`, this means a `git pull` on the server is enough to deploy new code — Bun detects the changed files, reloads the process, and the new version is live without a manual restart. See [Deploying Updates](#deploying-updates) below.

`--prod` is the flag your application should branch on for production behaviour (template caching, minified assets, no dev banners). The render layer reads `is_dev = Bun.argv.includes("--dev")` to decide; `--prod` is its complement.

`Environment=NODE_ENV=production` is included for any Node-compatible code that reads `NODE_ENV` (for example, third-party libraries you might add). Reepolee itself doesn't depend on it — the `--dev` / `--prod` flags are what matter.

<a name="installing-the-service"></a>

## Installing the Service

The `package.json` includes the installation commands as scripts so you can run them by name:

```bash
bun run service:install      # copy the unit file into /etc/systemd/system/
bun run service:reload       # tell systemd to re-read its unit files
bun run service:autostart    # enable on boot
bun run service:start        # start the service now
```

The scripts map to:

```bash
sudo cp ./reepolee.service /etc/systemd/system/reepolee.service
sudo systemctl daemon-reload
sudo systemctl enable reepolee
sudo systemctl start reepolee
```

After running them, confirm the service is active:

```bash
systemctl status reepolee
```

The output should show `active (running)` and the recent log lines from the application's startup. If it's `failed`, the same output shows the exit code and the last few log lines — usually enough to diagnose.

<a name="restart-on-config-change"></a>

## Restarting After a Service File Change

If you edit `reepolee.service` after the initial install, copy the new file in and reload systemd:

```bash
bun run service:install
bun run service:reload
sudo systemctl restart reepolee
```

`daemon-reload` re-reads the unit files; `restart` stops and starts the service so the new configuration takes effect. Just running `restart` without `daemon-reload` keeps systemd's cached old configuration.

<a name="deploying-updates"></a>

## Deploying Updates

Reepolee's recommended workflow is a `production` branch convention. From your local machine:

```bash
bun run release        # builds minified CSS and bumps the package version
bun run production     # force-pushes main to the production branch on GitHub
```

`release` runs `bun css:build` and bumps the patch version in `package.json`. `production` force-pushes `main` to the `production` branch — this is the branch your server pulls from, kept separate from `main` so you control exactly when the server picks up new code.

On the server:

```bash
cd /home/deploy/app
git pull origin production
bun install --frozen-lockfile   # only needed if dependencies changed
bun run css:build
```

Because `--hot` is watching for file changes, Bun restarts the process automatically as soon as `git pull` updates the source files. No manual service restart needed.

For automated server-side updates on push, see [GitHub Actions](/deployment/github-actions).

<a name="lifecycle-commands"></a>

## Lifecycle Commands

The full set of `systemctl` commands you'll use day-to-day:

| Command                           | Purpose                                          |
| --------------------------------- | ------------------------------------------------ |
| `sudo systemctl start reepolee`   | Start the service                                |
| `sudo systemctl stop reepolee`    | Stop the service                                 |
| `sudo systemctl restart reepolee` | Stop and start (full restart)                    |
| `sudo systemctl reload reepolee`  | Send `SIGHUP` (Bun ignores; use restart instead) |
| `sudo systemctl status reepolee`  | Show running status and recent log lines         |
| `sudo systemctl enable reepolee`  | Start on boot                                    |
| `sudo systemctl disable reepolee` | Don't start on boot                              |
| `journalctl -u reepolee -f`       | Follow the log live                              |

The `bun run service:logs` script runs `journalctl -u reepolee -f` for the common case of "show me what the server is doing right now." See [Logs](/deployment/logs) for the full logging story.

<a name="restart-and-resilience"></a>

## Restart and Resilience

`Restart=always` plus `RestartSec=5` means systemd restarts the process whenever it exits, after a 5-second pause. The pause prevents tight crash loops from burning CPU when something is fundamentally broken.

If the application crashes repeatedly within a short window, systemd will start to back off (and eventually give up if you've configured `StartLimitBurst` and `StartLimitInterval`). The defaults are forgiving — most projects won't need to tune them.

For graceful shutdown — finishing in-flight requests before exiting — Bun handles `SIGTERM` correctly out of the box. `systemctl stop reepolee` sends `SIGTERM`, the server stops accepting new requests, in-flight ones finish, and the process exits. No additional handler needed.

<a name="multiple-environments"></a>

## Multiple Environments on One Server

To run staging and production on the same machine, copy `reepolee.service` to a second name and adjust the paths:

```ini
# /etc/systemd/system/reepolee-staging.service
[Service]
User=deploy
WorkingDirectory=/home/deploy/staging
ExecStart=/home/deploy/.bun/bin/bun --hot /home/deploy/staging/server.ts --prod
Restart=always
Environment=PORT=2339
```

Use a different `PORT` (`2339` here) and a different `WorkingDirectory`. The reverse proxy points different hostnames at the different ports. Both services are independently start/stop/status-able by their unit name.

For more than two environments, the file copying gets repetitive — at that point, switch to systemd templates (`reepolee@.service`) and pass the environment name as the instance argument (`reepolee@staging`, `reepolee@production`).
