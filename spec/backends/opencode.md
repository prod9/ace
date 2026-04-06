**Status: WIP — not currently implemented. Backend module removed to simplify codebase.**

# Backend: OpenCode

Binary: `opencode` | Dir: `.opencode` | Instructions: `AGENTS.md`

## Readiness

`~/.local/share/opencode/auth.json` exists and is non-empty. Stores provider auth tokens;
missing or empty `{}` means no providers authenticated.

`OPENCODE_HOME` overrides `~/.local/share/opencode`. The DB file (`opencode.db`) is created on
first command run, but auth is the meaningful readiness signal.

## Session Prompt

Passed via `--system-prompt <prompt>` CLI flag.

## Yolo Mode

Not supported.

## MCP Registration

**Method: Direct config write** — no non-interactive CLI for adding servers.

Config file: `~/.config/opencode/opencode.json` (JSONC format).

ACE writes the config file directly — merging into existing content (preserving manually-added
entries) rather than overwriting. Deferred until OpenCode backend is fully implemented.

## Linked Folders

| Folder      | Supported |
|-------------|-----------|
| `skills/`   | ✓         |
| `rules/`    | ✗         |
| `commands/` | ✓         |
| `agents/`   | ✓         |
