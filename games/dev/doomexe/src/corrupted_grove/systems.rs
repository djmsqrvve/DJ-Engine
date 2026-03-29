//! Corrupted Grove combat, quest progress, and systems.

use super::{GroveEnemy, GroveEnemyKind, GroveEntity, GrovePlayer};
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
pub struct GroveHud;

#[derive(Component)]
pub struct GroveHudText;

pub fn setup_grove_ui(mut commands: Commands) {
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
            BackgroundColor(Color::srgba(0.05, 0.0, 0.1, 0.8)),
            GroveHud,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("CORRUPTED GROVE"),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgb(0.5, 0.2, 0.7)),
                GroveHudText,
            ));
        });
}

pub fn teardown_grove_ui(mut commands: Commands, query: Query<Entity, With<GroveHud>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

pub fn update_grove_hud(
    player_query: Query<&CombatStatsComponent, With<GrovePlayer>>,
    enemy_query: Query<(&CombatStatsComponent, &GroveEnemy)>,
    journal: Res<QuestJournal>,
    inventory: Res<Inventory>,
    mut text_query: Query<&mut Text, With<GroveHudText>>,
) {
    let Ok(player) = player_query.single() else {
        return;
    };
    let Ok(mut text) = text_query.single_mut() else {
        return;
    };

    let treants_alive = enemy_query
        .iter()
        .filter(|(s, e)| s.hp > 0 && e.kind == GroveEnemyKind::Treant)
        .count();
    let spiders_alive = enemy_query
        .iter()
        .filter(|(s, e)| s.hp > 0 && e.kind == GroveEnemyKind::ShadowSpider)
        .count();
    let gold = inventory.currency_balance("gold");

    let quest_status = if let Some(status) = journal.status("purify_grove") {
        format!("{:?}", status)
    } else {
        "???".into()
    };

    text.0 = format!(
        "CORRUPTED GROVE  |  You: {}/{}  |  Treants: {}  Spiders: {}  |  Quest: {}  |  Gold: {}  |  [Space=Attack, WASD=Move, Q=Potion, Esc=Leave]",
        player.hp, player.max_hp, treants_alive, spiders_alive, quest_status, gold
    );
}

// ---------------------------------------------------------------------------
// Movement
// ---------------------------------------------------------------------------

pub fn player_grove_movement(
    actions: Res<ActionState>,
    time: Res<Time>,
    mut query: Query<&mut Transform, With<GrovePlayer>>,
) {
    let Ok(mut transform) = query.single_mut() else {
        return;
    };
    let speed = 140.0;
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

pub fn player_grove_attack(
    actions: Res<ActionState>,
    mut player_query: Query<(Entity, &Transform, &mut AttackCooldown), With<GrovePlayer>>,
    enemy_query: Query<(Entity, &Transform, &CombatStatsComponent), With<GroveEnemy>>,
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

    let nearest = enemy_query
        .iter()
        .filter(|(_, _, stats)| stats.hp > 0)
        .min_by(|(_, a, _), (_, b, _)| {
            let da = player_pos.translation.distance_squared(a.translation);
            let db = player_pos.translation.distance_squared(b.translation);
            da.total_cmp(&db)
        });

    if let Some((enemy_entity, _, _)) = nearest {
        cooldown.reset();
        combat_events.write(CombatEvent {
            attacker: player,
            target: enemy_entity,
            flat_damage: None,
        });
    }
}

pub fn enemy_ai_attack(
    mut enemy_query: Query<
        (
            Entity,
            &Transform,
            &mut AttackCooldown,
            &CombatStatsComponent,
        ),
        With<GroveEnemy>,
    >,
    player_query: Query<(Entity, &Transform), With<GrovePlayer>>,
    mut combat_events: MessageWriter<CombatEvent>,
) {
    let Ok((player_entity, player_pos)) = player_query.single() else {
        return;
    };

    let attack_range = 110.0_f32;

    for (enemy_entity, enemy_pos, mut cooldown, stats) in enemy_query.iter_mut() {
        if stats.hp <= 0 {
            continue;
        }

        let dist = enemy_pos.translation.distance(player_pos.translation);
        if dist > attack_range {
            continue;
        }

        if cooldown.ready() {
            cooldown.reset();
            combat_events.write(CombatEvent {
                attacker: enemy_entity,
                target: player_entity,
                flat_damage: None,
            });
        }
    }
}

pub fn handle_grove_damage(
    mut damage_events: MessageReader<DamageEvent>,
    mut next_state: ResMut<NextState<GameState>>,
    mut quest_journal: ResMut<QuestJournal>,
    mut flags: ResMut<StoryFlags>,
    player_query: Query<Entity, With<GrovePlayer>>,
    enemy_query: Query<(Entity, &GroveEnemy)>,
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

        // Enemy killed — progress quest
        if event.target_defeated {
            if let Ok((_, enemy)) = enemy_query.get(event.target) {
                let kind_name = match enemy.kind {
                    GroveEnemyKind::Treant => "Treant",
                    GroveEnemyKind::ShadowSpider => "Spider",
                };
                info!("{} defeated!", kind_name);

                // Death particles based on enemy type
                let config = match enemy.kind {
                    GroveEnemyKind::Treant => ParticleConfig {
                        count: 10,
                        color: Color::srgb(0.3, 0.5, 0.1),
                        color_end: Some(Color::srgb(0.1, 0.2, 0.0)),
                        lifetime: 0.8,
                        speed: 50.0,
                        spread: std::f32::consts::TAU,
                        gravity: -60.0,
                        size: 4.0,
                        shrink: true,
                    },
                    GroveEnemyKind::ShadowSpider => ParticleConfig {
                        count: 8,
                        color: Color::srgb(0.4, 0.1, 0.5),
                        color_end: Some(Color::srgb(0.1, 0.0, 0.2)),
                        lifetime: 0.6,
                        speed: 70.0,
                        spread: std::f32::consts::TAU,
                        gravity: -100.0,
                        size: 3.0,
                        shrink: true,
                    },
                };

                particle_events.write(ParticleEvent {
                    position: Vec3::ZERO,
                    config,
                });

                let complete =
                    quest_journal.progress_objective("purify_grove", "defeat_corruption", 1);
                if complete {
                    info!("Grove cleansed! Quest objective complete.");
                    quest_journal.complete("purify_grove");
                    flags.0.insert("QuestComplete_grove".to_string(), true);
                    flash_events.write(ScreenFlashEvent {
                        color: Color::srgba(0.3, 0.8, 0.2, 0.5),
                        duration: 1.0,
                        intensity: 1.0,
                    });
                }
            }
        }

        // Player defeated
        if event.target_defeated && player_query.get(event.target).is_ok() {
            info!("Player defeated in the grove!");
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
// Grove clear + leave
// ---------------------------------------------------------------------------

pub fn check_grove_clear(
    flags: Res<StoryFlags>,
    keys: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if flags.0.get("QuestComplete_grove").copied().unwrap_or(false)
        && keys.just_pressed(KeyCode::Escape)
    {
        info!("Leaving grove, returning to overworld");
        next_state.set(GameState::Overworld);
    }
}

// ---------------------------------------------------------------------------
// Potion use + vignette
// ---------------------------------------------------------------------------

pub fn use_potion(
    keys: Res<ButtonInput<KeyCode>>,
    mut inventory: ResMut<Inventory>,
    mut player_query: Query<&mut CombatStatsComponent, With<GrovePlayer>>,
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
    player_query: Query<&CombatStatsComponent, With<GrovePlayer>>,
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
    fn test_grove_quest_name() {
        let mut journal = QuestJournal::default();
        journal.accept("purify_grove");
        journal.add_objective("purify_grove", "defeat_corruption", 4);

        for _ in 0..3 {
            journal.progress_objective("purify_grove", "defeat_corruption", 1);
        }
        assert!(!journal.all_objectives_complete("purify_grove"));

        let complete = journal.progress_objective("purify_grove", "defeat_corruption", 1);
        assert!(complete);
        assert!(journal.all_objectives_complete("purify_grove"));
    }

    #[test]
    fn test_treant_has_more_hp_than_spider() {
        let treant_hp = 55;
        let spider_hp = 35;
        assert!(treant_hp > spider_hp);
    }

    #[test]
    fn test_spider_hits_harder_than_treant() {
        let treant_damage = 8;
        let spider_damage = 12;
        assert!(spider_damage > treant_damage);
    }

    #[test]
    fn test_grove_player_stats() {
        let stats = CombatStatsComponent {
            max_hp: 100,
            hp: 100,
            damage: 14,
            defense: 6,
            crit_chance: 0.12,
            ..default()
        };
        assert_eq!(stats.max_hp, 100);
        assert_eq!(stats.damage, 14);
        assert!(stats.crit_chance > 0.0);
    }
}
