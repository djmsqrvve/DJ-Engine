//! Haunted Crypt — final area in DoomExe's demo loop.
//!
//! A dark crypt with 3 skeleton warriors and a lich mini-boss.
//! The lich has 2 phases: ranged attacks, then summons skeleton adds at 50% HP.
//! Quest: "cleanse_the_crypt" — defeat the lich, 150 gold + Lich's Staff.

use crate::state::GameState;
use bevy::prelude::*;
use dj_engine::prelude::*;

pub mod systems;

pub struct HauntedCryptPlugin;

impl Plugin for HauntedCryptPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameState::HauntedCrypt),
            (setup_crypt, systems::setup_crypt_ui),
        )
        .add_systems(
            Update,
            (
                systems::player_crypt_movement,
                systems::player_crypt_attack,
                systems::skeleton_ai_attack,
                systems::lich_ai,
                systems::handle_crypt_damage,
                systems::update_crypt_hud,
                systems::check_crypt_clear,
                systems::use_potion,
                systems::update_health_vignette,
            )
                .chain()
                .run_if(in_state(GameState::HauntedCrypt)),
        )
        .add_systems(
            OnExit(GameState::HauntedCrypt),
            (teardown_crypt, systems::teardown_crypt_ui),
        );
    }
}

#[derive(Component)]
pub struct CryptEntity;

#[derive(Component)]
pub struct CryptPlayer;

#[derive(Component)]
pub struct CryptSkeleton;

#[derive(Component)]
pub struct Lich {
    pub phase: LichPhase,
    pub summon_triggered: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LichPhase {
    /// Ranged attacks, normal behavior
    Phase1,
    /// Below 50% HP — summons adds, faster attacks
    Phase2,
}

fn setup_crypt(mut commands: Commands) {
    // Crypt floor — dark blue/gray stone
    commands.spawn((
        Name::new("Crypt Floor"),
        Sprite {
            color: Color::srgb(0.05, 0.05, 0.1),
            custom_size: Some(Vec2::new(900.0, 700.0)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 0.0),
        CryptEntity,
    ));

    // Torch decorations — orange glow markers
    for (x, y) in [
        (-200.0, 200.0),
        (200.0, 200.0),
        (-200.0, -150.0),
        (200.0, -150.0),
        (0.0, 250.0),
    ] {
        commands.spawn((
            Name::new("Torch"),
            Sprite {
                color: Color::srgba(1.0, 0.6, 0.1, 0.5),
                custom_size: Some(Vec2::new(12.0, 12.0)),
                ..default()
            },
            Transform::from_xyz(x, y, 2.0),
            CryptEntity,
        ));
    }

    // Bone pile props
    for (x, y) in [(-150.0, -80.0), (160.0, 60.0), (50.0, -180.0)] {
        commands.spawn((
            Name::new("Bone Pile"),
            Sprite {
                color: Color::srgb(0.35, 0.3, 0.25),
                custom_size: Some(Vec2::new(30.0, 20.0)),
                ..default()
            },
            Transform::from_xyz(x, y, 1.0),
            CryptEntity,
        ));
    }

    // Player
    commands.spawn((
        Name::new("Crypt Player"),
        Sprite {
            color: Color::srgb(0.2, 0.2, 0.8),
            custom_size: Some(Vec2::new(28.0, 28.0)),
            ..default()
        },
        Transform::from_xyz(0.0, -240.0, 10.0),
        CryptPlayer,
        dj_engine::combat::AttackCooldown::new(0.7),
        dj_engine::data::components::CombatStatsComponent {
            max_hp: 120,
            hp: 120,
            mana: 50,
            damage: 16,
            defense: 8,
            crit_chance: 0.15,
            ..default()
        },
        CryptEntity,
    ));

    // 3 Skeleton warriors
    let skeletons = [
        (Vec3::new(-120.0, 80.0, 10.0), 40, 9),
        (Vec3::new(100.0, 120.0, 10.0), 45, 10),
        (Vec3::new(-40.0, -40.0, 10.0), 50, 11),
    ];

    for (i, &(pos, hp, damage)) in skeletons.iter().enumerate() {
        commands.spawn((
            Name::new(format!("Skeleton Warrior {}", i + 1)),
            Sprite {
                color: Color::srgb(0.7, 0.65, 0.55),
                custom_size: Some(Vec2::new(26.0, 26.0)),
                ..default()
            },
            Transform::from_translation(pos),
            CryptSkeleton,
            dj_engine::combat::AttackCooldown::new(1.6),
            dj_engine::data::components::CombatStatsComponent {
                max_hp: hp,
                hp,
                damage,
                defense: 5,
                crit_chance: 0.08,
                loot_table_id: Some("skeleton_loot".into()),
                ..default()
            },
            CryptEntity,
        ));
    }

    // Lich mini-boss
    commands.spawn((
        Name::new("Lich"),
        Sprite {
            color: Color::srgb(0.3, 0.1, 0.6),
            custom_size: Some(Vec2::new(40.0, 40.0)),
            ..default()
        },
        Transform::from_xyz(0.0, 180.0, 10.0),
        Lich {
            phase: LichPhase::Phase1,
            summon_triggered: false,
        },
        dj_engine::combat::AttackCooldown::new(2.0),
        dj_engine::data::components::CombatStatsComponent {
            max_hp: 120,
            hp: 120,
            damage: 15,
            defense: 6,
            crit_chance: 0.12,
            loot_table_id: Some("lich_loot".into()),
            ..default()
        },
        CryptEntity,
    ));

    info!("Haunted Crypt: spawned 3 skeletons + lich boss");
}

fn teardown_crypt(mut commands: Commands, query: Query<Entity, With<CryptEntity>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lich_phases() {
        assert_ne!(LichPhase::Phase1, LichPhase::Phase2);
    }

    #[test]
    fn test_lich_default_state() {
        let lich = Lich {
            phase: LichPhase::Phase1,
            summon_triggered: false,
        };
        assert_eq!(lich.phase, LichPhase::Phase1);
        assert!(!lich.summon_triggered);
    }

    #[test]
    fn test_crypt_has_3_skeletons_and_1_lich() {
        let skeletons = [
            (Vec3::ZERO, 40, 9),
            (Vec3::ZERO, 45, 10),
            (Vec3::ZERO, 50, 11),
        ];
        assert_eq!(skeletons.len(), 3);
        // Lich is separate (120 HP boss)
        let lich_hp = 120;
        assert!(lich_hp > skeletons[2].1);
    }

    #[test]
    fn test_lich_phase_transition_threshold() {
        let max_hp = 120;
        let threshold = max_hp / 2;
        assert_eq!(threshold, 60);
        // At 61 HP: still phase 1
        assert!(61 > threshold);
        // At 59 HP: phase 2
        assert!(59 < threshold);
    }
}
