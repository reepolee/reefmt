---
title: "Preparing the Server"
---

# Preparing the Server

<a name="introduction"></a>

## Introduction

A Reepolee application is a single Bun process — there's no separate runtime, no application server to configure, no PHP-FPM or Gunicorn equivalent. Deployment is a matter of getting your code onto a server, building the production CSS, creating a systemd unit, and pointing a reverse proxy at the running port.

This page covers the once-per-server prep work: installing Bun, creating a deploy user, cloning the repository, and bringing the application up to a state where you can run it with `bun start`. The other pages in this section cover the parts that come after — running it as a long-lived service ([systemd](/deployment/systemd)), automating deploys ([GitHub Actions](/deployment/github-actions)), terminating TLS ([Reverse Proxy](/deployment/reverse-proxy)), and reading what the server tells you ([Logs](/deployment/logs)).

The walkthrough assumes a fresh Ubuntu or Debian VPS. The steps adapt cleanly to any other Linux distribution; only the package manager commands change.

<a name="installing-bun"></a>

## Installing Bun

SSH into the server and install Bun for the user that will run the application:

```bash
curl -fsSL https://bun.sh/install | bash
source ~/.bashrc
bun --version
```

The installer drops Bun into `~/.bun/bin/bun` and adds the directory to `$PATH` via the user's shell rc file. The `source ~/.bashrc` makes the change available in the current shell without logging out and back in.

Pin the Bun version explicitly if you want to avoid auto-upgrades on subsequent runs:

```bash
bun upgrade --canary
# or
curl -fsSL https://bun.sh/install | bash -s "bun-v1.3.13"
```

<a name="installing-tailwind"></a>

## Installing the Tailwind CLI

The CSS build runs once per deploy. Install the Tailwind v4 CLI globally:

```bash
bun add -g @tailwindcss/cli
```

You only need it on the server during the build step — the running application never invokes it. If you build CSS in CI rather than on the server (recommended for larger projects), skip this on the server and ship the compiled `static/app.css` as part of your deploy artifact.

<a name="creating-a-deploy-user"></a>

## Creating a Deploy User

Running the application as a dedicated user — rather than as `root` or your personal SSH user — isolates it from the rest of the system. The systemd unit later targets this user explicitly:

```bash
sudo adduser deploy
sudo su - deploy
```

The deploy user gets its own home directory and can be granted exactly the permissions it needs (write access to its own files, read access to system libraries). It has no sudo rights by default, which is what you want.

If you want to deploy via SSH (recommended), copy your public key into the deploy user's `~/.ssh/authorized_keys` so you can `ssh deploy@server` without using the root account:

```bash
mkdir -p ~/.ssh
chmod 700 ~/.ssh
echo "ssh-ed25519 AAAA... your-key-comment" >> ~/.ssh/authorized_keys
chmod 600 ~/.ssh/authorized_keys
```

<a name="cloning-the-repository"></a>

## Cloning the Repository

From inside the deploy user's home directory, clone your application:

```bash
git clone https://github.com/your-org/your-app.git app
cd app
```

For a private repository, set up an SSH deploy key on GitHub and configure git to use it. The simplest version: generate a key for the deploy user, paste the public key into the repo's "Deploy keys" settings on GitHub, then clone via the SSH URL:

```bash
ssh-keygen -t ed25519 -C "deploy@your-server" -f ~/.ssh/deploy_key
cat ~/.ssh/deploy_key.pub   # paste this into GitHub Deploy Keys

cat >> ~/.ssh/config <<EOF
Host github.com
    IdentityFile ~/.ssh/deploy_key
    IdentitiesOnly yes
EOF

git clone git@github.com:your-org/your-app.git app
```

<a name="environment-and-database"></a>

## Environment and Database

Copy the example environment file and fill in your production values. `.env` is gitignored and must be created manually on the server — never commit it:

```bash
cp .env.example .env
```

A typical production `.env`:

```
CONNECTION_STRING="sqlite:/home/deploy/app/app.db"
PORT=2338
TIME_ZONE=Europe/Ljubljana
SMTP_HOST="smtp.your-provider.com"
SMTP_PORT=587
SMTP_USERNAME="..."
SMTP_PASSWORD="..."
SMTP_FROM="hello@yourdomain.com"
```

For SQLite, point `CONNECTION_STRING` at an absolute path inside the deploy user's home directory. For MySQL, point it at your database server and leave the SQLite line commented out. See [Database](/database/getting-started) for the driver details.

The `.env` file should be readable only by the deploy user — `chmod 600 .env` makes it `rw-------`. Anything else is a credential leak waiting to happen.

<a name="installing-dependencies"></a>

## Installing Dependencies

Install the dev dependencies (Reepolee has no runtime dependencies — devDependencies are needed for the build):

```bash
bun install --frozen-lockfile
```

The `--frozen-lockfile` flag ensures the install matches `bun.lock` exactly, refusing to install if the lockfile is out of date. This is the right default for production: it catches drift between the lockfile you tested with locally and what the server would otherwise resolve.

<a name="building-css"></a>

## Building CSS

Build the minified production stylesheet:

```bash
bun run css:build
```

This writes to `static/app.css`. The dev variant (`static/app-dev.css`) is gitignored and not needed in production. Run this step after every deploy that touches templates or CSS — the layout loads `static/app.css` in production based on the `is_dev` flag, so an outdated file means stale styles.

If you build in CI instead, ship `static/app.css` as part of your deploy artifact and skip this step on the server. CI builds avoid having the Tailwind CLI on production hosts at all.

<a name="first-run"></a>

## The First Run

Before configuring systemd, confirm the application starts cleanly by running it directly:

```bash
bun start
```

`bun start` runs `bun server.ts --prod` (the script is in `package.json`). In production you run it under systemd, whose unit adds `--hot` so a `git pull` reloads the process automatically — see [systemd](/deployment/systemd). The output should look like:

```
App started at  ...
your-app v0.3.1
🚀 Server running at
http://localhost:2338
```

Visit `http://localhost:2338` from a browser on the server — `curl localhost:2338 | head -20` is the quickest check from the command line. If you get the home page back, the application is up and you're ready to make it persistent. If something fails, the error is in the same terminal — fix it before moving on.

`Ctrl+C` to stop, then proceed to [systemd](/deployment/systemd) to keep it running.

<a name="firewall"></a>

## Firewall

Reepolee listens on the port you set in `PORT` (default `2338`). For a typical deployment:

- **Port 80 and 443** are open to the world (the reverse proxy).
- **Port 2338** is open only to localhost — the proxy reaches the application over the loopback interface.

A minimal `ufw` setup:

```bash
sudo ufw allow 22/tcp       # SSH
sudo ufw allow 80/tcp       # HTTP (Certbot redirect target)
sudo ufw allow 443/tcp      # HTTPS
sudo ufw enable
```

Don't expose `2338` (or whatever port you picked) to the public internet — let only Nginx talk to it. The reverse proxy is the only entry point users hit; the application sits behind it.
