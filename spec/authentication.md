# Authentication

## Service Authentication (MCP OAuth)

ACE delegates service authentication entirely to the backend. All three supported backends
(Claude Code, OpenCode, Codex) support remote MCP servers with OAuth 2.0 discovery — the
backend handles token acquisition, storage, and refresh.

### Backend OAuth Support

| Backend  | Auth Command                    | Token Storage                              |
|----------|---------------------------------|--------------------------------------------|
| Claude   | `/mcp` (interactive)            | System keychain (macOS)                     |
| OpenCode | `opencode mcp auth <name>`      | `~/.local/share/opencode/mcp-auth.json`     |
| Codex    | `codex mcp login <name>`        | `~/.codex/auth.json` or OS keyring          |

### Flow

1. School declares `[[mcp]]` entries with remote URLs (not Docker images) for services that
   support remote MCP with OAuth.
2. ACE registers the remote MCP endpoint into the backend via CLI or config file.
3. The backend detects the server requires auth (HTTP 401) and initiates OAuth discovery.
4. User authorizes in the browser. Backend stores the token.
5. On subsequent sessions, the backend injects stored tokens automatically.

### What ACE Does NOT Do

- ACE does not implement PKCE or any OAuth flow itself.
- ACE does not store service tokens.
- ACE does not manage token refresh or expiry.

### Docker-Based MCP (Non-OAuth)

For MCP servers distributed as Docker images (no remote endpoint), authentication is handled
via environment variables in the `[[mcp]]` entry's `env` field. These values may reference
user config via `{{ services.<name>.token }}` templates — the user sets the token manually in
`~/.config/ace/config.toml`.

## School Repository Authentication

Access to the school repository itself is authenticated naturally through the git provider's
existing credentials (SSH keys, personal access tokens, credential helpers, etc.). ACE does not
manage or configure git authentication — it relies on the user's existing git setup. If the user
can `git clone` the school source URL, they're authenticated.
