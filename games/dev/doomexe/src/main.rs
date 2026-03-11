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
