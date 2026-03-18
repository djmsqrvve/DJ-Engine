use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Core player character data.
#[derive(Component, Debug, Clone, PartialEq, Serialize, Deserialize, Default, Reflect)]
#[reflect(Component)]
pub struct PlayerCharacterComponent {
    pub level: u32,
    pub experience: u64,
    pub class_id: String,
    pub skill_ids: Vec<String>,
}

/// Equipment slot assignments.
#[derive(Component, Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default, Reflect)]
#[reflect(Component)]
pub struct EquipmentSlotsComponent {
    pub head: Option<String>,
    pub neck: Option<String>,
    pub chest: Option<String>,
    pub legs: Option<String>,
    pub feet: Option<String>,
    pub hands: Option<String>,
    pub main_hand: Option<String>,
    pub off_hand: Option<String>,
    pub back: Option<String>,
    pub trinket1: Option<String>,
    pub trinket2: Option<String>,
}

/// An active status effect (buff or debuff).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default, Reflect)]
pub struct ActiveEffect {
    pub effect_id: String,
    pub remaining_duration: f32,
    pub stacks: u32,
}

/// Currently active status effects on an entity.
#[derive(Component, Debug, Clone, PartialEq, Serialize, Deserialize, Default, Reflect)]
#[reflect(Component)]
pub struct StatusEffectsComponent {
    pub effects: Vec<ActiveEffect>,
}

/// Tracks cooldown remaining for each ability.
#[derive(Component, Debug, Clone, PartialEq, Serialize, Deserialize, Default, Reflect)]
#[reflect(Component)]
pub struct AbilityCooldownsComponent {
    pub cooldowns: HashMap<String, f32>,
}

pub(super) fn register_types(app: &mut App) {
    app.register_type::<PlayerCharacterComponent>()
        .register_type::<EquipmentSlotsComponent>()
        .register_type::<ActiveEffect>()
        .register_type::<StatusEffectsComponent>()
        .register_type::<AbilityCooldownsComponent>();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_player_character_default() {
        let pc = PlayerCharacterComponent::default();
        assert_eq!(pc.level, 0);
        assert_eq!(pc.experience, 0);
        assert!(pc.class_id.is_empty());
        assert!(pc.skill_ids.is_empty());
    }

    #[test]
    fn test_equipment_slots_default() {
        let eq = EquipmentSlotsComponent::default();
        assert!(eq.head.is_none());
        assert!(eq.neck.is_none());
        assert!(eq.chest.is_none());
        assert!(eq.legs.is_none());
        assert!(eq.feet.is_none());
        assert!(eq.hands.is_none());
        assert!(eq.main_hand.is_none());
        assert!(eq.off_hand.is_none());
        assert!(eq.back.is_none());
        assert!(eq.trinket1.is_none());
        assert!(eq.trinket2.is_none());
    }

    #[test]
    fn test_status_effects_add_remove() {
        let mut se = StatusEffectsComponent::default();
        assert!(se.effects.is_empty());

        se.effects.push(ActiveEffect {
            effect_id: "poison".into(),
            remaining_duration: 5.0,
            stacks: 1,
        });
        se.effects.push(ActiveEffect {
            effect_id: "haste".into(),
            remaining_duration: 10.0,
            stacks: 2,
        });
        assert_eq!(se.effects.len(), 2);

        // Remove expired effects (duration <= 0)
        se.effects[0].remaining_duration = 0.0;
        se.effects.retain(|e| e.remaining_duration > 0.0);
        assert_eq!(se.effects.len(), 1);
        assert_eq!(se.effects[0].effect_id, "haste");
    }

    #[test]
    fn test_ability_cooldowns_tick() {
        let mut ac = AbilityCooldownsComponent::default();
        ac.cooldowns.insert("fireball".into(), 3.0);
        ac.cooldowns.insert("heal".into(), 8.0);

        assert!(ac.cooldowns.contains_key("fireball"));
        assert_eq!(ac.cooldowns["fireball"], 3.0);
        assert!(ac.cooldowns.contains_key("heal"));
        assert_eq!(ac.cooldowns["heal"], 8.0);
    }

    #[test]
    fn test_player_character_serialization() {
        let pc = PlayerCharacterComponent {
            level: 5,
            experience: 1200,
            class_id: "warrior".into(),
            skill_ids: vec!["slash".into(), "block".into()],
        };
        let json = serde_json::to_string(&pc).unwrap();
        let deser: PlayerCharacterComponent = serde_json::from_str(&json).unwrap();
        assert_eq!(pc, deser);
    }

    #[test]
    fn test_equipment_slots_serialization() {
        let eq = EquipmentSlotsComponent {
            head: Some("iron_helm".into()),
            chest: Some("plate_armor".into()),
            main_hand: Some("longsword".into()),
            ..Default::default()
        };
        let json = serde_json::to_string(&eq).unwrap();
        let deser: EquipmentSlotsComponent = serde_json::from_str(&json).unwrap();
        assert_eq!(eq, deser);
    }

    #[test]
    fn test_status_effects_serialization() {
        let se = StatusEffectsComponent {
            effects: vec![
                ActiveEffect {
                    effect_id: "burn".into(),
                    remaining_duration: 3.5,
                    stacks: 2,
                },
                ActiveEffect {
                    effect_id: "shield".into(),
                    remaining_duration: 12.0,
                    stacks: 1,
                },
            ],
        };
        let json = serde_json::to_string(&se).unwrap();
        let deser: StatusEffectsComponent = serde_json::from_str(&json).unwrap();
        assert_eq!(se, deser);
    }

    #[test]
    fn test_ability_cooldowns_serialization() {
        let mut ac = AbilityCooldownsComponent::default();
        ac.cooldowns.insert("fireball".into(), 2.5);
        ac.cooldowns.insert("teleport".into(), 15.0);

        let json = serde_json::to_string(&ac).unwrap();
        let deser: AbilityCooldownsComponent = serde_json::from_str(&json).unwrap();
        assert_eq!(ac, deser);
    }
}
