//! Status effect and ability cooldown tick systems.
//!
//! Ticks [`StatusEffectsComponent`] durations each frame, removes expired effects,
//! and fires [`StatusEffectExpired`] events. Also ticks [`AbilityCooldownsComponent`]
//! and fires [`AbilityReady`] when a cooldown reaches zero.

use bevy::prelude::*;

use crate::data::components::{AbilityCooldownsComponent, ActiveEffect, StatusEffectsComponent};

/// Fired when a status effect expires (duration reaches zero).
#[derive(Message, Debug, Clone, PartialEq)]
pub struct StatusEffectExpired {
    pub entity: Entity,
    pub effect_id: String,
}

/// Fired when an ability comes off cooldown.
#[derive(Message, Debug, Clone, PartialEq)]
pub struct AbilityReady {
    pub entity: Entity,
    pub ability_id: String,
}

/// Apply a status effect to an entity. Stacks if already present.
pub fn apply_effect(
    effects: &mut StatusEffectsComponent,
    effect_id: &str,
    duration: f32,
    stacks: u32,
) {
    if let Some(existing) = effects
        .effects
        .iter_mut()
        .find(|e| e.effect_id == effect_id)
    {
        existing.remaining_duration = existing.remaining_duration.max(duration);
        existing.stacks += stacks;
    } else {
        effects.effects.push(ActiveEffect {
            effect_id: effect_id.to_string(),
            remaining_duration: duration,
            stacks,
        });
    }
}

/// Remove all stacks of an effect.
pub fn remove_effect(effects: &mut StatusEffectsComponent, effect_id: &str) -> bool {
    let before = effects.effects.len();
    effects.effects.retain(|e| e.effect_id != effect_id);
    effects.effects.len() < before
}

/// System that ticks status effect durations and removes expired ones.
pub fn tick_status_effects(
    time: Res<Time>,
    mut query: Query<(Entity, &mut StatusEffectsComponent)>,
    mut expired_events: MessageWriter<StatusEffectExpired>,
) {
    let dt = time.delta_secs();
    for (entity, mut effects) in query.iter_mut() {
        let mut expired = Vec::new();

        for effect in &mut effects.effects {
            effect.remaining_duration -= dt;
            if effect.remaining_duration <= 0.0 {
                expired.push(effect.effect_id.clone());
            }
        }

        for effect_id in &expired {
            expired_events.write(StatusEffectExpired {
                entity,
                effect_id: effect_id.clone(),
            });
        }

        effects.effects.retain(|e| e.remaining_duration > 0.0);
    }
}

/// System that ticks ability cooldowns and removes completed ones.
pub fn tick_ability_cooldowns(
    time: Res<Time>,
    mut query: Query<(Entity, &mut AbilityCooldownsComponent)>,
    mut ready_events: MessageWriter<AbilityReady>,
) {
    let dt = time.delta_secs();
    for (entity, mut cooldowns) in query.iter_mut() {
        let mut ready = Vec::new();

        for (ability_id, remaining) in cooldowns.cooldowns.iter_mut() {
            *remaining -= dt;
            if *remaining <= 0.0 {
                ready.push(ability_id.clone());
            }
        }

        for ability_id in &ready {
            ready_events.write(AbilityReady {
                entity,
                ability_id: ability_id.clone(),
            });
        }

        cooldowns.cooldowns.retain(|_, remaining| *remaining > 0.0);
    }
}

/// Put an ability on cooldown.
pub fn start_cooldown(cooldowns: &mut AbilityCooldownsComponent, ability_id: &str, duration: f32) {
    cooldowns.cooldowns.insert(ability_id.to_string(), duration);
}

/// Check if an ability is on cooldown.
pub fn is_on_cooldown(cooldowns: &AbilityCooldownsComponent, ability_id: &str) -> bool {
    cooldowns
        .cooldowns
        .get(ability_id)
        .map(|r| *r > 0.0)
        .unwrap_or(false)
}

/// Plugin providing status effect and cooldown tick systems.
pub struct StatusPlugin;

impl Plugin for StatusPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<StatusEffectExpired>()
            .add_message::<AbilityReady>()
            .add_systems(Update, (tick_status_effects, tick_ability_cooldowns));

        use crate::contracts::{AppContractExt, ContractEntry, PluginContract};
        app.register_contract(PluginContract {
            name: "StatusPlugin".into(),
            description: "Status effect duration ticking and ability cooldowns".into(),
            resources: vec![],
            components: vec![],
            events: vec![
                ContractEntry::of::<StatusEffectExpired>("Status effect expired"),
                ContractEntry::of::<AbilityReady>("Ability off cooldown"),
            ],
            system_sets: vec![],
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_effect_new() {
        let mut effects = StatusEffectsComponent::default();
        apply_effect(&mut effects, "poison", 5.0, 1);
        assert_eq!(effects.effects.len(), 1);
        assert_eq!(effects.effects[0].effect_id, "poison");
        assert_eq!(effects.effects[0].stacks, 1);
    }

    #[test]
    fn test_apply_effect_stacks() {
        let mut effects = StatusEffectsComponent::default();
        apply_effect(&mut effects, "poison", 5.0, 1);
        apply_effect(&mut effects, "poison", 3.0, 2);
        assert_eq!(effects.effects.len(), 1);
        assert_eq!(effects.effects[0].stacks, 3);
        // Duration takes the max
        assert!((effects.effects[0].remaining_duration - 5.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_remove_effect() {
        let mut effects = StatusEffectsComponent::default();
        apply_effect(&mut effects, "poison", 5.0, 1);
        apply_effect(&mut effects, "haste", 10.0, 1);
        assert!(remove_effect(&mut effects, "poison"));
        assert_eq!(effects.effects.len(), 1);
        assert_eq!(effects.effects[0].effect_id, "haste");
    }

    #[test]
    fn test_tick_expires_effects() {
        // Unit test the tick logic directly without Bevy App
        let mut effects = StatusEffectsComponent::default();
        apply_effect(&mut effects, "short_buff", 0.1, 1);
        apply_effect(&mut effects, "long_buff", 10.0, 1);

        // Simulate 0.2s tick
        for effect in &mut effects.effects {
            effect.remaining_duration -= 0.2;
        }
        effects.effects.retain(|e| e.remaining_duration > 0.0);

        assert_eq!(effects.effects.len(), 1);
        assert_eq!(effects.effects[0].effect_id, "long_buff");
    }

    #[test]
    fn test_start_cooldown_and_check() {
        let mut cooldowns = AbilityCooldownsComponent::default();
        assert!(!is_on_cooldown(&cooldowns, "fireball"));

        start_cooldown(&mut cooldowns, "fireball", 3.0);
        assert!(is_on_cooldown(&cooldowns, "fireball"));
    }

    #[test]
    fn test_tick_expires_cooldowns() {
        // Unit test the tick logic directly without Bevy App
        let mut cooldowns = AbilityCooldownsComponent::default();
        start_cooldown(&mut cooldowns, "fireball", 0.1);
        start_cooldown(&mut cooldowns, "heal", 10.0);

        // Simulate 0.2s tick
        for (_, remaining) in cooldowns.cooldowns.iter_mut() {
            *remaining -= 0.2;
        }
        cooldowns.cooldowns.retain(|_, remaining| *remaining > 0.0);

        assert!(!cooldowns.cooldowns.contains_key("fireball"));
        assert!(cooldowns.cooldowns.contains_key("heal"));
    }

    #[test]
    fn test_remove_nonexistent_effect() {
        let mut effects = StatusEffectsComponent::default();
        assert!(!remove_effect(&mut effects, "not_here"));
    }

    #[test]
    fn test_apply_multiple_different_effects() {
        let mut effects = StatusEffectsComponent::default();
        apply_effect(&mut effects, "poison", 5.0, 1);
        apply_effect(&mut effects, "haste", 10.0, 2);
        apply_effect(&mut effects, "shield", 3.0, 1);
        assert_eq!(effects.effects.len(), 3);
    }

    #[test]
    fn test_effect_duration_refresh_takes_max() {
        let mut effects = StatusEffectsComponent::default();
        apply_effect(&mut effects, "buff", 3.0, 1);
        apply_effect(&mut effects, "buff", 8.0, 0);
        assert!((effects.effects[0].remaining_duration - 8.0).abs() < f32::EPSILON);

        // Shorter refresh doesn't reduce
        apply_effect(&mut effects, "buff", 2.0, 0);
        assert!((effects.effects[0].remaining_duration - 8.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_cooldown_not_started_is_not_on_cooldown() {
        let cooldowns = AbilityCooldownsComponent::default();
        assert!(!is_on_cooldown(&cooldowns, "any_ability"));
    }

    #[test]
    fn test_multiple_cooldowns_independent() {
        let mut cooldowns = AbilityCooldownsComponent::default();
        start_cooldown(&mut cooldowns, "fireball", 3.0);
        start_cooldown(&mut cooldowns, "heal", 1.5);

        assert!(is_on_cooldown(&cooldowns, "fireball"));
        assert!(is_on_cooldown(&cooldowns, "heal"));
        assert!(!is_on_cooldown(&cooldowns, "ice_bolt"));
    }
}
