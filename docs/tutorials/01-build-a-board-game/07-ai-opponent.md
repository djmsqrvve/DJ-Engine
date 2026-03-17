# Chapter 7: AI Opponent

Add a random-move AI for the Blue team with a brief thinking delay.

## What You'll Add

- `ai.rs` -- Timer resource, move selection, combat execution (116 lines)

## ai.rs

> **File: `games/dev/stratego/src/ai.rs`**

```rust
//! Simple random AI for the Blue team with a brief delay.

use bevy::prelude::*;
use rand::seq::SliceRandom;

use crate::board::StrategoBoard;
use crate::input::FeedbackMessage;
use crate::pieces::Team;
use crate::rules::{self, CombatResult};
use crate::state::{GamePhase, GameResult};

/// Timer for AI thinking delay.
#[derive(Resource, Debug)]
pub struct AiTimer {
    pub timer: Timer,
    pub ready: bool,
}

impl Default for AiTimer {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(0.8, TimerMode::Once),
            ready: false,
        }
    }
}

/// AI waits briefly, then takes a random valid move for Blue.
pub fn ai_turn_system(
    time: Res<Time>,
    mut ai_timer: ResMut<AiTimer>,
    mut board: ResMut<StrategoBoard>,
    mut next_state: ResMut<NextState<GamePhase>>,
    mut game_result: ResMut<GameResult>,
    mut feedback: ResMut<FeedbackMessage>,
) {
    ai_timer.timer.tick(time.delta());

    if !ai_timer.timer.is_finished() {
        return;
    }

    if ai_timer.ready {
        return; // Already moved this turn.
    }
    ai_timer.ready = true;

    let mut candidates: Vec<(usize, usize, Vec<(usize, usize)>)> = Vec::new();

    for (x, y, cell) in board.grid.iter() {
        if let Some(piece) = &cell.piece {
            if piece.team == Team::Blue && piece.rank.can_move() {
                let moves = rules::valid_moves(&board, Team::Blue, x, y);
                if !moves.is_empty() {
                    candidates.push((x, y, moves));
                }
            }
        }
    }

    if candidates.is_empty() {
        game_result.winner = Some(Team::Red);
        next_state.set(GamePhase::GameOver);
        ai_timer.timer.reset();
        ai_timer.ready = false;
        return;
    }

    let mut rng = rand::thread_rng();
    let (fx, fy, moves) = candidates.choose(&mut rng).unwrap();
    let &(tx, ty) = moves.choose(&mut rng).unwrap();

    // Snapshot ranks for feedback.
    let attacker_rank = board
        .get(*fx, *fy)
        .and_then(|c| c.piece.as_ref())
        .map(|p| p.rank);
    let defender_rank = board
        .get(tx, ty)
        .and_then(|c| c.piece.as_ref())
        .map(|p| p.rank);

    let combat = rules::execute_move(&mut board, *fx, *fy, tx, ty);

    // Combat feedback.
    if let (Some(combat), Some(atk), Some(def)) = (combat, attacker_rank, defender_rank) {
        match combat {
            CombatResult::AttackerWins => {
                feedback.set(format!("AI's {} defeated your {}!", atk.name(), def.name()));
            }
            CombatResult::DefenderWins => {
                feedback.set(format!("Your {} held against AI's {}!", def.name(), atk.name()));
            }
            CombatResult::BothDie => {
                feedback.set(format!("AI's {} and your {} destroyed each other!", atk.name(), def.name()));
            }
            CombatResult::FlagCaptured(loser) => {
                game_result.winner = Some(loser.opponent());
                next_state.set(GamePhase::GameOver);
                ai_timer.timer.reset();
                ai_timer.ready = false;
                return;
            }
        }
    }

    if let Some(CombatResult::FlagCaptured(loser)) = combat {
        game_result.winner = Some(loser.opponent());
        next_state.set(GamePhase::GameOver);
    } else {
        next_state.set(GamePhase::RedTurn);
    }

    ai_timer.timer.reset();
    ai_timer.ready = false;
}
```

Key patterns:

- **Double-guard**: The timer must finish (`is_finished()`) AND the ready flag must be false. This prevents the AI from moving multiple times in the same turn. Both checks are needed because `is_finished()` stays true until the timer is reset.
- **Rank snapshot**: `attacker_rank` and `defender_rank` are captured BEFORE `execute_move` because the move modifies the board (the attacker's source cell is cleared).
- **No-moves detection**: If no Blue piece can move, Red wins by default.
- **FlagCaptured early return**: If the AI captures the player's flag, we immediately transition to GameOver and reset the timer.
- **Timer reset**: After every AI move, both `timer.reset()` and `ready = false` are called to prepare for the next BlueTurn.
- **Cross-module import**: `use crate::input::FeedbackMessage` -- the AI uses the same feedback system as the player's input handler.

## Update main.rs

> **File: `games/dev/stratego/src/main.rs`**

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

Changes from Chapter 5:

- Added `mod ai;` and `.init_resource::<ai::AiTimer>()`
- Added BlueTurn system: `ai::ai_turn_system.run_if(in_state(GamePhase::BlueTurn))`
- Added `.after(ai::ai_turn_system)` to `sync_pieces_system`

## Checkpoint

```sh
make stratego
```

The full game loop now works:

1. Place all 25 pieces in the red zone
2. Blue army fills automatically
3. Click a piece, click a green cell to move
4. After your move, "Opponent is thinking..." appears
5. After 0.8 seconds, the AI makes a random move
6. Combat feedback shows ("AI's Captain defeated your Scout!")
7. Turns alternate until a flag is captured
8. "You win!" or "You lose!" -- press R to restart

## Next

[Chapter 8: Tutorial Overlay](08-polish.md) -- Add an in-game tutorial and final polish.
