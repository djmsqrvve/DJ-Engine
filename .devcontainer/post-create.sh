#!/usr/bin/env bash

set -euo pipefail

cat <<'EOF'
Codespaces desktop is available on forwarded port 6080 (password: vscode).
Runtime smoke commands:
  ./dj e --test-mode
  timeout 20s ./dj d
EOF
