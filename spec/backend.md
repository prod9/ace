# Backend Configuration

## Backend Enum

| Value      | Binary     | Backend Dir | Instructions File | Details                          |
|------------|------------|-------------|-------------------|----------------------------------|
| `claude`   | `claude`   | `.claude`   | `CLAUDE.md`       | [backends/claude.md](backends/claude.md)     |
| `opencode` | `opencode` | `.opencode` | `AGENTS.md`       | [backends/opencode.md](backends/opencode.md) |
| `codex`    | `codex`    | `.agents`   | `AGENTS.md`       | [backends/codex.md](backends/codex.md)       |
| `droid`    | `droid`    | `.factory`  | `AGENTS.md`       | [backends/droid.md](backends/droid.md)       |

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

## Backend Contract

Each backend must provide:

- **`binary()`** — executable name on `$PATH`, used for exec.
- **`backend_dir()`** — project directory where school folders are linked.
- **`instructions_file()`** — markdown file generated per-project during setup.
- **`is_ready()`** — heuristic check that the backend is authenticated/configured.
- **`yolo_args()`** — CLI flags to skip permission prompts, or error if unsupported.
- **`mcp_list()`** — list currently registered MCP server names.
- **`mcp_add(entry)`** — register a remote MCP server.

See per-backend specs for implementation details.

## MCP Server Registration

ACE registers `[[mcp]]` entries from `school.toml` into the active backend. All entries are
remote MCP endpoints — see [mcp.md](mcp.md) for the remote-only design rationale.

**Strategy: CLI-first.** Prefer invoking the backend's CLI to add MCP servers. Only fall back
to writing config files when the CLI lacks non-interactive or user-scoped support.

## Linked Folders

ACE links school folders (`skills/`, `rules/`, `commands/`, `agents/`) into the project's
backend directory. Not all backends support every folder — see per-backend specs for the
support matrix.

Some backends use different directory names (e.g. DROID uses `droids/` for `agents/`). The
Link action handles remapping.

## Session Prompt

Backends receive the session prompt via CLI flag (typically `--system-prompt`). See per-backend
specs for flag details and exceptions.

## Readiness Check

Before exec, ACE verifies the backend is ready to use — not just installed. If the backend is
installed but not initialized, ACE prompts the user to run the backend's login/init flow rather
than launching into a session that will immediately fail.
