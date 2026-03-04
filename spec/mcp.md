# MCP Servers

## Design Decision: Remote-Only (2026-03-04)

ACE supports remote MCP servers exclusively. Schools declare `[[mcp]]` entries with URLs pointing
to hosted MCP endpoints. The backend handles OAuth discovery, token acquisition, storage, and
refresh — ACE only registers the endpoint.

Docker-based stdio MCP (container images with injected tokens) is not supported. This was a
deliberate simplification based on the state of the MCP ecosystem.

## Why Remote-Only

As of early 2026, remote MCP with OAuth 2.1 is the dominant model. 80+ official vendor-hosted
servers exist. Every major developer tool category has remote MCP coverage:

| Category           | Services with remote MCP endpoints                    |
|--------------------|-------------------------------------------------------|
| Code hosting       | GitHub, GitLab, Buildkite                             |
| Project management | Jira/Confluence (Atlassian), Linear, Notion, Asana    |
| Observability      | Sentry, Datadog, PagerDuty, Cloudflare                |
| Cloud              | AWS (60+ servers), GCP (preview)                      |
| Databases          | Supabase, Neon, Prisma (managed Postgres)             |
| Payments           | Stripe, PayPal, Square                                |
| Design             | Figma, Canva, Webflow                                 |
| Deployment         | Vercel, Netlify                                       |

Raw database access (direct Postgres/MySQL/MongoDB) has no vendor-hosted remote endpoint, but
managed providers (Supabase, Neon, Prisma, AlloyDB) cover this via their own MCP servers with
OAuth. Internal services can be exposed through self-hosted MCP gateways (Cloudflare Workers,
etc.) that implement OAuth.

The Docker stdio model — where ACE injects tokens via env vars into containers — has no remaining
use case that cannot be served by hosting a remote MCP endpoint with OAuth instead.

## school.toml Format

```toml
[[mcp]]
name = "github"
url = "https://api.githubcopilot.com/mcp/"

[[mcp]]
name = "linear"
url = "https://mcp.linear.app/sse"

[[mcp]]
name = "jira"
url = "https://mcp.atlassian.com/v1/sse"

[[mcp]]
name = "sentry"
url = "https://mcp.sentry.dev/sse"
```

Fields:

- `name` — Identifier for the MCP server.
- `url` — Remote MCP endpoint URL. The backend discovers OAuth metadata via `.well-known`.

No `image`, `env`, or token-related fields. No template syntax.

## Registration

ACE registers each `[[mcp]]` entry into the active backend. See
[backend.md](backend.md#mcp-server-registration) for per-backend registration strategy.

## Authentication

Handled entirely by the backend. When the backend connects to a remote MCP endpoint and receives
a 401, it initiates OAuth discovery and prompts the user to authorize in the browser. Tokens are
stored by the backend (keychain, auth files, etc.).

| Backend  | Auth behavior                              | Token storage                             |
|----------|------------------------------------------|-------------------------------------------|
| Claude   | Auto-prompts on 401                      | System keychain                           |
| OpenCode | Auto-prompts on 401                      | `~/.local/share/opencode/mcp-auth.json`   |
| Codex    | Requires explicit `codex mcp login <name>` | `~/.codex/auth.json` or OS keyring      |

ACE does not implement OAuth, store tokens, or manage token refresh.

### First-Run Auth Prompt

After registering MCP entries, ACE should prompt the user to authenticate any servers that
haven't been authorized yet. For Claude and OpenCode this happens automatically on first use
(401 triggers OAuth inline). For Codex, the user must run `codex mcp login <name>` explicitly.

ACE detects newly registered entries (entries added since last run) and prints a message:

- Claude/OpenCode: `"New MCP server '<name>' registered — you'll be prompted to authorize on first use."`
- Codex: `"New MCP server '<name>' registered — run 'codex mcp login <name>' to authenticate."`

This is informational only — ACE does not block on auth completion.

## Transport

The MCP spec supports two HTTP transports:

- **Streamable HTTP** (`/mcp` endpoints) — current standard.
- **SSE** (`/sse` endpoints) — deprecated in spec but still widely deployed.

ACE does not distinguish between them. The URL is passed to the backend as-is. The backend's MCP
client handles transport negotiation.

## Ecosystem References

- [MCP Registry](https://registry.modelcontextprotocol.io/) — official server directory.
- [GitHub MCP Registry](https://github.blog/ai-and-ml/github-copilot/meet-the-github-mcp-registry-the-fastest-way-to-discover-mcp-servers/) — GitHub's registry (preview).
- [PulseMCP](https://www.pulsemcp.com/servers) — community directory (8,600+ servers).
- [MCP Authorization Spec](https://modelcontextprotocol.io/specification/draft/basic/authorization) — OAuth 2.1 extension.
