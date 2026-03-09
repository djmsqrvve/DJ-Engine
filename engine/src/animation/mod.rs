//! Animation system for DJ Engine.
//!
//! Provides procedural breathing, blinking, and idle motion animations.

use bevy::prelude::*;

pub mod components;
pub mod systems;

pub use components::{BlinkingAnimation, BreathingAnimation, IdleMotion};

/// Animation plugin that registers all animation systems.
pub struct DJAnimationPlugin;

impl Plugin for DJAnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                systems::breathing_system,
                systems::blinking_system,
                systems::idle_motion_system,
            ),
        );
    }
}
