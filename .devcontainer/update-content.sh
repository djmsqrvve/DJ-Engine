#!/usr/bin/env bash

set -euo pipefail

export RUSTC_WRAPPER=
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-/tmp/dj_engine_bevy18_codespace}"

mkdir -p "$CARGO_TARGET_DIR"

cargo fetch

# Keep provisioning fast enough that the codespace becomes reachable.
# Full runtime warmup remains available via .devcontainer/warm-runtime.sh.
cargo check -p dj_engine --no-default-features
