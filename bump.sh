#!/usr/bin/env bash
set -euo pipefail

VERSION="${1:?Usage: ./bump.sh <version>}"
VERSION="${VERSION#v}"

if ! cargo set-version --help &>/dev/null; then
  echo "Error: cargo set-version not found."
  echo ""
  echo "  cargo install cargo-edit"
  echo ""
  exit 1
fi

cargo set-version "$VERSION"
git commit -am "Bump version to $VERSION"
git tag "v$VERSION"

echo "==> Tagged v$VERSION — run ./release.sh to publish"
