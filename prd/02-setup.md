# Setup Flow

`ace setup` is a required first step before using ACE. It must be run explicitly — ACE does not
auto-detect or auto-initialize.

## Usage

```
ace setup <school-name> <source>
```

- `school-name` — name for this school (e.g. `prodigy9`, `acme`). Used as the key in config.
- `source` — git-cloneable URL or local path to the school repository.

Example:

```
ace setup prodigy9 https://github.com/prod9/school.git
```

Both arguments are required. No interactive prompts for these — explicit is better.

## What It Does

1. **Clone the school** — `git clone <source>` into `~/.cache/ace/<school-name>/`.
2. **Parse `school.toml`** — read school metadata, service declarations, MCP declarations.
3. **Authenticate** — run PKCE flow for each `[[services]]` entry declared in the school.
4. **Write config** — create/update `~/.config/ace/config.toml` with the school, source, and
   tokens.
5. **Write project config** — if run inside a project directory, write `ace.toml` with
   `school = "<school-name>"`.

## When to Run

Once, before first use. ACE refuses to operate without a config.

For adding schools, switching projects, or re-running auth, see
[06-context-management.md](06-context-management.md).

## Error Cases

- No network access — fail with clear message, setup requires cloning.
- Invalid school source — fail if URL is not git-cloneable or `school.toml` is missing/invalid.
- Auth failure — warn per service, continue with remaining services. User can re-auth later
  with `ace auth <name>`.
