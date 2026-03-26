# Testing

## Decision

Use `tempfile` crate for integration test isolation. Dagger/testcontainers deferred — only
revisit if multi-distro or network-dependent testing becomes necessary.

Rationale: ACE's integration surface is filesystem + git + symlinks. A sandboxed temp directory
covers this without container overhead.

## Test Categories

### Unit tests

- Live in `src/` alongside the code they test (`#[cfg(test)]`)
- Pure logic only — no filesystem, no git, no network
- Follow conventions in the `rust-coding` skill

### Integration tests

- Live in `tests/` directory
- Exercise filesystem, git, symlinks, and cross-module interactions
- Each test gets its own `TestEnv` — no shared state, no `#[serial]`

## TestEnv

Sandboxed filesystem root backed by `tempfile::TempDir`. RAII cleanup on drop.

See `tests/common/mod.rs` for the full API. Key design points:

- **Sandbox isolation**: `ace()` returns a `Command` with `env_clear()`, `HOME`/`XDG_CONFIG_HOME`/`XDG_CACHE_HOME` pointed at sandbox subdirs. Tests never touch real user files.
- **Escape prevention**: all path methods go through `path()`, which panics on absolute paths.
- **Remote school fixture**: `setup_remote_school()` creates a bare origin repo, cache clone, index entry, and ace.toml — everything needed to test Update/Pull without network access.

## File Layout

```
tests/
  common/
    mod.rs              # TestEnv, RemoteSchool, FlaudeRecord, helpers
  <command>_test.rs     # one file per CLI command / action area
```

## Conventions

- One `TestEnv` per test function — no sharing between tests
- `unwrap()` / `expect()` are fine in test code
- Test file names: `tests/*_test.rs`
- Helper functions go in `tests/common/mod.rs`
