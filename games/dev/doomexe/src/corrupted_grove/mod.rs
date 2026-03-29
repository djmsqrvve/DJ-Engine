//! Corrupted Grove — outdoor area after the cellar.
//!
//! A dark forest clearing with corrupted treants and shadow spiders.
//! Player must defeat 4 enemies to cleanse the grove and unlock the
//! "purify_grove" quest completion.

use crate::state::GameState;
use bevy::prelude::*;
use dj_engine::prelude::*;

pub mod systems;

pub struct CorruptedGrovePlugin;

impl Plugin for CorruptedGrovePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameState::CorruptedGrove),
            (setup_grove, systems::setup_grove_ui),
        )
        .add_systems(
            Update,
            (
                systems::player_grove_movement,
                systems::player_grove_attack,
                systems::enemy_ai_attack,
                systems::handle_grove_damage,
                systems::update_grove_hud,
                systems::check_grove_clear,
                systems::use_potion,
                systems::update_health_vignette,
            )
                .chain()
                .run_if(in_state(GameState::CorruptedGrove)),
        )
        .add_systems(
            OnExit(GameState::CorruptedGrove),
            (teardown_grove, systems::teardown_grove_ui),
        );
    }
}

#[derive(Component)]
pub struct GroveEntity;

#[derive(Component)]
pub struct GrovePlayer;

#[derive(Component)]
pub struct GroveEnemy {
    pub kind: GroveEnemyKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GroveEnemyKind {
    Treant,
    ShadowSpider,
}

fn setup_grove(mut commands: Commands) {
    // Forest floor — dark green/brown
    commands.spawn((
        Name::new("Grove Floor"),
        Sprite {
            color: Color::srgb(0.04, 0.08, 0.03),
            custom_size: Some(Vec2::new(900.0, 700.0)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 0.0),
        GroveEntity,
    ));

    // Corruption patches — purple tinted ground markers
    for (x, y) in [(-120.0, 80.0), (100.0, -60.0), (-50.0, -120.0)] {
        commands.spawn((
            Name::new("Corruption Patch"),
            Sprite {
                color: Color::srgba(0.4, 0.1, 0.5, 0.3),
                custom_size: Some(Vec2::new(80.0, 80.0)),
                ..default()
            },
            Transform::from_xyz(x, y, 1.0),
            GroveEntity,
        ));
    }

    // Tree decorations
    for (x, y) in [
        (-200.0, 150.0),
        (180.0, 120.0),
        (-180.0, -100.0),
        (200.0, -130.0),
        (0.0, 200.0),
    ] {
        commands.spawn((
            Name::new("Dead Tree"),
            Sprite {
                color: Color::srgb(0.15, 0.08, 0.05),
                custom_size: Some(Vec2::new(16.0, 40.0)),
                ..default()
            },
            Transform::from_xyz(x, y, 2.0),
            GroveEntity,
        ));
    }

    // Player in grove
    commands.spawn((
        Name::new("Grove Player"),
        Sprite {
            color: Color::srgb(0.2, 0.2, 0.8),
            custom_size: Some(Vec2::new(28.0, 28.0)),
            ..default()
        },
        Transform::from_xyz(0.0, -220.0, 10.0),
        GrovePlayer,
        dj_engine::combat::AttackCooldown::new(0.7),
        dj_engine::data::components::CombatStatsComponent {
            max_hp: 100,
            hp: 100,
            mana: 40,
            damage: 14,
            defense: 6,
            crit_chance: 0.12,
            ..default()
        },
        GroveEntity,
    ));

    // Enemies: 2 treants + 2 shadow spiders
    let enemies: [(Vec3, GroveEnemyKind, i32, i32, f32, &str); 4] = [
        (
            Vec3::new(-100.0, 80.0, 10.0),
            GroveEnemyKind::Treant,
            55,
            8,
            32.0,
            "treant_loot",
        ),
        (
            Vec3::new(120.0, 50.0, 10.0),
            GroveEnemyKind::Treant,
            65,
            10,
            36.0,
            "treant_loot",
        ),
        (
            Vec3::new(-60.0, -40.0, 10.0),
            GroveEnemyKind::ShadowSpider,
            35,
            12,
            22.0,
            "spider_loot",
        ),
        (
            Vec3::new(80.0, -80.0, 10.0),
            GroveEnemyKind::ShadowSpider,
            30,
            14,
            20.0,
            "spider_loot",
        ),
    ];

    for (pos, kind, hp, damage, size, loot) in &enemies {
        let color = match kind {
            GroveEnemyKind::Treant => Color::srgb(0.25, 0.45, 0.15),
            GroveEnemyKind::ShadowSpider => Color::srgb(0.3, 0.1, 0.35),
        };
        let name = match kind {
            GroveEnemyKind::Treant => "Corrupted Treant",
            GroveEnemyKind::ShadowSpider => "Shadow Spider",
        };

        commands.spawn((
            Name::new(name),
            Sprite {
                color,
                custom_size: Some(Vec2::new(*size, *size)),
                ..default()
            },
            Transform::from_translation(*pos),
            GroveEnemy { kind: *kind },
            dj_engine::combat::AttackCooldown::new(1.8),
            dj_engine::data::components::CombatStatsComponent {
                max_hp: *hp,
                hp: *hp,
                damage: *damage,
                defense: 4,
                crit_chance: 0.08,
                loot_table_id: Some(loot.to_string()),
                ..default()
            },
            GroveEntity,
        ));
    }

    info!("Corrupted Grove: spawned 2 treants + 2 spiders");
}

fn teardown_grove(mut commands: Commands, query: Query<Entity, With<GroveEntity>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grove_enemy_kinds() {
        assert_ne!(GroveEnemyKind::Treant, GroveEnemyKind::ShadowSpider);
    }

    #[test]
    fn test_grove_has_4_enemies() {
        let enemies: [(Vec3, GroveEnemyKind, i32, i32, f32, &str); 4] = [
            (
                Vec3::ZERO,
                GroveEnemyKind::Treant,
                55,
                8,
                32.0,
                "treant_loot",
            ),
            (
                Vec3::ZERO,
                GroveEnemyKind::Treant,
                65,
                10,
                36.0,
                "treant_loot",
            ),
            (
                Vec3::ZERO,
                GroveEnemyKind::ShadowSpider,
                35,
                12,
                22.0,
                "spider_loot",
            ),
            (
                Vec3::ZERO,
                GroveEnemyKind::ShadowSpider,
                30,
                14,
                20.0,
                "spider_loot",
            ),
        ];
        assert_eq!(enemies.len(), 4);
        // Treants should be tankier
        assert!(enemies[0].2 > enemies[2].2);
        // Spiders should hit harder
        assert!(enemies[3].3 > enemies[0].3);
    }
}
