#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

if ! git -C "$ROOT" rev-parse --is-inside-work-tree >/dev/null 2>&1; then
  echo "Error: not inside a git repository." >&2
  exit 1
fi

if [ "$(git -C "$ROOT" rev-parse --abbrev-ref HEAD)" != "main" ]; then
  echo "Error: publish from main." >&2
  exit 1
fi

if ! git -C "$ROOT" diff --quiet -- www/dist || ! git -C "$ROOT" diff --cached --quiet -- www/dist; then
  echo "Error: www/dist has uncommitted changes. Commit them on main first." >&2
  exit 1
fi

git -C "$ROOT" subtree push --prefix=www/dist --rejoin gh gh-pages
