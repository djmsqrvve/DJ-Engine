# Current Gaps

This document is the high-level gap map for the current engine state. It is
intended to complement the historical roadmap/spec docs with a shorter,
current-repo view.

## Status Snapshot

DJ Engine now has:

- an engine-first launch surface (`make dev`, `make preview`, `make game`)
- a manifest-driven editor shell
- a separate engine-owned runtime preview path
- project-scoped preview saves and continue flow
- snapshot-based editor dirty tracking and guarded reloads
- a registry-driven custom-document scaffold under `data/registry.json`

That gives the repo a solid reusable foundation. The biggest remaining gaps are
no longer about basic launchability; they are about authoring depth, extension
seams, and proving the engine can carry richer non-sample games cleanly.

## Near-Term Gaps

These are the next practical gaps to close in the next few focused slices.

- Custom-document authoring is still scaffold-level.
  Current support is discovery, validation, selection, and raw JSON editing.
  The engine still needs richer typed editing, better reference pickers, and
  friendlier validation surfaces.
- Runtime preview is still a baseline playable loop.
  It proves `Title -> Dialogue -> Overworld`, but it does not yet cover richer
  gameplay handoff, data-driven action loops, or deeper preview presets.
- Editor/runtime extension points need another pass.
  The shell can host custom documents, but game-specific panels, toolbar
  actions, and richer preview launch presets are still early.
- Current docs are much cleaner now, but some older planning/spec artifacts are
  still historical rather than fully reconciled with the present repo shape.

## Mid-Term Gaps

These are the next milestone-level gaps once the current scaffolding is made
pleasant to use.

- Generic authoring tools need to grow from raw scaffolding into real workflows:
  table editing, graph editing, shared property inspection, and reference-aware
  browsing for custom game data.
- The runtime bridge from authored documents to game plugins needs to be proven
  more deeply.
  The engine can now load custom documents, but games still need stronger,
  cleaner ways to consume them without engine-core branching.
- Validation needs a clearer split between structural engine validation and
  game-side semantic validation so large authored projects stay debuggable.
- Runtime debug surfaces need to mature.
  Event logs, mounted data inspection, save/settings inspection, and preview
  state visibility should become first-class tools instead of mostly code/test
  level capabilities.

## Long-Term Goal Gaps

These are the big-picture gaps between the current repo and the long-term engine
vision.

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
- Validate the engine against a richer external consumer, such as a Helix-style
  action/survival project, while keeping engine APIs generic and reusable.

## Practical Reading Of The Current State

The foundation is no longer the risky part. The real work ahead is making the
engine pleasant and extensible enough that a new game can mount into it without
turning the engine back into a one-off runtime.
