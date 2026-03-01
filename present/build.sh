#!/usr/bin/env bash
set -euo pipefail

if ! command -v typst &>/dev/null; then
  echo "typst not found. Install: brew install typst" >&2
  exit 1
fi

cd "$(dirname "$0")"
typst compile slides.typ slides.pdf "$@"
