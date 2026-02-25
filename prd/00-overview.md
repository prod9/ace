# ACE Overview

ACE (AI Coding Environment) is a CLI gateway into Claude Code or OpenCode. It ensures the
development environment is properly configured and up-to-date before handing off to the
underlying AI coding tool.

## Table of Contents

- [00-overview.md](00-overview.md) — This file. Philosophy, school concept, lifecycle.
- [01-configuration.md](01-configuration.md) — Config file locations, layering, format.
- [02-architecture.md](02-architecture.md) — Layers, data flow, dependency direction.
- [03-setup.md](03-setup.md) — `ace setup` first-run flow.
- [04-skills-sync.md](04-skills-sync.md) — Skill installation and sync.
- [06-authentication.md](06-authentication.md) — OAuth PKCE flow for services.
- [school/00-overview.md](school/00-overview.md) — School repository structure.
- [school/01-school-toml.md](school/01-school-toml.md) — `school.toml` format reference.
- [school/02-school-commands.md](school/02-school-commands.md) — `ace school` subcommands.

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

Skills are intentionally unversioned. Always track latest main.

In an LLM world, what matters is capturing **intent and preferences** — not locking specific tool
versions to prevent compatibility problems. Versioning an entire suite of context (à la Tessel) is
a non-goal. The skills folder optimizes for the LLM to work with, not for reproducible builds.

Old tools + new model = different results anyway, so pinning versions provides false stability.
Skills that ship companion scripts or binaries make this worse — once committed into the project's
git history, the tool version is locked to that commit. Updating the skill means the new tool runs
against old code, but checking out old code forces the old tool. The version matrix is unwinnable.
Keeping skills outside the project repo as symlinks to an always-latest cache sidesteps this
entirely. Workflow tooling should never cause compatibility problems with building the actual
project artifact — it sits outside the build graph entirely.

This is a deliberate departure from traditional package management. We are not replicating old
build paradigms in the LLM era.

## School

A school is a git-cloneable source repository containing skills, conventions, agent configs, and
other shared resources for an organization. See [school/00-overview.md](school/00-overview.md) for
full details on specifiers, structure, and relationship to projects.

## Lifecycle

1. **Discover config files** — find user-global, project-local, project-committed
2. **Setup check** — if no config found, error and tell the user to run `ace setup` (see [03-setup.md](03-setup.md))
3. **Parse and merge** — layer configs together
4. **Authenticate** — validate tokens for the active school
5. **Fetch school** — `git fetch` the school's repo (clone on first run)
7. **Sync skills/conventions** — pull latest and sync all skills from the school into the project
9. **Check tooling** — required CLI tools, language runtimes, etc.
10. **Check project setup** — CLAUDE.md, MCP configs, project-specific requirements from source
11. **Select backend** — Claude Code or OpenCode
12. **Inject prompt** — prepend system context about skills and `ace school propose` workflow
13. **Exec** — replace process with the chosen tool
