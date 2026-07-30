#!/usr/bin/env bash
set -euo pipefail

if ! command -v codex >/dev/null 2>&1; then
  echo "Codex CLI is not installed or is not available in PATH." >&2
  exit 1
fi

if [[ ! -f "CODEX_PROMPT.md" ]]; then
  echo "Run this script from the Hyper Get repository root." >&2
  exit 1
fi

codex exec --sandbox workspace-write "$(cat CODEX_PROMPT.md)"
