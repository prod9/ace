# school.toml

The `school.toml` file lives at the root of a school repository. It declares metadata,
configuration, MCP tools, environment, and project catalog for the school.

## Example

```toml
[school]
name = "acme"
description = "Acme Corp engineering school"

[env]
NODE_VERSION = "22"
PYTHON_VERSION = "3.12"
LITELLM_BASE_URL = "https://llm.acme.corp/v1"
DOCKER_REGISTRY = "registry.acme.corp"

[[auth]]
name = "github"
authorize_url = "https://github.com/login/oauth/authorize"
token_url = "https://github.com/login/oauth/access_token"
client_id = "Iv1.abc123"
scopes = ["repo", "read:org"]

[[auth]]
name = "jira"
authorize_url = "https://auth.atlassian.com/authorize"
token_url = "https://auth.atlassian.com/oauth/token"
client_id = "xyz789"
scopes = ["read:jira-work", "write:jira-work"]

[[mcp]]
name = "jira"
image = "ghcr.io/acme-corp/mcp-jira:latest"
env = { JIRA_URL = "https://acme.atlassian.net", JIRA_TOKEN = "{{ tokens.jira }}" }

[[mcp]]
name = "db"
image = "ghcr.io/acme-corp/mcp-db:latest"
env = { DB_HOST = "localhost" }

[[projects]]
name = "backend"
repo = "github.com/acme-corp/backend"
description = "Go API server. Handles auth, billing, and core business logic."

[projects.env]
SERVICE_NAME = "backend"

[[projects.mcp]]
name = "migrations"
image = "ghcr.io/acme-corp/mcp-migrations:latest"
env = { DB_TOKEN = "{{ tokens.db }}" }

[[projects]]
name = "frontend"
repo = "github.com/acme-corp/frontend"
description = "Next.js web app. Customer-facing dashboard and admin portal."

[projects.env]
SERVICE_NAME = "frontend"

[[projects]]
name = "infra"
repo = "github.com/acme-corp/infra"
description = "Terraform and Kubernetes configs for AWS deployment."
```

## Sections

### `[school]`

- `name` — Human-readable name for the school. Used in logs and UI.
- `description` — Short description of the school/organization. Included in AI system context.

### `[env]`

Key-value pairs of environment variables. Set in the shell before exec-ing Claude Code or
OpenCode. Use for shared endpoints, API base URLs, feature flags, etc.

These are not secrets. Tokens and credentials belong in the user-level config
(`~/.config/ace/config.toml` under `context.*.tokens`).

### `[[auth]]`

Array of OAuth service declarations. Each entry defines a service that developers need to
authenticate against. See [05-authentication.md](../05-authentication.md) for the full PKCE
flow and token lifecycle.

- `name` — Token identifier. Referenced in templates as `{{ tokens.<name> }}`.
- `authorize_url` — OAuth authorization endpoint.
- `token_url` — OAuth token exchange endpoint.
- `client_id` — OAuth app client ID. Not a secret — safe to commit.
- `scopes` — List of OAuth scopes to request.

### `[[mcp]]`

Array of MCP server declarations. Each entry defines a containerized MCP tool server. ACE uses
Dagger to spin up the containers and connects them to Claude Code or OpenCode.

MCP servers are packaged as container images. This eliminates host dependency management — no
need to install runtimes, packages, or tools required by individual MCP servers.

- `name` — Identifier for the MCP server.
- `image` — Container image reference (e.g. `ghcr.io/acme-corp/mcp-jira:latest`).
- `env` — Optional. Environment variables passed into the container. Supports template syntax
  for secrets (see below).

### Template Syntax

Values in `env` fields can reference tokens from the user's local config using
`{{ tokens.<name> }}`. At runtime, ACE resolves these against `context.*.tokens` in
`~/.config/ace/config.toml`. The `<name>` matches the `name` field in `[[auth]]`.

```toml
env = { JIRA_TOKEN = "{{ tokens.jira }}" }
```

If a token cannot be resolved (not yet authorized by the user), ACE warns and skips that MCP
server. It does not block.

### `[[projects]]`

Catalog of projects in the organization. Gives the AI context about available repositories and
what they do, enabling better cross-project reasoning and navigation.

- `name` — Short project identifier.
- `repo` — Git-cloneable URL for the project.
- `description` — What the project is and does. Written for AI/LLM consumption — be specific
  about tech stack, domain, and responsibilities.
- `env` — Optional. Project-specific environment variables. Merged with top-level `[env]`
  (project values override).
- `mcp` — Optional. Project-specific MCP servers. Added alongside top-level `[[mcp]]` servers.
  Same schema as top-level `[[mcp]]`.
