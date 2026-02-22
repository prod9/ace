# Backend Configuration

## Backend Enum

Two supported backends:

| Value      | Binary     | Skills Dir  |
|------------|------------|-------------|
| `claude`   | `claude`   | `.claude`   |
| `opencode` | `opencode` | `.opencode` |

## TOML Syntax

```toml
backend = "claude"
```

Valid in `ace.toml`, `ace.local.toml`, user `config.toml`, and `school.toml` (`[school]` section).

## Resolution Order

Last `Some` wins. Order (earliest to latest):

1. `school.toml` — `[school].backend`
2. User-global — `~/.config/ace/config.toml`
3. Project-committed — `ace.toml`
4. Project-local — `ace.local.toml`

Fallback if no layer specifies backend: `claude`.

## Per-Backend Conventions

- **Binary name**: `backend.binary()` — used for exec.
- **Skills directory**: `backend.skills_dir()` — skills are linked into `{skills_dir}/skills/`.

## Session Prompt

All backends receive the session prompt via `--system-prompt` CLI flag. No environment variable
fallback (`ACE_SYSTEM_PROMPT` is removed).
