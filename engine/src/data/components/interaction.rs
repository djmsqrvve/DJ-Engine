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
