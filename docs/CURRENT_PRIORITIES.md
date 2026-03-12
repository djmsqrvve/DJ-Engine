# Current Priorities

This document expands the short summary in `CURRENT_GAPS.md` into the practical
near-term and mid-term work the engine should focus on next.

## How To Read This

- Near-term priorities are the next focused slices that should improve the
  engine quickly and reduce friction for current development.
- Mid-term priorities are the next milestone-level investments once the current
  scaffolding is stable enough to build on comfortably.
- This is a planning document for the current repo shape, not a historical
  roadmap.

## Near-Term Priorities

### 1. Make custom documents pleasant to author

Current state:

- The engine can discover custom documents from `data/registry.json`
- It can validate them structurally
- The editor can browse them and edit raw JSON

Gap:

- Authoring is still too low-level for day-to-day use

What the engine needs next:

- typed editors for common document shapes
- reference-aware pickers instead of manual string editing
- clearer validation messages and field targeting
- friendlier save/load/reload flow for custom documents

Why this matters:

- The custom-document platform is the foundation for Helix-style and other
  non-scene game data
- If authoring remains raw JSON-first for too long, the engine seam will exist
  technically but still feel bad in practice

Success looks like:

- a contributor can create or update a custom document without hand-editing most
  references
- the editor can point directly at the broken field when validation fails
- common document kinds can be edited through reusable UI instead of only raw JSON

### 2. Strengthen the editor extension seam

Current state:

- The editor shell now has the right direction
- It can carry project state, custom docs, dirty tracking, and runtime handoff

Gap:

- Extension points for game-specific tools are still early

What the engine needs next:

- custom panel registration that feels first-class
- custom toolbar actions for mounted games/tools
- better document routing beyond the current raw browser/editor surface
- preview presets that can be selected from richer authored context

Why this matters:

- The engine will only stay reusable if games can extend the shell without
  forking it

Success looks like:

- a game can mount a custom panel beside the engine shell
- a game can register custom document routes cleanly
- preview launch behavior can be customized without engine-core branches

### 3. Expand runtime preview beyond the current baseline loop

Current state:

- `runtime_preview` proves `Title -> Dialogue -> Overworld`
- It supports mounted projects, startup defaults, preview profiles, and continue

Gap:

- The preview loop is still a baseline proof, not a rich reusable runtime bridge

What the engine needs next:

- better preview-profile-driven boot flows
- clearer status/error reporting when startup content is missing or invalid
- stronger bridging from loaded custom documents into game-side runtime plugins
- more confidence that preview can host non-DoomExe game shapes

Why this matters:

- Runtime preview is now the engine’s most important playable proof path

Success looks like:

- a mounted project can boot a more meaningful authored slice
- preview failure states are visible and actionable
- game plugins can consume mounted authored data without engine rewrites

### 4. Keep current docs aligned with the live repo

Current state:

- The onboarding path is much healthier than it was

Gap:

- Older planning/spec docs still exist and can confuse new contributors if left
  unframed

What the engine needs next:

- continue tightening current docs when engine behavior changes
- keep the handoff suite, branch log, and current planning docs current
- avoid letting new features land without at least a short current-doc update

Why this matters:

- Clean docs reduce drift, repeated reconstruction work, and accidental regressions

Success looks like:

- new contributors can follow the current docs without falling into historical dead ends
- current docs stay short and trustworthy

## Mid-Term Priorities

### 1. Build generic authoring tools on top of the scaffold

What this means:

- reusable table editing for record-heavy document kinds
- reusable graph editing for branching/relationship-heavy document kinds
- reusable property inspection for scalar/list/enum/reference fields
- reusable filtering, search, and reference browsing across document kinds

Why it matters:

- This is how the engine stops being a JSON shell and becomes a real authoring platform

Success looks like:

- at least two meaningful custom document kinds can be authored without bespoke panels
- custom panels become the exception, not the default

### 2. Clarify validation ownership

What this means:

- engine validates structure, IDs, discovery, and reference integrity
- game plugins validate semantics, balance, and domain rules
- validation output should make that boundary visible

Why it matters:

- Large authored projects become painful quickly if structural and semantic
  validation are mixed without discipline

Success looks like:

- validation issues are grouped clearly
- engine-side fixes and game-side fixes are easy to distinguish
- game validators can plug in cleanly without muddying engine rules

### 3. Mature runtime inspection and debug tooling

What this means:

- runtime event log improvements
- mounted document inspection
- save/settings inspection
- better visibility into preview state and loaded authored context

Why it matters:

- More authored/runtime complexity demands better introspection, not just more tests

Success looks like:

- a contributor can understand what preview loaded and why a runtime path failed
- debugging authored-data mistakes is faster than reading logs and code manually

### 4. Prove the runtime/document bridge with a richer consumer

What this means:

- a game-side plugin should consume mounted authored data through stable engine seams
- the engine should not need to absorb that game’s semantics to make it work

Why it matters:

- This is the actual proof that the current direction is working

Success looks like:

- a richer external consumer, like a Helix-style project, can mount into the
  engine with mostly additive game-side code
- the engine still reads as generic after that work lands
