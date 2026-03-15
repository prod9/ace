# MCP Servers

Remote-only. See [decisions/002-remote-only-mcp.md](decisions/002-remote-only-mcp.md) for rationale.

## school.toml Format

```toml
# OAuth-based — backend handles auth automatically
[[mcp]]
name = "linear"
url = "https://mcp.linear.app/mcp"

[[mcp]]
name = "sentry"
url = "https://mcp.sentry.dev/mcp"

# PAT-based — requires user credential via placeholder
[[mcp]]
name = "github"
url = "https://api.githubcopilot.com/mcp/"
instructions = "Create a fine-grained PAT at https://github.com/settings/personal-access-tokens/new with repository permissions: Contents (Read and write), Pull requests (Read and write)."

[mcp.headers]
Authorization = "Bearer {{ github_pat }}"
```

Fields:

- `name` — Identifier for the MCP server.
- `url` — Remote MCP endpoint URL. The backend discovers OAuth metadata via `.well-known`.
- `headers` — (optional) HTTP headers to pass to the backend. Values may contain
  `{{ placeholder }}` template syntax (see [configuration.md](configuration.md#placeholder-substitution)).
  ACE prompts the user for each placeholder on first registration and passes the resolved headers
  to the backend CLI. ACE does not store the values.
- `instructions` — (optional) Human-readable setup instructions. Printed to the terminal before
  prompting for placeholders. Also injected into the session prompt so the AI can guide the user
  if auth issues arise mid-session.

## Registration

ACE registers each `[[mcp]]` entry into the active backend at **user scope** (not project scope),
because users in a school typically share the same company infrastructure (GitHub org, Linear
workspace, etc.) and MCP servers should be available across all their projects.

Registration flow:

1. **Check** — `claude mcp get <name>`. If already registered at any scope, print a warning
   (same pattern as school update warnings) and skip. Do not overwrite existing config.
2. **Prompt** — If `headers` contain `{{ placeholder }}` values, print `instructions` (if
   present), then prompt the user for each placeholder value.
3. **Substitute** — Replace placeholders with user-provided values.
4. **Register** — Call the backend CLI to add the MCP server with resolved headers.
5. **Inform** — Print auth guidance (OAuth servers: "you'll be prompted on first use";
   PAT servers: confirm registration succeeded).

See [backend.md](backend.md#mcp-server-registration) for per-backend CLI commands.

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
