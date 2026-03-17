//! Animation system for DJ Engine.
//!
//! Provides procedural breathing, blinking, and idle motion animations.

use crate::contracts::{AppContractExt, ContractEntry, PluginContract};
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

        app.register_contract(PluginContract {
            name: "DJAnimationPlugin".into(),
            description: "Procedural breathing, blinking, and idle motion".into(),
            resources: vec![],
            components: vec![
                ContractEntry::of::<BreathingAnimation>("Breathing cycle animation"),
                ContractEntry::of::<BlinkingAnimation>("Blinking interval animation"),
                ContractEntry::of::<IdleMotion>("Idle sway motion"),
            ],
            events: vec![],
            system_sets: vec![],
        });
    }
}
