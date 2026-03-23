# Configuration Management

## Format

TOML.

## Layers

Resolved by merging (later overrides earlier):

1. **User-global** `~/.config/ace/config.toml`
2. **Project-committed** `ace.toml` (checked into git, shared)
3. **Project-local** `ace.local.toml` (gitignored, per-machine)

### Fields

Each layer can set:

- `school` — school specifier (last non-empty wins)
- `backend` — `"claude"`, `"opencode"`, or `"codex"` (highest-priority `Some` wins: local → project → user; fallback `claude`). See [backend.md](backend.md).
- `role` — selected role name (last non-empty wins). Must match a `[[roles]]` entry in the school's `school.toml`. Typically set in `ace.local.toml` via interactive selection. See [roles.md](roles.md).
- `session_prompt` — additional prompt text (last non-empty wins)
- `env` — environment variables (additive merge, later keys override)

## Loading vs Validation

Config loading and config validation are separate concerns.

### Loading

Serde handles deserialization only. All config structs use `#[derive(Default)]` and
`#[serde(default)]` at the struct level. Every field parses successfully regardless of what's
present in the TOML — missing keys get their type's `Default` value (empty string, empty vec,
`None`, etc.).

This means a TOML with no `name` key produces `SchoolToml { name: "".into(), .. }`
rather than a serde error. Partial or empty files always parse.

### Validation

After loading, a separate validation pass checks invariants and produces clear, actionable errors.
Validation runs on the merged config (after all three layers are combined), not on individual files.

Rules are expressed in code, not via serde attributes. Examples:

- `name` (top-level) must be non-empty.
- No duplicate `mcp[].name` entries.
- `projects[].repo` must be a valid specifier.

Validation errors reference the offending key path: e.g. `name: must not be empty`,
`mcp[0].name: duplicate entry`.

### Why

- Serde's "missing field" errors are opaque and unhelpful to users.
- Required-vs-optional is a validation concern, not a parsing concern.
- Validation on the merged config catches cross-layer issues (e.g. a layer overrides a field to
  an invalid value).
- Richer checks (URL format, uniqueness, non-empty) cannot be expressed through serde alone.

## Placeholder Substitution

Config string values may contain `{{ name }}` placeholders that are resolved at runtime by
prompting the user. This is a general-purpose mechanism — currently used by MCP header values
(see [mcp.md](mcp.md)) but available wherever user-specific values are needed.

### Syntax

- `{{ name }}` — placeholder, resolved by prompting the user.
- Whitespace inside braces is flexible: `{{name}}`, `{{ name }}`, `{{  name  }}` all match.
- Name must be `[a-zA-Z0-9_]+`.
- Literal `{{` that should not be treated as placeholders: not supported yet (no escaping).

### Engine

Hand-rolled 4-state parser (Text → MaybeOpen → Name → MaybeClose). Two pure functions:

- `extract_placeholders(input) -> Vec<String>` — returns unique placeholder names in
  order of first appearance.
- `substitute(input, values) -> String` — replaces each `{{ name }}` with the corresponding
  value from the map. Missing keys resolve to empty string.

No regex dependency. Lives in its own module (`src/template.rs` or similar), independent of
config or MCP logic.

### Future

Current engine is intentionally minimal. May be replaced with a mature template engine
(Jinja-compatible, Go template-compatible, etc.) if more complex substitution needs arise.
