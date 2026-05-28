#!/bin/bash
# Authored by big-pickle (opencode/big-pickle) - AI-generated
set -euo pipefail

mkdir -p docs

if [ ! -f /tmp/generated_docs.md ]; then
  echo "No generated docs found"
  exit 0
fi

python3 << 'PYEOF'
import os, re

with open('/tmp/generated_docs.md') as f:
    content = f.read()

current_file = None
lines = content.split('\n')
written = False

for i, line in enumerate(lines):
    m = re.match(r'^#+\s*FILE:\s*(.+\.md)$', line)
    if m:
        current_file = m.group(1).strip()
        doc_dir = os.path.join('docs', os.path.dirname(current_file))
        os.makedirs(doc_dir, exist_ok=True)
        file_content = []
        j = i + 1
        while j < len(lines) and not re.match(r'^#+\s*FILE:', lines[j]):
            if j < len(lines):
                file_content.append(lines[j])
            j += 1
        if file_content:
            filepath = os.path.join('docs', current_file)
            with open(filepath, 'w') as f:
                f.write('\n'.join(file_content).strip() + '\n')
            print(f'Written: {filepath}')
            written = True

if not written:
    os.makedirs('docs', exist_ok=True)
    with open('docs/overview.md', 'w') as f:
        f.write(content.strip() + '\n')
    print('Written: docs/overview.md')
PYEOF
