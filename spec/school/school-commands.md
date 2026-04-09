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
4. Create `CLAUDE.md` if missing.
5. Create `skills/ace-school/SKILL.md` from built-in template if missing.
6. Done. User commits and pushes to their school repo.

Prerequisites: create and clone a git repo first (e.g. `gh repo create org/school --private`).

## Update and Edit Safety

The school cache is a live working copy. Users may have uncommitted edits (skills modified
through symlinks). The **Update** action must check for dirty state before pulling:

1. `git status --porcelain` — if dirty, warn and abort. Tell user to propose changes when
   ready.
2. `git fetch origin`
3. Fast-forward to `origin/main` (only when the cache is confirmed clean).

The dirty guard in step 1 ensures user edits are never silently discarded.

## Skill Modification Workflow

When ACE execs into the backend (lifecycle step 13), it injects a session prompt that:

1. Tells the AI that skills are loaded from the linked school and are editable.
2. Instructs it to propose changes back to the school repo when skills are modified.

The AI backend handles the full PR workflow: `ace diff` to review, branch in the school
cache, commit, push, create PR via GitHub MCP. No dedicated `ace` command needed — the AI
has all the tools (git + GitHub MCP).

The `ace-school` skill (created by `ace school init`) provides detailed instructions for
this workflow.

## `ace import <source> [--skill <name>] [--all]`

Import a skill from an external repository into the school. Top-level command (not under
`ace school`) for convenience.

- **source** — GitHub `owner/repo` shorthand or full URL (same convention as school specifiers).
- **--skill** — Specific skill name or glob pattern (e.g. `"frontend-*"`).
- **--all** — Import all skills from the source. Shorthand for `--skill "*"`.

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
- When multiple skills are found and no `--skill` or `--all` is given, prompts for selection.
- Glob patterns (`--skill "frontend-*"` or `--all`) record an `[[imports]]` entry and print
  a hint to run `ace school update`. No skills are copied immediately — resolution happens
  during update.

### Parent school pattern

To inherit all skills from a company-wide school:

```sh
ace import company/school --all
ace school update
```

This adds `skill = "*"` to `[[imports]]` and fetches all skills on update. New skills added
to the parent are picked up automatically on subsequent updates.

## `ace school update`

Re-fetch all imported skills from their sources.

### Flow

1. Read `[[imports]]` from `school.toml`.
2. If empty, print "no imports to update" and return.
3. Group imports by source (avoid cloning same repo twice).
4. For each source group: clone to temp dir, discover skills.
   - **Exact imports**: copy the named skill over existing.
   - **Wildcard imports**: match discovered skills against the glob pattern, copy all matches.
     Skip skills that exist locally and are not tracked in `[[imports]]` (child wins).
5. Report which skills were updated to stderr.

### Important

- Exact imports update only the named skill. If not found in the source, warns and skips.
- Wildcard imports re-discover on every update — new skills matching the pattern are picked up
  automatically.
- The school's own non-imported skills are never overwritten by wildcard imports.

## `ace diff`

Show uncommitted changes in the school cache, including untracked files.

- Runs `git add -N .` (intent-to-add) before diffing so new files appear in the output.
- Prints `# school-cache\t<path>` as the first line (metadata, tab-separated).
- Resolves school specifier from `ace.toml`.
- Errors if no school configured or school is embedded (no cache directory).
- Passes raw diff output through to stdout (human-readable, not tab-separated).
- Prints metadata line even if the cache is clean (diff output may be empty).
- Output is a valid unified diff (patch-compatible).
