//! Story graph data structures for dialogue and narrative.
//!
//! This module provides serializable story graph types that complement
//! the existing `story_graph::StoryNode` runtime types with JSON support.

pub mod graph;
pub mod nodes;
pub mod types;
pub mod validation;

pub use graph::StoryGraphData;
pub use nodes::{
    ActionNodeData, CameraNodeData, ChoiceNodeData, ChoiceOption, ConditionalNodeData,
    DialogueNodeData, EndNodeData, StartNodeData, StoryNodeData, StoryNodeVariant,
    TimeControlNodeData,
};
pub use types::{
    ConditionOperator, EffectType, EndType, LocalizedString, RequiredEntity, RequiredItem,
    StoryCondition, StoryEffect, StoryGraphType, StoryNodeType,
};
pub use validation::{SceneValidationError, ValidationError};

use bevy::prelude::*;

pub(crate) fn register_types(app: &mut App) {
    types::register_types(app);
    nodes::register_types(app);
    graph::register_types(app);
}
