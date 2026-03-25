# Current Gaps

This document is the short snapshot for the current engine state. It points to
the more detailed planning docs rather than trying to hold every level of
planning in one file.

## Status Snapshot

DJ Engine now has:

- an engine-first launch surface (`make dev`, `make preview`, `make game`)
- a manifest-driven editor shell with new-game scaffolding (`make new-game`)
- a separate engine-owned runtime preview path
- project-scoped preview saves and continue flow
- snapshot-based editor dirty tracking and guarded reloads
- a registry-driven custom-document scaffold under `data/registry.json`
- structured metadata editing, reference-link pickers, a table editor with
  inline top-level scalar editing for record-heavy document kinds, and a
  recursive property inspector for nested object/array payload fields
- a working editor extension seam: toolbar action dispatch, preview preset
  selection, and custom panel registration (Helix data plugin is the first
  real consumer)
- structured panel data export for all editor surfaces (documents, scene,
  story graph, console, assets)
- an interactive tutorial overlay system with JSON-driven step definitions,
  panel highlighting, and auto-advance on user actions
- a typed TOML pipeline for Helix data: `helix-data` crate dependency,
  `HelixRegistries` Bevy Resource with 22 typed `Registry<T>` (2,681 entities),
  bridge layer converting Helix types to engine DB types, balance overlays,
  and a contract validation dashboard (`make helix-dashboard`)
- 444 passing tests across the workspace, zero clippy warnings
- combat formula system with configurable damage, crit, defense, and variance
- quest journal with accept/progress/complete/turn-in/abandon lifecycle
- NPC interaction system (proximity detection, InteractionEvent, dialogue routing)
- sprite animation player (frame cycling, loop/one-shot, speed control)
- spawner wave system (timed waves, SpawnWaveEvent for enemy instantiation)
- Lua ECS bridge: set_field, set_position, get_entities query, get_document access

That gives the repo a solid reusable foundation. The biggest remaining gaps are
no longer about basic launchability; they are about authoring depth, extension
seams, and proving the engine can carry richer non-sample games cleanly.

## Planning Docs

Use this file as the one-page summary, then go deeper here:

- [Current Priorities](CURRENT_PRIORITIES.md)
  - near-term and mid-term execution priorities
- [Long-Term Goals](LONG_TERM_GOALS.md)
  - engine vision, guardrails, and strategic proof points

## Near-Term Gaps

Short version:

- Custom-document authoring now supports structured editing, table browsing,
  inline top-level scalar table edits, field-targeted validation cues, and
  recursive nested property editing in the inspector. The remaining authoring
  gap is graph-style editing for relationship-heavy document kinds.
- Runtime preview is still a baseline playable loop.
- Editor extension seams are now fully wired: toolbar action handlers dispatch
  queued actions, preview preset selection feeds into runtime launch, and
  custom panel draw callbacks render game-provided UI. The Helix data plugin
  is the first real consumer.
- The editor now has a panel data export system and an interactive tutorial
  overlay. Entity placement auto-selects the placed entity.
- Documentation is current: AI/legacy docs archived, all links verified,
  crate metadata complete, game READMEs written, Helix integration documented.

Details live in [Current Priorities](CURRENT_PRIORITIES.md).

## Mid-Term Gaps

Short version:

- Generic authoring tools need to grow from raw scaffolding into real workflows:
  table editing, graph editing, shared property inspection, and reference-aware
  browsing for custom game data.
- The runtime bridge from authored documents to game plugins needs to be proven
  more deeply.
- Validation needs a clearer split between structural engine validation and
  game-side semantic validation so large authored projects stay debuggable.
- Runtime debug surfaces need to mature.

Details live in [Current Priorities](CURRENT_PRIORITIES.md).

## Long-Term Goal Gaps

Short version:

- Prove DJ Engine as a reusable multi-game authoring/runtime platform, not just
  a cleaned-up DoomExe shell.
- Support data-authored games with custom progression/combat/content models
  without leaking those semantics into engine core.
- Deliver a full extension model where games can register document kinds,
  validators, editor panels, preview presets, and runtime data consumers
  without forking the shell.
- Add long-horizon project infrastructure such as schema evolution, migration
  hooks, stronger save/settings extensibility, and more mature debugging and
  inspection workflows.
- Validate the engine against a richer external consumer — specifically, a Helix
  game consuming `helix_standardization` data — while keeping engine APIs generic
  and reusable.

Details live in [Long-Term Goals](LONG_TERM_GOALS.md).

## Practical Reading Of The Current State

The foundation is no longer the risky part. The real work ahead is making the
engine pleasant and extensible enough that a new game can mount into it without
turning the engine back into a one-off runtime.
