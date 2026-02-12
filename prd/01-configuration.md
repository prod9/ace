# Configuration Management

## Format

TOML.

## Layers

Resolved by merging (later overrides earlier):

1. **User-global** `~/.config/ace/config.toml`
2. **Project-local** `ace.local.toml` (gitignored, per-machine)
3. **Project-committed** `ace.toml` (checked into git, shared)

## Contexts

Multiple named contexts. A context represents a single organizational scope (e.g. a company
or team). Selected per-project or via CLI flag.

```toml
[context.acme]
source = "github.com/acme-corp/ace"

[context.acme.tokens]
github = "ghp_..."
jira = "jira_..."

[context.personal]
source = "github.com/myuser/ace"

[context.personal.tokens]
github = "ghp_..."
```

- `source` -- Git-cloneable URL for the organization's school (ACE source repository). Contains
  application component descriptions, skills, conventions, agent configs, etc.
- `tokens.github` -- GitHub access token.
- `tokens.jira` -- JIRA access token.

