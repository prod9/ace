# school.toml

The `school.toml` file lives at the root of a school repository. It declares metadata,
configuration, MCP tools, environment, and project catalog for the school.

## Example

```toml
[school]
name = "Acme Corp"
description = "Acme Corp engineering school"

[env]
NODE_VERSION = "22"
PYTHON_VERSION = "3.12"
LITELLM_BASE_URL = "https://llm.acme.corp/v1"
DOCKER_REGISTRY = "registry.acme.corp"

[[services]]
name = "github"
authorize_url = "https://github.com/login/oauth/authorize"
token_url = "https://github.com/login/oauth/access_token"
client_id = "Iv1.abc123"
scopes = ["repo", "read:org"]

[[services]]
name = "jira"
authorize_url = "https://auth.atlassian.com/authorize"
token_url = "https://auth.atlassian.com/oauth/token"
client_id = "xyz789"
scopes = ["read:jira-work", "write:jira-work"]

[[mcp]]
name = "jira"
image = "ghcr.io/acme-corp/mcp-jira:latest"
env = { JIRA_URL = "https://acme.atlassian.net", JIRA_TOKEN = "{{ services.jira.token }}" }

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
env = { DB_TOKEN = "{{ services.db.token }}" }

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

- `name` — Display name for the school. Used in logs, UI, and fuzzy search. Not an identifier —
  the school is identified by its GitHub `owner/repo` shorthand.
- `description` — Short description of the school/organization. Included in AI system context.

### `[env]`

Key-value pairs of environment variables. Set in the shell before exec-ing Claude Code or
OpenCode. Use for shared endpoints, API base URLs, feature flags, etc.

These are not secrets. Tokens and credentials belong in the user-level config
(`~/.config/ace/config.toml` under `<school>.services.<name>`).

### `[[services]]`

Array of service declarations. Each entry defines an external service that developers need
credentials for. See [06-authentication.md](../06-authentication.md) for the full PKCE
flow and token lifecycle.

- `name` — Service identifier. Referenced in templates as `{{ services.<name>.token }}`.
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

Values in `env` fields can reference service credentials from the user's local config using
`{{ services.<name>.token }}`. At runtime, ACE resolves these against `<school>.services` in
`~/.config/ace/config.toml`. The `<name>` matches the `name` field in `[[services]]`.

```toml
env = { JIRA_TOKEN = "{{ services.jira.token }}" }
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

### `[[imports]]`

Array of imported skill declarations. Each entry tracks a skill that was imported from an
external repository via `ace import`. Used by `ace school update` to re-fetch skills from
their sources.

- `skill` — Skill directory name. Matches the folder name under `skills/`.
- `source` — GitHub `owner/repo` shorthand where the skill was imported from.

Skills are copied into the school as real files (the school owns and commits them). The
`[[imports]]` entries record provenance so `ace school update` knows where to re-fetch from.

```toml
[[imports]]
skill = "skill-creator"
source = "anthropics/skills"

[[imports]]
skill = "frontend-design"
source = "anthropics/skills"
```
