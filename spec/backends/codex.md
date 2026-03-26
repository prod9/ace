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

**Method: Direct config write** — CLI only writes user-scope, no `--scope` flag.

Config file: `~/.codex/config.toml` (TOML format).

ACE writes the config file directly — merging into existing content (preserving manually-added
entries) rather than overwriting. Deferred until Codex backend is fully implemented.

## Linked Folders

| Folder      | Supported |
|-------------|-----------|
| `skills/`   | ✓         |
| `rules/`    | ✗         |
| `commands/` | ✗         |
| `agents/`   | ✗         |
