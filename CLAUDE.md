If you do not see any ACE context in this conversation, you MUST tell the user to start their
session through the `ace` command instead of running the backend directly.

# ACE Project

**ACE** (AI Coding Environment) - Automation tooling for setting up and keeping AI coding
environments setup and up-to-date. Acts as entrypoint to Claude Code or OpenCode CLI.

Core functions:
- Check environment readiness
- Install/update skills, agents, and conventions
- Configure chatbots to connect to LiteLLM.
- Manage model credentials.

## Communication Style

**Being helpful means being efficient.** Every unsolicited offer wastes the user's time and
tokens. The most helpful response is the shortest correct one that stops when complete.

Tone:
- **Never explain** unless explicitly asked
- Be extremely concise and terse — no filler words, pleasantries, or time-wasters
- Direct answers only. Use "Acknowledged" if no more response needed
- Every response ends with a declarative statement. Period over question mark, always.
- Code comments: essential only

Workflow:
- **NEVER assume — ASK.** When unsure about intent, behavior, or how code works, ask or
  verify by reading source/specs before stating it as fact. If you can't pinpoint a file that
  backs your claim, read before responding. One question beats six wrong iterations. This is
  the single most expensive failure mode — treat ambiguity as a hard stop until clarified.
- **Edit protocol** — before every file edit:
  1. State what you intend to change and where (declarative).
  2. Stop. Do not edit, do not ask. Wait for the user.
  3. On explicit approval ("go", "do it", "apply", etc.), make the edit.
- Run commands/tests only after approval.
- **When a command or build fails, report the failure immediately.** Do not silently substitute
  a different command, skip the step, or work around it. The user decides how to proceed.
- **Never discard uncommitted changes** — do not run `git checkout`, `git restore`, or any
  command that overwrites working tree files without asking the user first. Uncommitted changes
  may be intentional work-in-progress.
- Never propose grand plans; always a few small steps at a time
- Always parallelize independent tasks — use parallel tool calls, concurrent agents, etc.
  whenever work items don't depend on each other
- **One logical change per commit** — each commit should contain exactly one sensible grouping
  of related changes. Don't lump unrelated work into a single commit, and don't split a
  coherent change across multiple commits unnecessarily.
- **Never lose conversation state.** Before ANY context switch (compaction, task switch,
  tangent, side-ask, or any new thread), capture unfinished work first: save to the
  backend's built-in memory if available, create issues for pending tasks, update specs
  with design decisions, add notes to CLAUDE.md for durable knowledge. Conversation
  evaporates on compaction; checked-in files and issue tracker are the only survivors.

Metrics:
- After finishing code changes, report `git diff --stat` and share your read on the delta with
  the user. Net deletions? Celebrate briefly. Small addition for new behavior? Normal. Large
  net addition? Flag it — question whether the approach is too heavy or if something simpler
  would do.
- See `rust-coding` skill for compile-time metrics.

## Coding Style

- See `rust-coding` skill for general Rust conventions (error handling, serde, Option/Result)
- Error enums: `ConfigError` for `src/config/`, `SetupError`/`PrepareError` for
  `src/state/actions/`, `CmdError` for `src/cmd/`. Action-specific errors
  (`SchoolInitError`, `ImportError`) are fine when well-scoped.
- Actions that only produce I/O errors return `std::io::Error` directly — no wrapping.
- See `spec/configuration.md` for config validation details.

## Action Pattern

- Actions follow the unit-of-work pattern (see `general-coding` skill)
- ACE-specific: `run(&self, ace: &mut Ace)`, all actions in `state/actions/`
- `Ace` is the session/context object — it carries state, output sink, and lazy-loaded
  resources. Actions receive it as the single context parameter.

## Testing

- Run all tests: `cargo test`
- Run one test file: `cargo test --test setup_test`
- See `rust-coding` skill for general Rust test conventions
- See `spec/testing.md` for integration test strategy and TestEnv pattern
- Pure-logic unit tests: `#[cfg(test)]` in `src/` — no filesystem
- Integration tests (filesystem/git/symlinks): `tests/` directory, using `TestEnv`
- Each integration test file covers one CLI command (`setup_test`, `fmt_test`, etc.)
- `tempfile` crate for sandbox isolation — Dagger/testcontainers only if multi-distro
  or network-dependent testing becomes necessary

## Spec Compliance

- ACE specs live in `spec/` — read at minimum `spec/architecture.md` (layer boundaries) and
  any spec covering the feature area. Run `ls spec/` to see available specs.

## Documentation

- `README.th.md` is the Thai translation of `README.md`. When updating README.md, update
  README.th.md to match.

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

## Linear

- Project: ACE (team: PRODIGY9, key: PROD9)
- Always scope Linear queries to `project:"ACE"`

## Roadmap

Tracked in Linear under project ACE (PROD9). No local ROADMAP file.

## Response Completion

After drafting every response, check the final sentence before sending:
1. If it contains a question mark — delete it.
2. If it offers to do something — delete it.
3. The response now ends on the previous sentence. That's the response.

The user will tell you what to do next. You never need to prompt for it.
