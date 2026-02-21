# School Commands

The `ace school` subcommand manages school repositories. It operates in two contexts
depending on where it is invoked:

- **School repo context** — `school.toml` exists at cwd root. Commands operate on the
  school directly.
- **App repo context** — no `school.toml` at cwd root, but `ace.toml` links to a school.
  Commands operate against the linked school's local clone.

Detection: if `school.toml` exists in the current working directory root, treat as school
repo context. Otherwise, resolve the linked school from `ace.toml`.

## `ace school init`

Initialize a new school repository. Must be run inside a git repo.

Steps:

1. Check cwd is a git repo.
2. Ask for school display name (or accept via `--name` arg).
3. Write minimal `school.toml`:
   ```toml
   [school]
   name = "<name>"
   ```
4. Done. User commits and pushes to their school repo.

Prerequisites: create and clone a git repo first (e.g. `gh repo create org/school --private`).

## `ace school propose`

Propose local school changes back to the upstream school repo. Replaces the former
`ace learn` concept.

Flow:

1. Detect context (school repo or app repo with linked school).
2. In app repo context, resolve the school's local clone path.
3. Create a branch from the current school state.
4. Commit local changes to the branch.
5. Push branch and open a PR to the school repo.

This allows developers to modify skills/conventions locally and propose them back to the
shared school without direct pushes to main.

TODO: Define conflict handling, branch naming, and PR template.

## Skill Modification Prompt

When ACE execs into Claude Code or OpenCode (lifecycle step 12), it injects a system prompt
that:

1. Tells the AI that skills are loaded from the linked school and are editable.
2. Instructs it that if any skill files are modified during the session, the user should run
   `ace school propose` afterward to propose changes back to the school repo.

This enables a natural workflow: developers use skills, notice improvements, edit them
in-place, and propose changes — all without leaving their coding session.
