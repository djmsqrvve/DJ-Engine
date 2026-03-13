# Current Gaps

This document is the short snapshot for the current engine state. It points to
the more detailed planning docs rather than trying to hold every level of
planning in one file.

## Status Snapshot

DJ Engine now has:

- an engine-first launch surface (`make dev`, `make preview`, `make game`)
- a manifest-driven editor shell
- a separate engine-owned runtime preview path
- project-scoped preview saves and continue flow
- snapshot-based editor dirty tracking and guarded reloads
- a registry-driven custom-document scaffold under `data/registry.json`
- structured metadata editing, reference-link pickers, and a table editor for
  record-heavy document kinds (abilities, items, mobs)

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

- Custom-document authoring has structured editing and table browsing but still
  lacks inline cell editing and field-level validation targeting.
- Runtime preview is still a baseline playable loop.
- Editor/runtime extension points need another pass.
- Current docs are much cleaner now, but some older planning/spec artifacts are
  still historical rather than fully reconciled with the present repo shape.

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
