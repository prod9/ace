# Configuration Management

## Format

TOML.

## Layers

Resolved by merging (later overrides earlier):

1. **User-global** `~/.config/ace/config.toml`
2. **Project-local** `ace.local.toml` (gitignored, per-machine)
3. **Project-committed** `ace.toml` (checked into git, shared)

## Schools

Schools are identified by GitHub `owner/repo` shorthand. This is used as the key in config,
cache paths, and project references.

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

