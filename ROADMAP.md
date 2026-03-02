# ACE Roadmap

## Priority

- [ ] **PKCE auth flow** — blocker for multi-user rollout. `authenticate.rs` is a stub.
      `ace auth` command removed; fold auth into setup flow when implemented.
      Also blocks `ace school propose` and proposing pending school cache changes.

## Features

- [ ] **MCP registration** — register `[[mcp]]` from school.toml into the active backend.
      Claude (first): `claude mcp add-json -s project` per entry. CLI handles merging.
      OpenCode/Codex: deferred until those backends ship (requires direct file writes).
      All backends spawn `docker run -i --rm` as stdio child — ACE doesn't manage containers.
- [ ] **Codex backend** — third Backend variant. See research notes in MEMORY.md.
      Instructions file: `AGENTS.md` (not `CLAUDE.md`). Config: TOML in `.codex/config.toml`.
      Skills in `.agents/skills/`. Exec: `codex` (interactive) or `codex exec` (scripted).
      LiteLLM: native via `OPENAI_BASE_URL` or `model_providers` config.
- [x] **TUI school picker** — multi-school selection when multiple cached schools exist
- [ ] Add `tool` field to AceToml so Link knows backend-specific target dir
- [ ] `role` and `description` fields in ace_toml.rs for non-dev roles (PM, requirements-only)

## School

- [x] **Service entry instructions** — school CLAUDE.md guidance for AI to add `[[services]]`
      entries to school.toml. Covers CLI, TOML format, GitHub/Linear providers, scope guidance.
- [ ] Propose pending school cache changes (general-coding, rust-coding, typst-coding skills) — blocked on auth
- [ ] Update school CLAUDE.md template: commit messages should be detailed (PR-description level)

## Backlog

- [ ] Setup modes discussion — see `spec/` notes
- [ ] Auto `--continue` magic
- [ ] Cross-build script (`cargo` native, `cross` for cross-platform)
- [ ] Release workflow — blocked on [github-mcp-server#1012](https://github.com/github/github-mcp-server/issues/1012)
- [ ] Self-update — transparent auto-update for the ace binary
- [ ] Skill diff tool — compare skill versions after update
