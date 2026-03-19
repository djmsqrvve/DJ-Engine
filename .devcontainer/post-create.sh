#!/usr/bin/env bash

set -euo pipefail

cat <<'EOF'
DJ Engine Codespace ready (build-only, no GUI).

Build targets:
  make dev-exe      Windows .exe (cross-compile, release, stripped)
  make linux-exe    Linux binary (release, stripped)
  make check        Type-check workspace
  make test         Run all tests
  make validate     Full quality gate (fmt + lint + test + contracts)

Optional full warmup (~15 min):
  bash .devcontainer/warm-runtime.sh

GUI/browser viewing is prototype-level — run built binaries locally.
To use the desktop/VNC Codespace instead, select "DJ Engine (desktop)" config.
EOF
