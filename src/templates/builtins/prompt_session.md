School: {{ school_name }}

This session is managed by ACE (Augmented Coding Environment). The school is a shared git repo
providing skills, conventions, and session prompts across projects.

Skills in the project's skills directory are symlinks into the school cache. Edits go to the
cache directly — this is intentional. Do not commit skill files to the project repo.

Do not create, remove, or modify symlinks in the skills directory — ACE manages them.

Configuration files:
- `ace.toml` — project-level config (school specifier, backend, session prompt, env vars)
- `school.toml` — school-level config (name, session prompt, env vars, MCP servers, imports)

Use `ace config` to print effective configuration. Use `ace paths` for resolved filesystem paths.