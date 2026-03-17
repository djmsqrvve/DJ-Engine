# Chapter 8: Tutorial Overlay

Add an in-game tutorial that guides the player through their first game.

## What You'll Add

- Replace the `tutorial_steps.rs` stub with the full implementation (128 lines)
- Register the tutorial system in main.rs

## tutorial_steps.rs (full)

Replace the stub from Chapter 5 with the complete implementation:

> **File: `games/dev/stratego/src/tutorial_steps.rs`**

```rust
//! In-game tutorial that guides the player through their first game.

use bevy::prelude::*;

use crate::board::StrategoBoard;
use crate::input::{FeedbackMessage, SetupQueue};
use crate::pieces::{PieceRank, Team};
use crate::rendering::{TutorialPanel, TutorialText};
use crate::state::GamePhase;

/// Current tutorial step.
#[derive(Resource, Debug)]
pub struct TutorialState {
    pub step: usize,
    pub timer: f32,
    pub first_move_done: bool,
    pub first_combat_done: bool,
    pub enabled: bool,
}

impl Default for TutorialState {
    fn default() -> Self {
        Self {
            step: 0,
            timer: 0.0,
            first_move_done: false,
            first_combat_done: false,
            enabled: true,
        }
    }
}

const TUTORIAL_MESSAGES: &[&str] = &[
    "Welcome to Stratego! This is a 10x10 board with two lakes in the center.",
    "Place your Flag first — it can't move, so hide it in the red zone!",
    "Now place your Marshal (10) — your strongest piece. Higher rank wins!",
    "Place the rest of your army. It auto-fills when you're done.",
    "Your turn! Click one of your red pieces to select it.",
    "Move into an enemy piece to attack! Higher rank wins.",
    "Capture the enemy Flag to win. Equal ranks destroy each other. Good luck!",
];

fn tutorial_message(step: usize) -> &'static str {
    TUTORIAL_MESSAGES
        .get(step)
        .copied()
        .unwrap_or("Play on!")
}

/// Advance the tutorial based on game events. Runs every frame in all phases.
pub fn tutorial_system(
    time: Res<Time>,
    phase: Res<State<GamePhase>>,
    board: Res<StrategoBoard>,
    queue: Res<SetupQueue>,
    feedback: Res<FeedbackMessage>,
    mut tutorial: ResMut<TutorialState>,
    mut text_q: Query<&mut Text2d, With<TutorialText>>,
    mut panel_q: Query<&mut Visibility, With<TutorialPanel>>,
) {
    if !tutorial.enabled {
        // Hide panel when tutorial is off.
        if let Ok(mut vis) = panel_q.single_mut() {
            *vis = Visibility::Hidden;
        }
        return;
    }

    // Don't override feedback messages.
    if feedback.is_active() {
        return;
    }

    tutorial.timer += time.delta_secs();

    let should_advance = match tutorial.step {
        0 => tutorial.timer > 3.0,

        1 => board.grid.iter().any(|(_, _, cell)| {
            cell.piece
                .as_ref()
                .map(|p| p.team == Team::Red && p.rank == PieceRank::Flag)
                .unwrap_or(false)
        }),

        2 => board.grid.iter().any(|(_, _, cell)| {
            cell.piece
                .as_ref()
                .map(|p| p.team == Team::Red && p.rank == PieceRank::Marshal)
                .unwrap_or(false)
        }),

        3 => queue.remaining.is_empty(),

        4 => tutorial.first_move_done,

        5 => {
            tutorial.first_combat_done
                || board.grid.iter().any(|(_, _, cell)| {
                    cell.piece.as_ref().map(|p| p.revealed).unwrap_or(false)
                })
        }

        6 => *phase.get() == GamePhase::GameOver,

        _ => false,
    };

    if should_advance {
        tutorial.step += 1;
        tutorial.timer = 0.0;
    }

    // Show tutorial text in the separate panel below the board.
    if tutorial.step < TUTORIAL_MESSAGES.len() {
        if let Ok(mut text) = text_q.single_mut() {
            **text = tutorial_message(tutorial.step).to_string();
        }
        if let Ok(mut vis) = panel_q.single_mut() {
            *vis = Visibility::Visible;
        }
    } else {
        // Tutorial complete — hide the panel.
        if let Ok(mut vis) = panel_q.single_mut() {
            *vis = Visibility::Hidden;
        }
    }
}
```

Key design:

- **7 event-driven steps**, each advancing based on a different game event:
  - Step 0: Auto-advance after 3 seconds (welcome message)
  - Step 1: Board scan finds a Red Flag (player placed it)
  - Step 2: Board scan finds a Red Marshal
  - Step 3: Setup queue is empty (all pieces placed)
  - Step 4: `first_move_done` flag set by `player_click_system`
  - Step 5: `first_combat_done` OR any piece is revealed
  - Step 6: Game phase is GameOver

- **Cross-module integration**: Steps 4-5 depend on flags set by `player_click_system` in `input.rs`. When the player makes their first move, `tutorial.first_move_done = true`. When combat occurs, `tutorial.first_combat_done = true`. This was already wired in Chapter 5.

- **Feedback deference**: When a feedback message is active (e.g., "Your Marshal defeated their Captain!"), the tutorial doesn't update its panel text. This prevents the tutorial tip from overwriting important combat results.

- **Panel visibility**: The tutorial panel below the board shows when `step < 7` and hides when complete. It also hides when `enabled` is false (after restart, it re-enables via `TutorialState::default()`).

## Final main.rs

> **File: `games/dev/stratego/src/main.rs`**

This is the final version, matching the actual source exactly:

```rust
//! Stratego-lite — a turn-based board game tutorial for DJ Engine.

use bevy::prelude::*;
use bevy::window::WindowResolution;

mod ai;
mod board;
mod input;
mod pieces;
mod rendering;
mod rules;
mod state;
mod tutorial_steps;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "DJ Engine - Stratego".into(),
                        resolution: WindowResolution::new(800, 800)
                            .with_scale_factor_override(1.0),
                        position: WindowPosition::Centered(MonitorSelection::Primary),
                        present_mode: bevy::window::PresentMode::AutoVsync,
                        ..default()
                    }),
                    ..default()
                }),
        )
        .insert_resource(ClearColor(Color::srgb(0.15, 0.15, 0.2)))
        .init_state::<state::GamePhase>()
        .init_resource::<board::StrategoBoard>()
        .init_resource::<state::GameResult>()
        .init_resource::<input::PlayerSelection>()
        .init_resource::<input::SetupQueue>()
        .init_resource::<input::FeedbackMessage>()
        .init_resource::<tutorial_steps::TutorialState>()
        .init_resource::<ai::AiTimer>()
        // Startup
        .add_systems(Startup, (setup_camera, rendering::spawn_board_system))
        // Global (runs in all states)
        .add_systems(Update, (
            input::tick_feedback_system,
            tutorial_steps::tutorial_system,
            rendering::sync_pieces_system
                .after(input::setup_click_system)
                .after(input::player_click_system)
                .after(ai::ai_turn_system),
            rendering::sync_setup_zone_system,
        ))
        // Setup phase
        .add_systems(OnEnter(state::GamePhase::Setup), input::init_setup_system)
        .add_systems(
            Update,
            (
                input::setup_click_system,
                input::setup_status_system,
            )
                .run_if(in_state(state::GamePhase::Setup)),
        )
        // Red turn
        .add_systems(
            Update,
            (
                input::player_click_system,
                input::play_status_system,
                rendering::sync_highlights_system,
            )
                .run_if(in_state(state::GamePhase::RedTurn)),
        )
        // Blue turn (AI with delay)
        .add_systems(
            Update,
            ai::ai_turn_system.run_if(in_state(state::GamePhase::BlueTurn)),
        )
        // Game over
        .add_systems(OnEnter(state::GamePhase::GameOver), input::game_over_system)
        .add_systems(
            Update,
            input::restart_system.run_if(in_state(state::GamePhase::GameOver)),
        )
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}
```

The only change from Chapter 7 is adding `tutorial_steps::tutorial_system` to the global Update systems.

## Final Architecture

```text
main.rs           App setup, system registration, state wiring (89 lines)
pieces.rs         PieceRank, Team, PlacedPiece, army composition (137 lines)
board.rs          Grid<Cell> board, terrain, placement, auto-fill (222 lines)
rules.rs          Combat resolution, move validation, move execution (346 lines)
state.rs          GamePhase enum, GameResult (21 lines)
rendering.rs      Sprites, coordinate conversion, highlights, tutorial panel (302 lines)
input.rs          Click handlers, setup queue, feedback, restart (333 lines)
ai.rs             Random AI with timer delay (116 lines)
tutorial_steps.rs 7-step tutorial overlay (128 lines)
```

9 modules. ~1,700 lines. 20 tests. A complete, playable game.

## Checkpoint

```sh
make stratego
```

The full experience:

1. Tutorial panel appears below the board: "Welcome to Stratego!"
2. After 3 seconds: "Place your Flag first..."
3. Place Flag in bottom rows -> "Now place your Marshal..."
4. Place all pieces -> Blue army fills, tutorial says "Your turn!"
5. Select and move a piece -> combat feedback appears
6. Turns alternate with AI until a flag is captured
7. "You win!" or "You lose!" -- press R to restart
8. Tutorial panel hides after step 7

```sh
cargo test -p stratego
# 20 tests pass
```

## What's Next

Ideas for extending this game:

- **Smarter AI**: Prioritize attacking, protect the flag, avoid equal-rank trades
- **Undo**: Store move history, add Ctrl+Z support
- **Sound effects**: Use the engine's audio system for combat, placement, victory
- **Animations**: Lerp piece movement between cells instead of instant teleport
- **Online play**: Implement the `GameServer` trait for networked multiplayer

Or start a new tutorial: build a different kind of game using the same engine patterns.
