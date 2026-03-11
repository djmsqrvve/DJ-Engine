//! Camera setup for DJ Engine rendering.
//!
//! Provides camera configuration for pixel-perfect rendering.

use crate::story_graph::CameraCommand;
use bevy::prelude::*;

/// Marker component for the main game camera.
#[derive(Component)]
pub struct MainCamera;

/// The target resolution for the game canvas.
pub const GAME_WIDTH: f32 = 320.0;
pub const GAME_HEIGHT: f32 = 240.0;

pub fn handle_camera_commands(mut commands: MessageReader<CameraCommand>) {
    for cmd in commands.read() {
        warn!(
            "CameraCommand received but camera transitions not yet implemented: {:?}",
            cmd
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_dimensions() {
        assert_eq!(GAME_WIDTH, 320.0);
        assert_eq!(GAME_HEIGHT, 240.0);
    }
}
