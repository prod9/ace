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

## Config and Data Structs

- **Loading vs validation**: Serde handles parsing only. Validation is a separate pass in code after loading. See `prd/01-configuration.md` for details.
- All config/DTO structs use `#[derive(Default)]` + `#[serde(default)]` at the struct level. No per-field `#[serde(default)]`.
- **Prefer default**: Prefer `String` (defaults to `""`) over `Option<String>` when there is no meaningful distinction between absent and empty. Same for `Vec<T>` (empty vec) vs `Option<Vec<T>>`. Reserve `Option<T>` for cases where absence carries distinct semantics from the zero value.

## Action Pattern

- Actions are structs with params as fields, single method: `run(&self, session: &mut Session)`
- No extra parameters in `run()` — everything goes on the struct
- All actions live in `state/actions/`
- Session bundles `&mut State` — passed to every action

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

## TUI Pattern

- `term_ui` module: `Tui::new(&mut Ace)` + `tui.show(Screen::...)` — no trait, no dynamic dispatch
- `Screen` is a dumb enum (no data). Tui drives rendering, input, and action execution.
- Each screen method: `ratatui::init()` → render loop → `ratatui::restore()` → run action
- Always restore terminal before returning (even on error)
- `web_ui` will be a separate module later with its own screen types
- Session has no UI field — interactive input is handled entirely by term_ui/web_ui

## Cache Layout

- Git clones: `~/.cache/ace/repos/{owner/repo}/`
- Index: `~/.cache/ace/index.toml` — tracks downloaded schools (specifier, repo, path)
- `DownloadSchool` action clones/pulls repos, writes index entries
- `list_cached_schools` reads index.toml, never scans filesystem
- Git operations use `std::process::Command`, no sqlite or git crate

## Pending Work

- Setup modes discussion: see `prd/` notes
