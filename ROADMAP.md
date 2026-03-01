# ACE Roadmap

## Current Focus

1. **Clean up compiler warnings** — remove unused structs/imports
2. **Authentication PRD update** — fold auth into setup flow (no separate `ace auth`)
3. **TUI school picker** — when multiple cached schools exist
4. **Tech debt** — deduplicate `dirs_or_home` in `user_config.rs`
5. **Dogfood** — embed ACE's own school into this repo

## Implementation

- [ ] Authentication — PKCE OAuth (folded into setup flow, not separate command)
- [ ] TUI school picker — multi-school selection in setup

## Future Ideas

- **Self-update** — transparent auto-update mechanism for the ace binary. Check for new versions
  on run, download and replace in-place. Should be as seamless as possible — no manual steps.
- **Skill diff tool** — compare code changes between different versions of skills (e.g. after
  `ace setup` pulls a newer school). Show what changed so users can review before accepting updates.

