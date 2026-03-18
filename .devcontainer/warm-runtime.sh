#!/usr/bin/env bash

set -euo pipefail

export RUSTC_WRAPPER=
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-/home/vscode/.cache/dj-engine/cargo-target/dj_engine_bevy18}"

mkdir -p "$CARGO_TARGET_DIR"

echo "Building release exe (this will take ~15 minutes)..."
make dev-exe
