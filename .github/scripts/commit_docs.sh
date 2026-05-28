#!/bin/bash
# Authored by big-pickle (opencode/big-pickle) - AI-generated
set -euo pipefail

if [ ! -d docs ] || [ -z "$(ls -A docs 2>/dev/null)" ]; then
  echo "No documentation generated"
  exit 0
fi

git config user.name "github-actions[bot]"
git config user.email "github-actions[bot]@users.noreply.github.com"
git add docs/

if git diff --cached --quiet; then
  echo "No documentation changes to commit"
else
  git commit -m "docs: auto-generate documentation from recent commits [skip ci]"
  git push
fi
