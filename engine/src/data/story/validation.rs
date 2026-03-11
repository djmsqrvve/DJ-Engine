use std::collections::{HashSet, VecDeque};

use crate::data::scene::{EntityType, Scene};

use super::{graph::StoryGraphData, nodes::StoryNodeVariant};

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

pub fn validate_graph(graph: &StoryGraphData) -> Vec<ValidationError> {
    let mut errors = Vec::new();
    let node_ids: HashSet<_> = graph.nodes.iter().map(|node| node.id.as_str()).collect();

    if !node_ids.contains(graph.root_node_id.as_str()) {
        errors.push(ValidationError::MissingRootNode(graph.root_node_id.clone()));
    }

    for node in &graph.nodes {
        for next_id in node.next_node_ids() {
            if !node_ids.contains(next_id) {
                errors.push(ValidationError::BrokenReference {
                    from_node: node.id.clone(),
                    to_node: next_id.to_string(),
                });
            }
        }

        if node.next_node_ids().is_empty() && !matches!(node.data, StoryNodeVariant::End(_)) {
            errors.push(ValidationError::DeadEnd(node.id.clone()));
        }
    }

    if node_ids.contains(graph.root_node_id.as_str()) {
        let mut reachable = HashSet::new();
        let mut queue = VecDeque::new();
        queue.push_back(graph.root_node_id.as_str());
        reachable.insert(graph.root_node_id.as_str());

        while let Some(current) = queue.pop_front() {
            if let Some(node) = graph.find_node(current) {
                for next in node.next_node_ids() {
                    if reachable.insert(next) {
                        queue.push_back(next);
                    }
                }
            }
        }

        for node in &graph.nodes {
            if !reachable.contains(node.id.as_str()) {
                errors.push(ValidationError::UnreachableNode(node.id.clone()));
            }
        }
    }

    errors
}

pub fn validate_graph_against_scene(
    graph: &StoryGraphData,
    scene: &Scene,
) -> Vec<SceneValidationError> {
    let mut errors = Vec::new();

    for node in &graph.nodes {
        for requirement in &node.required_entities {
            match scene.find_entity(&requirement.entity_id) {
                Some(entity) => {
                    if let Some(expected_type) = requirement.entity_type {
                        if entity.entity_type != expected_type {
                            errors.push(SceneValidationError::WrongEntityType {
                                node_id: node.id.clone(),
                                entity_id: requirement.entity_id.clone(),
                                expected: expected_type,
                                found: entity.entity_type,
                            });
                        }
                    }
                }
                None => {
                    errors.push(SceneValidationError::MissingRequiredEntity {
                        node_id: node.id.clone(),
                        entity_id: requirement.entity_id.clone(),
                    });
                }
            }
        }
    }

    errors
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::scene::{Entity, EntityType, Scene};
    use crate::data::story::{StoryGraphData, StoryNodeData};
    use std::collections::HashMap;

    #[test]
    fn test_validation_missing_root() {
        let graph = StoryGraphData {
            id: "test".to_string(),
            name: "Test".to_string(),
            description: String::new(),
            graph_type: super::super::StoryGraphType::Dialogue,
            root_node_id: "nonexistent".to_string(),
            variables: HashMap::new(),
            nodes: vec![],
        };

        let errors = validate_graph(&graph);
        assert!(errors
            .iter()
            .any(|error| matches!(error, ValidationError::MissingRootNode(_))));
    }

    #[test]
    fn test_validate_against_scene() {
        let mut graph = StoryGraphData::new("test", "Test");
        let mut node = StoryNodeData::dialogue("node1", "Hero", "Hi");
        node.required_entities.push(super::super::RequiredEntity {
            entity_id: "hero_01".to_string(),
            entity_type: Some(EntityType::Npc),
        });
        graph.add_node(node);

        let scene = Scene::default();
        let errors = validate_graph_against_scene(&graph, &scene);
        assert_eq!(errors.len(), 1);
        assert!(matches!(
            errors[0],
            SceneValidationError::MissingRequiredEntity { .. }
        ));

        let mut scene = Scene::default();
        let mut entity = Entity::new("hero_01", "Hero");
        entity.entity_type = EntityType::Enemy;
        scene.add_entity(entity);

        let errors = validate_graph_against_scene(&graph, &scene);
        assert_eq!(errors.len(), 1);
        assert!(matches!(
            errors[0],
            SceneValidationError::WrongEntityType { .. }
        ));

        let mut scene = Scene::default();
        let mut entity = Entity::new("hero_01", "Hero");
        entity.entity_type = EntityType::Npc;
        scene.add_entity(entity);

        assert!(validate_graph_against_scene(&graph, &scene).is_empty());
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

        let errors = validate_graph(&graph);
        assert!(errors
            .iter()
            .any(|error| matches!(error, ValidationError::UnreachableNode(id) if id == "orphan")));
    }

    #[test]
    fn test_validation_all_reachable() {
        let mut graph = StoryGraphData::new("test", "Test");
        graph.root_node_id = "start".into();
        graph.add_node(StoryNodeData::start("start", Some("end")));
        graph.add_node(StoryNodeData::end("end"));

        let errors = validate_graph(&graph);
        assert!(!errors
            .iter()
            .any(|error| matches!(error, ValidationError::UnreachableNode(_))));
    }
}
