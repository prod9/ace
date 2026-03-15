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

- [x] **~~MCP registration~~** — register `[[mcp]]` from school.toml into the active backend.
      Claude: `claude mcp add-json -s project` per entry. OpenCode/Codex: deferred (hint shown).
- [x] **~~Codex backend~~** — Backend variant fully wired. Setup is now backend-aware: resolves
      backend from config layers, uses correct skills_dir (.agents/) and instructions file
      (AGENTS.md). Exec already supported via Backend enum.
- [x] **TUI school picker** — multi-school selection when multiple cached schools exist
- [x] ~~Add `tool` field to AceToml so Link knows backend-specific target dir~~ — superseded by
      backend-aware setup. Backend resolution from config layers determines skills_dir.
- [x] ~~`role` and `description` fields in ace_toml.rs~~ — for non-dev roles (PM, requirements-only).
      Last-Some-wins resolution across config layers. Injected into session prompt.

## School

- [ ] Propose pending school cache changes (general-coding, rust-coding, typst-coding skills)
- [x] ~~Update school CLAUDE.md template~~: commit messages as policy memos (see `spec/school/overview.md`)

## Testing

- [x] **~~Network-dependent tests~~** — `#[ignore]` strategy with `ACE_TEST_NETWORK` env var.
      Run with `cargo test -- --ignored`. Tests in `tests/network_test.rs`.
- [x] **~~DRY up test code~~** — `setup_embedded_school()` and `setup_embedded()` helpers
      added to TestEnv. Config and paths tests simplified.
- [x] **~~Extract shared git/fs utilities~~** — `clone_repo`, `copy_dir_recursive`,
      `discover_skills` moved to `actions/utils.rs`. `import_skill.rs` and `school_update.rs`
      import from shared module.

## Backlog

- [x] **~~Ctrl+C / signal handling~~** — SIGINT handler restores cursor visibility before exit.
      Only in human mode. Uses signal-hook flag + watchdog thread. Exit code 130.
- [ ] Setup modes discussion — see `spec/` notes
- [ ] Auto `--continue` magic
- [ ] Cross-build script (`cargo` native, `cross` for cross-platform)
- [ ] Release workflow — blocked on [github-mcp-server#1012](https://github.com/github/github-mcp-server/issues/1012)
- [ ] Self-update — transparent auto-update for the ace binary
- [ ] `ace switch` — switch between backends (re-link folders, regenerate instructions file)
- [ ] Skill diff tool — compare skill versions after update
