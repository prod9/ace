# Prompt Templating

Covers session prompt composition, template rendering, and delivery to the backend.

## Session Prompt

The session prompt is the instruction text ACE passes to the backend at launch. It is distinct
from the backend's own system prompt.

### Composition

Built by concatenating layers, separated by blank lines:

1. **Built-in** — `prompt_session.md`, always present. Contains school name, skill/symlink
   rules, debugging tips. Rendered with `{{ school_name }}`.
2. **School** — `session_prompt` field in `school.toml`. Domain-specific instructions from the
   school maintainer.
3. **Project** — `session_prompt` field in `ace.toml`. Project-specific overrides or additions.
4. **Skill change summary** — only when skills changed since last session. Lists added,
   updated, and removed skills.
5. **School changes** — `prompt_school_changes.md`, only when a school cache exists (remote
   schools). Contains proposal workflow steps. Rendered with `{{ school_cache }}`.
6. **Previous skills** — `prompt_previous_skills.md`, only when a `previous-skills/` directory
   exists. Migration guidance. Rendered with `{{ skills_dir }}`.

Empty/absent layers are skipped.

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

## Template Engine

Lives in `src/templates/` — `mod.rs` (Template struct) and `parser.rs` (state machine).

`Template::parse(input)` parses `{{ key }}` placeholders in a single pass, zero-copy. The
parsed template supports `placeholders()` (returns unique names) and `substitute(values)`
(returns rendered string with missing keys resolved to empty string).

### Syntax

- `{{ name }}` — placeholder, resolved by `substitute()`.
- Whitespace inside braces is flexible: `{{name}}`, `{{ name }}`, `{{  name  }}` all match.
- Name must be `[a-zA-Z0-9_]+`.
- Newlines inside `{{ }}` abort the placeholder (treated as literal).
- Single braces `{name}` are not placeholders — passthrough.
- Invalid names (hyphens, spaces, etc.) are preserved as literal text.

### Current Placeholders

| Placeholder        | Used in                   | Source                        | Example                              |
|--------------------|---------------------------|-------------------------------|--------------------------------------|
| `{{ school_name }}`  | `prompt_session.md`, project/school CLAUDE.md templates | School display name | `Acme` |
| `{{ skills_dir }}`   | `prompt_previous_skills.md`, project CLAUDE.md template | Backend skills directory name | `.claude` |
| `{{ school_cache }}` | `prompt_school_changes.md` | School cache path             | `/home/user/.cache/ace/repos/org/school` |

### Adding a Placeholder

1. Use `{{ field_name }}` in any `.md` file under `src/templates/builtins/`.
2. Pass the value in the `HashMap` at the call site (in `session.rs` or the action that
   renders the template).

### Module Layout

```
src/templates/
  mod.rs          — Template struct (parse, substitute, placeholders)
  parser.rs       — Parser state machine
  builtins.rs     — include_str! constants for all .md files
  session.rs      — build_session_prompt() composition
  builtins/       — .md template files
```
