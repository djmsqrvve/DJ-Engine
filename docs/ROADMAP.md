# DJ-Engine Roadmap

Status as of 2026-03-10: 121 tests passing, Phase 1-4 complete.

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
- TODOs in `engine/src/rendering/mod.rs:18-20`

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
- TODOs at spawner.rs:180-182

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

| Session | Work | Tests After |
|---------|------|-------------|
| Next | Phase 1: Quick wins (docs, validation, helper, MIDI) | ~107 |
| +1 | Phase 2: Audio crossfade | ~112 |
| +2 | Phase 3: Save/load | ~118 |
| +3-5 | Phase 4: CRT pipeline | ~122 |
| +6-8 | Phase 5: Physics/collision | ~130 |
