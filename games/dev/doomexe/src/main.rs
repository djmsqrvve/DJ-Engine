#![allow(
    clippy::too_many_arguments,
    clippy::type_complexity,
    clippy::module_inception
)]
use bevy::prelude::*;
use dj_engine::prelude::*;
use types::DoomExeAppConfig;

mod assets;
mod battle;
mod dialogue;
mod hamster;
mod hud;
mod overworld;
mod scripting;
mod state;
mod story;
mod title;
mod types;

fn main() {
    let app_config = DoomExeAppConfig::default();

    App::new()
        .add_plugins(
            DefaultPlugins
                .set(bevy::log::LogPlugin {
                    filter: "wgpu=error,naga=warn,bevy_render::camera=error,bevy_render=warn,bevy_ecs=warn,bevy_app=warn,bevy_winit=warn,dj_engine=info,doomexe=info".into(),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()) // Pixel art friendly
                .set(WindowPlugin {
                    primary_window: Some(app_config.primary_window()),
                    ..default()
                }),
        )
        // Engine plugins (bundled)
        .add_plugins(DJEnginePlugin::default())
        .insert_resource(AutoLoadMidi {
            path: "music/overworld_theme.mid".into(),
        })
        // Game database with loot table for glitch battle
        .insert_resource({
            let mut db = dj_engine::data::database::Database::default();
            let mut loot = dj_engine::data::database::LootTableRow::new("glitch_loot");
            loot.add_entry("corrupted_data", 1.0, 1);
            loot.add_entry("glitch_shard", 0.5, 1);
            db.loot_tables.push(loot);
            db
        })
        // Game-specific scripting extensions
        .add_plugins(scripting::GameScriptingPlugin)
        // Game state
        .init_state::<state::GameState>()
        // Game plugins
        .add_plugins(title::TitlePlugin)
        .add_plugins(story::StoryPlugin)
        .add_plugins(hamster::HamsterPlugin)
        .add_plugins(overworld::OverworldPlugin)
        .add_plugins(hud::HudPlugin)
        .add_plugins(dialogue::DialoguePlugin)
        .add_plugins(battle::BattlePlugin)
        .add_plugins(assets::GameAssetsPlugin)
        .run();
}
