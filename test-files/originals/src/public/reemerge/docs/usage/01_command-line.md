---
title: "Command line"
---

# Command line

Run `reemerge` inside any git repository:

```bash
reemerge
```

There are no required arguments — Reemerge is interactive. It fetches your
branches and walks you through selecting what to cherry-pick onto a fresh PR
branch.

## Interactive workflow

1. **Select target branch** — the branch you want to open the PR against
   (default: `main`).
2. **Select source branch** — the branch that holds the changes
   (default: `develop`).
3. **Pick commits** — multi-select the commits from the source branch that
   aren't already on the target.
4. **Preview diffs** — optionally review the changes before applying them.
5. **Select files per commit** — for each commit, choose exactly which files to
   include and deselect the rest.
6. **Name your PR branch** — auto-suggested as `pr/<source-branch-name>`.
7. **Confirm and apply** — the selected changes are cherry-picked onto the new
   branch, created off your target.
8. **Push** — optionally push the new branch to `origin` when you're ready.

When a selected commit is a merge commit, Reemerge automatically retries the
cherry-pick with `-m 1` so the mainline parent is used.

## Options

| Flag              | Description                                                             |
| ----------------- | ----------------------------------------------------------------------- |
| `--diff`          | Always show the diff preview for selected commits, skipping the prompt. |
| `--version`, `-V` | Print the version and exit.                                             |

## Use case: partial cherry-pick

Say you have a branch with 5 commits touching 20 files, but you only want 3
specific commits — and only certain files from them. Reemerge lets you:

1. Select only those 3 commits.
2. For each commit, deselect the files you don't want.
3. Apply just the selected changes to a clean branch off `main`.

This is the fast way to split a large branch into smaller, focused PRs.
