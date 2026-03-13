# DJ Engine Helix Data Plugin

`dj_engine_helix` is the workspace plugin crate that bridges `helix_standardization`
data into DJ Engine’s registry-driven custom-document flow without putting Helix
semantics into engine core.

## What It Adds

- `HelixDataPlugin` for registering `helix_abilities`, `helix_items`, and
  `helix_mobs` with `EditorDocumentRoute::Table` for browsable table editing
- `HelixDocumentIndex` for runtime/editor lookup of loaded Helix documents
- `helix_import` for turning a Helix `dist/` tree into a mounted DJ Engine project
- `helix_editor` and `helix_runtime_preview` wrapper binaries for proving the flow

## Import Loop

```bash
make helix-import HELIX_DIST=~/dev/helix/helix_standardization/dist PROJECT=/tmp/helix_project
make helix-editor PROJECT=/tmp/helix_project
make helix-preview PROJECT=/tmp/helix_project
```

The importer preserves the raw Helix payloads inside DJ custom-document
envelopes, annotates them with source-bucket tags, and derives only safe
document references.
