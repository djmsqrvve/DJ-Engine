#!/usr/bin/env bash

set -euo pipefail

rustup show

# plugins/helix_data depends on a separate repo (helix-data) via relative
# path that doesn't exist in Codespaces. Create a stub so cargo can resolve
# the workspace, then fetch and check only what we need.
HELIX_STUB="../../../../helix/helix_3d_render_prototype/crates/helix-data"
if [ ! -d "$HELIX_STUB" ]; then
    mkdir -p "$HELIX_STUB/src"
    cat > "$HELIX_STUB/Cargo.toml" <<'TOML'
[package]
name = "helix-data"
version = "0.1.0"
edition = "2021"
TOML
    echo "// stub for codespace" > "$HELIX_STUB/src/lib.rs"
    echo "Created helix-data stub for workspace resolution"
fi

cargo fetch
