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
ace setup prod9/school                       # clone a school, auth services, write config
ace                                          # launch backend (claude/opencode)
ace -- --continue                            # pass flags through to the backend
ace import anthropics/skills --skill commit  # import a skill from an external repo
ace school update                            # re-fetch all imported skills
```

## Commands

| Command | Description |
|---------|-------------|
| `ace setup [specifier]` | Clone a school, authenticate services, write config |
| `ace auth <service>` | Re-authenticate a service |
| `ace config` | Print effective configuration |
| `ace paths` | Print resolved filesystem paths |
| `ace import <source> [--skill <name>]` | Import a skill from an external repository |
| `ace school init` | Initialize a new school repository |
| `ace school propose` | Propose local school changes back to upstream |
| `ace school update` | Re-fetch all imported skills from their sources |

## How it works

ACE manages **schools** — shared repositories of skills, conventions, and configuration for AI
coding tools. When you run `ace`, it:

1. Resolves which school to use (from `ace.toml`)
2. Fetches/updates the school repository
3. Symlinks skills into your project
4. Launches the configured backend with the school's session prompt

## Skills workflow

Schools contain a shared `skills/` folder. When you run `ace`, the entire folder is symlinked
into your project — everyone on the same school works against the same skills.

**First-time setup with existing skills:** If your project already has hand-written skills in
`.claude/skills/`, ACE moves them to `previous-skills/` on first run. The LLM will then help
you merge them into the school's skills folder via `ace school propose`.

**Changing skills:** Edit skills in the school repo, not in your project. Use `ace school propose`
to push changes back to the shared school.

## Configuration

- `ace.toml` — project-level config (school specifier, backend, env)
- `ace.local.toml` — local overrides (gitignored)
- `~/.config/ace/config.toml` — user-level config (credentials)
- `school.toml` — school metadata (name, services, MCP servers, projects)

## License

MIT
