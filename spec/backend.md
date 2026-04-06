# Backend Configuration

## Backend Enum

| Value      | Binary     | Backend Dir | Instructions File | Details                          |
|------------|------------|-------------|-------------------|----------------------------------|
| `claude`   | `claude`   | `.claude`   | `CLAUDE.md`       | [backends/claude.md](backends/claude.md)     |
| `codex`    | `codex`    | `.agents`   | `AGENTS.md`       | [backends/codex.md](backends/codex.md)       |

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
- **`supports_trust(trust)`** — validate whether the backend supports the given trust level.
- **`exec_session(opts)`** — launch the backend session. Each backend builds its own Command
  internally from `SessionOpts` (trust, session prompt, project dir, env, extra args).
- **`mcp_list()`** — list currently registered MCP server names.
- **`mcp_add(entry)`** — register a remote MCP server.
- **`mcp_remove(name)`** — unregister a remote MCP server by name.
- **`mcp_check(names)`** — runtime usability check for registered MCP servers. This is not a
  static config parse — the backend executes a one-shot prompt that exercises each server from
  inside the backend's own environment (auth state, token storage, MCP client). Returns a list
  of name/ok pairs. Best-effort: returns empty on failure or if unsupported.

See per-backend specs for implementation details.

## MCP Server Registration

ACE registers `[[mcp]]` entries from `school.toml` into the active backend. All entries are
remote MCP endpoints — see [mcp.md](mcp.md) for the remote-only design rationale.

**Strategy: CLI-first.** Prefer invoking the backend's CLI to add MCP servers. Only fall back
to writing config files when the CLI cannot express the needed configuration cleanly.

ACE owns registration into the backend. Backend-native auth and MCP management should remain in
the backend wherever possible.

## Linked Folders

ACE links school folders (`skills/`, `rules/`, `commands/`, `agents/`) into the project's
backend directory. Not all backends support every folder — see per-backend specs for the
support matrix.

Some backends may use different directory names for linked folders. The Link action handles
remapping when needed.

## Session Prompt

Backends receive the session prompt via their native invocation surface. For some backends this
is a CLI flag such as `--system-prompt`; for others it is an initial positional prompt. See
per-backend specs for the exact delivery mechanism.

## Readiness Check

Backends may expose an `is_ready()` heuristic so ACE can warn or gate execution when the backend
is clearly not initialized. Whether ACE should enforce readiness before exec is a product
decision and may vary by backend or evolve over time.
