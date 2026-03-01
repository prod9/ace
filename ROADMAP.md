# ACE Roadmap

## Priority

- [ ] **PKCE auth flow** — blocker for multi-user rollout. `authenticate.rs` is a stub.
      `ace auth` command removed; fold auth into setup flow when implemented.
      Also blocks `ace school propose` and proposing pending school cache changes.

## Features

- [ ] **MCP templating** — template `[[mcp]]` from school.toml into Claude's native MCP config.
      Don't reimplement Docker lifecycle — Claude manages containers natively.
      Check OpenCode MCP support for parity.
- [ ] **Codex backend** — investigate OpenAI Codex CLI. Third Backend variant?
- [ ] **TUI school picker** — multi-school selection when multiple cached schools exist
- [ ] Add `tool` field to AceToml so Link knows backend-specific target dir
- [ ] `role` and `description` fields in ace_toml.rs for non-dev roles (PM, requirements-only)
- [ ] Dogfood — embed ACE's own school into this repo

## School

- [ ] Propose pending school cache changes (general-coding, rust-coding, typst-coding skills) — blocked on auth
- [ ] Update school CLAUDE.md template: commit messages should be detailed (PR-description level)

## Backlog

- [ ] Setup modes discussion — see `spec/` notes
- [ ] Auto `--continue` magic
- [ ] Cross-build script (`cargo` native, `cross` for cross-platform)
- [ ] Release workflow — blocked on [github-mcp-server#1012](https://github.com/github/github-mcp-server/issues/1012)
- [ ] Self-update — transparent auto-update for the ace binary
- [ ] Skill diff tool — compare skill versions after update
- [ ] Dagger integration tests — containers for isolated filesystem/git scenarios
