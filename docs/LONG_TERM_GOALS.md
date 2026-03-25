# Long-Term Goals

This document captures the longer-horizon goals for DJ Engine so the current
execution work stays aligned with the larger engine vision.

## Core Long-Term Goal

DJ Engine should become a reusable authoring and runtime platform for multiple
data-authored games, not a cleaned-up shell around one sample project.

That means:

- games can mount into the engine through stable extension seams
- the engine owns reusable structure and tooling
- game semantics stay in game plugins and game data

## Long-Term Goals

### 1. Reusable multi-game authoring platform

The editor should eventually feel like a true engine shell that can host
different game families without being rewritten each time.

Desired end state:

- multiple games can mount into the same editor shell
- different document kinds can share reusable authoring surfaces
- custom panels feel like extensions, not forks

### 2. Strong generic data platform

The engine should support rich authored data beyond scenes and story graphs.

Desired end state:

- projects can register new document kinds cleanly
- references across scenes, custom docs, story graphs, and assets are stable
- schema versioning and migration hooks exist when projects outgrow v1 layouts
- save/settings integration can carry both engine-owned and game-owned data

### 3. Runtime preview as a real engine bridge

`runtime_preview` should mature into a reusable playable path for authored
projects, not just a narrow demo loop.

Desired end state:

- mounted projects can launch meaningful vertical slices
- preview uses authored startup context and richer preview presets
- preview can hand off cleanly to game-side runtime systems
- preview remains engine-generic rather than turning into a hidden game runtime

### 4. Clean extension model

The strongest version of DJ Engine is one where future games do not require
engine-core edits for every meaningful customization.

Desired end state:

- document kinds register through stable APIs
- validators register through stable APIs
- editor panels and toolbar actions register through stable APIs
- runtime data consumers register through stable APIs

### 5. Durable debugging and migration story

As projects get larger, the engine needs stronger long-horizon maintenance
support.

Desired end state:

- validation stays understandable at scale
- migration/version hooks exist for both documents and saves
- runtime/debug inspection helps explain authored-data failures quickly
- old game data can be prepared for the engine without silent semantic drift

## Guardrails

These are the long-term guardrails that should shape design decisions.

### No game semantics in engine core

The engine should not absorb:

- Helix DNA rules
- Helix evolution logic
- DoomExe battle semantics
- sample-game HUD logic
- game-specific balance formulas

If a feature only makes sense for one game’s meaning, it belongs in the game.
`helix_standardization` data flows through game plugins, not engine core.

### Prefer extension seams over hardcoded branches

When adding support for a new workflow, prefer:

- registries
- traits
- typed document contracts
- editor extension points
- runtime plugin hooks

over:

- game-name-specific matches
- special-case branches in engine core
- one-off bespoke pathways that only fit one project

### Keep current docs short and trustworthy

The engine can have rich historical/planning docs, but the active onboarding
and current-planning docs should stay concise and accurate.

## Strategic Proof Points

These are the clearest long-term proofs that the engine is succeeding.

### Proof point 1

A non-DoomExe project can mount custom data, validate it, open it in the editor,
and preview a meaningful slice without engine-core game branches.

### Proof point 2

At least one richer external consumer — specifically, a Helix game consuming
data from `helix_standardization` (`~/dev/helix/helix_standardization`) — can
use the same editor/runtime infrastructure while keeping its semantics outside
the engine crate.

### Proof point 3

The engine remains understandable:

- current docs stay aligned with the live repo
- extension points are discoverable
- contributors do not need historical tribal knowledge to use the engine

## Helix Data Integration Path

The DJ multiverse includes a Helix timeline with multiple game variants (Helix2000,
Helix 3D, potential Helix MMORPG) that may run on different engines or languages.
`helix_standardization` (`~/dev/helix/helix_standardization`) is the shared data
standard across all of them: ~14,500 entities, 26 curated TOML files in
`dist/helix3d/` (2,681 entities), covering abilities, items, mobs, quests, zones,
equipment, mounts, currencies, and more.

The integration path into DJ-Engine is:

- **Typed TOML pipeline (current):** `dist/helix3d/*.toml` → `helix-data` crate
  `Registry<T>::from_toml_str()` → `HelixRegistries` Bevy Resource (22 typed
  registries) → bridge layer converts to engine DB types → balance overlays
  apply per-engine tuning
- **Legacy JSON pipeline:** `dist/` raw JSON → `plugins/helix_data/` importer →
  `CustomDocument<Value>` envelopes → `HelixDocumentIndex`
- The `helix-data` crate (`~/dev/helix/helix_3d_render_prototype/crates/helix-data/`)
  is a zero-Bevy pure serde+toml crate shared between DJ Engine and Helix 3D
- The engine never depends on `helix_standardization` at compile time or runtime
- Each game variant maps standardized data to its own runtime needs
- Production builds carry their own data — `helix_standardization` is a
  dev/testing bridge only
- `make helix-dashboard` validates all 22 registries for schema contracts

The test fixtures in `engine/src/runtime_preview/mod.rs` intentionally use
Helix-shaped document kinds (`abilities`, `enemy_archetypes`, `evolution_tree`,
`waves`) as proof that the custom document platform can carry real game data
shapes. These are engine tests exercising the generic platform, not engine-level
Helix dependencies.

## Relationship To Current Planning

- [CURRENT_GAPS.md](CURRENT_GAPS.md) is the short snapshot
- [CURRENT_PRIORITIES.md](CURRENT_PRIORITIES.md) is the practical near/mid-term
  execution layer
- this document is the longer-horizon goal and guardrail layer
