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
        .init_resource::<tutorial_steps::TutorialState>()
        // Startup
        .add_systems(Startup, (setup_camera, rendering::spawn_board_system))
        // Setup phase
        .add_systems(OnEnter(state::GamePhase::Setup), input::init_setup_system)
        .add_systems(
            Update,
            (
                input::setup_click_system,
                input::setup_status_system,
                rendering::sync_pieces_system,
                tutorial_steps::tutorial_system,
            )
                .run_if(in_state(state::GamePhase::Setup)),
        )
        // Red turn
        .add_systems(
            Update,
            (
                input::player_click_system,
                input::play_status_system,
                rendering::sync_pieces_system,
                rendering::sync_highlights_system,
                tutorial_steps::tutorial_system,
            )
                .run_if(in_state(state::GamePhase::RedTurn)),
        )
        // Blue turn (AI)
        .add_systems(OnEnter(state::GamePhase::BlueTurn), ai::ai_turn_system)
        .add_systems(
            Update,
            rendering::sync_pieces_system.run_if(in_state(state::GamePhase::BlueTurn)),
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
