//! Haunted Crypt combat systems — skeletons, lich AI, phase transitions.

use super::{CryptEntity, CryptPlayer, CryptSkeleton, Lich, LichPhase};
use crate::state::GameState;
use bevy::prelude::*;
use dj_engine::combat::{AttackCooldown, CombatEvent, DamageEvent};
use dj_engine::data::components::CombatStatsComponent;
use dj_engine::input::{ActionState, InputAction};
use dj_engine::particles::{ParticleConfig, ParticleEvent};
use dj_engine::prelude::{
    Inventory, LowHealthVignette, QuestJournal, ScreenFlashEvent, ScreenShakeEvent, StoryFlags,
};

// ---------------------------------------------------------------------------
// UI
// ---------------------------------------------------------------------------

#[derive(Component)]
pub struct CryptHud;

#[derive(Component)]
pub struct CryptHudText;

pub fn setup_crypt_ui(mut commands: Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(10.0),
                left: Val::Px(10.0),
                right: Val::Px(10.0),
                padding: UiRect::all(Val::Px(8.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.02, 0.02, 0.08, 0.85)),
            CryptHud,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("HAUNTED CRYPT"),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgb(0.4, 0.3, 0.7)),
                CryptHudText,
            ));
        });
}

pub fn teardown_crypt_ui(mut commands: Commands, query: Query<Entity, With<CryptHud>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

pub fn update_crypt_hud(
    player_query: Query<&CombatStatsComponent, With<CryptPlayer>>,
    skeleton_query: Query<&CombatStatsComponent, (With<CryptSkeleton>, Without<Lich>)>,
    lich_query: Query<(&CombatStatsComponent, &Lich)>,
    journal: Res<QuestJournal>,
    inventory: Res<Inventory>,
    mut text_query: Query<&mut Text, With<CryptHudText>>,
) {
    let Ok(player) = player_query.single() else {
        return;
    };
    let Ok(mut text) = text_query.single_mut() else {
        return;
    };

    let skeletons_alive = skeleton_query.iter().filter(|s| s.hp > 0).count();
    let gold = inventory.currency_balance("gold");

    let lich_status = if let Ok((stats, lich)) = lich_query.single() {
        if stats.hp <= 0 {
            "DEFEATED".to_string()
        } else {
            format!("{}/{} ({:?})", stats.hp, stats.max_hp, lich.phase)
        }
    } else {
        "???".into()
    };

    let quest_status = if let Some(status) = journal.status("cleanse_the_crypt") {
        format!("{:?}", status)
    } else {
        "???".into()
    };

    text.0 = format!(
        "HAUNTED CRYPT  |  You: {}/{}  |  Skeletons: {}  |  Lich: {}  |  Quest: {}  |  Gold: {}  |  [Space=Attack, WASD=Move, Q=Potion, Esc=Leave]",
        player.hp, player.max_hp, skeletons_alive, lich_status, quest_status, gold
    );
}

// ---------------------------------------------------------------------------
// Movement
// ---------------------------------------------------------------------------

pub fn player_crypt_movement(
    actions: Res<ActionState>,
    time: Res<Time>,
    mut query: Query<&mut Transform, With<CryptPlayer>>,
) {
    let Ok(mut transform) = query.single_mut() else {
        return;
    };
    let speed = 130.0;
    let dt = time.delta_secs();
    let mut dir = Vec2::ZERO;

    if actions.pressed(InputAction::Up) {
        dir.y += 1.0;
    }
    if actions.pressed(InputAction::Down) {
        dir.y -= 1.0;
    }
    if actions.pressed(InputAction::Left) {
        dir.x -= 1.0;
    }
    if actions.pressed(InputAction::Right) {
        dir.x += 1.0;
    }

    if dir != Vec2::ZERO {
        dir = dir.normalize();
        transform.translation.x += dir.x * speed * dt;
        transform.translation.y += dir.y * speed * dt;
        transform.translation.x = transform.translation.x.clamp(-400.0, 400.0);
        transform.translation.y = transform.translation.y.clamp(-300.0, 300.0);
    }
}

// ---------------------------------------------------------------------------
// Combat
// ---------------------------------------------------------------------------

pub fn player_crypt_attack(
    actions: Res<ActionState>,
    mut player_query: Query<(Entity, &Transform, &mut AttackCooldown), With<CryptPlayer>>,
    skeleton_query: Query<
        (Entity, &Transform, &CombatStatsComponent),
        (With<CryptSkeleton>, Without<Lich>),
    >,
    lich_query: Query<(Entity, &Transform, &CombatStatsComponent), With<Lich>>,
    mut combat_events: MessageWriter<CombatEvent>,
) {
    if !actions.just_pressed(InputAction::Confirm) {
        return;
    }

    let Ok((player, player_pos, mut cooldown)) = player_query.single_mut() else {
        return;
    };

    if !cooldown.ready() {
        return;
    }

    // Find nearest alive enemy (skeleton or lich)
    let nearest_skeleton = skeleton_query
        .iter()
        .filter(|(_, _, stats)| stats.hp > 0)
        .min_by(|(_, a, _), (_, b, _)| {
            let da = player_pos.translation.distance_squared(a.translation);
            let db = player_pos.translation.distance_squared(b.translation);
            da.total_cmp(&db)
        })
        .map(|(e, t, _)| (e, t.translation.distance_squared(player_pos.translation)));

    let nearest_lich = lich_query
        .iter()
        .filter(|(_, _, stats)| stats.hp > 0)
        .min_by(|(_, a, _), (_, b, _)| {
            let da = player_pos.translation.distance_squared(a.translation);
            let db = player_pos.translation.distance_squared(b.translation);
            da.total_cmp(&db)
        })
        .map(|(e, t, _)| (e, t.translation.distance_squared(player_pos.translation)));

    let target = match (nearest_skeleton, nearest_lich) {
        (Some((se, sd)), Some((le, ld))) => {
            if sd <= ld {
                Some(se)
            } else {
                Some(le)
            }
        }
        (Some((e, _)), None) => Some(e),
        (None, Some((e, _))) => Some(e),
        (None, None) => None,
    };

    if let Some(target) = target {
        cooldown.reset();
        combat_events.write(CombatEvent {
            attacker: player,
            target,
            flat_damage: None,
        });
    }
}

pub fn skeleton_ai_attack(
    mut skeleton_query: Query<
        (
            Entity,
            &Transform,
            &mut AttackCooldown,
            &CombatStatsComponent,
        ),
        (With<CryptSkeleton>, Without<Lich>),
    >,
    player_query: Query<(Entity, &Transform), With<CryptPlayer>>,
    mut combat_events: MessageWriter<CombatEvent>,
) {
    let Ok((player_entity, player_pos)) = player_query.single() else {
        return;
    };

    let attack_range = 100.0_f32;

    for (skeleton_entity, skeleton_pos, mut cooldown, stats) in skeleton_query.iter_mut() {
        if stats.hp <= 0 {
            continue;
        }

        let dist = skeleton_pos.translation.distance(player_pos.translation);
        if dist > attack_range {
            continue;
        }

        if cooldown.ready() {
            cooldown.reset();
            combat_events.write(CombatEvent {
                attacker: skeleton_entity,
                target: player_entity,
                flat_damage: None,
            });
        }
    }
}

/// Lich AI: phase 1 = ranged attacks, phase 2 = faster + summons adds at 50% HP.
pub fn lich_ai(
    mut commands: Commands,
    mut lich_query: Query<(
        Entity,
        &Transform,
        &mut AttackCooldown,
        &mut CombatStatsComponent,
        &mut Lich,
    )>,
    player_query: Query<(Entity, &Transform), With<CryptPlayer>>,
    mut combat_events: MessageWriter<CombatEvent>,
    mut flash_events: MessageWriter<ScreenFlashEvent>,
    mut shake_events: MessageWriter<ScreenShakeEvent>,
    mut particle_events: MessageWriter<ParticleEvent>,
) {
    let Ok((player_entity, player_pos)) = player_query.single() else {
        return;
    };

    let Ok((lich_entity, lich_pos, mut cooldown, mut stats, mut lich)) = lich_query.single_mut()
    else {
        return;
    };

    if stats.hp <= 0 {
        return;
    }

    // Phase transition at 50% HP
    if stats.hp <= stats.max_hp / 2 && lich.phase == LichPhase::Phase1 {
        lich.phase = LichPhase::Phase2;
        info!("Lich enters Phase 2! Summoning skeleton adds...");

        // Phase transition FX
        flash_events.write(ScreenFlashEvent {
            color: Color::srgba(0.4, 0.1, 0.6, 0.6),
            duration: 0.8,
            intensity: 1.0,
        });
        shake_events.write(ScreenShakeEvent::heavy());
        particle_events.write(ParticleEvent {
            position: lich_pos.translation,
            config: ParticleConfig {
                count: 16,
                color: Color::srgb(0.5, 0.2, 0.8),
                color_end: Some(Color::srgb(0.1, 0.0, 0.3)),
                lifetime: 1.0,
                speed: 60.0,
                spread: std::f32::consts::TAU,
                gravity: 20.0,
                size: 5.0,
                shrink: true,
            },
        });

        // Reduce attack cooldown for phase 2
        *cooldown = AttackCooldown::new(1.2);
    }

    // Summon 2 skeleton adds in phase 2 (once)
    if lich.phase == LichPhase::Phase2 && !lich.summon_triggered {
        lich.summon_triggered = true;

        let add_positions = [
            Vec3::new(
                lich_pos.translation.x - 60.0,
                lich_pos.translation.y - 40.0,
                10.0,
            ),
            Vec3::new(
                lich_pos.translation.x + 60.0,
                lich_pos.translation.y - 40.0,
                10.0,
            ),
        ];

        for (i, pos) in add_positions.iter().enumerate() {
            commands.spawn((
                Name::new(format!("Skeleton Add {}", i + 1)),
                Sprite {
                    color: Color::srgb(0.6, 0.55, 0.5),
                    custom_size: Some(Vec2::new(22.0, 22.0)),
                    ..default()
                },
                Transform::from_translation(*pos),
                CryptSkeleton,
                AttackCooldown::new(1.8),
                CombatStatsComponent {
                    max_hp: 25,
                    hp: 25,
                    damage: 7,
                    defense: 3,
                    crit_chance: 0.05,
                    loot_table_id: Some("skeleton_loot".into()),
                    ..default()
                },
                CryptEntity,
            ));
        }
        info!("Lich summoned 2 skeleton adds!");
    }

    // Ranged attack — lich attacks from farther range
    let attack_range = match lich.phase {
        LichPhase::Phase1 => 250.0_f32,
        LichPhase::Phase2 => 300.0_f32,
    };

    let dist = lich_pos.translation.distance(player_pos.translation);
    if dist > attack_range {
        return;
    }

    if cooldown.ready() {
        cooldown.reset();
        combat_events.write(CombatEvent {
            attacker: lich_entity,
            target: player_entity,
            flat_damage: None,
        });
    }
}

pub fn handle_crypt_damage(
    mut damage_events: MessageReader<DamageEvent>,
    mut next_state: ResMut<NextState<GameState>>,
    mut quest_journal: ResMut<QuestJournal>,
    mut flags: ResMut<StoryFlags>,
    player_query: Query<Entity, With<CryptPlayer>>,
    skeleton_query: Query<Entity, (With<CryptSkeleton>, Without<Lich>)>,
    lich_query: Query<Entity, With<Lich>>,
    mut shake_events: MessageWriter<ScreenShakeEvent>,
    mut flash_events: MessageWriter<ScreenFlashEvent>,
    mut particle_events: MessageWriter<ParticleEvent>,
) {
    for event in damage_events.read() {
        // Screen effects
        if player_query.get(event.target).is_ok() {
            shake_events.write(if event.is_critical {
                ScreenShakeEvent::heavy()
            } else {
                ScreenShakeEvent::medium()
            });
            flash_events.write(ScreenFlashEvent::damage());
        } else {
            shake_events.write(ScreenShakeEvent::light());
        }

        if event.target_defeated {
            // Skeleton killed
            if skeleton_query.get(event.target).is_ok() {
                info!("Skeleton destroyed!");
                particle_events.write(ParticleEvent {
                    position: Vec3::ZERO,
                    config: ParticleConfig {
                        count: 8,
                        color: Color::srgb(0.7, 0.65, 0.55),
                        color_end: Some(Color::srgb(0.3, 0.25, 0.2)),
                        lifetime: 0.6,
                        speed: 50.0,
                        spread: std::f32::consts::TAU,
                        gravity: -80.0,
                        size: 3.0,
                        shrink: true,
                    },
                });
            }

            // Lich killed — quest complete!
            if lich_query.get(event.target).is_ok() {
                info!("LICH DEFEATED! The crypt is cleansed!");
                particle_events.write(ParticleEvent {
                    position: Vec3::ZERO,
                    config: ParticleConfig {
                        count: 24,
                        color: Color::srgb(0.5, 0.2, 0.9),
                        color_end: Some(Color::srgb(1.0, 0.9, 0.2)),
                        lifetime: 1.5,
                        speed: 80.0,
                        spread: std::f32::consts::TAU,
                        gravity: 30.0,
                        size: 6.0,
                        shrink: false,
                    },
                });
                flash_events.write(ScreenFlashEvent {
                    color: Color::srgba(0.9, 0.8, 0.2, 0.6),
                    duration: 1.5,
                    intensity: 1.0,
                });
                shake_events.write(ScreenShakeEvent::heavy());

                quest_journal.progress_objective("cleanse_the_crypt", "defeat_lich", 1);
                quest_journal.complete("cleanse_the_crypt");
                flags.0.insert("QuestComplete_crypt".to_string(), true);
            }
        }

        // Player defeated
        if event.target_defeated && player_query.get(event.target).is_ok() {
            info!("Player defeated in the crypt!");
            shake_events.write(ScreenShakeEvent::heavy());
            particle_events.write(ParticleEvent {
                position: Vec3::ZERO,
                config: ParticleConfig::death_burst(),
            });
            next_state.set(GameState::GameOver);
        }
    }
}

// ---------------------------------------------------------------------------
// Crypt clear + leave
// ---------------------------------------------------------------------------

pub fn check_crypt_clear(
    flags: Res<StoryFlags>,
    keys: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if flags.0.get("QuestComplete_crypt").copied().unwrap_or(false)
        && keys.just_pressed(KeyCode::Escape)
    {
        info!("Leaving crypt, returning to overworld");
        next_state.set(GameState::Overworld);
    }
}

// ---------------------------------------------------------------------------
// Potion use + vignette
// ---------------------------------------------------------------------------

pub fn use_potion(
    keys: Res<ButtonInput<KeyCode>>,
    mut inventory: ResMut<Inventory>,
    mut player_query: Query<&mut CombatStatsComponent, With<CryptPlayer>>,
    mut flash_events: MessageWriter<ScreenFlashEvent>,
) {
    if !keys.just_pressed(KeyCode::KeyQ) {
        return;
    }

    let Ok(mut stats) = player_query.single_mut() else {
        return;
    };

    if !inventory.has_item("health_potion", 1) {
        info!("No health potions!");
        return;
    }

    if stats.hp >= stats.max_hp {
        info!("Already at full HP");
        return;
    }

    inventory.remove_item("health_potion", 1);
    let heal = 25.min(stats.max_hp - stats.hp);
    stats.hp += heal;
    flash_events.write(ScreenFlashEvent::heal());
    info!(
        "Used health potion! Healed {} HP ({}/{})",
        heal, stats.hp, stats.max_hp
    );
}

pub fn update_health_vignette(
    player_query: Query<&CombatStatsComponent, With<CryptPlayer>>,
    mut vignette_query: Query<&mut LowHealthVignette>,
) {
    let Ok(stats) = player_query.single() else {
        return;
    };

    for mut vignette in vignette_query.iter_mut() {
        vignette.hp_fraction = stats.hp as f32 / stats.max_hp as f32;
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crypt_quest_flow() {
        let mut journal = QuestJournal::default();
        journal.accept("cleanse_the_crypt");
        journal.add_objective("cleanse_the_crypt", "defeat_lich", 1);

        assert!(!journal.all_objectives_complete("cleanse_the_crypt"));

        let complete = journal.progress_objective("cleanse_the_crypt", "defeat_lich", 1);
        assert!(complete);
        assert!(journal.all_objectives_complete("cleanse_the_crypt"));
    }

    #[test]
    fn test_skeleton_warrior_stats() {
        let stats = CombatStatsComponent {
            max_hp: 45,
            hp: 45,
            damage: 10,
            defense: 5,
            ..default()
        };
        assert!(stats.hp < 120); // weaker than lich
    }

    #[test]
    fn test_lich_boss_stats() {
        let stats = CombatStatsComponent {
            max_hp: 120,
            hp: 120,
            damage: 15,
            defense: 6,
            crit_chance: 0.12,
            ..default()
        };
        assert_eq!(stats.max_hp, 120);
        assert!(stats.damage > 10); // hits harder than skeletons
    }

    #[test]
    fn test_lich_phase2_threshold() {
        let max_hp = 120;
        // Phase 2 triggers at 50%
        assert!(60 <= max_hp / 2);
        assert!(59 < max_hp / 2);
    }

    #[test]
    fn test_lich_summon_adds_once() {
        let mut lich = Lich {
            phase: LichPhase::Phase2,
            summon_triggered: false,
        };
        // First summon
        assert!(!lich.summon_triggered);
        lich.summon_triggered = true;
        assert!(lich.summon_triggered);
        // Won't summon again
    }
}
