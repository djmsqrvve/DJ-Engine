#![allow(clippy::too_many_arguments, clippy::type_complexity)]
//! RPG Demo — minimal reference game for DJ Engine.
//!
//! Demonstrates every major engine system working together:
//! combat, quests, inventory, NPC interaction, abilities, loot,
//! status effects, and Lua scripting hooks.
//!
//! Run with: `make rpg-demo`

use bevy::prelude::*;
use dj_engine::prelude::*;

mod gameplay;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(bevy::log::LogPlugin {
                    filter: "wgpu=error,naga=warn,bevy_render::camera=error,bevy_render=warn,bevy_ecs=warn,dj_engine=info,rpg_demo=info".into(),
                    ..default()
                })
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "DJ Engine — RPG Demo".into(),
                        resolution: bevy::window::WindowResolution::new(960, 640),
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins(DJEnginePlugin::default().without_diagnostics())
        .init_state::<gameplay::GamePhase>()
        .add_plugins(gameplay::GameplayPlugin)
        .run();
}
