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
- **`[[mcp]]`** — remote MCP server endpoints (name, url). The backend handles OAuth.
- **`[[projects]]`** — project catalog (name, repo, description, optional per-project env)
- **`[[imports]]`** — provenance tracking for skills imported via `ace import`

## Adding MCP Servers

Append directly to school.toml:

```toml
[[mcp]]
name = "github"
url = "https://api.githubcopilot.com/mcp/"

[[mcp]]
name = "linear"
url = "https://mcp.linear.app/sse"
```

Fields: `name` (unique identifier), `url` (remote MCP endpoint). The backend discovers
OAuth metadata and handles authentication automatically.

## Useful Commands

- `ace school update` — re-fetch all imported skills from their sources
- `ace diff` — show uncommitted changes in the school cache
- `ace import <source>` — import a skill from an external repo