//! Camera setup for DJ Engine rendering.
//!
//! Provides camera configuration for pixel-perfect rendering.

use bevy::prelude::*;

/// Marker component for the main game camera.
#[derive(Component)]
pub struct MainCamera;

/// The target resolution for the game canvas.
pub const GAME_WIDTH: f32 = 320.0;
pub const GAME_HEIGHT: f32 = 240.0;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_dimensions() {
        assert_eq!(GAME_WIDTH, 320.0);
        assert_eq!(GAME_HEIGHT, 240.0);
    }
}

/// Sets up the main camera for 2D rendering.
pub fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Camera {
            order: -1,
            ..default()
        },
        Projection::from(OrthographicProjection {
            scale: 0.25, // Zoom in (4x)
            ..OrthographicProjection::default_2d()
        }),
        MainCamera,
        Transform::from_xyz(0.0, 0.0, 1000.0),
    ));
}
