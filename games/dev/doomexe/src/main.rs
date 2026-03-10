use bevy::prelude::*;
use bevy::window::WindowResolution;
use dj_engine::prelude::*;

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
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest()) // Pixel art friendly
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "DJ Engine V1.0".into(),
                        resolution: WindowResolution::new(800, 600).with_scale_factor_override(1.0),
                        position: WindowPosition::Centered(MonitorSelection::Primary),
                        present_mode: bevy::window::PresentMode::AutoVsync,
                        ..default()
                    }),
                    ..default()
                }),
        )
        // Engine plugins (bundled)
        .add_plugins(DJEnginePlugin::default())
        .insert_resource(AutoLoadMidi { path: "music/overworld_theme.mid".into() })
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
