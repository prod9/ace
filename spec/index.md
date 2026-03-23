# ACE Overview

ACE (AI Coding Environment) is a CLI gateway into Claude Code, OpenCode, or Codex. It ensures the
development environment is properly configured and up-to-date before handing off to the
underlying AI coding tool.

## Table of Contents

- [index.md](index.md) — This file. Philosophy, school concept, lifecycle.
- [configuration.md](configuration.md) — Config file locations, layering, format.
- [architecture.md](architecture.md) — Layers, data flow, dependency direction.
- [setup.md](setup.md) — `ace setup` first-run flow.
- [roles.md](roles.md) — Role definitions, selection, prompt injection.
- [skills-sync.md](skills-sync.md) — School folder sync (skills, rules, commands, agents).
- [prompt-templating.md](prompt-templating.md) — Session prompt composition and template rendering.
- [mcp.md](mcp.md) — MCP server design (remote-only, OAuth delegation).
- [authentication.md](authentication.md) — Authentication (MCP OAuth, school repo access).
- [school/overview.md](school/overview.md) — School repository structure.
- [school/school-toml.md](school/school-toml.md) — `school.toml` format reference.
- [school/school-commands.md](school/school-commands.md) — `ace school` subcommands.
- [testing.md](testing.md) — Integration test strategy, TestEnv pattern.

## Philosophy

ACE is strictly a development tool. It optimizes for developer ergonomics over production
security concerns. Sharing credentials in config is acceptable since no production secrets
should ever be managed through ACE.

Convention over configuration. Do the obvious right thing automatically. Never assume
non-obvious defaults — ask instead.

GitHub is the assumed default host. `owner/repo` shorthand maps to
`https://github.com/owner/repo`.

Get the user into coding as fast as possible. Never block on operations that can be deferred.

## Versioning Philosophy

Skills always track latest main — projects never pin to a specific version.

Version-pinning assumes a dumb consumer that breaks on interface changes. An LLM reads the skill,
adapts, and can fix regressions itself. The execution engine is non-deterministic at every level:
same prompt + same model produces different code on different runs, model versions change
underneath you, and prompts evolve independently of code. Pinning skill versions cannot make a
non-reproducible pipeline reproducible.

Schools evolve independently of projects. Skills should work across as many project versions as
possible — deploying into any project at any point in its history. When compatibility issues arise,
the LLM understands both the skill and the project code, and resolves the gap itself.

Skills that ship companion scripts or binaries make version-pinning especially harmful — new
prompts against old code, old tools against new code, new models interpreting old prompts
differently. The combinatorial matrix is unwinnable. Symlinks to an always-latest cache sidestep
this entirely.

This is a deliberate departure from lockfile-and-pin paradigms. The skills folder captures intent
and preferences, not reproducible builds.

## School

A school is a git-cloneable source repository containing skills, conventions, agent configs, and
other shared resources for an organization. See [school/overview.md](school/overview.md) for
full details on specifiers, structure, and relationship to projects.

## Lifecycle

1. **Discover config files** — find user-global, project-local, project-committed
2. **Setup check** — if no config found, error and tell the user to run `ace setup` (see [setup.md](setup.md))
3. **Parse and merge** — layer configs together
4. **Register MCP servers** — register `[[mcp]]` entries into the backend
5. **Fetch school** — `git fetch` the school's repo (clone on first run)
6. **Sync school folders** — pull latest and link school folders (skills, rules, commands, agents) into the project
7. **Select role** — if school defines `[[roles]]` and no role set, prompt user to pick (see [roles.md](roles.md))
8. **Check tooling** — required CLI tools, language runtimes, etc.
9. **Check project setup** — CLAUDE.md, MCP configs, project-specific requirements from source
10. **Select backend** — Claude Code, OpenCode, or Codex
11. **Inject prompt** — prepend system context about skills, role, and school workflow
12. **Exec** — replace process with the chosen tool
