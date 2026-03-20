#!/usr/bin/env bash
set -euo pipefail

# Cross-build ACE for all release targets.
# Local-arch: plain cargo. Cross-arch: `cross` (requires Docker).

TARGETS=(
  aarch64-apple-darwin
  x86_64-apple-darwin
  aarch64-unknown-linux-gnu
  x86_64-unknown-linux-gnu
)

OUTDIR="${1:-target/dist}"
mkdir -p "$OUTDIR"

HOST_TARGET="$(rustc -vV | awk '/^host:/ { print $2 }')"

# Check for `cross` if any non-host targets exist.
needs_cross=false
for target in "${TARGETS[@]}"; do
  [ "$target" != "$HOST_TARGET" ] && needs_cross=true
done
if $needs_cross && ! command -v cross &>/dev/null; then
  echo "Error: cross not found (needed for non-host targets)."
  echo ""
  echo "  cargo install cross"
  echo ""
  exit 1
fi

failed=()

for target in "${TARGETS[@]}"; do
  echo "==> $target"

  if [ "$target" = "$HOST_TARGET" ]; then
    cmd=(cargo build --release --target "$target")
  else
    cmd=(cross build --release --target "$target")
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
