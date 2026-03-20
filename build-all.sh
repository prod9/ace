#!/usr/bin/env bash
set -euo pipefail

# Cross-build ACE for all release targets using cargo-zigbuild.
#
# Must be run from macOS (Apple SDK required for darwin targets).
#
# Prerequisites:
#   cargo install cargo-zigbuild
#   zig (v0.12+) — https://ziglang.org/download/

TARGETS=(
  aarch64-apple-darwin
  x86_64-apple-darwin
  aarch64-unknown-linux-gnu
  x86_64-unknown-linux-gnu
  aarch64-unknown-linux-musl
  x86_64-unknown-linux-musl
)

OUTDIR="${1:-target/dist}"
mkdir -p "$OUTDIR"

HOST_TARGET="$(rustc -vV | awk '/^host:/ { print $2 }')"
HOST_OS="$(uname -s)"

# --- Preflight checks -------------------------------------------------------

if [ "$HOST_OS" != "Darwin" ]; then
  echo "Error: build-all.sh must be run from macOS (need Apple SDK for darwin targets)."
  exit 1
fi

if ! command -v cargo-zigbuild &>/dev/null; then
  echo "Error: cargo-zigbuild not found."
  echo ""
  echo "  cargo install cargo-zigbuild"
  echo ""
  exit 1
fi

if ! command -v zig &>/dev/null; then
  echo "Error: zig not found."
  echo ""
  echo "  See https://ziglang.org/download/"
  echo ""
  exit 1
fi

# Ensure SDKROOT is set for framework linking.
if [ -z "${SDKROOT:-}" ]; then
  SDKROOT="$(xcrun --show-sdk-path 2>/dev/null || true)"
  if [ -z "$SDKROOT" ]; then
    echo "Error: SDKROOT not set and xcrun failed."
    exit 1
  fi
  export SDKROOT
fi

# Ensure all required Rust targets are installed.
for target in "${TARGETS[@]}"; do
  if ! rustup target list --installed | grep -qx "$target"; then
    echo "Installing Rust target: $target"
    rustup target add "$target"
  fi
done

# --- Build targets -----------------------------------------------------------

failed=()

for target in "${TARGETS[@]}"; do
  echo "==> $target"

  if [ "$target" = "$HOST_TARGET" ]; then
    cmd=(cargo build --release --target "$target")
  else
    cmd=(cargo zigbuild --release --target "$target")
  fi

  if "${cmd[@]}"; then
    cp "target/$target/release/ace" "$OUTDIR/ace-$target"
    echo "    -> $OUTDIR/ace-$target"
  else
    echo "    !! FAILED"
    failed+=("$target")
  fi
done

echo ""
echo "--- Results ---"
ls -lh "$OUTDIR"/ace-* 2>/dev/null || echo "(no binaries produced)"

if [ ${#failed[@]} -gt 0 ]; then
  echo ""
  echo "Failed targets:"
  printf "  %s\n" "${failed[@]}"
  exit 1
fi
