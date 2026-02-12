# School Overview

A school is a git-cloneable source repository containing skills, conventions, agent configs, and
other shared resources for an organization. Each context points to one school. ACE maintains a
local clone in `~/.cache/ace/{context}/` (or similar).

A school source can also be a local folder path. This is useful for development, testing, or
single-machine setups where git hosting is unnecessary.

## Purpose

The school is the single source of truth for how an organization's AI coding environment
behaves. It centralizes shared knowledge so that every developer on the team gets the same
skills, conventions, and agent configurations — regardless of which project they're working on.

## Structure

```
school.toml              # School metadata and configuration (see 01-school-toml.md)
skills/
  <name>/
    SKILL.md             # Skill definition (standard Claude/OpenCode skill format)
```

## Relationship to Projects

A school is independent of any single project. Multiple projects (e.g. frontend and backend
repos) share the same school through their context. ACE syncs the school into each project via
symlinks, so all projects see identical skill versions from a single local clone.

## Relationship to Contexts

Each context in ACE's configuration points to exactly one school via its `source` field. A user
can have multiple contexts (e.g. `acme`, `personal`), each pointing to a different school.
