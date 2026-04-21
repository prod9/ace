# School Folder Sync

Covers lifecycle steps 6–7: fetching the school and syncing school folders into the project.

## Linkable Folders

ACE links four folder types from the school into the project:

| Folder     | Purpose                                    |
|------------|--------------------------------------------|
| `skills/`  | Skill definitions (SKILL.md per skill)     |
| `rules/`   | Convention/rule files                      |
| `commands/`| Slash commands for the backend             |
| `agents/`  | Agent configurations                       |

All four use the same symlink + adoption pattern. Only folders that exist in the school are
linked — absent folders are silently skipped.

### Backend support matrix

Not all backends natively support every folder. ACE links regardless and warns for unsupported
combos:

| Folder     | Claude | Codex |
|------------|--------|-------|
| `skills/`  | ✓      | ✓     |
| `rules/`   | ✓      | ✗     |
| `commands/`| ✓      | ✗     |
| `agents/`  | ✓      | ✗     |

Linking still happens for unsupported combos — the warning is informational only (linked for
future compatibility).

## Fetch and Sync

On every run:

1. `git fetch` the school
2. Compare local HEAD SHA against remote HEAD SHA
3. If changed, `git pull` and symlink school folders into the project
4. If unchanged, skip — cached state is current

Always sync. No user prompt, no opt-out. Consistency across the team is more important than
saving a few seconds.

## Skill Selection

ACE does not decide which skills to apply. It syncs all skills from the school into the
project. Skill selection is handled by the project's backend instructions file (for example
`CLAUDE.md` or `AGENTS.md`), which tells the backend what school content exists and how to use
it.

## Symlinks Over Copies

Sync into projects using symlinks, not file copies. Multiple projects sharing the same school
(e.g. frontend and backend repos in the same org) all point to the same local clone. This avoids
redundant data and ensures all projects see the same skill versions immediately after a pull.

**One folder-level symlink per folder**, not per-entry symlinks. Each project folder is a
single symlink pointing to the school cache's corresponding directory:

```
project/.claude/skills/   → ~/.local/share/ace/{school}/skills/
project/.claude/rules/    → ~/.local/share/ace/{school}/rules/
project/.claude/commands/ → ~/.local/share/ace/{school}/commands/
project/.claude/agents/   → ~/.local/share/ace/{school}/agents/
```

No per-entry linking, no local overrides. Everyone on the same school works against the same
set of files. To change a skill, edit through symlinks and propose changes back to the school.

### First-time adoption

When a project has an existing real directory for any of the four folders (pre-ACE or
hand-written), ACE renames it to `previous-{name}/` before creating the symlink. This is a
single bulk rename — the whole folder is the management unit. This is a one-time migration to
allow bringing existing content into the school. After adoption, the symlink takes over and
`previous-{name}/` remains as a prompt for the user to consolidate them into the school.
The session prompt nudges the LLM to help merge `previous-skills/` into the school's skills
folder and propose the changes upstream.

## Storage

- School clones: `~/.local/share/ace/{owner/repo}/` (XDG_DATA_HOME). Schools are
  user data — `UpdateOutcome::Dirty` / `AheadOfOrigin` states can carry in-progress
  work that must survive OS cache hygiene.
- Import source cache: `~/.cache/ace/imports/{owner/repo}/` (XDG_CACHE_HOME).
  Read-only upstream snapshots used during `ace import` and `ace school update`;
  safe to delete.
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
