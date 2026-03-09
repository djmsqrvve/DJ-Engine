#!/usr/bin/env bash

set -euo pipefail

export RUSTC_WRAPPER=
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$HOME/.cache/dj-engine/cargo-target/dj_engine_bevy18}"

mkdir -p "$CARGO_TARGET_DIR"

cargo fetch --locked

# Warm the shared target dir with real binaries so Codespaces prebuilds
# reduce both compile-time validation and first editor/game launches.
cargo build --workspace
