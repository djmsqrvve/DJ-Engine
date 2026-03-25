use crate::hamster::components::{CharacterRoot, Expression};
use crate::state::GameState;
use crate::story::StoryState;
use bevy::prelude::*;
use dj_engine::combat::{CombatEvent, DamageEvent};
use dj_engine::data::components::CombatStatsComponent;
use dj_engine::input::{ActionState, InputAction};

/// Marker for the player's battle entity.
#[derive(Component)]
pub struct BattlePlayer;

/// Marker for the enemy's battle entity.
#[derive(Component)]
pub struct BattleEnemy;

/// Spawn battle entities with real combat stats.
pub fn setup_battle_entities(mut commands: Commands) {
    commands.spawn((
        BattlePlayer,
        CombatStatsComponent {
            max_hp: 80,
            hp: 80,
            mana: 30,
            damage: 20,
            defense: 5,
            crit_chance: 0.15,
            ..default()
        },
        Name::new("battle_player"),
    ));

    commands.spawn((
        BattleEnemy,
        CombatStatsComponent {
            max_hp: 40,
            hp: 40,
            mana: 0,
            damage: 12,
            defense: 3,
            crit_chance: 0.05,
            loot_table_id: Some("glitch_loot".into()),
            ..default()
        },
        Name::new("battle_enemy"),
    ));

    info!("Battle: entities spawned with real combat stats");
}

/// Player attacks when pressing Confirm (Space).
pub fn player_attack(
    actions: Res<ActionState>,
    player_query: Query<Entity, With<BattlePlayer>>,
    enemy_query: Query<Entity, With<BattleEnemy>>,
    mut combat_events: MessageWriter<CombatEvent>,
) {
    if !actions.just_pressed(InputAction::Confirm) {
        return;
    }

    let Ok(player) = player_query.single() else {
        return;
    };

    if let Some(enemy) = enemy_query.iter().next() {
        combat_events.write(CombatEvent {
            attacker: player,
            target: enemy,
            flat_damage: None,
        });
    }
}

/// Enemy attacks back after player attacks (simple turn-based).
pub fn enemy_counterattack(
    mut damage_events: MessageReader<DamageEvent>,
    player_query: Query<Entity, With<BattlePlayer>>,
    enemy_query: Query<Entity, With<BattleEnemy>>,
    mut combat_events: MessageWriter<CombatEvent>,
) {
    for event in damage_events.read() {
        // Only counterattack if the enemy was hit and survived
        if enemy_query.get(event.target).is_ok() && !event.target_defeated {
            if let Ok(player) = player_query.single() {
                combat_events.write(CombatEvent {
                    attacker: event.target,
                    target: player,
                    flat_damage: None,
                });
            }
        }
    }
}

/// Handle combat results — check for victory/defeat, update hamster.
pub fn handle_battle_damage(
    mut damage_events: MessageReader<DamageEvent>,
    mut hamster_query: Query<&mut CharacterRoot>,
    mut story: ResMut<StoryState>,
    mut next_state: ResMut<NextState<GameState>>,
    player_query: Query<Entity, With<BattlePlayer>>,
    enemy_query: Query<Entity, With<BattleEnemy>>,
) {
    for event in damage_events.read() {
        // Enemy defeated → victory
        if event.target_defeated && enemy_query.get(event.target).is_ok() {
            info!(
                "Battle WON! Final blow: {} damage (crit={})",
                event.final_damage, event.is_critical
            );
            for mut hamster in &mut hamster_query {
                hamster.expression = Expression::Happy;
                hamster.corruption = (hamster.corruption - 10.0).max(0.0);
            }
            story.add_flag("DefeatedGlitch");
            next_state.set(GameState::Overworld);
        }

        // Player defeated → defeat
        if event.target_defeated && player_query.get(event.target).is_ok() {
            info!("Battle LOST! Took {} damage", event.final_damage);
            for mut hamster in &mut hamster_query {
                hamster.expression = Expression::Angry;
                hamster.corruption = (hamster.corruption + 15.0).min(100.0);
            }
            next_state.set(GameState::Overworld);
        }
    }
}

/// Clean up battle entities on exit.
pub fn cleanup_battle_entities(
    mut commands: Commands,
    player_query: Query<Entity, With<BattlePlayer>>,
    enemy_query: Query<Entity, With<BattleEnemy>>,
) {
    for entity in player_query.iter().chain(enemy_query.iter()) {
        commands.entity(entity).despawn();
    }
}
