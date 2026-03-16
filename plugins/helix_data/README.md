# DJ Engine Helix Data Plugin

`dj_engine_helix` is the workspace plugin crate that bridges `helix_standardization`
data into DJ Engine's registry-driven custom-document flow without putting Helix
semantics into engine core.

## What It Adds

- `HelixDataPlugin` registers all 22 Helix document kinds with `EditorDocumentRoute::Table`
- `HelixRegistries` — typed `Registry<T>` for all 22 entity types from TOML
- `HelixDocumentIndex` — generic lookup by kind + id across all 22 kinds
- `HelixDatabase` — wrapper around engine `Database` populated from registries at startup
- `BalanceOverlays` — per-entity numeric balance tuning via TOML files
- `helix_import` for turning a Helix `dist/` tree into a mounted DJ Engine project
- `helix_dashboard` for contract validation from the CLI
- `helix_editor` and `helix_runtime_preview` wrapper binaries

## Two Pipelines

**Typed TOML (primary):**
```bash
make helix-import-toml HELIX3D=~/dev/helix/helix_standardization/dist/helix3d
make helix-dashboard HELIX3D=~/dev/helix/helix_standardization/dist/helix3d
make helix-editor HELIX_DIST=~/dev/helix/helix_standardization/dist PROJECT=/tmp/helix_project
```

**Legacy JSON (still supported):**
```bash
make helix-import HELIX_DIST=~/dev/helix/helix_standardization/dist PROJECT=/tmp/helix_project
make helix-editor PROJECT=/tmp/helix_project
make helix-preview PROJECT=/tmp/helix_project
```

## Data Flow

1. TOML registries load at startup → `HelixRegistries` resource
2. Bridge converts to engine `Database` → `HelixDatabase` resource
3. Sync system wraps TOML entities into `CustomDocument` envelopes → `LoadedCustomDocuments`
4. Editor table view shows all 22 kinds, filterable and sortable
5. Dashboard validation checks cross-refs, localization, TOML coverage
6. Balance overlays apply per-entity numeric tuning during bridge conversion
