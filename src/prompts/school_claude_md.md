# {school_name} — School Repository

This is an ACE school repository. It provides skills, conventions, and session prompts to
projects that subscribe to it via `ace setup`.

Run `ace` to start a coding session for developing this school.

## Structure

- `school.toml` — school configuration (see sections below)
- `skills/` — skill directories, each with a `SKILL.md`

## school.toml Sections

- **`name`** — school display name
- **`[env]`** — shared environment variables (endpoints, feature flags — not secrets)
- **`[[services]]`** — OAuth service declarations (name, authorize_url, token_url, client_id, scopes). Tokens are stored per-user, never committed. Referenced in MCP env as `{{ services.<name>.token }}`.
- **`[[mcp]]`** — containerized MCP tool servers (name, image, env). ACE templates these into the backend's native MCP config.
- **`[[projects]]`** — project catalog (name, repo, description, optional per-project env/mcp)
- **`[[imports]]`** — provenance tracking for skills imported via `ace import`

## Adding Services

Use `ace school add-service` to add an OAuth service entry to school.toml. Errors on duplicate
service names.

**IMPORTANT:** When invoked from an AI/LLM session or any non-interactive context, ALL flags
must be provided — the command falls back to interactive TUI prompts when flags are missing,
which is not available to tool use.

```
ace school add-service \
  --name github \
  --authorize-url https://github.com/login/oauth/authorize \
  --token-url https://github.com/login/oauth/access_token \
  --client-id Iv1.abc123 \
  --scopes repo,read:org
```

## Useful Commands

- `ace school update` — re-fetch all imported skills from their sources
- `ace diff` — show uncommitted changes in the school cache
- `ace import <source>` — import a skill from an external repo