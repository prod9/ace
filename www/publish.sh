#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DIST_DIR="$ROOT/www/dist"

if ! git -C "$ROOT" rev-parse --is-inside-work-tree >/dev/null 2>&1; then
  echo "Error: not inside a git repository." >&2
  exit 1
fi

if ! git -C "$ROOT" diff --cached --quiet; then
  echo "Error: staged changes present. Commit or unstage them first." >&2
  exit 1
fi

if [ ! -d "$DIST_DIR" ]; then
  echo "Error: www/dist does not exist. Build the site first with 'cd www && bun run build'." >&2
  exit 1
fi

if ! find "$DIST_DIR" -mindepth 1 -print -quit >/dev/null 2>&1; then
  echo "Error: www/dist is empty. Build the site first with 'cd www && bun run build'." >&2
  exit 1
fi

cleanup() {
  git -C "$ROOT" reset --quiet HEAD -- www/dist >/dev/null 2>&1 || true
}

trap cleanup EXIT

git -C "$ROOT" add -A www/dist

TREE="$(git -C "$ROOT" write-tree)"
PARENT="$(git -C "$ROOT" rev-parse HEAD)"
TEMP_COMMIT="$(
  printf 'Publish site\n' | git -C "$ROOT" commit-tree "$TREE" -p "$PARENT"
)"
SPLIT_COMMIT="$(git -C "$ROOT" subtree split --prefix=www/dist "$TEMP_COMMIT")"

git -C "$ROOT" branch -f gh-pages "$SPLIT_COMMIT" >/dev/null
git -C "$ROOT" push -f gh "$SPLIT_COMMIT:refs/heads/gh-pages"

echo "Published www/dist to gh-pages: $SPLIT_COMMIT"
