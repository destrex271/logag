#!/bin/bash
# Authored by big-pickle (opencode/big-pickle) - AI-generated
set -euo pipefail

SKILL=$(cat live_doc_skill.md | jq -Rs .)
DIFF=$(cat /tmp/changes.diff | jq -Rs .)

PAYLOAD=$(jq -n \
  --argjson skill "$SKILL" \
  --argjson diff "$DIFF" \
  '{
    "system_instruction": { "parts": [{ "text": $skill }] },
    "contents": [{
      "parts": [{
        "text": "Analyze these code changes from the past 24 hours and update documentation accordingly.\n\n```\n\($diff)\n```"
      }]
    }],
    "generationConfig": {
      "temperature": 0.2,
      "maxOutputTokens": 8192
    }
  }')

echo "$PAYLOAD" > /tmp/gemini_payload.json

curl -s -X POST \
  "https://generativelanguage.googleapis.com/v1beta/models/gemini-flash-latest:generateContent?key=${GEMINI_API_KEY}" \
  -H "Content-Type: application/json" \
  -d @/tmp/gemini_payload.json > /tmp/gemini_response.json

if ! jq -e '.candidates[0].content.parts[0].text' /tmp/gemini_response.json > /dev/null 2>&1; then
  echo "Gemini API error:"
  jq '.' /tmp/gemini_response.json
  exit 1
fi

jq -r '.candidates[0].content.parts[0].text' /tmp/gemini_response.json > /tmp/generated_docs.md
echo "Documentation generated successfully"
