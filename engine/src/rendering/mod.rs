//! Rendering system for DJ Engine.
//!
//! Provides offscreen rendering, palette swapping, and CRT post-processing.

use bevy::prelude::*;

pub mod camera;

pub use camera::{MainCamera, GAME_HEIGHT, GAME_WIDTH};

/// Rendering plugin that sets up the visual pipeline.
pub struct RenderingPlugin;

impl Plugin for RenderingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, camera::setup_camera);
        // TODO: Setup offscreen render target (320×240)
        // TODO: Setup upscaling to window
        // TODO: Register CRT post-processing pass
    }
}
