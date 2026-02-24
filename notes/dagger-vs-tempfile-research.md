# Integration Testing Research: Dagger vs Filesystem Isolation

## Approach 1: Dagger Engine

**Crate**: `dagger-sdk` v0.19.11 (community-maintained by kjuulh, not official Dagger team)

**How it works**: Connects to a local Dagger Engine daemon (BuildKit-based) over GraphQL, spins
up containers, mounts directories, runs commands inside, asserts on output. Full container
isolation.

**The problem for ACE**: Its dependency tree is enormous. It pulls in `reqwest`, `tokio`, `hyper`,
`h2`, `graphql_client`, `derive_builder`, `futures`, `flate2`, `tar`, `sha2`, `tracing`,
`tracing-subscriber`, `eyre` — dozens of crates that ACE deliberately avoids today by using
`smol` + `ureq`. This would be a severe compile-time regression.

Other issues:
- Requires Docker socket (CI complexity)
- Cold start: 10-30 seconds to pull/start the engine daemon
- Per-test overhead: 1-5 seconds per container
- Async-only API, forces `tokio` runtime (ACE uses `smol`)
- Linux containers only — cannot test macOS-specific path/symlink behavior
- Single community maintainer

## Approach 2: Filesystem Isolation Crates

| Crate | Good for ACE? | Why/why not |
|---|---|---|
| **`tempfile`** (already in tree) | **Yes** | Zero new deps, real FS/git/symlinks/process exec, <1ms per test |
| **`fstest`** | Maybe | Nice proc-macro sugar over tempfile+git, but thin wrapper, low adoption, needs `serial_test` |
| **`cap-std`** | Not ideal | True sandboxing but requires refactoring all FS code to use `Dir` type instead of bare `Path` |
| **`vfs`** | No | In-memory FS cannot run git commands, no symlink support, requires full abstraction layer |

## Comparison

| | Dagger | tempfile + helpers |
|---|---|---|
| Compile time impact | **Severe** (reqwest+tokio+graphql) | **Zero** (already in deps) |
| Test speed | 1-5s per test | <1ms per test |
| Git/symlink/exec | Yes (in container) | Yes (native) |
| macOS testing | No | Yes |
| Setup needed | Docker daemon | None |
| New dependencies | ~40+ transitive crates | 0 |

## Recommendation

**Use `tempfile` + a ~30-line `TestEnv` helper struct.** It's already in the dependency tree,
tests real code paths natively (including macOS), runs in microseconds, and adds zero
compile-time cost. The suggested `TestEnv` pattern provides `.with_git()`, `.write_file()`, and
`.symlink()` helpers with RAII cleanup.

Dagger (or more likely `testcontainers`) becomes relevant only if ACE later needs to test
against multiple Linux distros or network-dependent flows like LiteLLM proxy auth. Even then,
`testcontainers` would be preferable over `dagger-sdk` for lighter dependencies.

## Sources

- [dagger-sdk on crates.io](https://crates.io/crates/dagger-sdk)
- [dagger-sdk on GitHub](https://github.com/kjuulh/dagger-sdk)
- [cap-std (Bytecode Alliance)](https://github.com/bytecodealliance/cap-std)
- [vfs crate](https://crates.io/crates/vfs)
- [tempfile crate](https://crates.io/crates/tempfile)
- [fstest crate](https://crates.io/crates/fstest)
- [testcontainers-rs](https://github.com/testcontainers/testcontainers-rs)
- [Dagger.io](https://dagger.io/)
