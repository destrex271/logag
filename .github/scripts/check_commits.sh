#!/bin/bash
# Authored by big-pickle (opencode/big-pickle) - AI-generated
set -euo pipefail

SINCE=$(date -d '24 hours ago' +%s)
COMMITS=$(git log --since="@$SINCE" --format="%H" origin/main 2>/dev/null || true)

if [ -z "$COMMITS" ]; then
  echo "No commits in the last 24 hours"
  echo "has_changes=false" >> "$GITHUB_OUTPUT"
  exit 0
fi

echo "has_changes=true" >> "$GITHUB_OUTPUT"
{
  echo "commits<<EOF"
  echo "$COMMITS"
  echo "EOF"
} >> "$GITHUB_OUTPUT"
