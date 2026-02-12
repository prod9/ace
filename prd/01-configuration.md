# Configuration Management

## Format

TOML.

## Layers

Resolved by merging (later overrides earlier):

1. **User-global** `~/.config/ace/config.toml`
2. **Project-local** `ace.local.toml` (gitignored, per-machine)
3. **Project-committed** `ace.toml` (checked into git, shared)

## Schools

Multiple named schools. A school represents a single organizational scope (e.g. a company
or team). Selected per-project or via CLI flag.

```toml
[acme]
source = "https://github.com/acme-corp/ace.git"

[acme.services.github]
token = "ghp_..."

[acme.services.jira]
token = "jira_..."
username = "alice"

[personal]
source = "https://github.com/myuser/ace.git"

[personal.services.github]
token = "ghp_..."
```

- `source` -- Git-cloneable URL for the school repository. Contains
  application component descriptions, skills, conventions, agent configs, etc.
- `services.<name>.token` -- Access token for the service.
- `services.<name>.*` -- Additional service-specific fields (e.g. `username`).

