# Setup Flow

`ace setup <owner/repo>` is a required first step before using ACE in a project. It must be run
explicitly — ACE does not auto-detect or auto-initialize.

## Guards

Setup fails immediately if:

- **Not in a git repo** — error: `not in git repo, git init?`
- **`ace.toml` already exists** — error: `already set up, use 'ace' to run`

## Specifier Resolution

Before calling the Setup action, the CLI layer resolves which school to use:

- **`ace setup <owner/repo>`** — specifier is the argument.
- **`ace setup`** (no argument) — resolve from cache:
  - **One cached school** — use it automatically.
  - **Multiple cached schools** — TUI picker.
  - **No cached schools** — error: `no schools cached, ace setup <owner/repo>?`

This logic lives in the cmd/TUI layer. The Setup action always receives a resolved specifier.

## Setup Steps

1. Write `ace.toml` with `school = "<owner/repo>"`.
2. Call **Prepare** (see below).

That's it. Setup's only unique responsibility is writing `ace.toml`. Everything else is delegated
to Prepare, which is shared with the normal `ace` run.

## Prepare

Prepare ensures the school is ready to use. It is called by both `ace setup` and normal `ace`
runs.

1. **Is school cached?** (check `index.toml` for matching specifier)
   - **No** → **Install**: `git clone --depth 1` into `~/.cache/ace/repos/<owner>/<repo>/`, write
     `index.toml` entry, parse `school.toml`, register MCP servers, write user config.
   - **Yes** → **Update**: `git pull` on the cached repo.
2. **Link**: symlink school folders (`skills/`, `rules/`, `commands/`, `agents/`) from
   `<cache>/<folder>/` into `<project>/<backend_dir>/<folder>/`. One symlink per folder. Skips
   folders absent from the school. Preserves existing real directories by renaming them to
   `previous-<folder>/` before creating the symlink (first-time adoption).

## Normal `ace` Run

When the user runs `ace` (no subcommand) in a project that already has `ace.toml`:

1. Load state from `ace.toml`.
2. Call **Prepare** (install-if-needed / update-if-cached, then link).
3. Build system prompt from school config.
4. Detect and exec backend (Claude Code / OpenCode).

## Actions Summary

| Action    | Responsibility                                          | When                     |
|-----------|---------------------------------------------------------|--------------------------|
| Setup     | Guard checks, write `ace.toml`, call Prepare            | `ace setup <spec>`       |
| Prepare   | Orchestrate Install/Update + Link                       | Setup and normal `ace`   |
| Install   | `git clone`, index, register MCP, user config            | School not in cache      |
| Update    | `git pull --ff-only` on cached repo                     | School already cached    |
| Link      | Symlink school folders from cache into project          | Always (after install/update) |

## Error Cases

- **Not in git repo** — hard error.
- **Already set up** — hard error, use `ace` to run.
- **No network** — Install/Update fail with clear message.
- **Invalid school** — fail if not git-cloneable or `school.toml` missing/invalid.
- **MCP registration failure** — warn per server, continue. Backend handles auth on first use.
- **No cached schools (no-arg setup)** — error, suggest `ace setup <owner/repo>`.
