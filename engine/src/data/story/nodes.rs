use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::data::components::Vec3Data;

use super::types::{
    EndType, LocalizedString, RequiredEntity, RequiredItem, StoryCondition, StoryEffect,
    StoryNodeType,
};

/// Dialogue node data.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, Reflect)]
pub struct DialogueNodeData {
    /// Speaker ID (NPC, party member, or "narrator")
    pub speaker_id: String,
    /// Portrait asset ID
    #[serde(default)]
    pub portrait_id: Option<String>,
    /// Dialogue text per language
    pub text: LocalizedString,
    /// Voice line asset ID
    #[serde(default)]
    pub voice_line_id: Option<String>,
    /// Auto-advance duration (None = wait for input)
    #[serde(default)]
    pub duration: Option<f32>,
    /// Next node ID
    #[serde(default)]
    pub next_node_id: Option<String>,
}

/// A choice option in a choice node.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Reflect)]
pub struct ChoiceOption {
    /// Unique option identifier
    pub id: String,
    /// Display text per language
    pub text: LocalizedString,
    /// Target node ID when selected
    pub target_node_id: String,
    /// Conditions to show this option
    #[serde(default)]
    pub conditions: Vec<StoryCondition>,
    /// Effects when this option is selected
    #[serde(default)]
    pub effects: Vec<StoryEffect>,
}

/// Choice node data.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, Reflect)]
pub struct ChoiceNodeData {
    /// Optional prompt text per language
    #[serde(default)]
    pub prompt: LocalizedString,
    /// Available choice options
    pub options: Vec<ChoiceOption>,
}

/// Action node data.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, Reflect)]
pub struct ActionNodeData {
    /// Lua script ID to execute
    pub lua_script_id: String,
    /// Script parameters
    #[serde(default)]
    #[reflect(ignore)]
    pub params: HashMap<String, serde_json::Value>,
    /// Next node ID
    #[serde(default)]
    pub next_node_id: Option<String>,
}

/// Conditional node data.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Reflect)]
pub struct ConditionalNodeData {
    /// Condition to evaluate
    pub condition: StoryCondition,
    /// Node ID if condition is true
    pub true_target_node_id: String,
    /// Node ID if condition is false
    pub false_target_node_id: String,
}

/// Camera node data.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Reflect)]
pub struct CameraNodeData {
    /// Camera preset ID
    #[serde(default)]
    pub preset_id: Option<String>,
    /// Target position
    #[serde(default)]
    pub position: Vec3Data,
    /// Zoom level
    #[serde(default = "default_zoom")]
    pub zoom: f32,
    /// Camera angle (degrees)
    #[serde(default)]
    pub angle: f32,
    /// Transition duration in seconds
    #[serde(default = "default_duration")]
    pub duration: f32,
    /// Easing function name
    #[serde(default)]
    pub easing: String,
    /// Next node ID
    #[serde(default)]
    pub next_node_id: Option<String>,
}

fn default_zoom() -> f32 {
    1.0
}

fn default_duration() -> f32 {
    1.0
}

impl Default for CameraNodeData {
    fn default() -> Self {
        Self {
            preset_id: None,
            position: Vec3Data::default(),
            zoom: 1.0,
            angle: 0.0,
            duration: 1.0,
            easing: "linear".to_string(),
            next_node_id: None,
        }
    }
}

/// Time control node data.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Reflect)]
pub struct TimeControlNodeData {
    /// Whether to pause gameplay
    #[serde(default)]
    pub pause_gameplay: bool,
    /// Time scale (1.0 = normal, 0.5 = slow-mo)
    #[serde(default = "default_time_scale")]
    pub time_scale: f32,
    /// Next node ID
    #[serde(default)]
    pub next_node_id: Option<String>,
}

fn default_time_scale() -> f32 {
    1.0
}

impl Default for TimeControlNodeData {
    fn default() -> Self {
        Self {
            pause_gameplay: false,
            time_scale: 1.0,
            next_node_id: None,
        }
    }
}

/// End node data.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize, Reflect)]
pub struct EndNodeData {
    /// End behavior type
    #[serde(default)]
    pub end_type: EndType,
    /// Target scene ID (if end_type is LoadScene)
    #[serde(default)]
    pub target_scene_id: Option<String>,
}

/// Start node data.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, Reflect)]
pub struct StartNodeData {
    /// Next node ID logic should flow to
    #[serde(default)]
    pub next_node_id: Option<String>,
}

/// Story node variant data (tagged union).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Reflect)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StoryNodeVariant {
    Start(StartNodeData),
    Dialogue(DialogueNodeData),
    Choice(ChoiceNodeData),
    Action(ActionNodeData),
    Conditional(ConditionalNodeData),
    Camera(CameraNodeData),
    TimeControl(TimeControlNodeData),
    End(EndNodeData),
}

impl StoryNodeVariant {
    pub fn set_next_node_id(&mut self, id: String) -> bool {
        match self {
            Self::Start(data) => {
                data.next_node_id = Some(id);
                true
            }
            Self::Dialogue(data) => {
                data.next_node_id = Some(id);
                true
            }
            Self::Action(data) => {
                data.next_node_id = Some(id);
                true
            }
            Self::Camera(data) => {
                data.next_node_id = Some(id);
                true
            }
            Self::TimeControl(data) => {
                data.next_node_id = Some(id);
                true
            }
            Self::Choice(_) | Self::Conditional(_) | Self::End(_) => false,
        }
    }
}

/// A node in a story graph.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Reflect)]
pub struct StoryNodeData {
    /// Unique node identifier
    pub id: String,
    /// Node position in editor (for visual layout)
    #[serde(default)]
    pub position: Vec3Data,
    /// Node data variant
    pub data: StoryNodeVariant,
    /// Entities required by this node (e.g. speakers, targets)
    #[serde(default)]
    pub required_entities: Vec<RequiredEntity>,
    /// Items required by this node
    #[serde(default)]
    pub required_items: Vec<RequiredItem>,
}

impl StoryNodeData {
    /// Create a new dialogue node.
    pub fn dialogue(
        id: impl Into<String>,
        speaker: impl Into<String>,
        text: impl Into<String>,
    ) -> Self {
        let mut text_map = HashMap::new();
        text_map.insert("en".to_string(), text.into());
        Self {
            id: id.into(),
            position: Vec3Data::default(),
            data: StoryNodeVariant::Dialogue(DialogueNodeData {
                speaker_id: speaker.into(),
                text: text_map,
                ..Default::default()
            }),
            required_entities: Vec::new(),
            required_items: Vec::new(),
        }
    }

    /// Create a new start node.
    pub fn start(id: impl Into<String>, next_id: Option<impl Into<String>>) -> Self {
        Self {
            id: id.into(),
            position: Vec3Data::default(),
            data: StoryNodeVariant::Start(StartNodeData {
                next_node_id: next_id.map(Into::into),
            }),
            required_entities: Vec::new(),
            required_items: Vec::new(),
        }
    }

    /// Create a new end node.
    pub fn end(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            position: Vec3Data::default(),
            data: StoryNodeVariant::End(EndNodeData::default()),
            required_entities: Vec::new(),
            required_items: Vec::new(),
        }
    }

    /// Get the node type.
    pub fn node_type(&self) -> StoryNodeType {
        match &self.data {
            StoryNodeVariant::Start(_) => StoryNodeType::Start,
            StoryNodeVariant::Dialogue(_) => StoryNodeType::Dialogue,
            StoryNodeVariant::Choice(_) => StoryNodeType::Choice,
            StoryNodeVariant::Action(_) => StoryNodeType::Action,
            StoryNodeVariant::Conditional(_) => StoryNodeType::Conditional,
            StoryNodeVariant::Camera(_) => StoryNodeType::Camera,
            StoryNodeVariant::TimeControl(_) => StoryNodeType::TimeControl,
            StoryNodeVariant::End(_) => StoryNodeType::End,
        }
    }

    /// Get the next node ID(s) for this node.
    pub fn next_node_ids(&self) -> Vec<&str> {
        match &self.data {
            StoryNodeVariant::Start(data) => data.next_node_id.as_deref().into_iter().collect(),
            StoryNodeVariant::Dialogue(data) => data.next_node_id.as_deref().into_iter().collect(),
            StoryNodeVariant::Choice(data) => data
                .options
                .iter()
                .map(|option| option.target_node_id.as_str())
                .collect(),
            StoryNodeVariant::Action(data) => data.next_node_id.as_deref().into_iter().collect(),
            StoryNodeVariant::Conditional(data) => vec![
                data.true_target_node_id.as_str(),
                data.false_target_node_id.as_str(),
            ],
            StoryNodeVariant::Camera(data) => data.next_node_id.as_deref().into_iter().collect(),
            StoryNodeVariant::TimeControl(data) => {
                data.next_node_id.as_deref().into_iter().collect()
            }
            StoryNodeVariant::End(_) => vec![],
        }
    }
}

pub(super) fn register_types(app: &mut App) {
    app.register_type::<StoryNodeVariant>()
        .register_type::<StoryNodeData>()
        .register_type::<DialogueNodeData>()
        .register_type::<ChoiceNodeData>()
        .register_type::<ChoiceOption>()
        .register_type::<ActionNodeData>()
        .register_type::<ConditionalNodeData>()
        .register_type::<CameraNodeData>()
        .register_type::<TimeControlNodeData>()
        .register_type::<EndNodeData>()
        .register_type::<StartNodeData>();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::story::{ConditionOperator, StoryCondition};

    #[test]
    fn test_set_next_node_id_supported() {
        let cases: Vec<StoryNodeVariant> = vec![
            StoryNodeVariant::Start(StartNodeData { next_node_id: None }),
            StoryNodeVariant::Dialogue(DialogueNodeData::default()),
            StoryNodeVariant::Action(ActionNodeData::default()),
            StoryNodeVariant::Camera(CameraNodeData::default()),
            StoryNodeVariant::TimeControl(TimeControlNodeData::default()),
        ];

        for mut variant in cases {
            assert!(variant.set_next_node_id("target".into()));
        }
    }

    #[test]
    fn test_set_next_node_id_unsupported() {
        let cases: Vec<StoryNodeVariant> = vec![
            StoryNodeVariant::Choice(ChoiceNodeData::default()),
            StoryNodeVariant::Conditional(ConditionalNodeData {
                condition: StoryCondition {
                    variable: String::new(),
                    operator: ConditionOperator::Equals,
                    value: serde_json::Value::Null,
                },
                true_target_node_id: String::new(),
                false_target_node_id: String::new(),
            }),
            StoryNodeVariant::End(EndNodeData::default()),
        ];

        for mut variant in cases {
            assert!(!variant.set_next_node_id("target".into()));
        }
    }
}
