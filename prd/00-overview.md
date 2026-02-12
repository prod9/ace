# ACE Overview

ACE (AI Coding Environment) is a CLI gateway into Claude Code or OpenCode. It ensures the
development environment is properly configured and up-to-date before handing off to the
underlying AI coding tool.

## Philosophy

ACE is strictly a development tool. It optimizes for developer ergonomics over production
security concerns. Sharing credentials in config is acceptable since no production secrets
should ever be managed through ACE.

Convention over configuration. Do the obvious right thing automatically. Never assume
non-obvious defaults — ask instead.

Prefer vendor-agnostic approaches. Use plain git over host-specific APIs to support GitHub,
GitLab, and other hosts equally.

Get the user into coding as fast as possible. Never block on operations that can be deferred.

## School

A school is a git-cloneable source repository containing skills, conventions, agent configs, and
other shared resources for an organization. Each context points to one school. ACE maintains a
local clone in `~/.cache/ace/{context}/` (or similar).

## Lifecycle

1. **Discover config files** — find user-global, project-local, project-committed
2. **First-run check** — if no config found, switch to initialization flow (see [02-initialization.md](02-initialization.md))
3. **Parse and merge** — layer configs together
4. **Select context** — from CLI flag, project config, or prompt
5. **Authenticate** — validate tokens for the active context
6. **Fetch school** — `git fetch` the context's school repo (clone on first run)
7. **Sync skills/conventions** — pull latest and sync all skills from the school into the project
9. **Check tooling** — required CLI tools, language runtimes, etc.
10. **Check project setup** — CLAUDE.md, MCP configs, project-specific requirements from source
11. **Select backend** — Claude Code or OpenCode
12. **Exec** — replace process with the chosen tool
