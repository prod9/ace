# ACE Roadmap

## Priority

- [x] **~~PKCE auth flow~~** — superseded by remote MCP + OAuth. All three backends (Claude,
      OpenCode, Codex) handle OAuth discovery natively for remote MCP servers. ACE delegates
      auth entirely to the backend. `authenticate.rs` stub can be removed.
- [x] ~~Investigate adding rules() to school~~ — generalized to link all 4 school folders
      (skills, rules, commands, agents) with backend compatibility warnings
- [x] **~~Services removal~~** — `[[services]]`, `ServiceDecl`, `authenticate.rs`,
      `ace school add-service`, `{{ services.X.token }}` templating all removed. `[[mcp]]`
      simplified to `name` + `url` (remote-only). See `spec/mcp.md`.

## Features

- [ ] **MCP registration** — register `[[mcp]]` from school.toml into the active backend.
      Claude (first): `claude mcp add-json -s project` per entry. CLI handles merging.
      OpenCode/Codex: deferred until those backends ship (requires direct file writes).
      All entries are remote URLs — no Docker/stdio.
- [ ] **Codex backend** — third Backend variant. See research notes in MEMORY.md.
      Instructions file: `AGENTS.md` (not `CLAUDE.md`). Config: TOML in `.codex/config.toml`.
      Skills in `.agents/skills/`. Exec: `codex` (interactive) or `codex exec` (scripted).
      LiteLLM: native via `OPENAI_BASE_URL` or `model_providers` config.
- [x] **TUI school picker** — multi-school selection when multiple cached schools exist
- [ ] Add `tool` field to AceToml so Link knows backend-specific target dir
- [ ] `role` and `description` fields in ace_toml.rs for non-dev roles (PM, requirements-only)

## School

- [ ] Propose pending school cache changes (general-coding, rust-coding, typst-coding skills)
- [ ] Update school CLAUDE.md template: commit messages should be detailed (PR-description level)

## Backlog

- [ ] Setup modes discussion — see `spec/` notes
- [ ] Auto `--continue` magic
- [ ] Cross-build script (`cargo` native, `cross` for cross-platform)
- [ ] Release workflow — blocked on [github-mcp-server#1012](https://github.com/github/github-mcp-server/issues/1012)
- [ ] Self-update — transparent auto-update for the ace binary
- [ ] `ace switch` — switch between backends (re-link folders, regenerate instructions file)
- [ ] Skill diff tool — compare skill versions after update
