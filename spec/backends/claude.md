# Backend: Claude

Binary: `claude` | Dir: `.claude` | Instructions: `CLAUDE.md`

## Readiness

`~/.claude.json` exists with auth data. Created on first successful login.

## Session Prompt

Passed via `--system-prompt <prompt>` CLI flag.

## Yolo Mode

`--dangerously-skip-permissions`

## MCP Registration

**Method: CLI** — non-interactive, user-scoped, handles merging.

```sh
claude mcp add -t http -s user <name> <url> [-H "K: V" ...]
```

Examples:

```sh
# OAuth server (no headers)
claude mcp add -t http -s user linear https://mcp.linear.app/mcp

# PAT server (with header)
claude mcp add -t http -s user github https://api.githubcopilot.com/mcp/ \
  -H "Authorization: Bearer ghp_xxxxx"
```

User scope (`-s user`) makes the server available across all projects. The CLI writes to the
user-level config and merges with existing entries. ACE never touches `.mcp.json` directly.

Before adding, ACE checks `claude mcp get <name>` to detect existing registrations. If the
server is already registered at any scope, ACE warns and skips — it does not overwrite.

**MCP list**: parsed from `~/.claude.json`.

## Linked Folders

| Folder      | Supported |
|-------------|-----------|
| `skills/`   | ✓         |
| `rules/`    | ✓         |
| `commands/` | ✓         |
| `agents/`   | ✓         |
