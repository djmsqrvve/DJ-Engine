//! Rendering system for DJ Engine.
//!
//! Provides offscreen rendering at 320x240, upscaling to window,
//! and CRT post-processing (configurable via `CrtConfig`).

use bevy::prelude::*;

pub mod camera;
pub mod crt_material;
pub mod offscreen;

pub use camera::{MainCamera, GAME_HEIGHT, GAME_WIDTH};
pub use offscreen::{CrtConfig, DisplayCamera, DisplayQuad, OffscreenTarget};

/// Rendering plugin that sets up the visual pipeline.
pub struct RenderingPlugin;

impl Plugin for RenderingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(crt_material::CrtMaterialPlugin)
            .init_resource::<CrtConfig>()
            .register_type::<CrtConfig>()
            .add_systems(Startup, offscreen::setup_offscreen_pipeline)
            .add_systems(
                Update,
                (
                    camera::handle_camera_commands,
                    offscreen::resize_display_projection,
                    offscreen::sync_crt_config,
                ),
            );
    }
}
