# Skills Sync

Covers lifecycle steps 6–7: fetching the school and syncing skills into the project.

## Fetch and Sync

On every run:

1. `git fetch` the school
2. Compare local HEAD SHA against remote HEAD SHA
3. If changed, `git pull` and symlink skills/conventions into the project
4. If unchanged, skip — cached state is current

Always sync. No user prompt, no opt-out. Consistency across the team is more important than
saving a few seconds.

## Skill Selection

ACE does not decide which skills to apply. It syncs all skills from the school into the
project. Skill selection is handled by the project's CLAUDE.md (hand-maintained, committed to
the project repo), which tells Claude Code/OpenCode which skills to always load.

## Symlinks Over Copies

Sync into projects using symlinks, not file copies. Multiple projects sharing the same school
(e.g. frontend and backend repos in the same org) all point to the same local clone. This avoids
redundant data and ensures all projects see the same skill versions immediately after a pull.

**One folder-level symlink**, not per-skill symlinks. The project's `skills/` directory is a
single symlink pointing to the school cache's `skills/` directory:

```
project/.claude/skills/ → ~/.cache/ace/repos/{school}/skills/
```

No per-skill linking, no local overrides. Everyone on the same school works against the same
set of skills. To change a skill, edit through symlinks and propose changes back to the school.

### First-time adoption

When a project has an existing real skills directory (pre-ACE or hand-written), ACE renames the
entire directory to `previous-skills/` before creating the folder symlink. This is a single
bulk rename, not per-skill — the whole folder is the management unit. This is a one-time
migration to allow bringing existing skills into the school. After adoption, the symlink takes over and
`previous-skills/` remains as a prompt for the user to consolidate them into the school.
The session prompt nudges the LLM to help merge `previous-skills/` into the school's skills
folder and propose the changes upstream.

## Cache

- Git clones: `~/.cache/ace/repos/{owner/repo}/`
- Index: `~/.cache/ace/index.toml` — tracks downloaded schools
- Cache key: remote HEAD SHA
- On SHA match: no-op
- On SHA mismatch: pull + sync
- First run: full clone + index entry

### index.toml

```toml
[[school]]
specifier = "prod9/school"
repo = "prod9/school"
path = ""

[[school]]
specifier = "prod9/mono:school"
repo = "prod9/mono"
path = "school"
```

- `specifier` — full specifier as written in `ace.toml`
- `repo` — `owner/repo` portion (git clone target)
- `path` — subfolder within the repo containing `school.toml` (empty string if root)

`list_cached_schools` reads index.toml, not the filesystem.
