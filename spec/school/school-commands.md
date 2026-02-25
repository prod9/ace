# School Commands

The `ace school` subcommand manages school repositories. It operates in two contexts
depending on where it is invoked:

- **School repo context** — `school.toml` exists at cwd root. Commands operate on the
  school directly.
- **App repo context** — no `school.toml` at cwd root, but `ace.toml` links to a school.
  Commands operate against the linked school's local clone.

Detection: if `school.toml` exists in the current working directory root, treat as school
repo context. Otherwise, resolve the linked school from `ace.toml`.

## `ace school init`

Initialize a new school repository. Must be run inside a git repo.

Steps:

1. Check cwd is a git repo.
2. Ask for school display name (or accept via `--name` arg).
3. Write minimal `school.toml`:
   ```toml
   [school]
   name = "<name>"
   ```
4. Done. User commits and pushes to their school repo.

Prerequisites: create and clone a git repo first (e.g. `gh repo create org/school --private`).

## `ace school propose`

Propose local school changes back to the upstream school repo. Users edit skill files
in-place (through symlinks into the cache) during their coding session, then call
`school propose` when ready to submit.

### Flow

1. Resolve school cache path from `ace.toml` specifier.
2. Check for uncommitted changes in cache (`git status --porcelain`). If clean, error:
   `no changes to propose`.
3. Create branch: `git checkout -b ace/propose-<timestamp>`.
4. Stage and commit: `git add -A && git commit`.
5. Push: `git push -u origin <branch>`.
6. Create PR via GitHub API against `main`.
7. Reset cache back to main: `git checkout main && git reset --hard origin/main`.
8. Return PR URL to user.

### Important

- Edits happen through symlinks — modifying a linked skill file modifies the cache directly.
- The PR is based on wherever `main` was at clone/last-update time. GitHub shows conflicts
  if upstream has diverged. User resolves in GitHub.
- After propose, the cache is reset to `origin/main` so subsequent updates work cleanly.

## Update and Edit Safety

The school cache is a live working copy. Users may have uncommitted edits (skills modified
through symlinks). The **Update** action must check for dirty state before resetting:

1. `git status --porcelain` — if dirty, warn and abort. Tell user to `school propose` or
   discard changes first.
2. `git fetch origin`
3. `git reset --hard origin/main`

This ensures updates always track latest `main` and handle force pushes, but never
silently discard user edits.

## Skill Modification Prompt

When ACE execs into Claude Code or OpenCode (lifecycle step 12), it injects a system prompt
that:

1. Tells the AI that skills are loaded from the linked school and are editable.
2. Instructs it that if any skill files are modified during the session, the user should run
   `ace school propose` afterward to propose changes back to the school repo.

This enables a natural workflow: developers use skills, notice improvements, edit them
in-place, and propose changes — all without leaving their coding session.

## `ace import <source> [--skill <name>]`

Import a skill from an external repository into the school. Top-level command (not under
`ace school`) for convenience.

- **source** — GitHub `owner/repo` shorthand or full URL (same convention as school specifiers).
- **--skill** — Specific skill name within the repo (if repo has multiple skills).

### Flow

1. Resolve school context: if `school.toml` in cwd → school dir. Otherwise resolve linked
   school from `ace.toml` → school cache path.
2. Clone source repo to temp dir (`git clone --depth 1`).
3. Discover `SKILL.md` files in the repo (checks both `skills/` subdirectory and root-level).
4. Select skill:
   - `--skill` given → find by name.
   - Single skill in repo → auto-import.
   - Multiple skills → interactive `inquire::Select` prompt.
5. Copy skill folder into `{school_root}/skills/{skill_name}/`.
6. Append `[[imports]]` entry to `school.toml` (upsert — replace if skill name already exists).
7. Print confirmation to stderr.

### Important

- Skills are copied as real files — the school owns and commits them.
- Re-importing the same skill overwrites files and updates (not duplicates) the `[[imports]]`
  entry.
- Never imports all skills wholesale when multiple are present — always prompts for selection.

## `ace school update`

Re-fetch all imported skills from their sources.

### Flow

1. Read `[[imports]]` from `school.toml`.
2. If empty, print "no imports to update" and return.
3. Group imports by source (avoid cloning same repo twice).
4. For each source group: clone to temp dir, discover skills, copy matching ones over existing.
5. Report which skills were updated to stderr.

### Important

- Only updates skills listed in `[[imports]]` — does not discover or import new skills.
- If a skill is no longer found in the source repo, prints a warning and skips it.
