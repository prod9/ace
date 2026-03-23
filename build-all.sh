#!/usr/bin/env bash
set -euo pipefail

# Cross-build ACE for all release targets using cargo-zigbuild.
#
# Runs on macOS or Linux:
#   macOS  — builds all targets (darwin + linux)
#   Linux  — builds linux targets only (darwin targets require macOS runners)
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

# --- Filter targets by host OS ----------------------------------------------

build_targets=()
skipped=()

for target in "${TARGETS[@]}"; do
  case "$target" in
    *-apple-darwin)
      if [ "$HOST_OS" = "Darwin" ]; then
        build_targets+=("$target")
      else
        skipped+=("$target")
      fi
      ;;
    *)
      build_targets+=("$target")
      ;;
  esac
done

if [ ${#skipped[@]} -gt 0 ]; then
  echo "Skipping (need macOS runner): ${skipped[*]}"
fi

if [ ${#build_targets[@]} -eq 0 ]; then
  echo "Error: no buildable targets for this platform."
  exit 1
fi

# --- Preflight checks -------------------------------------------------------

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

# Ensure SDKROOT is set for framework linking (macOS only).
if [ "$HOST_OS" = "Darwin" ]; then
  if [ -z "${SDKROOT:-}" ]; then
    SDKROOT="$(xcrun --show-sdk-path 2>/dev/null || true)"
    if [ -z "$SDKROOT" ]; then
      echo "Error: SDKROOT not set and xcrun failed."
      exit 1
    fi
    export SDKROOT
  fi
fi

# Ensure all required Rust targets are installed.
for target in "${build_targets[@]}"; do
  if ! rustup target list --installed | grep -qx "$target"; then
    echo "Installing Rust target: $target"
    rustup target add "$target"
  fi
done

# --- Build targets -----------------------------------------------------------

failed=()

for target in "${build_targets[@]}"; do
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
