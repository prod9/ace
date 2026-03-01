---
name: ace-school
description: >
  ACE school management — proposing skill changes, creating PRs to the school
  repo, and understanding school structure. TRIGGER when: user wants to propose
  changes to skills, create a school PR, run `ace diff`, or asks about school
  structure/workflow. DO NOT TRIGGER for: normal coding tasks or project-specific
  work.
---

# ACE School Management

## What is an ACE school?

A school is a git repo containing skills, conventions, and session prompts shared across
projects. Structure:

- `school.toml` — school metadata, session prompt, imports, services, MCP servers
- `skills/` — skill directories, each with a `SKILL.md`

Projects subscribe via `ace setup`, which clones the school into a local cache
(`~/.cache/ace/…`) and symlinks `skills/` into the project.

## Editing skills

Skill files in the project are symlinks into the school cache. Edits go directly to the cache —
this is intentional. The school cache is a real git working copy.

## Proposing changes

When skill edits need to go upstream:

1. Run `ace diff` to review changes.
2. `cd $(ace paths school)` to enter the school cache directory.
3. `git checkout -b ace/{short-description}` — create a feature branch.
4. Stage and commit with a descriptive message.
5. `git push -u origin {branch}` — push to the school remote.
6. Create a PR via GitHub MCP (`mcp__github__create_pull_request`).
7. Do **NOT** reset the cache to main — that destroys uncommitted work across all branches.

## Good school PR guidelines

- **One skill or one coherent theme per PR.** Don't mix unrelated skill changes.
- **Title**: imperative, scoped (e.g. "Add self-audit checklist to general-coding").
- **Body**: what changed, why, which sessions revealed the need.
- **Keep skills generic** — no project-specific content. Skills must work across all projects
  that subscribe to the school.
- **Watch for conflicts** — skill instructions can interact with project `CLAUDE.md` and with
  each other. If a skill contradicts another skill or common project conventions, call it out
  in the PR description.