#!/usr/bin/env bash
set -euo pipefail

# Cross-build ACE for linux/mac × arm64/amd64
# macOS targets: built with cargo directly
# Linux targets: built with `cross` (requires Docker)

TARGETS_MAC=(
  aarch64-apple-darwin
  x86_64-apple-darwin
)

TARGETS_LINUX=(
  aarch64-unknown-linux-gnu
  x86_64-unknown-linux-gnu
)

OUTDIR="${1:-target/dist}"
TOOLCHAIN="${CROSS_TOOLCHAIN:-stable}"

mkdir -p "$OUTDIR"

failed=()

build_mac() {
  local target="$1"
  echo "==> $target (cargo)"
  if cargo +"$TOOLCHAIN" build --release --target "$target"; then
    cp "target/$target/release/ace" "$OUTDIR/ace-$target"
    echo "    -> $OUTDIR/ace-$target"
  else
    echo "    !! FAILED"
    failed+=("$target")
  fi
}

build_linux() {
  local target="$1"
  echo "==> $target (cross)"
  if RUSTUP_TOOLCHAIN="$TOOLCHAIN" cross build --release --target "$target"; then
    cp "target/$target/release/ace" "$OUTDIR/ace-$target"
    echo "    -> $OUTDIR/ace-$target"
  else
    echo "    !! FAILED"
    failed+=("$target")
  fi
}

# macOS targets — always attempt
for t in "${TARGETS_MAC[@]}"; do
  build_mac "$t"
done

# Linux targets — require Docker
if docker info >/dev/null 2>&1; then
  for t in "${TARGETS_LINUX[@]}"; do
    build_linux "$t"
  done
else
  echo ""
  echo "!! Docker not running — skipping Linux targets."
  echo "   Start Docker Desktop and re-run to build:"
  for t in "${TARGETS_LINUX[@]}"; do
    echo "     $t"
    failed+=("$t")
  done
fi

echo ""
echo "--- Results ---"
ls -lh "$OUTDIR"/ace-* 2>/dev/null || echo "(no binaries produced)"

if [ ${#failed[@]} -gt 0 ]; then
  echo ""
  echo "Failed targets:"
  printf "  %s\n" "${failed[@]}"
  exit 1
fi
