# Cross-Build Notes

Target matrix: linux/mac × arm64/amd64

| Target | Status | Notes |
|---|---|---|
| `aarch64-apple-darwin` | ✅ | Native host, works with `cargo build` |
| `x86_64-apple-darwin` | ✅ | Works with `cargo +nightly build --target x86_64-apple-darwin` |
| `x86_64-unknown-linux-gnu` | ❌ | Blocked (see below) |
| `aarch64-unknown-linux-gnu` | ❌ | Blocked (see below) |

## Blockers for Linux Targets

1. **`native-tls` requires OpenSSL** — `ureq` is configured with `native-tls`, which links
   against system OpenSSL. Cross-compiling from macOS→Linux needs Linux OpenSSL headers/libs
   which aren't available on the host.

2. **`cross` tool is broken on macOS→Linux with nightly** — `cross` (v0.2.5) tries to install
   a Linux-hosted toolchain locally (`nightly-x86_64-unknown-linux-gnu`) instead of using
   Docker. This fails because that toolchain can't run on macOS. Docker was confirmed running;
   the bug is in `cross`'s toolchain resolution.

3. **`cargo-zigbuild` needs zig runtime** — `cargo-zigbuild` can cross-compile without Docker
   by using Zig's bundled libc, but requires `zig` to be installed on the host.

4. **Edition 2024 requires nightly** — the script must use `+nightly`, not `+stable`.

## Options to Unblock

Pick one:

- **Switch `ureq` to `rustls`** — pure Rust TLS, no system deps. Simplest fix. Then plain
  `cargo build --target x86_64-unknown-linux-gnu` should work from macOS.
- **Install `zig` and use `cargo-zigbuild`** — handles cross-sysroot automatically.
  `cargo-zigbuild` is already installed. Just needs `zig` binary.
- **Use CI (GitHub Actions)** — build each target on its native runner. Avoids cross-compilation
  entirely. Most reliable long-term.
- **Fix `cross` or wait for update** — the toolchain detection bug may be fixed upstream.

## Script

`scripts/cross-build.sh` is ready. It builds macOS targets with `cargo` and Linux targets
with `cross` (when Docker is available). Once a Linux strategy is chosen, update the script.

## Installed Tooling

- `rustup` targets added (nightly): all 4 targets
- `cargo-zigbuild` v0.22.1 installed (needs `zig` binary)
- `cross` v0.2.5 installed (needs Docker + bug fix)
