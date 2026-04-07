# Decision: Local (stdio) MCP Servers (2026-04-07)

[Decision 002](002-remote-only-mcp.md) restricted ACE to remote-only MCP. This decision extends
MCP support to include local stdio servers while keeping remote as the default.

## Why

Local-first tools like mempalace (AI memory management) expose MCP servers via stdio — they run
as a local process, not a hosted endpoint. There is no remote equivalent. Schools that want to
mandate such tools need stdio MCP support.

The remote-only rationale (002) was sound for vendor-hosted services. It does not cover tools
that are inherently local: memory systems, local databases, project-specific tooling, dev
utilities. These tools will not move to remote hosting because their value comes from local
filesystem and process access.

## Scope

Extend `[[mcp]]` in `school.toml` to support stdio transport alongside the existing remote
(HTTP) transport. The two transports are mutually exclusive per entry — an entry has either
`url` or `command`, never both.

## What Changes

- `McpDecl` gains `command`, `args`, and `env` fields.
- Validation enforces: exactly one of `url` or `command` must be non-empty.
- `headers` is only valid when `url` is set.
- Backend `mcp_add()` implementations handle both transports.
- Registration scope for stdio servers: **project** (not user), because stdio servers often
  depend on project-specific paths or virtualenvs.

## What Does Not Change

- Remote MCP registration flow (unchanged).
- Auth model (backends still own OAuth/token storage for remote servers).
- Health check mechanism (`mcp_check()` works regardless of transport).
- Placeholder substitution (works in `env` values too, same `{{ name }}` syntax).
