# ACE Project

**ACE** (AI Coding Environment) - Automation tooling for setting up and keeping AI coding
environments setup and up-to-date. Acts as entrypoint to Claude Code or OpenCode CLI.

Core functions:
- Check environment readiness
- Install/update skills, agents, and conventions
- Configure chatbots to connect to LiteLLM.
- Manage model credentials.

## Communication Style

Tone:
- **Never explain** unless explicitly asked
- Be extremely concise and terse — no filler words, pleasantries, or time-wasters
- Direct answers only. Use "Acknowledged" if no more response needed
- Do not offer help or assume user needs one at the end unless a suggestion is explicitly requested
- Code comments: essential only

Workflow:
- Ask permission before editing files (group related files)
- Run commands/tests only after asking
- Never propose grand plans; always a few small steps at a time
- Always parallelize independent tasks — use parallel tool calls, concurrent agents, etc.
  whenever work items don't depend on each other

Metrics:
- Before any refactoring or code changes, run `cloc` on affected files to capture a baseline.
  After finishing, report `git diff --stat` to show lines changed.

## Dependencies

- Prioritize fast compilation times when choosing crates
- Prefer small crates with fewer compilation units over feature-rich heavy ones
- Crate must be stable and well-maintained
- Measure twice before adding a new dependency
- Check crate versions/metadata/docs via `cargo search` or `cargo info`, not web searches

## Coding Style

Clarity:
- Clarity over compression — prefer named variables for each branch over long chained expressions
- When there are multiple possible sources for a value, compute each into a named variable
  first, then combine (e.g. `xdg.or_else(|| home)`)
- Use monadic combinators (`map`, `and_then`, `unwrap_or`, etc.) on `Option`/`Result` where
  they simplify over match/if chains

Error handling:
- Prefer per-module error enums (e.g. `ParseError`) over `Box<dyn Error>`
- **NEVER use `.unwrap()`** — always propagate errors with `?` or handle explicitly. No exceptions.
- In tests, use `.expect("reason")` instead of `.unwrap()` so failures always have context.
- Be strict with error handling everywhere. No lazy shortcuts, no swallowing errors.

Structure:
- Keep indentation shallow — fail fast with `?` or early return instead of nesting. When
  branching is unavoidable, keep branch arms short: delegate to named functions rather than
  inlining logic inside match/if blocks.

## Code Grouping (Typography for Code)

Apply print typography principles to code layout. Code is read far more than written — visual
structure should communicate intent before the reader parses any syntax.

- **Proximity** — statements that work together belong together. Group related lines (e.g. a
  variable and the operation on it, or a sequence of setup steps) with no blank lines between
  them. The absence of space signals "these are one thought."
- **Paragraph breaks** — separate groups with a single blank line. Each group should represent
  one logical step or concern: setup, transformation, result. A blank line says "new thought
  starts here." Two blank lines is too many inside a function.
- **Chunking** — aim for groups of roughly 3-5 lines. Humans parse information in small chunks.
  A function that is a wall of 20 ungrouped lines is harder to read than four groups of five.
  Conversely, scattering every line with blank lines between them destroys grouping and makes
  everything feel equally (un)important.
- **Rhythm** — alternate between density (grouped logic) and space (blank line separators) to
  create a visual rhythm. The reader's eye should be able to skim group boundaries and
  understand the function's flow at a glance, like scanning paragraph starts in prose.
- **Method structure** — ideal functions follow a three-act pattern: (1) preconditions — validate
  inputs and fail fast with `?` or early return, (2) do work — the core logic, (3) postconditions
  — verify results if needed, then return. Each act is its own group separated by blank lines.
  This makes it immediately obvious where validation ends and real work begins.
- **Consistency** — apply the same grouping logic across the codebase. If `resolve()` groups
  as parse → compute → return, then similar functions should follow the same cadence. Consistent
  rhythm across files reduces cognitive load when navigating unfamiliar code.

## Config and Data Structs

- **Loading vs validation**: Serde handles parsing only. Validation is a separate pass in code
  after loading. See `prd/01-configuration.md` for details.
- All config/DTO structs use `#[derive(Default)]` + `#[serde(default)]` at the struct level.
  No per-field `#[serde(default)]`.
- **Prefer default**: Prefer `String` (defaults to `""`) over `Option<String>` when there is no
  meaningful distinction between absent and empty. Same for `Vec<T>` (empty vec) vs
  `Option<Vec<T>>`. Reserve `Option<T>` for cases where absence carries distinct semantics
  from the zero value.

## Action Pattern

- **Actions are for mutations only** — disk writes, git commands, process exec, network calls.
  Pure computation (building strings, merging data, validation) belongs on `State` or in
  helper modules under `state/`, not as actions.
- Actions are structs with params as fields, single method: `run(&self, session: &mut Session)`
- No extra parameters in `run()` — everything goes on the struct
- All actions live in `state/actions/`
- Session bundles `&mut State` — passed to every action

## PRD Compliance

- **Read `prd/02-architecture.md` before starting any coding task** — it defines the
  Config/State/Ace layer boundaries and data flow
- During coding tasks, flag any deviation from PRDs or missing PRD coverage
- Ask for directions before proceeding when implementation would differ from PRD

## Testing

- Unit tests are inline `#[cfg(test)] mod tests` in the same file
- Longer integration tests will go in an external `tests/` crate later
- **No tautological tests** — don't test trivial getters/accessors that just return a value
  (e.g. `assert_eq!(Backend::Claude.binary(), "claude")`). These restate the implementation
  and catch nothing. Similarly, don't test that serde serializes/deserializes correctly —
  that's testing the crate, not our code. Test behavior that involves logic, branching, or
  composition.

## CLI Conventions

- `ace paths` uses tab-separated `key\tvalue` for machine parseability
- Paths printed regardless of whether they exist on disk
- Command help text lives in clap doc comments / attributes, not in PRDs
- When modifying commands, ensure `--help` text stays aligned with code behavior

## TUI Pattern

- `term_ui` module: `Tui::new(&mut Ace)` + `tui.show(Screen::...)` — no trait, no dynamic dispatch
- `Screen` is a dumb enum (no data). Tui drives rendering, input, and action execution.
- Each screen method: `ratatui::init()` → render loop → `ratatui::restore()` → run action
- Always restore terminal before returning (even on error)
- Session has no UI field — interactive input is handled entirely by term_ui

## Cache Layout

- Git clones: `~/.cache/ace/repos/{owner/repo}/`
- Index: `~/.cache/ace/index.toml` — tracks downloaded schools (specifier, repo, path)
- Git operations use `std::process::Command`, no sqlite or git crate

## Tech Debt

- `user_config.rs` has duplicate `dirs_or_home`/`config_dir` — refactor to use `config::paths` versions

## Pending Work

- Setup modes discussion: see `prd/` notes
