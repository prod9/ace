# Backend: Codex

Binary: `codex` | Dir: `.agents` | Instructions: `AGENTS.md`

## Readiness

`~/.codex/auth.json` exists, **or** `OPENAI_API_KEY`/`CODEX_API_KEY` env var is set.

`CODEX_HOME` overrides `~/.codex`.

## Session Prompt

Passed via `--system-prompt <prompt>` CLI flag.

## Yolo Mode

Not supported.

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

`mcp_remove()` may follow later unless a user-facing ACE flow requires it sooner.

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
