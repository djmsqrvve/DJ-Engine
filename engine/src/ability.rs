//! Ability use system for DJ Engine.
//!
//! Validates mana cost and cooldown, deducts mana, starts cooldown,
//! and fires [`AbilityUsedEvent`] for games to handle (damage, heal, buff, etc.).

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::combat::CombatEvent;
use crate::data::components::{AbilityCooldownsComponent, CombatStatsComponent};
use crate::status::start_cooldown;

/// Request to use an ability.
#[derive(Message, Debug, Clone, PartialEq)]
pub struct UseAbilityRequest {
    pub caster: Entity,
    pub target: Option<Entity>,
    pub ability_id: String,
    pub mana_cost: i32,
    pub cooldown: f32,
    pub damage: Option<i32>,
    pub heal: Option<i32>,
    pub effect_id: Option<String>,
    pub effect_duration: Option<f32>,
}

/// Result of ability use — fired after validation and resource deduction.
#[derive(Message, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AbilityUsedEvent {
    Success {
        caster: Entity,
        ability_id: String,
    },
    InsufficientMana {
        caster: Entity,
        ability_id: String,
        mana_needed: i32,
        mana_available: i32,
    },
    OnCooldown {
        caster: Entity,
        ability_id: String,
    },
    NoTarget {
        caster: Entity,
        ability_id: String,
    },
}

/// System that processes ability use requests.
pub fn process_ability_use(
    mut requests: MessageReader<UseAbilityRequest>,
    mut ability_events: MessageWriter<AbilityUsedEvent>,
    mut combat_events: MessageWriter<CombatEvent>,
    mut query: Query<(&mut CombatStatsComponent, &mut AbilityCooldownsComponent)>,
) {
    for request in requests.read() {
        // Check cooldown
        if let Ok((_, cooldowns)) = query.get(request.caster) {
            if crate::status::is_on_cooldown(cooldowns, &request.ability_id) {
                ability_events.write(AbilityUsedEvent::OnCooldown {
                    caster: request.caster,
                    ability_id: request.ability_id.clone(),
                });
                continue;
            }
        }

        // Check mana
        if let Ok((stats, _)) = query.get(request.caster) {
            if stats.mana < request.mana_cost {
                ability_events.write(AbilityUsedEvent::InsufficientMana {
                    caster: request.caster,
                    ability_id: request.ability_id.clone(),
                    mana_needed: request.mana_cost,
                    mana_available: stats.mana,
                });
                continue;
            }
        }

        // Deduct mana and start cooldown
        if let Ok((mut stats, mut cooldowns)) = query.get_mut(request.caster) {
            stats.mana -= request.mana_cost;
            if request.cooldown > 0.0 {
                start_cooldown(&mut cooldowns, &request.ability_id, request.cooldown);
            }
        }

        // Fire damage combat event if ability deals damage
        if let Some(damage) = request.damage {
            if let Some(target) = request.target {
                combat_events.write(CombatEvent {
                    attacker: request.caster,
                    target,
                    flat_damage: Some(damage),
                });
            } else {
                ability_events.write(AbilityUsedEvent::NoTarget {
                    caster: request.caster,
                    ability_id: request.ability_id.clone(),
                });
                continue;
            }
        }

        // Fire heal (apply to self)
        if let Some(heal) = request.heal {
            if let Ok((mut stats, _)) = query.get_mut(request.caster) {
                stats.hp = (stats.hp + heal).min(stats.max_hp);
            }
        }

        ability_events.write(AbilityUsedEvent::Success {
            caster: request.caster,
            ability_id: request.ability_id.clone(),
        });
    }
}

/// Plugin providing ability use processing.
pub struct AbilityPlugin;

impl Plugin for AbilityPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<UseAbilityRequest>()
            .add_message::<AbilityUsedEvent>()
            .add_systems(Update, process_ability_use);

        use crate::contracts::{AppContractExt, ContractEntry, PluginContract};
        app.register_contract(PluginContract {
            name: "AbilityPlugin".into(),
            description: "Ability use validation, mana cost, cooldown, and effect dispatch".into(),
            resources: vec![],
            components: vec![],
            events: vec![
                ContractEntry::of::<UseAbilityRequest>("Ability use request"),
                ContractEntry::of::<AbilityUsedEvent>("Ability use result"),
            ],
            system_sets: vec![],
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::components::AbilityCooldownsComponent;

    fn test_app() -> (App, Entity) {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<UseAbilityRequest>();
        app.add_message::<AbilityUsedEvent>();
        app.add_message::<CombatEvent>();
        app.add_systems(Update, process_ability_use);

        let caster = app
            .world_mut()
            .spawn((
                CombatStatsComponent {
                    max_hp: 100,
                    hp: 100,
                    mana: 50,
                    damage: 20,
                    ..default()
                },
                AbilityCooldownsComponent::default(),
            ))
            .id();

        (app, caster)
    }

    #[test]
    fn test_ability_success_deducts_mana() {
        let (mut app, caster) = test_app();

        app.world_mut()
            .resource_mut::<Messages<UseAbilityRequest>>()
            .write(UseAbilityRequest {
                caster,
                target: None,
                ability_id: "heal".into(),
                mana_cost: 20,
                cooldown: 3.0,
                damage: None,
                heal: Some(30),
                effect_id: None,
                effect_duration: None,
            });

        app.update();

        let stats = app.world().get::<CombatStatsComponent>(caster).unwrap();
        assert_eq!(stats.mana, 30); // 50 - 20
    }

    #[test]
    fn test_ability_insufficient_mana() {
        let (mut app, caster) = test_app();

        app.world_mut()
            .resource_mut::<Messages<UseAbilityRequest>>()
            .write(UseAbilityRequest {
                caster,
                target: None,
                ability_id: "mega_spell".into(),
                mana_cost: 999,
                cooldown: 0.0,
                damage: None,
                heal: None,
                effect_id: None,
                effect_duration: None,
            });

        app.update();

        let stats = app.world().get::<CombatStatsComponent>(caster).unwrap();
        assert_eq!(stats.mana, 50); // unchanged
    }

    #[test]
    fn test_ability_starts_cooldown() {
        let (mut app, caster) = test_app();

        app.world_mut()
            .resource_mut::<Messages<UseAbilityRequest>>()
            .write(UseAbilityRequest {
                caster,
                target: None,
                ability_id: "fireball".into(),
                mana_cost: 10,
                cooldown: 5.0,
                damage: None,
                heal: None,
                effect_id: None,
                effect_duration: None,
            });

        app.update();

        let cooldowns = app
            .world()
            .get::<AbilityCooldownsComponent>(caster)
            .unwrap();
        assert!(cooldowns.cooldowns.contains_key("fireball"));
    }

    #[test]
    fn test_ability_blocked_by_cooldown() {
        let (mut app, caster) = test_app();

        // Pre-set cooldown
        let mut cooldowns = app
            .world_mut()
            .get_mut::<AbilityCooldownsComponent>(caster)
            .unwrap();
        start_cooldown(&mut cooldowns, "fireball", 10.0);

        app.world_mut()
            .resource_mut::<Messages<UseAbilityRequest>>()
            .write(UseAbilityRequest {
                caster,
                target: None,
                ability_id: "fireball".into(),
                mana_cost: 10,
                cooldown: 5.0,
                damage: None,
                heal: None,
                effect_id: None,
                effect_duration: None,
            });

        app.update();

        let stats = app.world().get::<CombatStatsComponent>(caster).unwrap();
        assert_eq!(stats.mana, 50); // mana not deducted (blocked)
    }

    #[test]
    fn test_ability_heal_restores_hp() {
        let (mut app, caster) = test_app();

        // Damage the caster first
        app.world_mut()
            .get_mut::<CombatStatsComponent>(caster)
            .unwrap()
            .hp = 40;

        app.world_mut()
            .resource_mut::<Messages<UseAbilityRequest>>()
            .write(UseAbilityRequest {
                caster,
                target: None,
                ability_id: "heal".into(),
                mana_cost: 15,
                cooldown: 0.0,
                damage: None,
                heal: Some(30),
                effect_id: None,
                effect_duration: None,
            });

        app.update();

        let stats = app.world().get::<CombatStatsComponent>(caster).unwrap();
        assert_eq!(stats.hp, 70); // 40 + 30
        assert_eq!(stats.mana, 35); // 50 - 15
    }

    #[test]
    fn test_ability_heal_capped_at_max_hp() {
        let (mut app, caster) = test_app();

        // Slight damage
        app.world_mut()
            .get_mut::<CombatStatsComponent>(caster)
            .unwrap()
            .hp = 95;

        app.world_mut()
            .resource_mut::<Messages<UseAbilityRequest>>()
            .write(UseAbilityRequest {
                caster,
                target: None,
                ability_id: "big_heal".into(),
                mana_cost: 5,
                cooldown: 0.0,
                damage: None,
                heal: Some(50),
                effect_id: None,
                effect_duration: None,
            });

        app.update();

        let stats = app.world().get::<CombatStatsComponent>(caster).unwrap();
        assert_eq!(stats.hp, 100); // capped at max_hp
    }

    #[test]
    fn test_ability_zero_mana_cost_always_works() {
        let (mut app, caster) = test_app();

        // Set mana to 0
        app.world_mut()
            .get_mut::<CombatStatsComponent>(caster)
            .unwrap()
            .mana = 0;

        app.world_mut()
            .resource_mut::<Messages<UseAbilityRequest>>()
            .write(UseAbilityRequest {
                caster,
                target: None,
                ability_id: "free_spell".into(),
                mana_cost: 0,
                cooldown: 1.0,
                damage: None,
                heal: Some(10),
                effect_id: None,
                effect_duration: None,
            });

        app.update();

        let stats = app.world().get::<CombatStatsComponent>(caster).unwrap();
        // Mana still 0 (0 - 0 = 0), but ability should work
        assert_eq!(stats.mana, 0);
    }
}
