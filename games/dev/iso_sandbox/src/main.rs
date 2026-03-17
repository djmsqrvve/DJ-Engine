//! Isometric sandbox — place entities on a 16x16 diamond-tile grid.

use bevy::prelude::*;
use bevy::window::WindowResolution;

mod grid;
mod input;
mod palette;
mod rendering;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "DJ Engine - Iso Sandbox".into(),
                        resolution: WindowResolution::new(1024, 768)
                            .with_scale_factor_override(1.0),
                        position: WindowPosition::Centered(MonitorSelection::Primary),
                        present_mode: bevy::window::PresentMode::AutoVsync,
                        ..default()
                    }),
                    ..default()
                }),
        )
        .insert_resource(ClearColor(Color::srgb(0.12, 0.12, 0.18)))
        .init_resource::<grid::IsoGrid>()
        .init_resource::<input::HoverTile>()
        .init_resource::<input::FeedbackMessage>()
        .init_resource::<palette::SelectedPalette>()
        // Startup
        .add_systems(Startup, (setup_camera, rendering::spawn_grid_system))
        // Update
        .add_systems(Update, (
            input::hover_system,
            input::click_place_system,
            input::click_remove_system,
            input::terrain_cycle_system,
            input::tick_feedback_system,
            input::status_system,
            palette::palette_system,
            rendering::sync_tiles_system
                .after(input::terrain_cycle_system),
            rendering::sync_entities_system
                .after(input::click_place_system)
                .after(input::click_remove_system),
            rendering::sync_hover_system
                .after(input::hover_system),
        ))
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Transform::from_xyz(0.0, -240.0, 0.0),
    ));
}
