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

### API

| Method                                 | Description                                                                      |
|----------------------------------------|----------------------------------------------------------------------------------|
| `new()`                                | Create a new temp directory                                                      |
| `root()` → `&Path`                    | Return the sandbox root path                                                     |
| `path(rel)` → `PathBuf`               | Resolve relative path under root. **Panics on absolute paths** (escape prevention) |
| `write_file(rel, contents)`            | Create file with contents, creating parent dirs as needed                        |
| `read_file(rel)` → `String`           | Read file contents                                                               |
| `mkdir(rel)`                           | Create directory (and parents)                                                   |
| `symlink(target, link)`               | Create symlink. Both paths relative to root                                      |
| `git_init()`                           | Run `git init` in the sandbox root                                               |
| `assert_exists(rel)`                   | Assert path exists                                                               |
| `assert_not_exists(rel)`               | Assert path does not exist                                                       |
| `assert_symlink(link, expected_target)` | Assert symlink points to expected target                                         |
| `assert_contains(rel, needle)`         | Assert file contains string                                                      |

### Escape Prevention

All methods resolve paths through `path()`, which panics on absolute paths. This ensures
tests cannot accidentally read/write outside the sandbox.

## File Layout

```
tests/
  common/
    mod.rs          # TestEnv struct + helpers
  link_test.rs      # symlink/link action tests
  sync_test.rs      # skill sync tests
  ...
```

## Migration Path

1. Move `TestFixture` from `src/state/actions/link.rs` → `TestEnv` in `tests/common/mod.rs`
2. Migrate existing tests one commit at a time
3. Remove inline `TestFixture` once all tests moved

## Conventions

- One `TestEnv` per test function — no sharing between tests
- `unwrap()` / `expect()` are fine in test code
- Test file names: `tests/*_test.rs`
- Helper functions go in `tests/common/mod.rs`
