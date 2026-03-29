use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Trigger type for interactive objects.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, Reflect)]
#[serde(rename_all = "snake_case")]
pub enum TriggerType {
    #[default]
    None,
    Door,
    Chest,
    Npc,
    Custom,
}

/// Event hooks for interactive objects.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize, Reflect)]
pub struct InteractivityEvents {
    /// Event/script to run on interaction (E key, click, etc.)
    pub on_interact: Option<String>,
    /// Event/script to run when player enters trigger
    pub on_enter: Option<String>,
    /// Event/script to run when player exits trigger
    pub on_exit: Option<String>,
    /// Event/script to run on entity death
    pub on_death: Option<String>,
}

/// Interactivity component data.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, Component, Reflect)]
#[reflect(Component)]
pub struct InteractivityComponent {
    /// Type of trigger
    #[serde(default)]
    pub trigger_type: TriggerType,
    /// Unique trigger identifier
    #[serde(default)]
    pub trigger_id: String,
    /// Custom parameters for the trigger
    #[serde(default)]
    #[reflect(ignore)]
    pub parameters: HashMap<String, serde_json::Value>,
    /// Lua script ID to execute
    #[serde(default)]
    pub lua_script_id: Option<String>,
    /// Event hooks
    #[serde(default)]
    pub events: InteractivityEvents,
}

pub(super) fn register_types(app: &mut App) {
    app.register_type::<TriggerType>()
        .register_type::<InteractivityEvents>()
        .register_type::<InteractivityComponent>();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trigger_type_default_is_none() {
        assert_eq!(TriggerType::default(), TriggerType::None);
    }

    #[test]
    fn test_interactivity_events_default_all_none() {
        let events = InteractivityEvents::default();
        assert!(events.on_interact.is_none());
        assert!(events.on_enter.is_none());
        assert!(events.on_exit.is_none());
        assert!(events.on_death.is_none());
    }

    #[test]
    fn test_interactivity_component_default() {
        let comp = InteractivityComponent::default();
        assert_eq!(comp.trigger_type, TriggerType::None);
        assert!(comp.trigger_id.is_empty());
        assert!(comp.lua_script_id.is_none());
        assert!(comp.parameters.is_empty());
    }

    #[test]
    fn test_trigger_type_serde() {
        let json = serde_json::to_string(&TriggerType::Npc).unwrap();
        assert_eq!(json, "\"npc\"");
        let tt: TriggerType = serde_json::from_str(&json).unwrap();
        assert_eq!(tt, TriggerType::Npc);
    }

    #[test]
    fn test_interactivity_serde_roundtrip() {
        let comp = InteractivityComponent {
            trigger_type: TriggerType::Door,
            trigger_id: "exit_door".into(),
            lua_script_id: Some("open_door.lua".into()),
            events: InteractivityEvents {
                on_interact: Some("open".into()),
                ..Default::default()
            },
            ..Default::default()
        };
        let json = serde_json::to_string(&comp).unwrap();
        let comp2: InteractivityComponent = serde_json::from_str(&json).unwrap();
        assert_eq!(comp.trigger_type, comp2.trigger_type);
        assert_eq!(comp.trigger_id, comp2.trigger_id);
    }
}
