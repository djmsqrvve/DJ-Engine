#!/usr/bin/env bash

set -euo pipefail

export RUSTC_WRAPPER=
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-/tmp/dj_engine_bevy18_codespace}"

mkdir -p "$CARGO_TARGET_DIR"

# Build the runnable workspace artifacts after the codespace is already usable.
cargo build --workspace
