//! Story graph data structures for dialogue and narrative.
//!
//! This module provides serializable story graph types that complement
//! the existing `story_graph::StoryNode` runtime types with JSON support.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::components::Vec3Data;
use super::scene::{EntityType, Scene};

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

// ... (skipping unchanged code if possible, but replace_file_content needs contiguous block? No, I can target specific blocks)
// Since node_type() and next_node_ids() are further down, I will use multiple replace chunks if they were separate tool calls, but here I can just replace the enum first.

// Wait, I messed up the thinking. I can use MultiReplaceFileContent or multiple ReplaceFileContent.
// I'll do StoryNodeType update first.

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
            Self::Start(d) => {
                d.next_node_id = Some(id);
                true
            }
            Self::Dialogue(d) => {
                d.next_node_id = Some(id);
                true
            }
            Self::Action(a) => {
                a.next_node_id = Some(id);
                true
            }
            Self::Camera(c) => {
                c.next_node_id = Some(id);
                true
            }
            Self::TimeControl(t) => {
                t.next_node_id = Some(id);
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
                next_node_id: next_id.map(|s| s.into()),
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
            StoryNodeVariant::Start(s) => s.next_node_id.as_deref().into_iter().collect(),
            StoryNodeVariant::Dialogue(d) => d.next_node_id.as_deref().into_iter().collect(),
            StoryNodeVariant::Choice(c) => c
                .options
                .iter()
                .map(|o| o.target_node_id.as_str())
                .collect(),
            StoryNodeVariant::Action(a) => a.next_node_id.as_deref().into_iter().collect(),
            StoryNodeVariant::Conditional(c) => vec![
                c.true_target_node_id.as_str(),
                c.false_target_node_id.as_str(),
            ],
            StoryNodeVariant::Camera(c) => c.next_node_id.as_deref().into_iter().collect(),
            StoryNodeVariant::TimeControl(t) => t.next_node_id.as_deref().into_iter().collect(),
            StoryNodeVariant::End(_) => vec![],
        }
    }
}

/// Validation error for story graphs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationError {
    /// Root node ID doesn't exist
    MissingRootNode(String),
    /// A node references a non-existent node
    BrokenReference { from_node: String, to_node: String },
    /// Node has no outgoing edges (dead end, excluding End nodes)
    DeadEnd(String),
    /// Unreachable node
    UnreachableNode(String),
}

/// Validation error when checking against a scene.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SceneValidationError {
    /// Node requires an entity that is missing from the scene
    MissingRequiredEntity { node_id: String, entity_id: String },
    /// Node requires an entity of a specific type, but found different type
    WrongEntityType {
        node_id: String,
        entity_id: String,
        expected: EntityType,
        found: EntityType,
    },
}

/// A complete story graph.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, Reflect)]
pub struct StoryGraphData {
    /// Unique graph identifier
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Description
    #[serde(default)]
    pub description: String,
    /// Graph type
    #[serde(default)]
    pub graph_type: StoryGraphType,
    /// Root node ID (entry point)
    #[serde(default)]
    pub root_node_id: String,
    /// Initial variable values
    #[serde(default)]
    #[reflect(ignore)]
    pub variables: HashMap<String, serde_json::Value>,
    /// All nodes in the graph
    pub nodes: Vec<StoryNodeData>,
}

impl StoryGraphData {
    /// Create a new empty story graph.
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: String::new(),
            graph_type: StoryGraphType::Dialogue,
            root_node_id: String::new(),
            variables: HashMap::new(),
            nodes: Vec::new(),
        }
    }

    /// Add a node to the graph.
    pub fn add_node(&mut self, node: StoryNodeData) {
        self.nodes.push(node);
    }

    /// Find a node by ID.
    pub fn find_node(&self, id: &str) -> Option<&StoryNodeData> {
        self.nodes.iter().find(|n| n.id == id)
    }

    /// Validate the story graph and return any errors.
    pub fn validate(&self) -> Vec<ValidationError> {
        let mut errors = Vec::new();
        let node_ids: std::collections::HashSet<_> =
            self.nodes.iter().map(|n| n.id.as_str()).collect();

        // Check root node exists
        if !node_ids.contains(self.root_node_id.as_str()) {
            errors.push(ValidationError::MissingRootNode(self.root_node_id.clone()));
        }

        // Check all references are valid
        for node in &self.nodes {
            for next_id in node.next_node_ids() {
                if !node_ids.contains(next_id) {
                    errors.push(ValidationError::BrokenReference {
                        from_node: node.id.clone(),
                        to_node: next_id.to_string(),
                    });
                }
            }

            // Check for dead ends (nodes with no outgoing edges that aren't End nodes)
            if node.next_node_ids().is_empty() && !matches!(node.data, StoryNodeVariant::End(_)) {
                errors.push(ValidationError::DeadEnd(node.id.clone()));
            }
        }

        // Check for unreachable nodes via BFS from root
        if node_ids.contains(self.root_node_id.as_str()) {
            let mut reachable = std::collections::HashSet::new();
            let mut queue = std::collections::VecDeque::new();
            queue.push_back(self.root_node_id.as_str());
            reachable.insert(self.root_node_id.as_str());
            while let Some(current) = queue.pop_front() {
                if let Some(node) = self.find_node(current) {
                    for next in node.next_node_ids() {
                        if reachable.insert(next) {
                            queue.push_back(next);
                        }
                    }
                }
            }
            for node in &self.nodes {
                if !reachable.contains(node.id.as_str()) {
                    errors.push(ValidationError::UnreachableNode(node.id.clone()));
                }
            }
        }

        errors
    }

    /// Validate the story graph against a specific scene.
    pub fn validate_against_scene(&self, scene: &Scene) -> Vec<SceneValidationError> {
        let mut errors = Vec::new();

        for node in &self.nodes {
            for req in &node.required_entities {
                match scene.find_entity(&req.entity_id) {
                    Some(entity) => {
                        if let Some(expected_type) = req.entity_type {
                            if entity.entity_type != expected_type {
                                errors.push(SceneValidationError::WrongEntityType {
                                    node_id: node.id.clone(),
                                    entity_id: req.entity_id.clone(),
                                    expected: expected_type,
                                    found: entity.entity_type,
                                });
                            }
                        }
                    }
                    None => {
                        errors.push(SceneValidationError::MissingRequiredEntity {
                            node_id: node.id.clone(),
                            entity_id: req.entity_id.clone(),
                        });
                    }
                }
            }
        }

        errors
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_story_graph_serialization() {
        let mut graph = StoryGraphData::new("intro", "Introduction");
        graph.root_node_id = "start".to_string();
        graph.add_node(StoryNodeData::dialogue("start", "Narrator", "Welcome!"));
        graph.add_node(StoryNodeData::end("end"));

        let json = serde_json::to_string_pretty(&graph).unwrap();
        let parsed: StoryGraphData = serde_json::from_str(&json).unwrap();
        assert_eq!(graph.id, parsed.id);
        assert_eq!(graph.nodes.len(), parsed.nodes.len());
    }

    #[test]
    fn test_validation_missing_root() {
        let graph = StoryGraphData {
            id: "test".to_string(),
            name: "Test".to_string(),
            description: String::new(),
            graph_type: StoryGraphType::Dialogue,
            root_node_id: "nonexistent".to_string(),
            variables: HashMap::new(),

            nodes: vec![],
        };

        let errors = graph.validate();
        assert!(errors
            .iter()
            .any(|e| matches!(e, ValidationError::MissingRootNode(_))));
    }

    #[test]
    fn test_validate_against_scene() {
        use crate::data::scene::{Entity, EntityType, Scene};

        let mut graph = StoryGraphData::new("test", "Test");
        let mut node = StoryNodeData::dialogue("node1", "Hero", "Hi");
        node.required_entities.push(RequiredEntity {
            entity_id: "hero_01".to_string(),
            entity_type: Some(EntityType::Npc),
        });
        graph.add_node(node);

        // Case 1: Entity missing
        let scene = Scene::default();
        let errors = graph.validate_against_scene(&scene);
        assert_eq!(errors.len(), 1);
        assert!(matches!(
            errors[0],
            SceneValidationError::MissingRequiredEntity { .. }
        ));

        // Case 2: Entity exists but wrong type
        let mut scene = Scene::default();
        let mut entity = Entity::new("hero_01", "Hero");
        entity.entity_type = EntityType::Enemy; // Wrong type
        scene.add_entity(entity);

        let errors = graph.validate_against_scene(&scene);
        assert_eq!(errors.len(), 1);
        assert!(matches!(
            errors[0],
            SceneValidationError::WrongEntityType { .. }
        ));

        // Case 3: Correct
        let mut scene = Scene::default();
        let mut entity = Entity::new("hero_01", "Hero");
        entity.entity_type = EntityType::Npc; // Correct type
        scene.add_entity(entity);

        let errors = graph.validate_against_scene(&scene);
        assert!(errors.is_empty());
    }

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

    #[test]
    fn test_validation_unreachable_node() {
        let mut graph = StoryGraphData::new("test", "Test");
        graph.root_node_id = "start".into();
        graph.add_node(StoryNodeData::start("start", Some("end")));
        graph.add_node(StoryNodeData::end("end"));
        graph.add_node(StoryNodeData::dialogue(
            "orphan",
            "Ghost",
            "Nobody reaches me",
        ));

        let errors = graph.validate();
        assert!(errors
            .iter()
            .any(|e| matches!(e, ValidationError::UnreachableNode(id) if id == "orphan")));
    }

    #[test]
    fn test_validation_all_reachable() {
        let mut graph = StoryGraphData::new("test", "Test");
        graph.root_node_id = "start".into();
        graph.add_node(StoryNodeData::start("start", Some("end")));
        graph.add_node(StoryNodeData::end("end"));

        let errors = graph.validate();
        assert!(!errors
            .iter()
            .any(|e| matches!(e, ValidationError::UnreachableNode(_))));
    }
}
