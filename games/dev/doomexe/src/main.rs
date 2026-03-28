#![allow(
    clippy::too_many_arguments,
    clippy::type_complexity,
    clippy::module_inception
)]
use bevy::prelude::*;
use dj_engine::prelude::*;
use dj_engine::screen_fx::LowHealthVignette;
use types::DoomExeAppConfig;

mod assets;
mod battle;
mod cellar;
mod dialogue;
mod gameover;
mod hamster;
mod hud;
mod overworld;
mod pause;
mod scripting;
mod state;
mod story;
mod title;
mod types;

fn build_game_database() -> dj_engine::data::database::Database {
    use dj_engine::data::database::*;

    let mut db = Database::default();

    // -- Items --
    db.items.push(ItemRow {
        id: "health_potion".into(),
        name: [("en".into(), "Health Potion".into())].into(),
        heal_amount: 25,
        price: 10,
        sell_value: 5,
        max_stack: 10,
        ..Default::default()
    });
    db.items.push(ItemRow {
        id: "rat_tail".into(),
        name: [("en".into(), "Rat Tail".into())].into(),
        price: 5,
        sell_value: 3,
        max_stack: 20,
        ..Default::default()
    });
    db.items.push(ItemRow {
        id: "rusty_sword".into(),
        name: [("en".into(), "Rusty Sword".into())].into(),
        damage: 3,
        price: 25,
        sell_value: 10,
        max_stack: 1,
        ..Default::default()
    });
    db.items.push(ItemRow {
        id: "corrupted_data".into(),
        name: [("en".into(), "Corrupted Data".into())].into(),
        sell_value: 8,
        ..Default::default()
    });
    db.items.push(ItemRow {
        id: "glitch_shard".into(),
        name: [("en".into(), "Glitch Shard".into())].into(),
        sell_value: 15,
        ..Default::default()
    });

    // -- Consumables --
    db.consumables.push(ConsumableRow {
        id: "health_potion".into(),
        name: [("en".into(), "Health Potion".into())].into(),
        consumable_type: "potion".into(),
        stack_size: 10,
        ..Default::default()
    });

    // -- Loot Tables --
    let mut glitch_loot = LootTableRow::new("glitch_loot");
    glitch_loot.add_entry("corrupted_data", 1.0, 1);
    glitch_loot.add_entry("glitch_shard", 0.5, 1);
    db.loot_tables.push(glitch_loot);

    let mut rat_loot = LootTableRow::new("rat_loot");
    rat_loot.add_entry("rat_tail", 0.8, 1);
    rat_loot.add_entry("health_potion", 0.3, 1);
    db.loot_tables.push(rat_loot);

    db
}

fn spawn_health_vignette(mut commands: Commands) {
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(0.0),
            left: Val::Px(0.0),
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.0)),
        LowHealthVignette::default(),
    ));
}

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
        // Game database: items, loot, consumables for the gameplay loop
        .insert_resource(build_game_database())
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
        .add_plugins(cellar::CellarPlugin)
        .add_plugins(gameover::GameOverPlugin)
        .add_plugins(pause::PausePlugin)
        .add_plugins(assets::GameAssetsPlugin)
        .add_systems(Startup, spawn_health_vignette)
        .run();
}
