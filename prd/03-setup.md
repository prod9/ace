# Setup Flow

`ace setup` is a required first step before using ACE. It must be run explicitly — ACE does not
auto-detect or auto-initialize.

## Modes

### `ace setup <owner/repo>` — Install a school

Clones and configures a school. If run inside a git repo, also links the project.

```
ace setup prod9/school
```

Steps:

1. **Download the school** — `git clone https://github.com/<owner/repo>` into
   `~/.cache/ace/repos/<owner>/<repo>/`. If already cached, `git pull` instead.
   Writes an entry to `~/.cache/ace/index.toml` after successful clone.
2. **Parse `school.toml`** — read school metadata, service declarations, MCP declarations.
3. **Authenticate** — run PKCE flow for each `[[services]]` entry declared in the school.
4. **Write config** — create/update `~/.config/ace/config.toml` with school entry keyed by
   `owner/repo`.
5. **Write project config** — if run inside a git repo, write `ace.toml` with
   `school = "<owner/repo>"`.

### `ace setup` — Link a project to a cached school

Picks from already-cached schools and writes `ace.toml`. No cloning or auth.

```
ace setup
```

Requirements:

- Must be inside a git repo. If not, error with suggestion to `git init` or use
  `ace setup <owner/repo>`.
- At least one school must be listed in `~/.cache/ace/index.toml`. If not, error with
  suggestion to run `ace setup <owner/repo>` first.

Behavior:

- **One cached school** — use it automatically.
- **Multiple cached schools** — fuzzy search prompt (matches `owner/repo` and display name from
  `school.toml`).
- Writes `ace.toml` with `school = "<owner/repo>"`.

## Scenario Matrix

| Scenario | In git repo? | ace.toml? | School cached? | What happens                                                          |
|----------|--------------|-----------|----------------|-----------------------------------------------------------------------|
| A        | no           | no        | no             | `ace setup <owner/repo>` — clone, config, auth. No project linking.   |
| B        | no           | no        | yes            | No project to link. Suggest `git init`.                               |
| C        | yes          | no        | no             | `ace setup <owner/repo>` — clone, config, auth, write ace.toml.       |
| D        | yes          | no        | yes            | `ace setup` — pick from cached schools, write ace.toml.               |
| E        | yes          | yes       | no             | `ace setup <owner/repo>` — ace.toml names school but no cache. Clone. |
| F        | yes          | yes       | yes            | Already set up. Re-auth or verify only.                               |

## When to Run

Once, before first use. ACE refuses to operate without a config.

## Error Cases

- No network access — fail with clear message, setup requires cloning.
- Invalid school source — fail if URL is not git-cloneable or `school.toml` is missing/invalid.
- Auth failure — warn per service, continue with remaining services. User can re-auth later
  with `ace auth <name>`.
- Not in a git repo (no-arg mode) — error, suggest `git init` or `ace setup <owner/repo>`.
- No cached schools (no-arg mode) — error, suggest `ace setup <owner/repo>`.
