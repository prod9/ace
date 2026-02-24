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

- Check crate versions/metadata/docs via `cargo search` or `cargo info`, not web searches

## Coding Style

- Use monadic combinators (`map`, `and_then`, `unwrap_or`, etc.) on `Option`/`Result` where
  they simplify over match/if chains

Error handling:
- **One error enum per folder** — `ConfigError` for `src/config/`, `SetupError`/`PrepareError`
  for `src/state/actions/`, `CmdError` for `src/cmd/`. Action-specific errors
  (`SchoolInitError`, `SchoolProposeError`, `ImportError`) are fine when well-scoped.
  Don't create wrapper enums that just re-wrap the same `io::Error` / `toml` errors —
  consolidate into the folder-level enum.
- Actions that only produce I/O errors return `std::io::Error` directly — no wrapping.
- **NEVER use `.unwrap()`** — always propagate errors with `?` or handle explicitly. No exceptions.
- In tests, use `.expect("reason")` instead of `.unwrap()` so failures always have context.
- Be strict with error handling everywhere. No lazy shortcuts, no swallowing errors.

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

- Actions follow the unit-of-work pattern (see `general-coding` skill)
- ACE-specific: `run(&self, session: &mut Session)`, all actions in `state/actions/`
- Session bundles `&mut State` — passed to every action

## Testing

- Unit tests are inline `#[cfg(test)] mod tests` in the same file
- Longer integration tests will go in an external `tests/` crate later
- **Future**: Use Dagger for integration tests — spin up test containers for isolated
  filesystem/git scenarios instead of temp dirs
- Don't test that serde serializes/deserializes correctly — that's testing the crate, not our code.

## PRD Compliance

- ACE specs live in `prd/` — read at minimum `prd/02-architecture.md` (layer boundaries) and
  any PRD covering the feature area. Run `ls prd/` to see available PRDs.

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
- `rust-coding` skill (Rust-specific conventions) — extract from this file next
- Add some magic? For example, auto --continue ?
- Cross-build script — produce binaries for linux/mac × arm64/amd64 targets
