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
- **NEVER assume intent — ASK.** When the user flags something, do not guess what they mean
  and silently re-plan or re-implement. Ask one clarifying question. Re-planning loops burn
  tokens and time for nothing. One question beats six wrong iterations. This is the single
  most expensive failure mode — treat ambiguous feedback as a hard stop until clarified.
- Ask permission before editing files (group related files)
- Run commands/tests only after asking
- **Never discard uncommitted changes** — do not run `git checkout`, `git restore`, or any
  command that overwrites working tree files without asking the user first. Uncommitted changes
  may be intentional work-in-progress.
- Never propose grand plans; always a few small steps at a time
- Always parallelize independent tasks — use parallel tool calls, concurrent agents, etc.
  whenever work items don't depend on each other
- **One logical change per commit** — each commit should contain exactly one sensible grouping
  of related changes. Don't lump unrelated work into a single commit, and don't split a
  coherent change across multiple commits unnecessarily.

Metrics:
- After finishing code changes, report `git diff --stat` and share your read on the delta with
  the user. Net deletions? Celebrate briefly. Small addition for new behavior? Normal. Large
  net addition? Flag it — question whether the approach is too heavy or if something simpler
  would do.
- After `cargo build` or `cargo test`, report the compilation time shown in the output. Flag
  regressions — if a change noticeably increases compile time, question whether a lighter
  approach exists.

## Dependencies

- Prioritize fast compilation times when choosing crates
- Prefer small crates with fewer compilation units over feature-rich heavy ones
- Crate must be stable and well-maintained
- Measure twice before adding a new dependency
- Check crate versions/metadata/docs via `cargo search` or `cargo info`, not web searches

## Coding Style

Naming:
- **Names must be unambiguous in context** — choose names that have one clear reading given
  the surrounding code, dependencies, and domain. E.g. in a serde-heavy codebase, `Encode`
  is a better error variant than `Serialize` because `Serialize` already means the serde trait.

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

- **Read all relevant PRDs before starting any coding task** — at minimum
  `prd/02-architecture.md` (layer boundaries) and any PRD covering the feature area.
  Run `ls prd/` to see available PRDs. Flag deviations or missing coverage.
- Ask for directions before proceeding when implementation would differ from PRD

## Testing

- **TDD flow**: If a change warrants tests (per the rules below), write the failing test first,
  run it to confirm failure, then implement. Do not write tests and implementation together.
- Unit tests are inline `#[cfg(test)] mod tests` in the same file
- Longer integration tests will go in an external `tests/` crate later
- **Future**: Use Dagger for integration tests — spin up test containers for isolated
  filesystem/git scenarios instead of temp dirs
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

- `term_ui` module: `Tui::new(&mut Ace)` + `tui.run(Workflow::...)` — no trait, no dynamic dispatch
- `Workflow` enum dispatches to methods that use `inquire` for inline prompts (Text, Select)
- Standalone `term_ui::select()` for cases where the caller runs the action (e.g. async actions)
- Session has no UI field — interactive input is handled entirely by term_ui

## Cache Layout

- Git clones: `~/.cache/ace/repos/{owner/repo}/`
- Index: `~/.cache/ace/index.toml` — tracks downloaded schools (specifier, repo, path)
- Git operations use `std::process::Command`, no sqlite or git crate

## Tech Debt

- `user_config.rs` has duplicate `dirs_or_home`/`config_dir` — refactor to use `config::paths` versions

## Pending Work

Priority:
- **PKCE auth flow** — blocker for multi-user rollout. `authenticate.rs` is still a stub.

Backlog:
- Setup modes discussion: see `prd/` notes
- Move skills.md into proper skills directory, then `ace school propose` to upstream to prod9
- Split CLAUDE.md notes into school skills: `general-coding` (language-agnostic rules) then
  `rust-coding` (Rust-specific conventions)
- Add some magic? For example, auto --continue ?
- Cross-build script — produce binaries for linux/mac × arm64/amd64 targets
