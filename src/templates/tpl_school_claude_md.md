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

### Via CLI

Use `ace school add-service` to add an OAuth service entry. Errors on duplicate names.

**IMPORTANT:** In AI/LLM sessions, ALL flags must be provided — missing flags trigger
interactive TUI prompts which are unavailable to tool use.

```
ace school add-service \
  --name <name> \
  --authorize-url <url> \
  --token-url <url> \
  --client-id <id> \
  --scopes <comma-separated>
```

### Via TOML

Append directly to school.toml:

```toml
[[services]]
name = "github"
authorize_url = "https://github.com/login/oauth/authorize"
token_url = "https://github.com/login/oauth/access_token"
client_id = "Iv1.abc123"
scopes = ["repo", "read:org"]
```

Fields: `name` (unique identifier, referenced as `{{ services.<name>.token }}`),
`authorize_url`, `token_url`, `client_id` (all required), `scopes` (optional array).
`client_id` is not a secret — safe to commit. Tokens are never stored in school.toml.

### Common Providers

**GitHub:**
- authorize_url: `https://github.com/login/oauth/authorize`
- token_url: `https://github.com/login/oauth/access_token`
- Common scopes: `repo` (full repo access), `read:org` (org membership), `read:user` (profile), `workflow` (Actions)
- Docs: https://docs.github.com/en/apps/oauth-apps/building-oauth-apps/scopes-for-oauth-apps

**Linear:**
- authorize_url: `https://linear.app/oauth/authorize`
- token_url: `https://api.linear.app/oauth/token`
- Common scopes: `read` (always present), `write` (write access), `issues:create` (new issues only), `comments:create` (new comments only)
- Docs: https://linear.app/developers/oauth-2-0-authentication

### Scope Selection

Choose the narrowest scopes that cover the use case:
- Read-only integrations → read scopes only (e.g. `read:org` for GitHub, `read` for Linear)
- Issue/PR automation → add write scopes for the specific resource (e.g. `repo` for GitHub, `issues:create` for Linear)
- Avoid admin/full-access scopes unless the tool genuinely needs them

## Useful Commands

- `ace school update` — re-fetch all imported skills from their sources
- `ace diff` — show uncommitted changes in the school cache
- `ace import <source>` — import a skill from an external repo