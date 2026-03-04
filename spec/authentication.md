# Authentication

## MCP Server Authentication

ACE delegates MCP authentication entirely to the backend. All three supported backends
(Claude Code, OpenCode, Codex) handle OAuth discovery, token acquisition, storage, and refresh
for remote MCP servers. See [mcp.md](mcp.md) for full details on the remote-only MCP design.

| Backend  | Auth behavior                              | Token storage                             |
|----------|--------------------------------------------|-------------------------------------------|
| Claude   | Auto-prompts on 401                        | System keychain                           |
| OpenCode | Auto-prompts on 401                        | `~/.local/share/opencode/mcp-auth.json`   |
| Codex    | Requires explicit `codex mcp login <name>` | `~/.codex/auth.json` or OS keyring        |

ACE does not implement OAuth, store tokens, or manage token refresh.

## School Repository Authentication

Access to the school repository itself is authenticated naturally through the git provider's
existing credentials (SSH keys, personal access tokens, credential helpers, etc.). ACE does not
manage or configure git authentication — it relies on the user's existing git setup. If the user
can `git clone` the school source URL, they're authenticated.
