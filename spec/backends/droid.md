**Status: WIP — not currently implemented. Backend module removed to simplify codebase.**

# Backend: Droid (Factory.ai)

Binary: `droid` | Dir: `.factory` | Instructions: `AGENTS.md`

## Readiness

`~/.factory/settings.json` exists. Created on first browser-based sign-in.

## Session Prompt

**No `--system-prompt` flag.** Interactive mode accepts an inline prompt as positional args
(`droid "prompt here"`), and `droid exec` takes a `prompt` argument — but neither supports
system prompt injection.

Options under consideration:

1. Prepend session prompt to `AGENTS.md` during setup (written content, not runtime flag).
2. Use `droid exec -f <tmpfile>` with prompt baked in.
3. Wait for Factory to add a `--system-prompt` flag.

**This needs a design decision before implementation.**

## Yolo Mode

`--skip-permissions-unsafe`

Also supports tiered autonomy via `--auto <low|medium|high>` on `droid exec`.

## MCP Registration

**Method: CLI** — similar to Claude.

```sh
droid mcp add <name> <url> --type http [--header "K: V" ...]
```

Examples:

```sh
# OAuth server (no headers)
droid mcp add linear https://mcp.linear.app/mcp --type http

# PAT server (with header)
droid mcp add github https://api.githubcopilot.com/mcp/ --type http \
  --header "Authorization: Bearer ghp_xxxxx"
```

Key differences from Claude's CLI:
- `--type http` instead of `-t http`
- `--header` instead of `-H`
- No scope flag — user-level by default

**MCP config files:**
- User-level: `~/.factory/mcp.json`
- Project-level: `.factory/mcp.json`

User config takes priority. Project-defined servers cannot be removed via CLI.

**MCP list**: parsed from `~/.factory/mcp.json`.

## Linked Folders

| Folder      | Supported | Target Dir Name |
|-------------|-----------|-----------------|
| `skills/`   | ✓         | `skills/`       |
| `rules/`    | ✗         | —               |
| `commands/` | ✓         | `commands/`     |
| `agents/`   | ✓         | `droids/`       |

Note: school `agents/` folder is symlinked as `.factory/droids/` — DROID uses `droids/` as
its convention for custom subagents.
