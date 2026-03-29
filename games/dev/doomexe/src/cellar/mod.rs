//! Cellar dungeon zone for DoomExe.
//!
//! Dark basement with 5 rat enemies (3 normal, 1 small, 1 alpha). Player fights rats (real-time combat),
//! collects loot, finds a weapon chest, and progresses the "clear_the_cellar" quest.

use crate::state::GameState;
use bevy::prelude::*;
use dj_engine::data::{BodyType, CollisionComponent, Vec3Data};
use dj_engine::prelude::*;

pub mod systems;

pub struct CellarPlugin;

impl Plugin for CellarPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameState::Cellar),
            (setup_cellar, systems::setup_cellar_ui),
        )
        .add_systems(
            Update,
            (
                systems::player_cellar_movement,
                systems::player_cellar_attack,
                systems::rat_ai_attack,
                systems::handle_cellar_damage,
                systems::update_cellar_hud,
                systems::check_cellar_clear,
                systems::chest_interaction,
                systems::use_potion,
                systems::update_health_vignette,
            )
                .chain()
                .run_if(in_state(GameState::Cellar)),
        )
        .add_systems(
            OnExit(GameState::Cellar),
            (teardown_cellar, systems::teardown_cellar_ui),
        );
    }
}

#[derive(Component)]
pub struct CellarEntity;

#[derive(Component)]
pub struct CellarPlayer;

#[derive(Component)]
pub struct CellarRat {
    pub index: usize,
}

#[derive(Component)]
pub struct WeaponChest {
    pub opened: bool,
}

fn setup_cellar(mut commands: Commands) {
    // Dark floor
    commands.spawn((
        Name::new("Cellar Floor"),
        Sprite {
            color: Color::srgb(0.06, 0.04, 0.02),
            custom_size: Some(Vec2::new(800.0, 600.0)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 0.0),
        CellarEntity,
    ));

    // Player in cellar
    commands.spawn((
        Name::new("Cellar Player"),
        Sprite {
            color: Color::srgb(0.2, 0.2, 0.8),
            custom_size: Some(Vec2::new(28.0, 28.0)),
            ..default()
        },
        Transform::from_xyz(0.0, -180.0, 10.0),
        CellarPlayer,
        dj_engine::combat::AttackCooldown::new(0.8),
        dj_engine::data::components::CombatStatsComponent {
            max_hp: 80,
            hp: 80,
            mana: 30,
            damage: 10,
            defense: 5,
            crit_chance: 0.1,
            ..default()
        },
        CellarEntity,
    ));

    // 5 Rats spread across the cellar (varied sizes and stats)
    let rats: [(Vec3, f32, i32, i32); 5] = [
        (Vec3::new(-120.0, 60.0, 10.0), 24.0, 30, 5), // normal rat
        (Vec3::new(80.0, 100.0, 10.0), 24.0, 30, 5),  // normal rat
        (Vec3::new(0.0, -20.0, 10.0), 24.0, 30, 5),   // normal rat
        (Vec3::new(-60.0, 130.0, 10.0), 20.0, 20, 4), // small rat
        (Vec3::new(120.0, 40.0, 10.0), 30.0, 45, 7),  // big rat (alpha)
    ];

    for (i, &(pos, size, hp, damage)) in rats.iter().enumerate() {
        let color = if size > 26.0 {
            Color::srgb(0.6, 0.2, 0.1) // alpha rat = darker
        } else if size < 22.0 {
            Color::srgb(0.55, 0.35, 0.25) // small rat = lighter
        } else {
            Color::srgb(0.5, 0.25, 0.15) // normal
        };

        commands.spawn((
            Name::new(format!("Rat {}", i + 1)),
            Sprite {
                color,
                custom_size: Some(Vec2::new(size, size)),
                ..default()
            },
            Transform::from_translation(pos),
            CellarRat { index: i },
            dj_engine::combat::AttackCooldown::new(2.0),
            dj_engine::data::components::CombatStatsComponent {
                max_hp: hp,
                hp,
                damage,
                defense: 2,
                crit_chance: 0.05,
                loot_table_id: Some("rat_loot".into()),
                ..default()
            },
            CellarEntity,
        ));
    }

    // Weapon chest in far corner
    commands.spawn((
        Name::new("Weapon Chest"),
        Sprite {
            color: Color::srgb(0.6, 0.5, 0.1),
            custom_size: Some(Vec2::new(32.0, 24.0)),
            ..default()
        },
        Transform::from_xyz(150.0, 150.0, 10.0),
        WeaponChest { opened: false },
        CellarEntity,
    ));

    info!("Cellar: spawned 5 rats (3 normal, 1 small, 1 alpha) + weapon chest");
}

fn teardown_cellar(mut commands: Commands, query: Query<Entity, With<CellarEntity>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}
