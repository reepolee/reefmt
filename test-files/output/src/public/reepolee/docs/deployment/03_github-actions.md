---
title: "GitHub Actions"
---

# GitHub Actions

<a name="introduction"></a>

## Introduction

The manual deploy flow — `git pull` on the server, `bun install`, `bun run css:build` — works fine for a project with one or two developers and infrequent deploys. Past that, automating it is one less thing to remember and one less window where a deploy can be half-applied.

GitHub Actions is the easiest path because the repository is already on GitHub. The workflow below SSHes into the server and runs the same commands you'd run by hand — no container registry to set up, no Docker images to build, no Kubernetes manifests. The application stays a single Bun process; the only thing the workflow adds is "do this on push instead of when I remember."

<a name="the-workflow-file"></a>

## The Workflow File

<media-frame label="Screenshot — GitHub Actions deploy run (green checks)" ratio="16/9"></media-frame>

Create `.github/workflows/deploy.yml` at the root of your repository:

```yaml
name: Deploy

on:
    push:
        branches: [production]

jobs:
    deploy:
        runs-on: ubuntu-latest
        steps:
            - name: Deploy to server
              uses: appleboy/ssh-action@v1
              with:
                  host: ${{ secrets.SERVER_HOST }}
                  username: ${{ secrets.SERVER_USER }}
                  key: ${{ secrets.SERVER_SSH_KEY }}
                  script: |
                      cd /home/deploy/app
                      git pull origin production
                      bun install --frozen-lockfile
                      bun run css:build
```

The trigger — `on: push: branches: [production]` — means the workflow runs whenever someone pushes to the `production` branch. The Reepolee convention is to keep `main` as your working branch and force-push it to `production` when you want a deploy to happen (`bun run production` does this). This makes the deploy event explicit; merging a PR into `main` doesn't accidentally ship to production.

<a name="setting-the-secrets"></a>

## Setting the Secrets

The workflow references three secrets you set on the repository:

| Secret           | Value                                                                |
| ---------------- | -------------------------------------------------------------------- |
| `SERVER_HOST`    | The server's hostname or IP address                                  |
| `SERVER_USER`    | The deploy user (`deploy`, in the examples)                          |
| `SERVER_SSH_KEY` | The private key for an SSH user that can log into the deploy account |

Generate a dedicated key for the deploy workflow on your local machine, paste the public key into the deploy user's `~/.ssh/authorized_keys` on the server, and paste the private key into the GitHub repository's secrets:

```bash
# locally
ssh-keygen -t ed25519 -C "github-actions-deploy" -f ~/.ssh/gha_deploy
cat ~/.ssh/gha_deploy.pub   # → paste into authorized_keys on the server
cat ~/.ssh/gha_deploy       # → paste into the SERVER_SSH_KEY repository secret
```

Repository secrets live under **Settings → Secrets and variables → Actions** in GitHub. The private key goes in as-is (including the `-----BEGIN OPENSSH PRIVATE KEY-----` lines). Once set, the secrets aren't visible in logs — the workflow can use them but you can't read them back.

Don't reuse a personal SSH key. The deploy key only needs to log into the server and run a small script; isolating it means rotating it later doesn't disrupt your other SSH access.

<a name="the-deploy-script"></a>

## What the Deploy Script Does

The four commands the workflow runs are exactly the manual deploy flow:

- **`cd /home/deploy/app`** — switch to the application directory.
- **`git pull origin production`** — fetch the new code. Bun's `--hot` flag detects the file change and reloads the process.
- **`bun install --frozen-lockfile`** — install dependencies if `bun.lock` changed. Frozen-lockfile prevents drift.
- **`bun run css:build`** — rebuild the production stylesheet.

Add or remove steps as your project needs. Running database migrations, clearing a cache, sending a deploy notification — all fit into the same `script:` block.

<a name="building-css-in-ci"></a>

## Building CSS in CI Instead

The workflow above builds CSS on the server. For larger projects, building it in the GitHub Actions runner and rsyncing the result to the server keeps the production server free of build tooling. A pattern that works well:

```yaml
name: Deploy

on:
    push:
        branches: [production]

jobs:
    build:
        runs-on: ubuntu-latest
        steps:
            - uses: actions/checkout@v4
            - uses: oven-sh/setup-bun@v1
            - run: bun install --frozen-lockfile
            - run: bun run css:build
            - name: Push compiled CSS to server
              uses: easingthemes/ssh-deploy@main
              with:
                  SSH_PRIVATE_KEY: ${{ secrets.SERVER_SSH_KEY }}
                  REMOTE_HOST: ${{ secrets.SERVER_HOST }}
                  REMOTE_USER: ${{ secrets.SERVER_USER }}
                  SOURCE: "static/app.css"
                  TARGET: "/home/deploy/app/static/"
            - name: Pull source on server
              uses: appleboy/ssh-action@v1
              with:
                  host: ${{ secrets.SERVER_HOST }}
                  username: ${{ secrets.SERVER_USER }}
                  key: ${{ secrets.SERVER_SSH_KEY }}
                  script: |
                      cd /home/deploy/app
                      git pull origin production
                      bun install --frozen-lockfile
```

The build step runs in a fresh Ubuntu runner with the right tooling installed; the deploy step ships only the artifact. The server doesn't need `@tailwindcss/cli` installed at all.

For most projects, building on the server (the simpler workflow at the top of the page) is fine. Move to CI builds when you find the server build is the slow part of the deploy or when you want to keep production hosts minimal.

<a name="rolling-back"></a>

## Rolling Back

A bad deploy is rolled back the same way it was made — push the previous version to `production`. From your local machine:

```bash
git checkout main
git reset --hard <previous-good-sha>
git push origin main:production --force
```

The workflow runs again, the server pulls the previous SHA, and Bun's `--hot` reload picks up the rollback in seconds. Because nothing about the deploy state lives outside git, the rollback is the same operation as a forward deploy.

For a faster rollback (no waiting for the workflow), SSH into the server and `git reset --hard` directly:

```bash
ssh deploy@server
cd ~/app
git fetch origin production
git reset --hard <previous-good-sha>
```

The next time the workflow runs, the server's branch state matches the new push.

<a name="notifications"></a>

## Deploy Notifications

To know when a deploy starts, succeeds, or fails, add a notification step to the workflow. The shapes vary by service — for Slack via a webhook:

```yaml
- name: Notify Slack
  if: always()
  uses: rtCamp/action-slack-notify@v2
  env:
      SLACK_WEBHOOK: ${{ secrets.SLACK_WEBHOOK }}
      SLACK_TITLE: "Deploy ${{ job.status }}"
```

`if: always()` makes the notification fire whether the deploy succeeded or failed. For more control, use `if: success()` and `if: failure()` for separate notifications with different formatting.

<a name="when-not-to-use-actions"></a>

## When Not to Use GitHub Actions

For very small projects (one developer, infrequent deploys), `git pull` on the server is fine and the GitHub Actions workflow is one more thing to maintain. For very large ones (multiple environments, integration tests, security scans, staged rollouts), GitHub Actions is fine but you'll outgrow the simple "ssh in and run commands" workflow — at that scale it makes sense to look at proper deployment tooling (Argo, Spinnaker, your platform's native deploy system).

The workflow above is right for the middle: more than one developer, more than one deploy a week, you want pushes to `production` to actually update production without a manual step.
