# ACE Roadmap

## Current Focus

1. **Clean up compiler warnings** — remove unused structs/imports
2. **Authentication PRD update** — fold auth into setup flow (no separate `ace auth`)
3. **TUI school picker** — when multiple cached schools exist
4. **Tech debt** — deduplicate `dirs_or_home` in `user_config.rs`
5. **Dogfood** — embed ACE's own school into this repo

## PRD Completion

- [x] Core overview (`prd/00-overview.md`)
- [x] Configuration management (`prd/01-configuration.md`)
- [x] Architecture (`prd/02-architecture.md`)
- [x] Setup flow (`prd/03-setup.md`)
- [x] Skills sync (`prd/04-skills-sync.md`)
- [x] Authentication (`prd/06-authentication.md`) — PRD done, implementation pending
- [x] School overview (`prd/school/00-overview.md`)
- [x] school.toml spec (`prd/school/01-school-toml.md`)
- [x] School commands (`prd/school/02-school-commands.md`)

## Implementation

- [x] Project scaffolding (CLI arg parsing, error handling, config structs)
- [x] Config parsing — TOML loader with 3-layer merge
- [x] School fetch — git clone/pull for remote sources, local folder support
- [x] Skills sync — symlink school skills into project
- [x] Cache management — `~/.cache/ace/repos/`, index.toml tracking
- [x] Setup flow — `ace setup` with Prepare/Install/Update/Link
- [x] School propose — `ace school propose` / `ace school pr`
- [x] System prompt building and sync
- [x] Backend selection and exec — hand off to Claude Code or OpenCode
- [ ] Authentication — PKCE OAuth (folded into setup flow, not separate command)
- [ ] TUI school picker — multi-school selection in setup

## School Skills

- **Rust skill** — first skill to write. Must establish crate selection preferences: prefer
  small, ergonomic crates over large traditional ones. Optimize for ease of use, ease of
  reasoning, compilation time, and binary size. Example: smol over tokio for async runtime.
