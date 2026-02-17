# Setup Flow

`ace setup` is a required first step before using ACE. It must be run explicitly — ACE does not
auto-detect or auto-initialize.

## Usage

```
ace setup <owner/repo>
```

- `owner/repo` — GitHub `owner/repo` shorthand (e.g. `prod9/school`). Maps to
  `https://github.com/owner/repo`.

Example:

```
ace setup prod9/school
```

## What It Does

1. **Clone the school** — `git clone https://github.com/<owner/repo>` into
   `~/.cache/ace/<owner>/<repo>/`.
2. **Parse `school.toml`** — read school metadata, service declarations, MCP declarations.
3. **Authenticate** — run PKCE flow for each `[[services]]` entry declared in the school.
4. **Write config** — create/update `~/.config/ace/config.toml` with school entry keyed by
   `owner/repo`.
5. **Write project config** — if run inside a project directory, write `ace.toml` with
   `school = "<owner/repo>"`.

## When to Run

Once, before first use. ACE refuses to operate without a config.

For adding schools, switching projects, or re-running auth, see
[06-context-management.md](06-context-management.md).

## Error Cases

- No network access — fail with clear message, setup requires cloning.
- Invalid school source — fail if URL is not git-cloneable or `school.toml` is missing/invalid.
- Auth failure — warn per service, continue with remaining services. User can re-auth later
  with `ace auth <name>`.
