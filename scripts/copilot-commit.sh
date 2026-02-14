#!/usr/bin/env bash
set -euo pipefail
# Use the Copilot identity for a single commit.
# Usage: ./scripts/copilot-commit.sh -m "message" (or pass any git commit args)
export GIT_AUTHOR_NAME="Copilot CLI"
export GIT_AUTHOR_EMAIL="copilot@local"
export GIT_COMMITTER_NAME="Copilot CLI"
export GIT_COMMITTER_EMAIL="copilot@local"
exec git commit "$@"
