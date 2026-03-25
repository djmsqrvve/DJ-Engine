#![allow(clippy::too_many_arguments, clippy::type_complexity)]
//! Helix RPG — MMORPG prototype powered by Helix standardization data.
//!
//! Spawns NPCs, enemies, and abilities from the 2,681 entities in
//! helix_standardization's dist/helix3d/ TOML pipeline.
//!
//! Run with: `make helix-rpg`

use bevy::prelude::*;
use dj_engine::core::DJEnginePlugin;

mod world;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(bevy::log::LogPlugin {
                    filter: "wgpu=error,naga=warn,bevy_render=warn,bevy_ecs=warn,dj_engine=info,helix_rpg=info".into(),
                    ..default()
                })
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Helix RPG — DJ Engine".into(),
                        resolution: bevy::window::WindowResolution::new(1024, 768),
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins(DJEnginePlugin::default().without_diagnostics())
        .init_state::<world::GamePhase>()
        .add_plugins(world::WorldPlugin)
        .run();
}
