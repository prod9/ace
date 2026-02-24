# Session Prompt

The session prompt is the instruction text ACE passes to the backend (Claude Code / OpenCode) at
launch. It is distinct from the backend's own system prompt.

## Composition

The session prompt is built by concatenating three layers, separated by blank lines:

1. **Built-in** — minimal, embedded in ACE binary. Contains school name and propose workflow hint.
   Updated when users update ACE.
2. **School** — `session_prompt` field in `school.toml` `[school]` section. Domain-specific
   instructions from the school maintainer.
3. **Project** — `session_prompt` field in `ace.toml`. Project-specific overrides or additions.

Empty layers are skipped.

### Templating

Prompt `.md` files support `{key}` placeholders rendered at runtime via `PromptCtx`. This is a
simple `str::replace` per field — no regex, no crate.

Current placeholders:

| Placeholder     | Source                          | Example    |
|-----------------|---------------------------------|------------|
| `{skills_dir}`  | Backend skills directory name   | `.claude`  |

To add a new placeholder: add a field to `PromptCtx` in `src/prompts/mod.rs`, add a
`.replace()` call in `render()`, then use `{field_name}` in any prompt `.md` file.

## Config Fields

### `school.toml`

```toml
[school]
name = "acme"
session_prompt = "Follow Acme coding standards..."
```

### `ace.toml`

```toml
school = "acme/school"
session_prompt = "This project uses PostgreSQL..."
```

## Delivery

The composed prompt is passed to the backend via `--system-prompt <prompt>` CLI flag.
