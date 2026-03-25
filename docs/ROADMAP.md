# DJ-Engine Roadmap

Status as of 2026-03-25: 499 tests passing, zero failures, zero clippy warnings. Phase 1-5 complete, runtime deepening shipped. 18 gameplay systems: combat, combat FX (floating text), quests, inventory, interaction, animation, spawner, status effects, ability cooldowns, loot, economy (consumables, equipment, vendor), character (titles, weapon skills, bags), Lua API (8 tables), debug console, objective navigator. Five game crates: DoomExe, Stratego, Iso Sandbox, RPG Demo (SDK reference), Helix RPG (MMORPG data consumer). QA checklist with 6 visual test cards.

## Phase 1: Quick Wins (1 session)

### 1a. Stale Docs Fix DONE (2026-03-10)
- Replaced `/tmp/dj_engine_bevy18` with `~/.cargo-targets/dj_engine_bevy18`
- Replaced major Bevy 0.14/0.15/0.16 references with 0.18
- Added staleness banners to current navigation docs
- Added historical headers to older planning and handoff docs
- Fixed stale claims in AI Handoff Suite (asset loaders, CI `--no-run`, retired `./dj`)
- Rewrote `docs-summary-reference.md` and `MASTER-DOCUMENTATION-INDEX.md` around current docs plus `archive/`
- Normalized AI Handoff Suite current-state/workspace-map/session-handoff notes
- Moved `enginefeaturedraft.md`, `enginefeaturedraftjson.md` to `archive/`
- Rewrote `archive/README.md` as proper archive index

### 1b. Story Graph Validation
- Add unreachable node detection (BFS from root) in `engine/src/data/story.rs:531`
- Small: ~20 lines

### 1c. set_next Helper
- Add `StoryNodeVariant::set_next_node_id()` method to eliminate 8-arm match in `engine/src/editor/views.rs:292-311`
- Small: ~15 lines

### 1d. MIDI Path Warning
- Gate `start_midi_load` on a config check so editor binary doesn't emit spurious warnings
- File: `engine/src/midi/wav.rs:110`

## Phase 2: Audio Crossfade (1 session)

### What exists
- `AudioCommand::PlayBgm { track, crossfade }` and `StopBgm { fade_out }` already defined
- `handle_audio_commands` ignores crossfade/fade_out params (`engine/src/audio/mod.rs:136,155`)
- `AudioState` tracks volume, `BgmSource`/`SfxSource` markers exist

### What to build
- `BgmFade` component on BGM entities tracking elapsed/duration/direction
- Update system that ticks `BgmFade`, interpolates `AudioSink` volume
- `PlayBgm`: fade out current BGM, spawn new with fade in
- `StopBgm`: fade out current BGM over `fade_out` duration
- ~100 lines + tests

## Phase 3: Save/Load System (1-2 sessions)

### What exists
- Title screen with NEW GAME / CONTINUE / QUIT (`games/dev/doomexe/src/title.rs`)
- `StoryFlags`, `StoryVariables` resources (serializable state)
- `StoryGraphData` with JSON serialization patterns throughout `engine/src/data/`
- serde already in workspace

### What to build
- `SaveData` struct: story flags, story variables, current scene ID, graph position
- `save_game()` / `load_game()` functions writing JSON to `~/.local/share/dj_engine/saves/`
- Wire CONTINUE button to load, NEW GAME to reset
- ~150 lines + tests

## Phase 4: CRT Post-Processing Pipeline (2-3 sessions)

### What exists
- Camera at 320x240 with 4x zoom (`engine/src/rendering/camera.rs`)
- `GAME_WIDTH`/`GAME_HEIGHT` constants
- Offscreen render target and CRT shader already implemented (`engine/src/rendering/offscreen.rs`, `engine/src/rendering/crt_material.rs`)

### What to build
- Session A: Offscreen render target (320x240 texture), upscaling blit to window
- Session B: CRT shader (scanlines, slight barrel distortion, color bleeding)
- Session C: Toggle system (enable/disable CRT via config), intensity controls
- Requires Bevy 0.18 render graph / post-processing API research
- Largest feature gap

## Phase 5: Physics/Collision (2-3 sessions)

### What exists
- Full data types: `CollisionComponent` with 13 fields, `BodyType`, `CollisionShape` (`engine/src/data/components.rs:200-251`)
- `InteractivityComponent`, `AudioSourceComponent` data structures
- Spawner (`engine/src/data/spawner.rs`) receives data but doesn't insert physics components

### What to build
- Choose physics library (likely `avian2d` for Bevy 0.18 or custom AABB)
- Bridge `CollisionComponent` data -> Bevy/physics ECS components in spawner
- Collision layers and trigger detection
- Blocked on library choice decision

## Phase 6: Tech Debt (ongoing, interleave with above)

- Extract `engine/src/data/story.rs` validation into `validation.rs` (648 lines)
- Extract gameplay components from `engine/src/data/components.rs` (621 lines)
- Can be done in spare cycles between feature sessions

## Suggested Session Order

| Session | Work                     | Tests After |
| ------- | ------------------------ | ----------- |
| Done    | Phase 1: Quick wins      | 302         |
| Done    | Phase 2: Audio crossfade | 411         |
| Done    | Runtime deepening        | 499         |
| Next    | Phase 3: Save/load       | ~510        |
| +1      | Phase 4: CRT pipeline    | ~520        |
| +2      | Phase 5: Physics/collision | ~535      |
