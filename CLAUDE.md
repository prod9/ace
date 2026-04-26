If you do not see any ACE context in this conversation, you MUST tell the user to start their
session through the `ace` command instead of running the backend directly.

# ACE Project

**ACE** (Accelerated Coding Environment) - Automation tooling for setting up and keeping AI coding
environments setup and up-to-date. Acts as entrypoint to Claude Code or Codex.

Core functions:
- Check environment readiness
- Install/update skills, agents, and conventions
- Configure chatbots to connect to LiteLLM.
- Manage model credentials.

## Coding Style

- **`simplify` skill**: Must load and adhere to all coding skill principles (`general-coding`,
  `rust-coding`, etc.) before proposing changes. Simplification that violates a coding
  principle is a regression, not an improvement.
- See `rust-coding` skill for general Rust conventions (error handling, serde, Option/Result)
- Error enums: `ConfigError` for `src/config/`, `SetupError`/`PrepareError` and other
  action-scoped errors for `src/actions/`, `CmdError` for `src/cmd/`. Action-specific
  errors (`InitError`, `AddImportError`, `PullImportsError`) are fine when well-scoped.
- Actions that only produce I/O errors return `std::io::Error` directly — no wrapping.
- See `spec/configuration.md` for config validation details.

## Action Pattern

- Actions follow the unit-of-work pattern (see `general-coding` skill)
- ACE-specific: `run(&self, ace: &mut Ace)`, all actions in `src/actions/`
- Grouped by user role: `actions/project/` (consumer-side, user in their repo) vs
  `actions/school/` (maintainer-side, user in a school repo). See
  `spec/decisions/005-action-layout.md`.
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

- **Flaude is a test-only backend.** It exists so integration tests can exercise
  the Claude-family code paths without hitting a real binary. Do not mention
  Flaude on the website (`www/`), in user-facing help text, or in public docs.
  It's fine to reference in specs, code comments, and CLAUDE.md itself.

## CLI Conventions

- `ace paths` uses tab-separated `key\tvalue` for machine parseability
- Paths printed regardless of whether they exist on disk
- Command help text lives in clap doc comments / attributes, not in PRDs
- When modifying commands, ensure `--help` text stays aligned with code behavior

## Backcompat Policy

ACE has real users. Treat CLI verbs, subcommand names, config keys (in `ace.toml`,
`school.toml`, `ace.local.toml`), and storage paths as public contracts.

- **Renames** — add the new name, keep the old one as an alias. Use clap's
  `#[command(visible_alias = "...")]` for subcommands; add deprecation hints in
  help text where useful. Do not remove the old name in a minor/patch release.
- **Removals** — require a major version bump and a clear release-note callout.
- **Internal renames** (struct names, error-enum variants, module paths) have no
  backcompat obligation — they're not part of the contract.
- **Storage migrations** — prefer detect-and-hint (see `warn_stray_cache_dirs`
  in `src/main.rs`) over silent auto-migration. Users should know what changed.

When in doubt, add the alias and move on. Breaking a user's `ace ...` command
or their checked-in `ace.toml` is not acceptable without explicit deprecation.

## TUI Pattern

- `term_ui` module: `Tui::new(&mut Ace)` + `tui.run(Workflow::...)` — no trait, no dynamic dispatch
- `Workflow` enum dispatches to methods that use `inquire` for inline prompts (Text, Select)
- Standalone `term_ui::select()` for cases where the caller runs the action (e.g. async actions)
- Session has no UI field — interactive input is handled entirely by term_ui

## Storage Layout

- School clones: `~/.local/share/ace/{owner/repo}/` (XDG_DATA_HOME) — schools are
  user *data*, not cache. `UpdateOutcome::Dirty` / `AheadOfOrigin` states can carry
  in-progress work; losing it to OS cache hygiene would be a foot-gun.
- Import source cache: `~/.cache/ace/imports/{owner/repo}/` (XDG_CACHE_HOME) —
  read-only upstream snapshots used by `ace import` and `ace school update`.
  Safe to sweep; next invocation re-clones.
- Index: `~/.local/share/ace/index.toml` — tracks downloaded schools (specifier, repo,
  path). Sits alongside the school clones (user data, not cache). Legacy location
  `~/.cache/ace/index.toml` is one-shot migrated via `index_toml::load_or_migrate`;
  legacy file is left on disk and surfaced by `warn_stray_cache_dirs` for manual cleanup.
- Startup hint: bare `ace` invocation warns once if the pre-PROD9-76 flat layout
  (`~/.cache/ace/{owner/repo}/`) has stray entries. No auto-migration.
- Git operations use `std::process::Command`, no sqlite or git crate.

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
