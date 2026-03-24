# DJ Engine Documentation

This index covers the current, maintained documentation for DJ Engine.

## Guides

| Guide | Description | Audience |
|-------|-------------|----------|
| [Getting Started](GETTING_STARTED.md) | Setup, validation, and launch commands | New contributors |
| [Architecture](ARCHITECTURE.md) | Engine/editor/runtime/custom-document architecture | Contributors |
| [Testing Guide](TESTING.md) | Validation commands and test structure | Contributors |
| [Project Structure](PROJECT_STRUCTURE.md) | Workspace layout and mounted-project shape | Everyone |
| [Code Style](CODE_STYLE.md) | Coding standards and conventions | Contributors |

## Planning

| Document | Description |
|----------|-------------|
| [Roadmap](ROADMAP.md) | Phased development plan (Phases 1-4 complete, 302+ tests) |
| [Current Gaps](CURRENT_GAPS.md) | Near-, mid-, and long-term gap map |
| [Current Priorities](CURRENT_PRIORITIES.md) | Detailed execution priorities for upcoming work |
| [Long-Term Goals](LONG_TERM_GOALS.md) | Engine vision, guardrails, and proof points |

## Tutorials

| Tutorial | Description |
|----------|-------------|
| [Build a Board Game](tutorials/01-build-a-board-game/README.md) | 8-chapter Stratego walkthrough with full code |

## Current Engine Areas

| Area | Location | Notes |
|------|----------|-------|
| Engine core | `engine/src/` | Reusable engine/editor/runtime code |
| Sample game | `games/dev/doomexe/` | Hamster narrator JRPG prototype |
| Stratego | `games/dev/stratego/` | 10x10 board game tutorial with AI |
| Iso Sandbox | `games/dev/iso_sandbox/` | Isometric 16x16 tile grid |
| Helix plugin | `plugins/helix_data/` | Helix data bridge and TOML import tooling |
| Runtime preview | `engine/src/runtime_preview/` | Engine-owned playable preview loop |
| Custom documents | `engine/src/data/custom.rs` | Registry-driven authored data |
| Editor shell | `engine/src/editor/` | Egui editor, graph preview, runtime handoff |

The root [README.md](../README.md) and [engine/README.md](../engine/README.md)
are also kept current for the engine-first workflow.

## Historical Documentation

Legacy design specs, migration notes, and AI session handoff files live under
[archive/](../archive/). These are kept for historical reference but are not
current implementation guides.

## Need Help?

- Check existing [GitHub Issues](https://github.com/djmsqrvve/DJ-Engine/issues)
- Read the [Contributing Guide](../CONTRIBUTING.md)
- Review the [Maintainer Guide](../MAINTAINERS.md)
