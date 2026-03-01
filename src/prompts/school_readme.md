# {school_name}

ACE school repository — shared skills, conventions, and session prompts for your team.

## For Developers

Subscribe a project to this school:

```sh
ace setup {school_specifier}
```

This clones the school, symlinks skills into your project, and configures your AI coding
session. Run `ace` to start.

## Structure

```
school.toml       # School configuration
skills/           # Skill directories (each has a SKILL.md)
  ace-school/     # Built-in school management skill
CLAUDE.md         # AI session instructions for this repo
```

`school.toml` defines: env vars, OAuth services, MCP tool servers, project catalog, and
skill imports. See `CLAUDE.md` for section details.

## Managing Skills

| Task | Command |
|---|---|
| Import a skill | `ace import <owner/repo>` |
| Re-fetch imports | `ace school update` |
| Review local edits | `ace diff` |

Skills are the primary content of this repo. Each skill is a directory under `skills/` with
a `SKILL.md` that the AI backend reads during coding sessions.

## Managing Services

Add OAuth service entries so ACE can acquire tokens for MCP tool servers:

```sh
ace school add-service --name github \
  --authorize-url https://github.com/login/oauth/authorize \
  --token-url https://github.com/login/oauth/access_token \
  --client-id <client-id> \
  --scopes repo,read:org
```

Service tokens are stored per-user (`~/.config/ace/config.toml`), never committed.
MCP server configs reference them via `{{ services.<name>.token }}`.
