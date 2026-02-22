# Setup Flow

`ace setup` is a required first step before using ACE. It must be run explicitly — ACE does not
auto-detect or auto-initialize.

## Specifier Resolution

Before any action, the CLI layer resolves which school to use:

- **`ace setup <owner/repo>`** — specifier is the argument.
- **`ace setup`** (no argument) — resolve from cache:
  - **One cached school** — use it automatically.
  - **Multiple cached schools** — TUI picker (fuzzy search by `owner/repo` and display name).
  - **No cached schools** — error: `ace setup <owner/repo>?`

This logic lives in the cmd/TUI layer, not in the Setup action. Setup always receives a resolved
specifier.

## Steps

Once a specifier is resolved, Setup runs two sequential checks:

1. **Install** (if school not in cache) — download school to local cache:
   - `git clone https://github.com/<owner/repo>` into `~/.cache/ace/repos/<owner>/<repo>/`.
     If already cached, skip.
   - Write entry to `~/.cache/ace/index.toml`.
   - Parse `school.toml` — read school metadata, service declarations.
   - Authenticate — run PKCE flow for each `[[services]]` entry.
   - Write user config — create/update `~/.config/ace/config.toml` with school entry keyed by
     `owner/repo`.

2. **Link** (if in git repo) — connect project to the cached school:
   - Write `ace.toml` with `school = "<owner/repo>"`.
   - Sync skills from school cache into the project (symlinks).

Both steps run in a single `ace setup` invocation when needed. Install is skipped if the school
is already cached. Link is skipped if not in a git repo (install-only scenario).

## Scenario Matrix

| Scenario | In git repo? | School cached? | What happens                                |
|----------|--------------|----------------|---------------------------------------------|
| A        | no           | no             | Install only. No project linking.            |
| B        | no           | yes            | Nothing to do. Suggest `git init`.           |
| C        | yes          | no             | Install, then Link.                          |
| D        | yes          | yes            | Link only (skip install).                    |

## Requirements

- Must be inside a git repo for linking. If not in a git repo and no specifier given, error
  with suggestion to `git init` or use `ace setup <owner/repo>`.
- Specifier required if no schools are cached.

## When to Run

Once, before first use. ACE refuses to operate without a config.

## Error Cases

- No network access — fail with clear message, setup requires cloning.
- Invalid school source — fail if URL is not git-cloneable or `school.toml` is missing/invalid.
- Auth failure — warn per service, continue with remaining services. User can re-auth later
  with `ace auth <name>`.
- Not in a git repo (no-arg mode) — error, suggest `git init` or `ace setup <owner/repo>`.
- No cached schools (no-arg mode) — error, suggest `ace setup <owner/repo>`.
