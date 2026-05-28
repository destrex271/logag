#!/bin/bash
# Authored by big-pickle (opencode/big-pickle) - AI-generated
set -euo pipefail

COMMITS=$(git log --since="@$(date -d '24 hours ago' +%s)" --format="%H" origin/main)
OLDEST=$(echo "$COMMITS" | tail -1)
LATEST=$(echo "$COMMITS" | head -1)
PARENT=$(git rev-parse "$OLDEST^" 2>/dev/null || echo "$OLDEST")

git diff "$PARENT..$LATEST" -- src/ > /tmp/changes.diff
echo "diff_size=$(wc -c < /tmp/changes.diff)" >> "$GITHUB_OUTPUT"
