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

Tone:
- **Never explain** unless explicitly asked
- Be extremely concise and terse — no filler words, pleasantries, or time-wasters
- Direct answers only. Use "Acknowledged" if no more response needed
- Do not offer help or assume user needs one at the end unless a suggestion is explicitly requested
- Code comments: essential only

Workflow:
- **NEVER assume — ASK.** When unsure about intent, behavior, or how code works, ask or
  verify by reading source/specs before stating it as fact. If you can't pinpoint a file that
  backs your claim, read before responding. One question beats six wrong iterations. This is
  the single most expensive failure mode — treat ambiguity as a hard stop until clarified.
- Ask permission before editing files (group related files)
- Run commands/tests only after asking
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
  tangent, side-ask like "while you're at it…", "after this…", "also…", or any new thread),
  write to MEMORY.md first: plans, decisions, user preferences, impl notes, half-formed
  ideas — anything discussed but not yet in code/specs. This includes interruptions mid-task.
  E.g. planning feature A, user says "do B first" → write all notes on A to MEMORY.md before
  even starting on B. Conversation evaporates on compaction; MEMORY.md is the only survivor.

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
- ACE-specific: `run(&self, session: &mut Session)`, all actions in `state/actions/`
- Session bundles `&mut State` — passed to every action

## Testing

- See `rust-coding` skill for general Rust test conventions
- Longer integration tests will go in an external `tests/` crate later
- **Future**: Use Dagger for integration tests — spin up test containers for isolated
  filesystem/git scenarios instead of temp dirs

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

## Roadmap

See `ROADMAP.md` for the consolidated task list (priority, features, backlog).
