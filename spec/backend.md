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

## MCP Server Registration

ACE registers `[[mcp]]` entries from `school.toml` into the active backend. Two modes:

### Remote MCP (Preferred)

For services with remote MCP endpoints (GitHub, Linear, etc.), the school declares a URL.
The backend handles OAuth discovery, token acquisition, and storage — ACE just registers the
endpoint.

```toml
[[mcp]]
name = "linear"
url = "https://mcp.linear.app/sse"
```

### Docker MCP (Fallback)

For services without remote MCP endpoints, `image` field constructs a `docker run -i --rm`
command. Env entries become `-e KEY=VALUE` flags. Template values resolve from user config.

```toml
[[mcp]]
name = "jira"
image = "ghcr.io/acme-corp/mcp-jira:latest"
env = { JIRA_URL = "https://acme.atlassian.net", JIRA_TOKEN = "{{ services.jira.token }}" }
```

ACE resolves templates, builds the docker command, and delegates to the backend's own mechanism
to register the server.

### Strategy: CLI-First

Prefer invoking the backend's CLI to add MCP servers. Only fall back to writing config files
when the CLI lacks non-interactive or project-scoped support.

| Backend  | Method | Reason |
|----------|--------|--------|
| Claude   | `claude mcp add-json -s project <name> '<json>'` | Non-interactive, project-scoped, handles merging |
| OpenCode | Write `opencode.json` directly | No non-interactive CLI for adding servers |
| Codex    | Write `.codex/config.toml` directly | CLI only writes user-scope, no `--scope` flag |

**Claude example** — ACE runs:

```sh
claude mcp add-json -s project jira '{
  "type": "stdio",
  "command": "docker",
  "args": ["run", "-i", "--rm", "-e", "JIRA_URL=https://acme.atlassian.net",
           "-e", "JIRA_TOKEN=gho_...", "ghcr.io/acme-corp/mcp-jira:latest"]
}'
```

The CLI writes `.mcp.json` and merges with any existing entries. ACE never touches `.mcp.json`
directly for the Claude backend.

For OpenCode and Codex, ACE writes the config file directly — merging into existing content
(preserving manually-added entries) rather than overwriting. These are deferred until those
backends are fully implemented.

### Implementation Order

1. **Claude** — implement first. Single `std::process::Command` call per MCP entry.
2. **OpenCode / Codex** — implement when those backends ship. Requires JSON/TOML
   serialization and merge logic.

### Per-Backend Config Formats (Reference)

For backends that require direct file writes:

| Field   | Claude                          | OpenCode                        | Codex                           |
|---------|---------------------------------|---------------------------------|---------------------------------|
| command | `"command": "x", "args": [...]` | `"command": ["x", ...]`         | `command = "x"`, `args = [...]` |
| env     | `"env": { ... }`                | `"environment": { ... }`        | `[mcp_servers.name.env]`        |
| type    | `"type": "stdio"`               | `"type": "local"`               | (implicit stdio)                |

Note: `env`/`environment` fields set host-process env vars, not container env. Container env
must be passed via `-e` flags in args. ACE always uses `-e` flags for `[[mcp]]` env entries.

## Backend Readiness Check

Before exec, ACE should verify the backend is **ready to use** — not just installed. A backend
binary may exist on `$PATH` but the user may never have logged in or completed first-run setup.

Detection heuristic (per backend):

| Backend  | Check | Rationale |
|----------|-------|-----------|
| Claude   | `~/.claude.json` exists with auth data | Created on first successful login |
| OpenCode | `~/.local/share/opencode/auth.json` exists and is non-empty | Stores provider auth tokens; missing/empty `{}` = no providers authenticated |
| Codex    | `~/.codex/auth.json` exists, **or** `OPENAI_API_KEY`/`CODEX_API_KEY` env var set | Created on first login; env vars bypass file entirely |

Notes:
- OpenCode: `OPENCODE_HOME` overrides `~/.local/share/opencode`. The DB file (`opencode.db`)
  is created on first command run, but auth is the meaningful readiness signal.
- Codex: `CODEX_HOME` overrides `~/.codex`.

If the backend is installed but not initialized, ACE should **prompt the user** to run the
backend's login/init flow (e.g. `claude login`) rather than launching into a session that will
immediately fail.

## Linked Folders

ACE links school folders (`skills/`, `rules/`, `commands/`, `agents/`) into the project's
backend directory. Not all backends natively support every folder:

| Folder     | Claude | OpenCode | Codex |
|------------|--------|----------|-------|
| `skills/`  | ✓      | ✓        | ✓     |
| `rules/`   | ✓      | ✗        | ✗     |
| `commands/`| ✓      | ✓        | ✗     |
| `agents/`  | ✓      | ✓        | ✗     |

ACE links all folders regardless and warns for unsupported combos (linked for future
compatibility).

## Session Prompt

All backends receive the session prompt via `--system-prompt` CLI flag.
