# Prompt Templating

Covers session prompt composition, template rendering, and delivery to the backend.

## Session Prompt

The session prompt is the instruction text ACE passes to the backend at launch. It is distinct
from the backend's own system prompt.

### Composition

Built by concatenating three layers, separated by blank lines:

1. **Built-in** — minimal, embedded in ACE binary. Contains school name and propose workflow hint.
   Updated when users update ACE.
2. **School** — `session_prompt` field in `school.toml` (top-level). Domain-specific instructions
   from the school maintainer.
3. **Project** — `session_prompt` field in `ace.toml`. Project-specific overrides or additions.

Empty layers are skipped.

### Config Fields

`school.toml`:

```toml
name = "acme"
session_prompt = "Follow Acme coding standards..."
```

`ace.toml`:

```toml
school = "acme/school"
session_prompt = "This project uses PostgreSQL..."
```

### Delivery

The composed prompt is passed to the backend via `--system-prompt <prompt>` CLI flag.

## Template Rendering

Prompt `.md` files (built-in templates, CLAUDE.md templates) support `{key}` placeholders
rendered at runtime via `PromptCtx`. Simple `str::replace` per field — no regex, no crate.

### Current Placeholders

| Placeholder     | Source                          | Example    |
|-----------------|---------------------------------|------------|
| `{skills_dir}`  | Backend skills directory name   | `.claude`  |
| `{school_name}` | School display name             | `Acme`     |

### Adding a Placeholder

1. Add a field to `PromptCtx` in `src/templates/mod.rs`.
2. Add a `.replace()` call in `render()`.
3. Use `{field_name}` in any prompt `.md` file under `src/templates/`.
