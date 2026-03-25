# DJ Engine Architecture

This document describes the current high-level architecture of DJ Engine after
the engine/editor/runtime decoupling work.

## Overview

DJ Engine is a Bevy-based engine crate with three main entry surfaces:

- the editor shell (`dj_engine`)
- the engine-owned runtime preview (`runtime_preview`)
- optional game crates such as `doomexe`

At a high level:

```text
Mounted Project / Sample Game
        |
        v
+-----------------------------+
|         DJ Engine           |
| +-------------------------+ |
| | Editor Shell           | |
| | Runtime Preview        | |
| | Data + Custom Docs     | |
| | Story Graph            | |
| | Rendering / Audio      | |
| | Input / Collision      | |
| | Scripting / Save       | |
| +-------------------------+ |
+-----------------------------+
        |
        v
       Bevy
```

## Core Principles

### Engine-first, project-agnostic boundaries

The engine owns reusable structure and tooling. Sample-game logic stays outside
engine core.

- Engine owns project manifests, scene/story loading, runtime preview, custom
  document discovery, editor routing, and generic validation.
- Games own semantics such as combat rules, HUD logic, enemy behavior,
  progression rules, and game-specific scripting extensions.

### Data-driven project model

Mounted projects are rooted at `project.json` and can include:

- `scenes/`
- `story_graphs/`
- `assets/`
- `data/registry.json` plus custom document folders

The engine treats `data/registry.json` as the entrypoint for custom authored
documents that live beside scenes and story graphs.

### Separate serialized data from live ECS state

The editor, runtime preview, and save flows all rely on a data layer separate
from live runtime components. That keeps authored JSON, validation, and ECS
runtime state from collapsing into one type surface.

## Main Execution Paths

### Editor shell

- Binary: `engine/src/main.rs`
- Primary command: `make dev`
- Responsibilities:
  - mount projects from `project.json`
  - edit scenes and story graphs
  - browse and edit custom documents
  - track dirty state
  - hand off `Run Project` to the separate runtime preview process

The old idea of the editor becoming full runtime state has been replaced by a
separate handoff model. The editor stays an authoring tool.

### Runtime preview

- Binary: `engine/src/bin/runtime_preview.rs`
- Primary command: `make preview PROJECT=/path/to/project`
- Responsibilities:
  - mount a project
  - load startup scene/story data
  - load custom documents and preview profiles
  - run the generic `Title -> Dialogue -> Overworld` preview loop
  - support project-scoped continue checkpoints

### Sample game

- Binary: `games/dev/doomexe/src/main.rs`
- Primary command: `make game`
- Role:
  - exercise engine features as a sample consumer
  - remain separate from engine core semantics

## Data Architecture

### Built-in project data

Key engine-owned data modules:

- `engine/src/data/project.rs`
  - project manifest, project-relative paths, startup defaults
- `engine/src/data/scene.rs`
  - scene composition and entity data
- `engine/src/data/story/`
  - serialized story graph data
- `engine/src/data/spawner.rs`
  - scene-to-runtime spawn bridge
- `engine/src/data/loader.rs`
  - load/save helpers and data errors

### Custom document platform

Custom game data lives in `engine/src/data/custom.rs` and adds:

- `CustomDataManifest`
- `CustomDocumentEntry`
- `DocumentRef`
- `LoadedCustomDocuments`
- `ValidationIssue`
- `EditorDocumentRoute`

Games can register their own document kinds and validators without forcing
engine-core Helix- or DoomExe-specific branches.

## Editor Architecture

The editor is split across focused modules under `engine/src/editor/`.

Important responsibilities:

- mounted-project lifecycle
- level and story-graph views
- graph preview as an editor-only tool
- custom-document browser and raw JSON editing
- dirty tracking across scene/story/project/custom-doc state
- runtime preview launch/stop lifecycle and status reporting

## Runtime And Save Architecture

- `engine/src/runtime_preview/mod.rs`
  - preview state machine and project boot flow
- `engine/src/project_mount.rs`
  - shared path normalization and startup resolution
- `engine/src/save.rs`
  - save helpers, scoped saves, and runtime preview continue support

The preview path uses project startup defaults and preview profiles rather than
hardcoded sample-game assumptions.

## Extension Points

Current extension seams include:

- custom document kind registration
- custom document validators
- editor document routing
- runtime preview loading of custom document bundles
- game-side scripting/plugin layers in consumer crates

That is the architectural boundary to protect as new game-specific systems are
introduced.

## External Data Sources

The custom document platform is designed to accept data from external pipelines.
The engine never depends on any external data source at compile time or runtime —
external data flows through game plugins that register document kinds.

The primary planned external source is `helix_standardization`
(`~/dev/helix/helix_standardization`), a mature data standardization pipeline
with 2,681 curated entities across 26 TOML files in `dist/helix3d/` (abilities, items,
mobs, quests, zones, equipment, mounts, currencies, etc.). The full source pipeline
contains ~14,500 entities across 305+ categories, but the curated helix3d subset is
what DJ Engine and Helix 3D consume.

The intended data flow:

```text
helix_standardization dist/ JSON
        |
        v
  Game plugin maps categories to DJ-Engine document kinds
        |
        v
  Registered via CustomDocumentRegistration<T>
        |
        v
  Loaded through data/registry.json
        |
        v
  Editor browsing, structured editing, validation, runtime loading
```

This is a dev/testing integration. Production builds embed their own data and do
not depend on `helix_standardization` being present.

The test fixtures in `engine/src/runtime_preview/mod.rs` intentionally use
Helix-shaped document kinds (`abilities`, `enemy_archetypes`, `evolution_tree`,
`waves`) as proof that the custom document platform can carry real game data
shapes. These are engine tests exercising the generic platform, not engine-level
Helix dependencies.
