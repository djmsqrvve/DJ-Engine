use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::data::scene::Scene;

use super::nodes::StoryNodeData;
use super::types::StoryGraphType;
use super::validation::{
    validate_graph, validate_graph_against_scene, SceneValidationError, ValidationError,
};

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
        self.nodes.iter().find(|node| node.id == id)
    }

    /// Validate the story graph and return any errors.
    pub fn validate(&self) -> Vec<ValidationError> {
        validate_graph(self)
    }

    /// Validate the story graph against a specific scene.
    pub fn validate_against_scene(&self, scene: &Scene) -> Vec<SceneValidationError> {
        validate_graph_against_scene(self, scene)
    }
}

pub(super) fn register_types(app: &mut App) {
    app.register_type::<StoryGraphData>();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::story::StoryNodeData;
    use serde_json::json;

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
    fn test_story_graph_json_shape_stable() {
        let mut graph = StoryGraphData::new("intro", "Introduction");
        graph.root_node_id = "start".to_string();
        graph
            .variables
            .insert("intro_complete".to_string(), serde_json::json!(true));
        graph.add_node(StoryNodeData::start("start", Some("end")));
        graph.add_node(StoryNodeData::end("end"));

        let json = serde_json::to_value(&graph).unwrap();
        assert_eq!(json["id"], json!("intro"));
        assert_eq!(json["root_node_id"], json!("start"));
        assert_eq!(json["nodes"][0]["data"]["type"], json!("start"));
        assert_eq!(json["nodes"][1]["data"]["type"], json!("end"));
        assert_eq!(json["variables"]["intro_complete"], json!(true));
    }
}
