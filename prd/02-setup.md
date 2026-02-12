# Setup Flow

`ace setup` is a required first step before using ACE. It must be run explicitly — ACE does not
auto-detect or auto-initialize.

## Usage

```
ace setup <context-name> <school-source>
```

- `context-name` — name for this context (e.g. `prodigy9`, `acme`). Used as the key in config.
- `school-source` — git-cloneable URL or local path to the school repository.

Example:

```
ace setup prodigy9 github.com/prod9/school
```

Both arguments are required. No interactive prompts for these — explicit is better.

## What It Does

1. **Clone the school** — `git clone <school-source>` into `~/.cache/ace/<context-name>/`.
2. **Parse `school.toml`** — read school metadata, auth requirements, MCP declarations.
3. **Authenticate** — run PKCE flow for each `[[auth]]` service declared in the school.
4. **Write config** — create/update `~/.config/ace/config.toml` with the context, source, and
   tokens.
5. **Write project config** — if run inside a project directory, write `ace.toml` with
   `context = "<context-name>"`.

## When to Run

Once, before first use. ACE refuses to operate without a config.

For adding contexts, switching projects, or re-running auth, see
[06-context-management.md](06-context-management.md).

## Error Cases

- No network access — fail with clear message, setup requires cloning.
- Invalid school source — fail if URL is not git-cloneable or `school.toml` is missing/invalid.
- Auth failure — warn per service, continue with remaining services. User can re-auth later
  with `ace auth <name>`.
