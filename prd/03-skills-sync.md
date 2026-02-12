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

Sync into projects using symlinks, not file copies. Multiple projects sharing the same context
(e.g. frontend and backend repos in the same org) all point to the same local clone. This avoids
redundant data and ensures all projects see the same skill versions immediately after a pull.

## Cache

- Location: `~/.cache/ace/{context}/`
- Cache key: remote HEAD SHA
- On SHA match: no-op
- On SHA mismatch: pull + sync
- First run: full clone
