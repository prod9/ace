# Backend: Codex

Binary: `codex` | Dir: `.agents` | Instructions: `AGENTS.md`

## Readiness

`~/.codex/auth.json` exists, **or** `OPENAI_API_KEY`/`CODEX_API_KEY` env var is set.

`CODEX_HOME` overrides `~/.codex`.

## Session Prompt

Passed as Codex's initial positional prompt. Codex does not support a `--system-prompt` flag.

## Trust Modes

- `trust = "auto"` → `--full-auto`
- `trust = "yolo"` → `--dangerously-bypass-approvals-and-sandbox`

## MCP Registration

**Method: CLI-first.** Prefer `codex mcp add` for registration.

Fallback: edit `~/.codex/config.toml` directly only if the CLI cannot express the needed
configuration cleanly. Prefer the CLI because it remains aligned with Codex's evolving config
model.

Config file: `~/.codex/config.toml` (TOML format). Codex also supports project-level
`.codex/config.toml`, but ACE registers school MCP servers at user scope.

ACE should merge into existing config when using the fallback path. Never overwrite unrelated
user config.

## MCP Auth And Management

After registration, MCP auth and ongoing management happen inside Codex via `/mcp`.

ACE should not run a separate external OAuth flow for Codex. Once inside the backend session,
the user manages MCP connectivity there.

## Implementation Priorities

Codex support should be completed in this order:

1. `mcp_add()` — required to register school MCP servers through the native Codex CLI.
2. `mcp_list()` — required so ACE can avoid repeatedly offering or re-adding already
   registered servers.
3. `mcp_check()` — required in the same Codex pass because "registered" does not imply
   "working". MCP can be configured but unusable due to expired auth, invalid tokens, or
   backend-side state drift.
4. `mcp_remove()` — required because `ace mcp reset` is already part of ACE's user-facing
   command surface.

Automatic post-registration health checks in ACE's shared main flow are a separate
cross-backend product decision. Codex should implement `mcp_check()` now, but ACE should not
quietly introduce Codex-only auto-check behavior through the shared registration path.

## Linked Folders

| Folder      | Supported |
|-------------|-----------|
| `skills/`   | ✓         |
| `rules/`    | ✗         |
| `commands/` | ✗         |
| `agents/`   | ✗         |

Codex uses ACE school skills through the current `AGENTS.md` plus linked-folder workflow.
Unsupported entries here mean ACE should not assume richer native folder primitives for those
other school directories.
