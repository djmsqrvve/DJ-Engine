# DJ Engine Documentation

This index separates current repo guides from older design and planning docs.

## Start Here

Use these first when you need the current repo shape rather than historical design intent:

| Guide | Description | Audience |
|-------|-------------|----------|
| [AI Handoff Suite](AI_HANDOFF_SUITE/README.md) | Source-derived handoff for the current worktree and recent engine milestones | AI agents and contributors resuming work |
| [Getting Started](GETTING_STARTED.md) | Current setup, validation, and runtime launch commands | New contributors |
| [Architecture](ARCHITECTURE.md) | Current engine/editor/runtime/custom-document architecture | Contributors |
| [Current Gaps](CURRENT_GAPS.md) | High-level near-, mid-, and long-term gap map for the current engine state | Contributors and planners |
| [Current Priorities](CURRENT_PRIORITIES.md) | Detailed near- and mid-term execution priorities for the next engine slices | Contributors and planners |
| [Long-Term Goals](LONG_TERM_GOALS.md) | Longer-horizon engine goals, guardrails, and proof points | Contributors and planners |
| [Testing Guide](TESTING.md) | Current validation commands and test structure | Contributors |
| [Project Structure](PROJECT_STRUCTURE.md) | Current workspace layout and mounted-project shape | Everyone |
| [Code Style](CODE_STYLE.md) | Coding standards and conventions | Contributors |

The root [README.md](../README.md) and [engine/README.md](../engine/README.md)
are also kept current for the engine-first workflow.

## Current Areas

| Area | Location | Notes |
|------|----------|-------|
| Engine core | `engine/src/` | Reusable engine/editor/runtime code |
| Sample game | `games/dev/doomexe/` | Optional sample consumer, not the engine source of truth |
| Runtime preview | `engine/src/runtime_preview/` | Engine-owned playable preview loop for mounted projects |
| Custom documents | `engine/src/data/custom.rs` | Registry-driven authored data under `data/registry.json` |
| Editor shell | `engine/src/editor/` | Egui editor, graph preview, and runtime handoff |

## Historical And Planning Docs

These files are still useful context, but they are not the source of truth for
the current repository and may describe older DoomExe-first or hamster-era
assumptions:

| Document | Status |
|----------|--------|
| [ROADMAP.md](ROADMAP.md) | Historical milestone notes |
| [Game_Engine_Technical_Roadmap.md](Game_Engine_Technical_Roadmap.md) | Older long-range planning document |
| [EDITOR_Specification_Complete.md](EDITOR_Specification_Complete.md) | Older editor specification |
| [complete-detailed-docs.md](complete-detailed-docs.md) | Historical implementation draft |
| [DETAILED_TASK_DOCS.md](DETAILED_TASK_DOCS.md) | Historical task/spec draft |
| [Architecture_Specification.json](Architecture_Specification.json) | Historical high-level architecture artifact |

When any of those disagree with code, the code, Cargo manifests, `Makefile`,
root README, engine README, and AI Handoff Suite win.

## Need Help?

- Check existing [GitHub Issues](https://github.com/djmsqrvve/dj_engine/issues)
- Read the [Contributing Guide](../CONTRIBUTING.md)
- Review the [Maintainer Guide](../MAINTAINERS.md)
