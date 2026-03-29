//! Cellar combat, quest progress, loot, and chest systems.

use super::{CellarEntity, CellarPlayer, CellarRat, WeaponChest};
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
pub struct CellarHud;

#[derive(Component)]
pub struct CellarHudText;

pub fn setup_cellar_ui(mut commands: Commands) {
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
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
            CellarHud,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("CELLAR"),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgb(0.8, 0.6, 0.2)),
                CellarHudText,
            ));
        });
}

pub fn teardown_cellar_ui(mut commands: Commands, query: Query<Entity, With<CellarHud>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

pub fn update_cellar_hud(
    player_query: Query<&CombatStatsComponent, With<CellarPlayer>>,
    rat_query: Query<&CombatStatsComponent, With<CellarRat>>,
    journal: Res<QuestJournal>,
    inventory: Res<Inventory>,
    mut text_query: Query<&mut Text, With<CellarHudText>>,
) {
    let Ok(player) = player_query.single() else {
        return;
    };
    let Ok(mut text) = text_query.single_mut() else {
        return;
    };

    let rats_alive = rat_query.iter().filter(|s| s.hp > 0).count();
    let gold = inventory.currency_balance("gold");

    let quest_status = if let Some(status) = journal.status("clear_the_cellar") {
        format!("{:?}", status)
    } else {
        "???".into()
    };

    text.0 = format!(
        "CELLAR  |  You: {}/{}  |  Rats: {}  |  Quest: {}  |  Gold: {}  |  [Space=Attack, WASD=Move]",
        player.hp, player.max_hp, rats_alive, quest_status, gold
    );
}

// ---------------------------------------------------------------------------
// Movement
// ---------------------------------------------------------------------------

pub fn player_cellar_movement(
    actions: Res<ActionState>,
    time: Res<Time>,
    mut query: Query<&mut Transform, With<CellarPlayer>>,
) {
    let Ok(mut transform) = query.single_mut() else {
        return;
    };
    let speed = 120.0;
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
        // Clamp to cellar bounds
        transform.translation.x = transform.translation.x.clamp(-350.0, 350.0);
        transform.translation.y = transform.translation.y.clamp(-250.0, 250.0);
    }
}

// ---------------------------------------------------------------------------
// Combat
// ---------------------------------------------------------------------------

pub fn player_cellar_attack(
    time: Res<Time>,
    actions: Res<ActionState>,
    mut player_query: Query<(Entity, &Transform, &mut AttackCooldown), With<CellarPlayer>>,
    rat_query: Query<(Entity, &Transform, &CombatStatsComponent), With<CellarRat>>,
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

    // Attack nearest alive rat
    let nearest = rat_query
        .iter()
        .filter(|(_, _, stats)| stats.hp > 0)
        .min_by(|(_, a, _), (_, b, _)| {
            let da = player_pos.translation.distance_squared(a.translation);
            let db = player_pos.translation.distance_squared(b.translation);
            da.total_cmp(&db)
        });

    if let Some((rat_entity, _, _)) = nearest {
        cooldown.reset();
        combat_events.write(CombatEvent {
            attacker: player,
            target: rat_entity,
            flat_damage: None,
        });
    }
}

pub fn rat_ai_attack(
    mut rat_query: Query<
        (
            Entity,
            &Transform,
            &mut AttackCooldown,
            &CombatStatsComponent,
        ),
        With<CellarRat>,
    >,
    player_query: Query<(Entity, &Transform), With<CellarPlayer>>,
    mut combat_events: MessageWriter<CombatEvent>,
) {
    let Ok((player_entity, player_pos)) = player_query.single() else {
        return;
    };

    let attack_range = 100.0_f32;

    for (rat_entity, rat_pos, mut cooldown, stats) in rat_query.iter_mut() {
        if stats.hp <= 0 {
            continue;
        }

        let dist = rat_pos.translation.distance(player_pos.translation);
        if dist > attack_range {
            continue;
        }

        if cooldown.ready() {
            cooldown.reset();
            combat_events.write(CombatEvent {
                attacker: rat_entity,
                target: player_entity,
                flat_damage: None,
            });
        }
    }
}

pub fn handle_cellar_damage(
    mut damage_events: MessageReader<DamageEvent>,
    mut next_state: ResMut<NextState<GameState>>,
    mut quest_journal: ResMut<QuestJournal>,
    mut flags: ResMut<StoryFlags>,
    player_query: Query<Entity, With<CellarPlayer>>,
    rat_query: Query<Entity, With<CellarRat>>,
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

        // Rat killed — progress quest
        if event.target_defeated && rat_query.get(event.target).is_ok() {
            info!("Rat defeated!");
            particle_events.write(ParticleEvent {
                position: Vec3::ZERO,
                config: ParticleConfig::hit_sparks(),
            });

            let complete = quest_journal.progress_objective("clear_the_cellar", "kill_rats", 1);
            if complete {
                info!("All rats cleared! Quest objective complete.");
                quest_journal.complete("clear_the_cellar");
                flags.0.insert("QuestComplete_cellar".to_string(), true);
                flash_events.write(ScreenFlashEvent::gold());
            }
        }

        // Player defeated
        if event.target_defeated && player_query.get(event.target).is_ok() {
            info!("Player defeated in cellar!");
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
// Cellar clear check + chest
// ---------------------------------------------------------------------------

pub fn check_cellar_clear(
    flags: Res<StoryFlags>,
    keys: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    // After quest is complete, press Escape to return to village
    if flags
        .0
        .get("QuestComplete_cellar")
        .copied()
        .unwrap_or(false)
        && keys.just_pressed(KeyCode::Escape)
    {
        info!("Leaving cellar, returning to overworld");
        next_state.set(GameState::Overworld);
    }
}

pub fn chest_interaction(
    actions: Res<ActionState>,
    player_query: Query<&Transform, With<CellarPlayer>>,
    mut chest_query: Query<(&Transform, &mut WeaponChest, &mut Sprite)>,
    mut inventory: ResMut<Inventory>,
    mut flash_events: MessageWriter<ScreenFlashEvent>,
) {
    if !actions.just_pressed(InputAction::Confirm) {
        return;
    }

    let Ok(player_pos) = player_query.single() else {
        return;
    };

    for (chest_pos, mut chest, mut sprite) in chest_query.iter_mut() {
        if chest.opened {
            continue;
        }

        let dist = player_pos.translation.distance(chest_pos.translation);
        if dist < 50.0 {
            chest.opened = true;
            sprite.color = Color::srgb(0.3, 0.25, 0.05); // Dim opened chest
            inventory.add_item("rusty_sword", 1, 1);
            flash_events.write(ScreenFlashEvent::gold());
            info!("Opened chest! Found: Rusty Sword (+3 damage)");
        }
    }
}

// ---------------------------------------------------------------------------
// Potion use + vignette
// ---------------------------------------------------------------------------

pub fn use_potion(
    keys: Res<ButtonInput<KeyCode>>,
    mut inventory: ResMut<Inventory>,
    mut player_query: Query<&mut CombatStatsComponent, With<CellarPlayer>>,
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
    player_query: Query<&CombatStatsComponent, With<CellarPlayer>>,
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
    fn test_rat_stats() {
        let stats = CombatStatsComponent {
            max_hp: 30,
            hp: 30,
            damage: 5,
            defense: 2,
            crit_chance: 0.05,
            loot_table_id: Some("rat_loot".into()),
            ..default()
        };
        assert_eq!(stats.hp, 30);
        assert_eq!(stats.damage, 5);
        assert_eq!(stats.loot_table_id, Some("rat_loot".into()));
    }

    #[test]
    fn test_chest_starts_closed() {
        let chest = WeaponChest { opened: false };
        assert!(!chest.opened);
    }

    #[test]
    fn test_potion_heals_25_capped_at_max() {
        let max_hp = 80;
        let hp = 70;
        let heal = 25_i32.min(max_hp - hp);
        assert_eq!(heal, 10); // Only heals the deficit

        let hp2 = 30;
        let heal2 = 25_i32.min(max_hp - hp2);
        assert_eq!(heal2, 25); // Full heal amount
    }

    #[test]
    fn test_cellar_has_5_rats() {
        let rat_data: [(Vec3, f32, i32, i32); 5] = [
            (Vec3::new(-120.0, 60.0, 10.0), 24.0, 30, 5),
            (Vec3::new(80.0, 100.0, 10.0), 24.0, 30, 5),
            (Vec3::new(0.0, -20.0, 10.0), 24.0, 30, 5),
            (Vec3::new(-60.0, 130.0, 10.0), 20.0, 20, 4),
            (Vec3::new(120.0, 40.0, 10.0), 30.0, 45, 7),
        ];
        assert_eq!(rat_data.len(), 5);
        // Alpha rat has more HP
        assert!(rat_data[4].2 > rat_data[0].2);
        // Small rat is smaller
        assert!(rat_data[3].1 < rat_data[0].1);
    }

    #[test]
    fn test_quest_progress_tracking() {
        let mut journal = QuestJournal::default();
        journal.accept("clear_the_cellar");
        journal.add_objective("clear_the_cellar", "kill_rats", 5);

        // Kill 4 rats
        for _ in 0..4 {
            journal.progress_objective("clear_the_cellar", "kill_rats", 1);
        }
        assert!(!journal.all_objectives_complete("clear_the_cellar"));

        // Kill 5th rat
        let complete = journal.progress_objective("clear_the_cellar", "kill_rats", 1);
        assert!(complete);
        assert!(journal.all_objectives_complete("clear_the_cellar"));
    }
}
