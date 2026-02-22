```
░█▀█░█▀▀░█▀▀
░█▀█░█░░░█▀▀
░▀░▀░▀▀▀░▀▀▀
```

**ACE** (AI Coding Environment) — automation tooling for setting up and keeping AI coding
environments up-to-date. Acts as entrypoint to [Claude Code](https://docs.anthropic.com/en/docs/claude-code)
or [OpenCode](https://github.com/opencode-ai/opencode).

## Install

```sh
cargo install --path .
```

## Usage

```sh
ace setup prod9/school    # clone a school, auth services, write config
ace                       # launch backend (claude/opencode)
ace -- --continue         # pass flags through to the backend
```

## Commands

| Command | Description |
|---------|-------------|
| `ace setup [specifier]` | Clone a school, authenticate services, write config |
| `ace auth <service>` | Re-authenticate a service |
| `ace config` | Print effective configuration |
| `ace paths` | Print resolved filesystem paths |
| `ace school init` | Initialize a new school repository |
| `ace school propose` | Propose local school changes back to upstream |

## How it works

ACE manages **schools** — shared repositories of skills, conventions, and configuration for AI
coding tools. When you run `ace`, it:

1. Resolves which school to use (from `ace.toml`)
2. Fetches/updates the school repository
3. Symlinks skills into your project
4. Launches the configured backend with the school's session prompt

## Configuration

- `ace.toml` — project-level config (school specifier, backend, env)
- `ace.local.toml` — local overrides (gitignored)
- `~/.config/ace/config.toml` — user-level config (credentials)
- `school.toml` — school metadata (name, services, MCP servers, projects)

## License

MIT
