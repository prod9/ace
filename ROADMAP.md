# ACE Roadmap

## Current Focus

1. **School structure and format** — finalize PRDs for school layout, school.toml, and skills format
2. **Dogfood** — embed ACE's own school into this repo so ACE eats its own dog food

## PRD Completion

- [x] Core overview (`prd/00-overview.md`)
- [x] Configuration management (`prd/01-configuration.md`)
- [ ] Initialization flow (`prd/02-initialization.md`) — stub
- [x] Skills sync (`prd/03-skills-sync.md`)
- [x] Authentication (`prd/05-authentication.md`)
- [ ] `ace learn` (`prd/04-learn.md`) — stub
- [x] School overview (`prd/school/00-overview.md`)
- [x] school.toml spec (`prd/school/01-school-toml.md`)

## Implementation

- [ ] Project scaffolding (CLI arg parsing, error handling, config structs)
- [ ] Config parsing — TOML loader with 3-layer merge
- [ ] Context resolution — CLI flag / project config / prompt
- [ ] School fetch — git clone/pull for remote sources, local folder support
- [ ] Skills sync — symlink school skills into project
- [ ] Cache management — `~/.cache/ace/{context}/`, SHA-based freshness check
- [ ] Initialization flow — first-run setup when no config found
- [ ] `ace learn` — open Claude Code/OpenCode on the school clone
- [ ] Backend selection and exec — hand off to Claude Code or OpenCode
