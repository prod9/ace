# ACE Project

**ACE** (AI Coding Environment) - Automation tooling for setting up and keeping AI coding
environments setup and up-to-date. Acts as entrypoint to Claude Code or OpenCode CLI.

Core functions:
- Check environment readiness
- Install/update skills, agents, and conventions
- Configure chatbots to connect to LiteLLM.
- Manage model credentials.

## Communication Style

- **Never explain** unless explicitly asked
- Be extremely concise and terse
- No filler words, pleasantries, or time-wasters
- Direct answers only
- Use "Acknowledged" if no more response needed
- Ask permission before editing files (group related files)
- Run commands/tests only after asking
- Code comments: essential only
- Do not offer help or assume user needs one at the end unless a suggestion is explicitly requested
- Never propose grand plans; always a few small steps at a time

## Dependencies

- Prioritize fast compilation times when choosing crates
- Prefer small crates with fewer compilation units over feature-rich heavy ones
- Crate must be stable and well-maintained
- Measure twice before adding a new dependency
- Check crate versions/metadata/docs via `cargo search` or `cargo info`, not web searches

## Coding Style

- Clarity over compression — prefer named variables for each branch over long chained expressions
- When there are multiple possible sources for a value, compute each into a named variable first, then combine (e.g. `xdg.or_else(|| home)`)
- Prefer per-module error enums (e.g. `ParseError`) over `Box<dyn Error>`
- **NEVER use `.unwrap()`** — always propagate errors with `?` or handle explicitly. No exceptions.
- In tests, use `.expect("reason")` instead of `.unwrap()` so failures always have context.
- Be strict with error handling everywhere. No lazy shortcuts, no swallowing errors.

## Action Pattern

- Actions are structs with params as fields, single method: `run(&self, session: &mut Session)`
- No extra parameters in `run()` — everything goes on the struct
- All actions live in `state/actions/`
- Session bundles `&mut State` + `&dyn UI` — passed to every action

## PRD Compliance

- During coding tasks, flag any deviation from PRDs or missing PRD coverage
- Ask for directions before proceeding when implementation would differ from PRD

## Testing

- Unit tests are inline `#[cfg(test)] mod tests` in the same file
- Longer integration tests will go in an external `tests/` crate later

## Output Formats

- `ace paths` uses tab-separated `key\tvalue` for machine parseability
- Paths printed regardless of whether they exist on disk

## Help Text

- Command help text lives in clap doc comments / attributes, not in PRDs
- When modifying commands, ensure `--help` text stays aligned with code behavior

## Tech Debt

- `user_config.rs` has duplicate `dirs_or_home`/`config_dir` — refactor to use `config::paths` versions

## Pending Work

- Setup modes discussion: see `prd/` notes
