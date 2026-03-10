use crate::audio::AudioCommand;
use crate::data::story::{StoryGraphData, StoryNodeVariant};
use bevy::prelude::*;
use std::collections::HashMap;

/// Unique identifier for a node in the graph.
pub type NodeId = usize;

/// Represents a single logic or content step in the story.
#[derive(Debug, Clone, Reflect)]
pub enum StoryNode {
    /// Show dialogue and wait for user confirmation.
    Dialogue {
        speaker: String,
        text: String,
        portrait: Option<String>,
        next: Option<NodeId>,
    },
    /// Present a set of choices to the player.
    Choice {
        speaker: String,
        prompt: String,
        options: Vec<GraphChoice>,
    },
    /// Play a sound effect or music track.
    Audio {
        command: AudioCommand,
        next: Option<NodeId>,
    },
    /// Change the background scene.
    Scene {
        path: String,
        duration: f32,
        next: Option<NodeId>,
    },
    /// Conditional branch based on a story flag.
    Branch {
        flag: String,
        if_true: Option<NodeId>,
        if_false: Option<NodeId>,
    },
    /// Set or unset a story flag.
    SetFlag {
        flag: String,
        value: bool,
        next: Option<NodeId>,
    },
    /// Wait for a specified duration in seconds.
    Wait { duration: f32, next: Option<NodeId> },
    /// A generic event trigger for game-specific logic.
    Event {
        event_id: String,
        payload: String,
        next: Option<NodeId>,
    },
    /// Start execution of the graph.
    Start { next: Option<NodeId> },
    /// End execution of the current graph.
    End,
}

/// A choice option within a Choice node.
#[derive(Debug, Clone, Reflect)]
pub struct GraphChoice {
    pub text: String,
    pub next: Option<NodeId>,
    pub flag_required: Option<String>,
}

/// The graph container holding all nodes.
#[derive(Resource, Default, Clone, Reflect)]
#[reflect(Resource)]
pub struct StoryGraph {
    pub nodes: HashMap<NodeId, StoryNode>,
    pub start_node: Option<NodeId>,
    pub(crate) next_id: usize,
}

impl StoryGraph {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            start_node: None,
            next_id: 0,
        }
    }

    pub fn add(&mut self, node: StoryNode) -> NodeId {
        let id = self.next_id;
        self.nodes.insert(id, node);
        self.next_id += 1;
        id
    }

    pub fn set_start(&mut self, id: NodeId) {
        self.start_node = Some(id);
    }
}

/// Generic container for story flags (booleans).
#[derive(Resource, Default, Debug, Clone, Reflect)]
#[reflect(Resource)]
pub struct StoryFlags(pub HashMap<String, bool>);

impl StoryFlags {
    pub fn set(&mut self, flag: &str, value: bool) {
        self.0.insert(flag.to_string(), value);
    }

    pub fn get(&self, flag: &str) -> bool {
        *self.0.get(flag).unwrap_or(&false)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
pub enum ExecutionStatus {
    #[default]
    Idle,
    Running,
    WaitingForInput,
    WaitingForTimer,
    Paused,
}

#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
pub struct GraphExecutor {
    pub active_graph: Option<StoryGraph>,
    pub current_node: Option<NodeId>,
    pub status: ExecutionStatus,
    pub wait_timer: Timer,
}

impl GraphExecutor {
    pub fn start(&mut self, graph: StoryGraph) {
        let start = graph.start_node;
        self.active_graph = Some(graph);
        self.current_node = start;
        self.status = ExecutionStatus::Running;
    }

    /// Helper to bridge Editor Data -> Runtime Graph
    pub fn load_from_data(&mut self, data: &StoryGraphData) {
        let mut graph = StoryGraph::new();
        let mut id_map: HashMap<String, NodeId> = HashMap::new();

        // Pass 1: Allocate IDs
        for node_data in &data.nodes {
            let next_id = graph.next_id;
            graph.add(StoryNode::End);
            id_map.insert(node_data.id.clone(), next_id);
        }

        // Pass 2: Overwrite with actual data
        for node_data in &data.nodes {
            let runtime_id = id_map[&node_data.id];

            let resolve = |opt_id: &Option<String>| -> Option<NodeId> {
                opt_id.as_ref().and_then(|id| id_map.get(id).cloned())
            };

            let node = match &node_data.data {
                StoryNodeVariant::Start(d) => StoryNode::Start {
                    next: resolve(&d.next_node_id),
                },
                StoryNodeVariant::Dialogue(d) => StoryNode::Dialogue {
                    speaker: d.speaker_id.clone(),
                    text: d.text.get("en").cloned().unwrap_or_default(),
                    portrait: d.portrait_id.clone(),
                    next: resolve(&d.next_node_id),
                },
                StoryNodeVariant::Choice(c) => StoryNode::Choice {
                    speaker: "Player".into(),
                    prompt: c.prompt.get("en").cloned().unwrap_or_default(),
                    options: c
                        .options
                        .iter()
                        .map(|o| GraphChoice {
                            text: o.text.get("en").cloned().unwrap_or_default(),
                            next: Some(id_map[&o.target_node_id]),
                            flag_required: None,
                        })
                        .collect(),
                },
                StoryNodeVariant::Action(a) => StoryNode::Event {
                    event_id: "lua_script".into(),
                    payload: a.lua_script_id.clone(),
                    next: resolve(&a.next_node_id),
                },
                StoryNodeVariant::End(e) => {
                    if let Some(scene) = &e.target_scene_id {
                        StoryNode::Scene {
                            path: scene.clone(),
                            duration: 1.0,
                            next: None,
                        }
                    } else {
                        StoryNode::End
                    }
                }
                _ => StoryNode::End,
            };

            graph.nodes.insert(runtime_id, node);
        }

        if let Some(start_id) = id_map.get(&data.root_node_id) {
            graph.set_start(*start_id);
        }

        self.start(graph);
    }
}

/// Events sent FROM the Executor TO the UI/Game
#[derive(Message, Debug, Clone)]
pub enum StoryFlowEvent {
    ShowDialogue {
        speaker: String,
        text: String,
        portrait: Option<String>,
    },
    ShowChoices {
        prompt: String,
        options: Vec<String>,
    },
    GraphComplete,
}

/// Events sent FROM the UI/Game TO the Executor
#[derive(Message, Debug, Clone)]
pub enum StoryInputEvent {
    Advance,
    SelectChoice(usize),
}

#[derive(Message)]
pub struct StoryEvent {
    pub id: String,
    pub payload: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_story_graph_add_and_start() {
        let mut graph = StoryGraph::new();
        let id = graph.add(StoryNode::Start { next: None });
        graph.add(StoryNode::End);
        graph.set_start(id);
        assert_eq!(graph.nodes.len(), 2);
        assert_eq!(graph.start_node, Some(0));
    }

    #[test]
    fn test_story_graph_sequential_ids() {
        let mut graph = StoryGraph::new();
        let a = graph.add(StoryNode::End);
        let b = graph.add(StoryNode::End);
        let c = graph.add(StoryNode::End);
        assert_eq!(a, 0);
        assert_eq!(b, 1);
        assert_eq!(c, 2);
    }

    #[test]
    fn test_story_flags_set_get() {
        let mut flags = StoryFlags::default();
        flags.set("met_hamster", true);
        assert!(flags.get("met_hamster"));
        assert!(!flags.get("unset_flag"));
    }

    #[test]
    fn test_story_flags_overwrite() {
        let mut flags = StoryFlags::default();
        flags.set("x", true);
        flags.set("x", false);
        assert!(!flags.get("x"));
    }

    #[test]
    fn test_execution_status_default() {
        assert_eq!(ExecutionStatus::default(), ExecutionStatus::Idle);
    }

    #[test]
    fn test_executor_start() {
        let mut graph = StoryGraph::new();
        let start_id = graph.add(StoryNode::Start { next: None });
        graph.set_start(start_id);

        let mut executor = GraphExecutor::default();
        executor.start(graph);
        assert_eq!(executor.status, ExecutionStatus::Running);
        assert_eq!(executor.current_node, Some(start_id));
    }
}
