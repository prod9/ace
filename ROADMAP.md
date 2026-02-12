# ACE Roadmap

## Current Focus

1. **School structure and format** — finalize PRDs for school layout, school.toml, and skills format
2. **Dogfood** — embed ACE's own school into this repo so ACE eats its own dog food

## PRD Completion

- [x] Core overview (`prd/00-overview.md`)
- [x] Configuration management (`prd/01-configuration.md`)
- [x] Setup flow (`prd/02-setup.md`)
- [x] Skills sync (`prd/03-skills-sync.md`)
- [x] Authentication (`prd/05-authentication.md`)
- [ ] `ace learn` (`prd/04-learn.md`) — stub
- [ ] Context management (`prd/06-context-management.md`) — stub
- [x] School overview (`prd/school/00-overview.md`)
- [x] school.toml spec (`prd/school/01-school-toml.md`)

## Implementation

- [ ] Project scaffolding (CLI arg parsing, error handling, config structs)
- [ ] Config parsing — TOML loader with 3-layer merge
- [ ] Context resolution — CLI flag / project config / prompt
- [ ] School fetch — git clone/pull for remote sources, local folder support
- [ ] Skills sync — symlink school skills into project
- [ ] Cache management — `~/.cache/ace/{context}/`, SHA-based freshness check
- [ ] Setup flow — `ace setup` command for first-run and context management
- [ ] `ace learn` — open Claude Code/OpenCode on the school clone
- [ ] Backend selection and exec — hand off to Claude Code or OpenCode

## School Skills

- **Rust skill** — first skill to write. Must establish crate selection preferences: prefer
  small, ergonomic crates over large traditional ones. Optimize for ease of use, ease of
  reasoning, compilation time, and binary size. Example: smol over tokio for async runtime.
