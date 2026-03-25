//! Combat formula system for DJ Engine.
//!
//! Provides damage calculation, hit resolution, and combat events.
//! Games use [`CombatEvent`] to trigger attacks and [`DamageEvent`] to react
//! to resolved damage. The formulas are configurable via [`CombatConfig`].

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::data::components::CombatStatsComponent;

/// Configuration for combat formulas.
#[derive(Resource, Debug, Clone, Serialize, Deserialize, Reflect)]
#[reflect(Resource)]
pub struct CombatConfig {
    /// Minimum damage dealt (before defense). Prevents zero-damage hits.
    pub min_damage: i32,
    /// Critical hit damage multiplier.
    pub crit_multiplier: f32,
    /// Defense reduction formula: damage = max(min_damage, attack - defense * defense_factor)
    pub defense_factor: f32,
    /// Variance range (0.0 = no variance, 0.2 = +/- 20%)
    pub variance: f32,
}

impl Default for CombatConfig {
    fn default() -> Self {
        Self {
            min_damage: 1,
            crit_multiplier: 2.0,
            defense_factor: 0.5,
            variance: 0.15,
        }
    }
}

/// Request an attack be calculated.
#[derive(Message, Debug, Clone, PartialEq)]
pub struct CombatEvent {
    /// The entity performing the attack.
    pub attacker: Entity,
    /// The entity being attacked.
    pub target: Entity,
    /// Optional flat damage override (bypasses attacker stats).
    pub flat_damage: Option<i32>,
}

/// Result of a resolved combat event — emitted after damage calculation.
#[derive(Message, Debug, Clone, PartialEq)]
pub struct DamageEvent {
    pub attacker: Entity,
    pub target: Entity,
    pub raw_damage: i32,
    pub final_damage: i32,
    pub is_critical: bool,
    pub target_hp_after: i32,
    pub target_defeated: bool,
}

/// Compute damage from an attacker's stats against a target's defense.
pub fn calculate_damage(
    config: &CombatConfig,
    attack: i32,
    defense: i32,
    crit_chance: f32,
    rng_roll: f32,
    variance_roll: f32,
) -> (i32, bool) {
    let is_crit = rng_roll < crit_chance;

    let base =
        (attack as f32 - defense as f32 * config.defense_factor).max(config.min_damage as f32);
    let varied = base * (1.0 + config.variance * (variance_roll * 2.0 - 1.0));
    let multiplied = if is_crit {
        varied * config.crit_multiplier
    } else {
        varied
    };

    (multiplied.round() as i32, is_crit)
}

/// System that processes combat events and applies damage.
pub fn resolve_combat(
    config: Res<CombatConfig>,
    mut combat_events: MessageReader<CombatEvent>,
    mut damage_events: MessageWriter<DamageEvent>,
    mut stats_query: Query<&mut CombatStatsComponent>,
) {
    for event in combat_events.read() {
        let (attack_damage, crit_chance) = if let Some(flat) = event.flat_damage {
            (flat, 0.0)
        } else if let Ok(attacker_stats) = stats_query.get(event.attacker) {
            (attacker_stats.damage, attacker_stats.crit_chance)
        } else {
            warn!(
                "Combat: attacker {:?} has no CombatStatsComponent",
                event.attacker
            );
            continue;
        };

        let defense = stats_query
            .get(event.target)
            .map(|s| s.defense)
            .unwrap_or(0);

        let rng_roll = rand::random::<f32>();
        let variance_roll = rand::random::<f32>();

        let (final_damage, is_critical) = calculate_damage(
            &config,
            attack_damage,
            defense,
            crit_chance,
            rng_roll,
            variance_roll,
        );

        // Apply damage to target
        let (hp_after, defeated) = if let Ok(mut target_stats) = stats_query.get_mut(event.target) {
            target_stats.hp = (target_stats.hp - final_damage).max(0);
            (target_stats.hp, target_stats.hp <= 0)
        } else {
            (0, true)
        };

        damage_events.write(DamageEvent {
            attacker: event.attacker,
            target: event.target,
            raw_damage: attack_damage,
            final_damage,
            is_critical,
            target_hp_after: hp_after,
            target_defeated: defeated,
        });
    }
}

/// Plugin providing combat formulas and damage resolution.
pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CombatConfig>()
            .register_type::<CombatConfig>()
            .add_message::<CombatEvent>()
            .add_message::<DamageEvent>()
            .add_systems(Update, resolve_combat);

        use crate::contracts::{AppContractExt, ContractEntry, PluginContract};
        app.register_contract(PluginContract {
            name: "CombatPlugin".into(),
            description: "Damage calculation and combat resolution".into(),
            resources: vec![ContractEntry::of::<CombatConfig>(
                "Combat formula configuration",
            )],
            components: vec![],
            events: vec![
                ContractEntry::of::<CombatEvent>("Attack request"),
                ContractEntry::of::<DamageEvent>("Resolved damage result"),
            ],
            system_sets: vec![],
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_damage_calculation() {
        let config = CombatConfig::default();
        let (damage, is_crit) = calculate_damage(&config, 50, 20, 0.0, 0.5, 0.5);
        // 50 - 20*0.5 = 40, no variance at 0.5 roll, no crit
        assert_eq!(damage, 40);
        assert!(!is_crit);
    }

    #[test]
    fn test_critical_hit() {
        let config = CombatConfig::default();
        // rng_roll=0.0 < crit_chance=0.5 → critical
        let (damage, is_crit) = calculate_damage(&config, 50, 0, 0.5, 0.0, 0.5);
        // 50 * 2.0 crit = 100
        assert_eq!(damage, 100);
        assert!(is_crit);
    }

    #[test]
    fn test_high_defense_clamps_to_min() {
        let config = CombatConfig::default();
        let (damage, _) = calculate_damage(&config, 10, 100, 0.0, 0.5, 0.5);
        // 10 - 100*0.5 = -40, clamped to min_damage=1
        assert_eq!(damage, 1);
    }

    #[test]
    fn test_variance_affects_damage() {
        let config = CombatConfig {
            variance: 0.2,
            ..default()
        };
        // variance_roll=0.0 → multiplier = 1.0 + 0.2 * (0.0 - 1.0) = 0.8
        let (low, _) = calculate_damage(&config, 100, 0, 0.0, 0.5, 0.0);
        // variance_roll=1.0 → multiplier = 1.0 + 0.2 * (2.0 - 1.0) = 1.2
        let (high, _) = calculate_damage(&config, 100, 0, 0.0, 0.5, 1.0);
        assert!(low < high);
        assert_eq!(low, 80);
        assert_eq!(high, 120);
    }

    #[test]
    fn test_resolve_combat_applies_damage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<CombatConfig>();
        app.add_message::<CombatEvent>();
        app.add_message::<DamageEvent>();
        app.add_systems(Update, resolve_combat);

        let attacker = app
            .world_mut()
            .spawn(CombatStatsComponent {
                damage: 30,
                crit_chance: 0.0,
                ..default()
            })
            .id();

        let target = app
            .world_mut()
            .spawn(CombatStatsComponent {
                max_hp: 100,
                hp: 100,
                defense: 10,
                ..default()
            })
            .id();

        app.world_mut()
            .resource_mut::<Messages<CombatEvent>>()
            .write(CombatEvent {
                attacker,
                target,
                flat_damage: None,
            });

        app.update();

        let target_stats = app.world().get::<CombatStatsComponent>(target).unwrap();
        // 30 - 10*0.5 = 25, +/- variance → HP should be less than 100
        assert!(target_stats.hp < 100);
        assert!(target_stats.hp > 0);
    }

    #[test]
    fn test_flat_damage_bypasses_stats() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<CombatConfig>();
        app.add_message::<CombatEvent>();
        app.add_message::<DamageEvent>();
        app.add_systems(Update, resolve_combat);

        let attacker = app.world_mut().spawn_empty().id();
        let target = app
            .world_mut()
            .spawn(CombatStatsComponent {
                hp: 50,
                defense: 0,
                ..default()
            })
            .id();

        app.world_mut()
            .resource_mut::<Messages<CombatEvent>>()
            .write(CombatEvent {
                attacker,
                target,
                flat_damage: Some(20),
            });

        app.update();

        let target_stats = app.world().get::<CombatStatsComponent>(target).unwrap();
        // Flat 20 damage with 0 defense, +/- variance
        assert!(target_stats.hp < 50);
    }
}
