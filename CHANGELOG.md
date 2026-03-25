# Changelog

All notable changes to DJ Engine will be documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-03-24

First public release.

### Added

- **Engine core**: manifest-driven editor with tile system, story graph, custom document platform
- **Table editor**: inline scalar editing, recursive property inspector, reference-link pickers
- **Lua scripting**: mlua 0.10 integration with hot-reload, FFI bridge, ECS bridge
- **CRT pipeline**: offscreen render target, scanlines, barrel distortion, chromatic aberration
- **Procedural animation**: breathing, blinking, expression-driven character assembly
- **Palette corruption**: real-time palette swaps driven by corruption float (0-100)
- **Runtime preview**: save/load, title flow, continue flow, story graph execution
- **Tutorial overlay**: JSON-driven step definitions, panel highlighting, auto-advance
- **Custom document platform**: registry-driven game data with editor browsing and validation
- **DoomExe game**: dark-fantasy JRPG prototype with hamster narrator
- **Stratego game**: 10x10 board game with AI opponent and 8-chapter tutorial
- **Iso Sandbox game**: isometric 16x16 tile grid with entity placement
- **Helix data plugin**: typed TOML pipeline consuming 2,681 MMORPG entities via helix-data crate
- **Helix dashboard**: contract validation with 7 cross-reference checks
- **Balance overlays**: per-engine tuning applied during bridge conversion
- **CI pipeline**: formatting, clippy (fails on warnings), test count gate (400+)
- **Codespaces support**: devcontainer with desktop forwarding for Bevy windows
- **Cross-compile**: Windows .exe builds via `make dev-exe`
- **Documentation**: architecture guide, testing guide, project structure, 8-chapter Stratego tutorial

### Infrastructure

- Rust 1.94.0 pinned via rust-toolchain.toml
- Bevy 0.18 with workspace dependency management
- 411 tests, zero clippy warnings
- MIT licensed

[0.1.0]: https://github.com/djmsqrvve/DJ-Engine/releases/tag/v0.1.0
