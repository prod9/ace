# Local MCP Servers

Extends [mcp.md](mcp.md) with stdio transport. See
[decisions/004-local-mcp-servers.md](decisions/004-local-mcp-servers.md) for rationale.

## school.toml Format

```toml
# Stdio server — local process
[[mcp]]
name = "mempalace"
command = "python"
args = ["-m", "mempalace.mcp_server"]

# Stdio server with env vars and instructions
[[mcp]]
name = "local-db"
command = "npx"
args = ["-y", "@myorg/db-mcp"]
instructions = "Requires Node 22+. Run `npm install -g npx` if missing."

[mcp.env]
DATABASE_URL = "{{ database_url }}"

# Remote server (existing format, unchanged)
[[mcp]]
name = "github"
url = "https://api.githubcopilot.com/mcp/"

[mcp.headers]
Authorization = "Bearer {{ github_pat }}"
```

## Fields (stdio entries)

- `name` — Identifier. Same uniqueness constraint as remote entries.
- `command` — Executable to spawn. Must be on `$PATH` or an absolute path.
- `args` — (optional) Arguments passed to the command. TOML array of strings.
- `env` — (optional) Environment variables passed to the spawned process. Values may
  contain `{{ placeholder }}` syntax — ACE prompts the user on first registration, same as
  remote header placeholders.
- `instructions` — (optional) Setup guidance printed before prompting for placeholders.

## Validation

Checked on merged config (same phase as existing MCP validation):

- Each entry must have exactly one of `url` or `command` (not both, not neither).
- `headers` is invalid when `command` is set (headers are HTTP-only).
- `args` is invalid when `url` is set (args are stdio-only).
- `env` is valid for both transports (remote servers may need env vars too in future, but
  for now only validated on stdio entries).
- Duplicate `name` across all entries (remote + stdio) is an error.

## Registration

### Scope

Stdio servers are registered at **project scope**, not user scope. Unlike remote servers
(company-wide GitHub, Linear, etc.), stdio servers often depend on project-local paths,
virtualenvs, or binaries. Project scope keeps them contained.

Remote servers remain at user scope (unchanged).

### Flow

Same five-step flow as remote, with transport-specific differences:

1. **Check** — Already registered? Skip. Same `mcp_list()` check.
2. **Prompt** — If `env` contains `{{ placeholder }}` values, print `instructions` and prompt.
3. **Substitute** — Replace placeholders in env values.
4. **Register** — Call backend CLI with stdio transport args.
5. **Inform** — Print confirmation.

### Per-Backend CLI

#### Claude

```sh
# Minimal
claude mcp add --transport stdio --scope project <name> -- <command> [args...]

# With env vars
claude mcp add --transport stdio --scope project \
  --env KEY1=val1 --env KEY2=val2 \
  <name> -- <command> [args...]
```

`--scope project` writes to `.mcp.json` in the project directory. The `--` separator
divides Claude CLI flags from the spawned command.

#### Codex

Codex stdio MCP support via CLI is pending investigation. Two paths:

1. **CLI-first** (preferred): `codex mcp add --transport stdio <name> -- <command> [args...]`
   if/when Codex supports it.
2. **Config fallback**: Write directly to `~/.codex/config.toml` or `.codex/config.toml`
   (project scope) with the appropriate TOML structure for stdio MCP entries.

The existing Codex `mcp_add()` already falls back to config-file writes when the CLI cannot
express the needed configuration. Stdio entries follow the same fallback pattern.

## Config Struct Changes

```rust
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct McpDecl {
    pub name: String,

    // Remote transport
    pub url: String,
    #[serde(skip_serializing_if = "is_empty_map")]
    pub headers: HashMap<String, String>,

    // Stdio transport
    #[serde(skip_serializing_if = "is_empty_str")]
    pub command: String,
    #[serde(skip_serializing_if = "is_empty_vec")]
    pub args: Vec<String>,

    // Shared
    #[serde(skip_serializing_if = "is_empty_map")]
    pub env: HashMap<String, String>,
    #[serde(skip_serializing_if = "is_empty_str")]
    pub instructions: String,
}
```

`McpDecl` stays as one struct. Transport is determined by which fields are populated
(`url` → remote, `command` → stdio). No enum wrapper — keeps serde straightforward and
TOML representation flat.

## Backend Contract Changes

`mcp_add(entry: &McpDecl)` already receives the full struct. Backend implementations
inspect whether `entry.url` or `entry.command` is populated to choose transport. No
signature change needed.

New helper on `McpDecl`:

```rust
impl McpDecl {
    pub fn is_stdio(&self) -> bool {
        !self.command.is_empty()
    }

    pub fn is_remote(&self) -> bool {
        !self.url.is_empty()
    }
}
```

## Placeholder Substitution

Extended to cover `env` values in addition to `headers`. The `resolve_headers()` function
in `register_mcp.rs` becomes `resolve_placeholders()` and checks both `headers` and `env`
maps for `{{ name }}` patterns.

## Health Checks

No change. `mcp_check()` exercises servers through the backend regardless of transport.
The backend's MCP client handles stdio connections the same way it handles remote ones.
