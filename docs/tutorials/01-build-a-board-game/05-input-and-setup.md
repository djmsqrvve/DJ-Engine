# Chapter 5: Input and Setup

Handle mouse clicks for placing pieces during setup and moving them during play.

## What You'll Add

- `input.rs` -- Click handling, feedback, setup queue, selection, all game systems (333 lines)
- `tutorial_steps.rs` -- Stub with just the `TutorialState` resource (replaced in Chapter 8)

We need the `tutorial_steps` stub because `input.rs` imports `TutorialState` to set tutorial progress flags during gameplay.

## Step 1: tutorial_steps.rs (stub)

> **File: `games/dev/stratego/src/tutorial_steps.rs`** (temporary -- replaced in Chapter 8)

```rust
//! In-game tutorial — stub for now, full implementation in Chapter 8.

use bevy::prelude::*;

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
```

This is just the data structure. The tutorial system that advances through steps comes in Chapter 8.

## Step 2: input.rs

> **File: `games/dev/stratego/src/input.rs`**

```rust
//! Player input handling: setup placement, piece selection, movement.

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::board::StrategoBoard;
use crate::pieces::{army_composition, PieceRank, PlacedPiece, Team};
use crate::rendering::{world_to_cell, StatusText};
use crate::rules::{self, CombatResult};
use crate::state::{GamePhase, GameResult};
use crate::tutorial_steps::TutorialState;

/// Temporary feedback message shown for a few seconds.
#[derive(Resource, Default, Debug)]
pub struct FeedbackMessage {
    pub text: String,
    pub timer: f32,
}

impl FeedbackMessage {
    pub fn set(&mut self, msg: impl Into<String>) {
        self.text = msg.into();
        self.timer = 2.0;
    }

    pub fn is_active(&self) -> bool {
        self.timer > 0.0
    }
}

/// Tick the feedback timer down each frame.
pub fn tick_feedback_system(time: Res<Time>, mut feedback: ResMut<FeedbackMessage>) {
    if feedback.timer > 0.0 {
        feedback.timer -= time.delta_secs();
    }
}

/// Tracks the player's current selection during play.
#[derive(Resource, Default, Debug)]
pub struct PlayerSelection {
    pub selected: Option<(usize, usize)>,
    pub valid_moves: Vec<(usize, usize)>,
}

/// Queue of pieces the player still needs to place during setup.
#[derive(Resource, Default, Debug)]
pub struct SetupQueue {
    pub remaining: Vec<PieceRank>,
}

/// Get the cursor position in world coordinates.
fn cursor_world_pos(
    windows: &Query<&Window, With<PrimaryWindow>>,
    camera_q: &Query<(&Camera, &GlobalTransform)>,
) -> Option<Vec2> {
    let window = windows.single().ok()?;
    let (camera, camera_transform) = camera_q.single().ok()?;
    let cursor_pos = window.cursor_position()?;
    camera.viewport_to_world_2d(camera_transform, cursor_pos).ok()
}

/// Initialize setup queue with all army pieces.
pub fn init_setup_system(mut queue: ResMut<SetupQueue>) {
    queue.remaining.clear();
    for (rank, count) in army_composition() {
        for _ in 0..count {
            queue.remaining.push(rank);
        }
    }
    // Place flag and marshal first for tutorial flow.
    queue.remaining.sort_by(|a, b| {
        let priority = |r: &PieceRank| match r {
            PieceRank::Flag => 0,
            PieceRank::Marshal => 1,
            _ => 2 + r.strength() as i32,
        };
        priority(a).cmp(&priority(b))
    });
}

/// Handle clicks during setup phase — place pieces one by one.
pub fn setup_click_system(
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    mut board: ResMut<StrategoBoard>,
    mut queue: ResMut<SetupQueue>,
    mut next_state: ResMut<NextState<GamePhase>>,
    mut feedback: ResMut<FeedbackMessage>,
) {
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    let Some(world_pos) = cursor_world_pos(&windows, &camera_q) else {
        return;
    };
    let Some((x, y)) = world_to_cell(world_pos) else {
        feedback.set("Click inside the board to place a piece.");
        return;
    };

    let Some(&rank) = queue.remaining.first() else {
        return;
    };

    let piece = PlacedPiece {
        rank,
        team: Team::Red,
        revealed: false,
    };

    if board.place_piece(x, y, piece) {
        queue.remaining.remove(0);

        if queue.remaining.is_empty() {
            board.auto_fill_army(Team::Blue);
            next_state.set(GamePhase::RedTurn);
        }
    } else if !board.is_setup_zone(x, y, Team::Red) {
        feedback.set("Place pieces in your zone (bottom 4 rows).");
    } else {
        feedback.set("That cell is already occupied or is a lake.");
    }
}

/// Update status text during setup.
pub fn setup_status_system(
    queue: Res<SetupQueue>,
    feedback: Res<FeedbackMessage>,
    mut text_q: Query<&mut Text2d, With<StatusText>>,
) {
    let Ok(mut text) = text_q.single_mut() else {
        return;
    };

    // Feedback messages take priority over status.
    if feedback.is_active() {
        **text = feedback.text.clone();
        return;
    }

    if let Some(rank) = queue.remaining.first() {
        **text = format!(
            "Place your {} ({} pieces remaining)",
            rank.name(),
            queue.remaining.len()
        );
    } else {
        **text = "Setup complete!".to_string();
    }
}

/// Handle clicks during Red's turn — select piece, then move.
pub fn player_click_system(
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    mut board: ResMut<StrategoBoard>,
    mut selection: ResMut<PlayerSelection>,
    mut next_state: ResMut<NextState<GamePhase>>,
    mut game_result: ResMut<GameResult>,
    mut tutorial: ResMut<TutorialState>,
    mut feedback: ResMut<FeedbackMessage>,
) {
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    let Some(world_pos) = cursor_world_pos(&windows, &camera_q) else {
        return;
    };
    let Some((x, y)) = world_to_cell(world_pos) else {
        return;
    };

    // If we have a selection and clicked a valid move target — execute the move.
    if let Some((sx, sy)) = selection.selected {
        if selection.valid_moves.contains(&(x, y)) {
            // Snapshot the attacker and defender ranks before the move for combat feedback.
            let attacker_rank = board
                .get(sx, sy)
                .and_then(|c| c.piece.as_ref())
                .map(|p| p.rank);
            let defender_rank = board
                .get(x, y)
                .and_then(|c| c.piece.as_ref())
                .map(|p| p.rank);

            let combat = rules::execute_move(&mut board, sx, sy, x, y);

            selection.selected = None;
            selection.valid_moves.clear();

            // Tutorial hooks.
            tutorial.first_move_done = true;
            if combat.is_some() {
                tutorial.first_combat_done = true;
            }

            // Combat feedback.
            if let (Some(combat), Some(atk), Some(def)) = (combat, attacker_rank, defender_rank) {
                match combat {
                    CombatResult::AttackerWins => {
                        feedback.set(format!(
                            "Your {} defeated their {}!",
                            atk.name(),
                            def.name()
                        ));
                    }
                    CombatResult::DefenderWins => {
                        feedback.set(format!(
                            "Their {} defeated your {}!",
                            def.name(),
                            atk.name()
                        ));
                    }
                    CombatResult::BothDie => {
                        feedback.set(format!(
                            "Both {}s destroyed each other!",
                            atk.name()
                        ));
                    }
                    CombatResult::FlagCaptured(loser) => {
                        game_result.winner = Some(loser.opponent());
                        next_state.set(GamePhase::GameOver);
                        return;
                    }
                }
            }

            if let Some(CombatResult::FlagCaptured(loser)) = combat {
                game_result.winner = Some(loser.opponent());
                next_state.set(GamePhase::GameOver);
            } else {
                next_state.set(GamePhase::BlueTurn);
            }
            return;
        }
    }

    // Otherwise, try to select a Red piece.
    if let Some(cell) = board.get(x, y) {
        if let Some(piece) = &cell.piece {
            if piece.team == Team::Red && piece.rank.can_move() {
                let moves = rules::valid_moves(&board, Team::Red, x, y);
                selection.selected = Some((x, y));
                selection.valid_moves = moves;
                return;
            }
            if piece.team == Team::Red && !piece.rank.can_move() {
                feedback.set("Your Flag can't move.");
                return;
            }
            if piece.team == Team::Blue {
                feedback.set("That's not your piece.");
                return;
            }
        }
    }

    // Clicked empty/invalid — deselect.
    selection.selected = None;
    selection.valid_moves.clear();
}

/// Update status text during play.
pub fn play_status_system(
    phase: Res<State<GamePhase>>,
    selection: Res<PlayerSelection>,
    feedback: Res<FeedbackMessage>,
    mut text_q: Query<&mut Text2d, With<StatusText>>,
) {
    let Ok(mut text) = text_q.single_mut() else {
        return;
    };

    // Feedback messages take priority.
    if feedback.is_active() {
        **text = feedback.text.clone();
        return;
    }

    match phase.get() {
        GamePhase::RedTurn => {
            if selection.selected.is_some() {
                **text = "Click a highlighted cell to move".to_string();
            } else {
                **text = "Your turn — click a piece to select it".to_string();
            }
        }
        GamePhase::BlueTurn => {
            **text = "Opponent is thinking...".to_string();
        }
        _ => {}
    }
}

/// Display game over screen.
pub fn game_over_system(
    game_result: Res<GameResult>,
    mut text_q: Query<&mut Text2d, With<StatusText>>,
) {
    let Ok(mut text) = text_q.single_mut() else {
        return;
    };
    let winner_name = match game_result.winner {
        Some(Team::Red) => "You win!",
        Some(Team::Blue) => "You lose!",
        None => "Draw!",
    };
    **text = format!("{winner_name} Press R to restart.");
}

/// Restart the game on R key.
pub fn restart_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut board: ResMut<StrategoBoard>,
    mut selection: ResMut<PlayerSelection>,
    mut game_result: ResMut<GameResult>,
    mut tutorial: ResMut<TutorialState>,
    mut feedback: ResMut<FeedbackMessage>,
    mut next_state: ResMut<NextState<GamePhase>>,
) {
    if keys.just_pressed(KeyCode::KeyR) {
        *board = StrategoBoard::new();
        *selection = PlayerSelection::default();
        *game_result = GameResult::default();
        *tutorial = TutorialState::default();
        *feedback = FeedbackMessage::default();
        next_state.set(GamePhase::Setup);
    }
}
```

Key patterns:

- **`cursor_world_pos`** is private -- used by both `setup_click_system` and `player_click_system`.
- **Setup queue sorting** puts Flag first, Marshal second. This aligns with the tutorial steps (Ch 8) which check for Flag placement, then Marshal placement.
- **`player_click_system`** has three branches:
  1. **Has selection + valid move target**: Execute the move, snapshot ranks for combat feedback, set tutorial flags, transition to BlueTurn
  2. **Clicked a Red piece**: Select it, compute valid moves
  3. **Clicked empty/invalid**: Deselect
- **Rank snapshots**: `attacker_rank` and `defender_rank` are captured BEFORE `execute_move` because the move modifies the board.
- **Tutorial hooks**: `tutorial.first_move_done` and `tutorial.first_combat_done` are set here. The tutorial system in Chapter 8 reads these flags to advance steps 4-5.
- **`restart_system`** resets every resource to default and transitions to Setup. `OnEnter(Setup)` fires again, reinitializing the setup queue.

## Step 3: Update main.rs

> **File: `games/dev/stratego/src/main.rs`**

```rust
//! Stratego-lite — a turn-based board game tutorial for DJ Engine.

use bevy::prelude::*;
use bevy::window::WindowResolution;

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
        // Startup
        .add_systems(Startup, (setup_camera, rendering::spawn_board_system))
        // Global (runs in all states)
        .add_systems(Update, (
            input::tick_feedback_system,
            rendering::sync_pieces_system
                .after(input::setup_click_system)
                .after(input::player_click_system),
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

Note: No BlueTurn systems yet (AI comes in Chapter 7). The `.after()` constraints on `sync_pieces_system` ensure piece sprites update in the same frame the board changes.

## Checkpoint

```sh
make stratego
```

You can now:

1. Click cells in the bottom 4 rows to place pieces one by one (Flag first, then Marshal, etc.)
2. See red squares with rank labels appear on the board
3. When all 25 are placed, Blue's army fills the top rows
4. Click a red piece to select it (yellow highlight), click a green cell to move
5. Combat feedback appears ("Your Marshal defeated their Captain!")
6. After moving, status shows "Opponent is thinking..." but nothing happens (AI comes next)
7. If you capture the Flag, "You win! Press R to restart."

## Next

[Chapter 6: State Machine Deep Dive](06-state-machine.md) -- Understand the game phase flow.
