# Configuration Management

## Format

TOML.

## Layers

Resolved by merging (later overrides earlier):

1. **User** `~/.config/ace/ace.toml` (or `$XDG_CONFIG_HOME/ace/ace.toml`) — personal defaults across all projects
2. **Project** `ace.toml` (checked into git, shared with team)
3. **Local** `ace.local.toml` (gitignored, per-machine overrides)

### Fields

Each layer can set:

- `school` — school specifier (last non-empty wins)
- `backend` — `"claude"` or `"codex"` (highest-priority `Some` wins: local → project → user; fallback `claude`). See [backend.md](backend.md).
- `role` — selected role name (last non-empty wins). Must match a `[[roles]]` entry in the school's `school.toml`. Typically set in `ace.local.toml` via interactive selection. See [roles.md](roles.md).
- `session_prompt` — additional prompt text (last non-empty wins)
- `env` — environment variables (additive merge, later keys override)
- `skip_update` — disable automatic version check and background upgrade. Default: `false`.
  See [upgrade.md](upgrade.md). Also overridden by `ACE_SKIP_UPDATE=1` env var.

### Personal-only fields

These fields are resolved from the **user** and **local** layers only — never from
project-committed `ace.toml` or `school.toml`. They are personal workflow preferences.

- `trust` — permission mode: `"default"`, `"auto"`, or `"yolo"`. Default: `"default"`.
- `resume` — auto-resume previous session on `ace` launch. Default: `true`.
  When `true`, `ace` passes resume flags to the backend if a previous session exists for
  the current project directory. `ace --new` (or `ace -n`) forces a fresh session regardless.
  Backends that don't support resume start fresh silently.

Resolution for personal-only fields: local wins over user. Project layer is skipped entirely.

## Scope Flags

Write commands (`ace config set`, `ace auto`, `ace yolo`) accept a scope flag to choose
which layer to write to:

- `--user` (alias: `--global`) — write to user-level `~/.config/ace/ace.toml`
- `--project` — write to project `ace.toml`
- `--local` — write to `ace.local.toml`

When no scope flag is given, the default is inferred from the key:

- Personal-only fields (`trust`, `resume`) → `--local`
- Shared fields (`school`, `backend`, `session_prompt`, `env.*`, `skip_update`) → `--project`

An explicit scope flag always overrides inference.

## Config Commands

### `ace config`

Bare `ace config` prints the effective resolved configuration (all layers merged).

### `ace config get <key>`

Print the effective resolved value for a single key. Outputs the raw value, one line.

Keys: `school`, `backend`, `trust`, `resume`, `session_prompt`, `skip_update`, `env.KEY`.

### `ace config set <key> <value> [--user|--project|--local]`

Write a single field to the appropriate layer. Loads the target file, modifies the field,
saves back. Other fields in that file are preserved.

Key syntax:
- Simple fields: `backend`, `school`, `trust`, `resume`, `session_prompt`, `skip_update`
- Env map entries: `env.KEY` — dot-path into the `[env]` table (e.g. `ace config set env.ANTHROPIC_API_KEY sk-...`)

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
