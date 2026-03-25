use crate::hamster::components::{CharacterRoot, Expression};
use crate::state::GameState;
use crate::story::{BattlePending, StoryState};
use bevy::prelude::*;
use dj_engine::combat::{AttackCooldown, CombatEvent, DamageEvent};
use dj_engine::data::components::CombatStatsComponent;
use dj_engine::input::{ActionState, InputAction};

/// Marker for the player's battle entity.
#[derive(Component)]
pub struct BattlePlayer;

/// Marker for the enemy's battle entity.
#[derive(Component)]
pub struct BattleEnemy;

/// Brief input lockout at battle start to prevent dialogue Space carrying over.
#[derive(Resource)]
pub struct BattleInputDelay(pub Timer);

/// Spawn battle entities with real combat stats.
pub fn setup_battle_entities(mut commands: Commands, mut battle_pending: ResMut<BattlePending>) {
    battle_pending.0 = false; // Clear the flag now that we've entered battle
    commands.spawn((
        BattlePlayer,
        CombatStatsComponent {
            max_hp: 80,
            hp: 80,
            mana: 30,
            damage: 10,
            defense: 5,
            crit_chance: 0.1,
            ..default()
        },
        AttackCooldown::new(1.0),
        Name::new("battle_player"),
    ));

    commands.spawn((
        BattleEnemy,
        CombatStatsComponent {
            max_hp: 80,
            hp: 80,
            mana: 0,
            damage: 8,
            defense: 4,
            crit_chance: 0.05,
            loot_table_id: Some("glitch_loot".into()),
            ..default()
        },
        Name::new("battle_enemy"),
    ));

    commands.insert_resource(BattleInputDelay(Timer::from_seconds(0.3, TimerMode::Once)));

    info!("Battle: entities spawned — press Space to attack!");
}

/// Player attacks when pressing Confirm (Space), gated by AttackCooldown.
pub fn player_attack(
    time: Res<Time>,
    actions: Res<ActionState>,
    mut player_query: Query<(Entity, &mut AttackCooldown), With<BattlePlayer>>,
    enemy_query: Query<Entity, With<BattleEnemy>>,
    mut combat_events: MessageWriter<CombatEvent>,
    mut delay: Option<ResMut<BattleInputDelay>>,
) {
    // Block input briefly at battle start
    if let Some(ref mut d) = delay {
        d.0.tick(time.delta());
        if !d.0.is_finished() {
            return;
        }
    }

    if !actions.just_pressed(InputAction::Confirm) {
        return;
    }

    let Ok((player, mut cooldown)) = player_query.single_mut() else {
        return;
    };

    if !cooldown.ready() {
        return;
    }

    if let Some(enemy) = enemy_query.iter().next() {
        cooldown.reset();
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
    mut flags: ResMut<dj_engine::story_graph::StoryFlags>,
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
            // Write to BOTH flag systems so graph executor + HUD tracker both see it
            story.add_flag("DefeatedGlitch");
            flags.0.insert("DefeatedGlitch".to_string(), true);
            info!("STATE: Battle -> Overworld (victory)");
            next_state.set(GameState::Overworld);
        }

        // Player defeated → defeat
        if event.target_defeated && player_query.get(event.target).is_ok() {
            info!("Battle LOST! Took {} damage", event.final_damage);
            for mut hamster in &mut hamster_query {
                hamster.expression = Expression::Angry;
                hamster.corruption = (hamster.corruption + 15.0).min(100.0);
            }
            info!("STATE: Battle -> Overworld (defeat)");
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
