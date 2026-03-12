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

At least one richer external consumer, such as a Helix-style project, can use
the same editor/runtime infrastructure while keeping its semantics outside the
engine crate.

### Proof point 3

The engine remains understandable:

- current docs stay aligned with the live repo
- extension points are discoverable
- contributors do not need historical tribal knowledge to use the engine

## Relationship To Current Planning

- [CURRENT_GAPS.md](CURRENT_GAPS.md) is the short snapshot
- [CURRENT_PRIORITIES.md](CURRENT_PRIORITIES.md) is the practical near/mid-term
  execution layer
- this document is the longer-horizon goal and guardrail layer
