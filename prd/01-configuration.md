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
- `backend` — `"claude"` or `"opencode"` (last `Some` wins, fallback `claude`). See [07-backend.md](07-backend.md).
- `session_prompt` — additional prompt text (last non-empty wins)
- `env` — environment variables (additive merge, later keys override)

## Schools

Schools are identified by a specifier in `ace.toml` (see [school/00-overview.md](school/00-overview.md#specifier)).
Credentials are keyed by the source portion (`owner/repo` or `.`) in config and cache paths.

```toml
["acme-corp/school"]

["acme-corp/school".services.github]
token = "ghp_..."

["acme-corp/school".services.jira]
token = "jira_..."
username = "alice"

["myuser/school"]

["myuser/school".services.github]
token = "ghp_..."
```

- `services.<name>.token` -- Access token for the service.
- `services.<name>.*` -- Additional service-specific fields (e.g. `username`).

## Loading vs Validation

Config loading and config validation are separate concerns.

### Loading

Serde handles deserialization only. All config structs use `#[derive(Default)]` and
`#[serde(default)]` at the struct level. Every field parses successfully regardless of what's
present in the TOML — missing keys get their type's `Default` value (empty string, empty vec,
`None`, etc.).

This means the TOML `[school]` with no `name` key produces `SchoolMeta { name: "".into(), .. }`
rather than a serde error. Partial or empty files always parse.

### Validation

After loading, a separate validation pass checks invariants and produces clear, actionable errors.
Validation runs on the merged config (after all three layers are combined), not on individual files.

Rules are expressed in code, not via serde attributes. Examples:

- `school.name` must be non-empty.
- `services[].authorize_url` and `services[].token_url` must be valid URLs.
- `services[].client_id` must be non-empty.
- No duplicate `services[].name` entries.
- No duplicate `mcp[].name` entries.
- `projects[].repo` must be a valid specifier.

Validation errors reference the offending key path: e.g. `school.name: must not be empty`,
`services[0].authorize_url: not a valid URL`.

### Why

- Serde's "missing field" errors are opaque and unhelpful to users.
- Required-vs-optional is a validation concern, not a parsing concern.
- Validation on the merged config catches cross-layer issues (e.g. a layer overrides a field to
  an invalid value).
- Richer checks (URL format, uniqueness, non-empty) cannot be expressed through serde alone.

