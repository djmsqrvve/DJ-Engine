use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::data::scene::EntityType;

/// Story graph type categorization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, Reflect)]
#[serde(rename_all = "snake_case")]
pub enum StoryGraphType {
    /// Dialogue/conversation
    #[default]
    Dialogue,
    /// Cinematic cutscene
    Cutscene,
    /// Mission/quest logic
    MissionLogic,
}

/// Story node type enumeration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Reflect)]
#[serde(rename_all = "snake_case")]
pub enum StoryNodeType {
    Start,
    /// Dialogue display
    Dialogue,
    /// Player choice
    Choice,
    /// Execute action/script
    Action,
    /// Conditional branch
    Conditional,
    /// Camera movement
    Camera,
    /// Time/pause control
    TimeControl,
    /// End of branch
    End,
}

/// Condition operator for story conditions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, Reflect)]
#[serde(rename_all = "snake_case")]
pub enum ConditionOperator {
    #[default]
    Equals,
    NotEquals,
    LessThan,
    LessThanOrEquals,
    GreaterThan,
    GreaterThanOrEquals,
    Contains,
}

/// End node behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, Reflect)]
#[serde(rename_all = "snake_case")]
pub enum EndType {
    /// Return to normal gameplay
    #[default]
    ReturnToGameplay,
    /// Load a different scene
    LoadScene,
    /// Quit to menu/exit
    Quit,
}

/// Effect type for story effects.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Reflect)]
#[serde(rename_all = "snake_case")]
pub enum EffectType {
    /// Set a variable
    SetVar,
    /// Add to a variable
    AddVar,
    /// Give item to player
    GiveItem,
    /// Remove item from player
    RemoveItem,
    /// Set quest state
    SetQuestState,
}

/// Requirement: Entity must exist in the scene.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Reflect)]
pub struct RequiredEntity {
    /// Entity ID that must exist
    pub entity_id: String,
    /// Expected entity type (optional check)
    #[serde(default)]
    pub entity_type: Option<EntityType>,
}

/// Requirement: Item must exist in inventory (or be available).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Reflect)]
pub struct RequiredItem {
    /// Item ID that is required
    pub item_id: String,
    /// Quantity required
    #[serde(default = "default_one")]
    pub quantity: u32,
}

fn default_one() -> u32 {
    1
}

/// Localized string (text in multiple languages).
pub type LocalizedString = HashMap<String, String>;

/// A condition for story branching.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Reflect)]
pub struct StoryCondition {
    /// Variable name to check
    pub variable: String,
    /// Comparison operator
    #[serde(default)]
    pub operator: ConditionOperator,
    /// Value to compare against
    #[reflect(ignore)]
    pub value: serde_json::Value,
}

/// An effect/action that modifies game state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Reflect)]
pub struct StoryEffect {
    /// Effect type
    #[serde(rename = "type")]
    pub effect_type: EffectType,
    /// Effect parameters
    #[serde(default)]
    #[reflect(ignore)]
    pub params: HashMap<String, serde_json::Value>,
}

pub(super) fn register_types(app: &mut App) {
    app.register_type::<StoryGraphType>()
        .register_type::<StoryNodeType>()
        .register_type::<ConditionOperator>()
        .register_type::<EndType>()
        .register_type::<EffectType>()
        .register_type::<RequiredEntity>()
        .register_type::<RequiredItem>()
        .register_type::<StoryCondition>()
        .register_type::<StoryEffect>();
}
