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
- `helix_dashboard` for contract validation from the CLI (boxed output with API health checks)
- `helix_editor` and `helix_runtime_preview` wrapper binaries
- API health checks (`api_health.rs`) — opt-in remote validation against standardization API (port 6800)

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
7. API health checks validate against standardization API (opt-in, 2s timeout)

## API Integration (opt-in)

When the standardization API is running on port 6800, the dashboard performs three
additional checks:

- **API Health** (`GET /health`) — confirms API is reachable, reports remote entity count and data age
- **Data Freshness** (local mtime vs remote) — warns if local TOML files are >60 min old
- **Remote Validation** (`POST /validate`) — validates a sample entity against API contract schemas

All checks use `curl` with a 2-second timeout. If the API is offline, the dashboard
reports Info-level status and continues — it never blocks or fails due to API unavailability.

Balance overlays can be serialized to JSON via `api_health::serialize_overlays_for_api()`
for future upload to a `POST /balance` endpoint.
