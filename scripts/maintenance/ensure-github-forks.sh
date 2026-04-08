#!/usr/bin/env bash
# Fork nasa/cFS, nasa/ci_lab, nasa/cFE into your GitHub org or account (default: GokServis).
# Requires: GITHUB_TOKEN or GH_TOKEN with repo scope.
# Usage: export GITHUB_TOKEN=ghp_... && ./scripts/maintenance/ensure-github-forks.sh

set -euo pipefail

TOKEN="${GITHUB_TOKEN:-${GH_TOKEN:-}}"
OWNER="${GITHUB_FORK_OWNER:-GokServis}"

if [[ -z "${TOKEN}" ]]; then
  echo "Set GITHUB_TOKEN (or GH_TOKEN) to create forks via the GitHub API." >&2
  exit 1
fi

fork_repo() {
  local upstream_repo="$1"
  local url="https://api.github.com/repos/${upstream_repo}/forks"
  echo "Forking ${upstream_repo} -> ${OWNER}..."
  curl -sf -X POST \
    -H "Authorization: Bearer ${TOKEN}" \
    -H "Accept: application/vnd.github+json" \
    -H "X-GitHub-Api-Version: 2022-11-28" \
    "${url}" \
    -d "{}"
  echo
}

fork_repo "nasa/cFS"
fork_repo "nasa/ci_lab"
fork_repo "nasa/cFE"

echo "Done. Verify: https://github.com/${OWNER}/cFS"
