use bevy::prelude::*;
use bevy::window::WindowResolution;

/// Minimal test to see if a red window with text can be rendered at all.
/// No Egui, no Audio, no Scene logic.
fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "MINIMAL TEST WINDOW".into(),
                resolution: WindowResolution::new(800, 600).with_scale_factor_override(1.0),
                position: WindowPosition::Centered(MonitorSelection::Primary),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::srgb(1.0, 0.0, 0.0))) // Bright Red
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    // UI Text
    commands.spawn((
        Text::new("IF YOU SEE THIS RED WINDOW, THE GRAPHICS DRIVER IS WORKING!"),
        TextFont {
            font_size: 30.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Percent(45.0),
            left: Val::Percent(10.0),
            ..default()
        },
    ));
}
