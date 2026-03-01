# Backend Configuration

## Backend Enum

Three supported backends:

| Value      | Binary     | Skills Dir  | Instructions File | MCP Config           |
|------------|------------|-------------|-------------------|----------------------|
| `claude`   | `claude`   | `.claude`   | `CLAUDE.md`       | `.mcp.json` (JSON)   |
| `opencode` | `opencode` | `.opencode` | `AGENTS.md`       | `opencode.json` (JSONC) |
| `codex`    | `codex`    | `.agents`   | `AGENTS.md`       | `.codex/config.toml` (TOML) |

## TOML Syntax

```toml
backend = "claude"
```

Valid in `ace.toml`, `ace.local.toml`, user `config.toml`, and `school.toml` (`[school]` section).

## Resolution Order

First `Some` wins in this priority order (highest to lowest):

1. Project-local — `ace.local.toml`
2. Project-committed — `ace.toml`
3. `school.toml` — school-level default
4. User-global — `~/.config/ace/config.toml`

Fallback if no layer specifies backend: `claude`.

## Per-Backend Conventions

- **Binary name**: `backend.binary()` — used for exec.
- **Skills directory**: `backend.skills_dir()` — skills are linked into `{skills_dir}/skills/`.
- **Instructions file**: `backend.instructions_file()` — generated per-project by ACE during setup.

## MCP Config Differences

ACE templates `[[mcp]]` entries from `school.toml` into backend-native MCP config. The `image`
field is not a native concept in any backend — ACE constructs a `docker run -i --rm` command
with the image as the final argument, and env entries become `-e KEY=VALUE` flags.

Given this school.toml entry:

```toml
[[mcp]]
name = "jira"
image = "ghcr.io/acme-corp/mcp-jira:latest"
env = { JIRA_URL = "https://acme.atlassian.net", JIRA_TOKEN = "{{ services.jira.token }}" }
```

ACE generates:

**Claude** (`.mcp.json`):
```json
{
  "mcpServers": {
    "jira": {
      "type": "stdio",
      "command": "docker",
      "args": ["run", "-i", "--rm", "-e", "JIRA_URL=https://acme.atlassian.net",
               "-e", "JIRA_TOKEN=gho_...", "ghcr.io/acme-corp/mcp-jira:latest"],
      "env": {}
    }
  }
}
```

**OpenCode** (`opencode.json`):
```jsonc
{
  "mcp": {
    "jira": {
      "type": "local",
      "command": ["docker", "run", "-i", "--rm", "-e", "JIRA_URL=https://acme.atlassian.net",
                  "-e", "JIRA_TOKEN=gho_...", "ghcr.io/acme-corp/mcp-jira:latest"]
    }
  }
}
```

**Codex** (`.codex/config.toml`):
```toml
[mcp_servers.jira]
command = "docker"
args = ["run", "-i", "--rm", "-e", "JIRA_URL=https://acme.atlassian.net",
        "-e", "JIRA_TOKEN=gho_...", "ghcr.io/acme-corp/mcp-jira:latest"]
```

Key format differences summary:

| Field   | Claude                          | OpenCode                        | Codex                           |
|---------|---------------------------------|---------------------------------|---------------------------------|
| command | `"command": "x", "args": [...]` | `"command": ["x", ...]`         | `command = "x"`, `args = [...]` |
| env     | `"env": { ... }`                | `"environment": { ... }`        | `[mcp_servers.name.env]`        |
| type    | `"type": "stdio"`               | `"type": "local"`               | (implicit stdio)                |

Note: `env`/`environment` fields set host-process env vars, not container env. Container env
must be passed via `-e` flags in args. ACE always uses `-e` flags for `[[mcp]]` env entries.

### Write Strategy

ACE uses the best available method per backend to write project-scoped MCP config:

| Backend  | Method | Reason |
|----------|--------|--------|
| Claude   | `claude mcp add-json -s project <name> '<json>'` | Designed for programmatic use, handles merging |
| OpenCode | Write `opencode.json` directly | No non-interactive CLI for adding servers |
| Codex    | Write `.codex/config.toml` directly | CLI only writes user-scope, no `--scope` flag |

For OpenCode and Codex, ACE merges into the existing config file (preserving manually-added
entries) rather than overwriting.

## Session Prompt

All backends receive the session prompt via `--system-prompt` CLI flag.
