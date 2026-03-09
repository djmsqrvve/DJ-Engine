//! Node-based story execution system.
//!
//! Replaces linear dialogue queues with a directed graph of nodes.
//! Supports branching logic, events, and complex narrative flow.

use crate::audio::AudioCommand;
use crate::scene::ChangeSceneEvent;
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
    next_id: usize,
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
    // For prototype simplicity, we store the struct directly.
    pub active_graph: Option<StoryGraph>,
    pub current_node: Option<NodeId>,
    pub status: ExecutionStatus,
    pub wait_timer: Timer,
}

use crate::data::story::{StoryGraphData, StoryNodeVariant};

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
            let next_id = graph.next_id; // Peek next ID
                                         // We insert a placeholder to reserve the ID
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
                    speaker: "Player".into(), // Default?
                    prompt: c.prompt.get("en").cloned().unwrap_or_default(),
                    options: c
                        .options
                        .iter()
                        .map(|o| GraphChoice {
                            text: o.text.get("en").cloned().unwrap_or_default(),
                            next: Some(id_map[&o.target_node_id]), // Choices must have targets?
                            flag_required: None,
                        })
                        .collect(),
                },
                StoryNodeVariant::Action(a) => {
                    // For now, assume action is Event? or Lua script?
                    // Maps to Event for prototype
                    StoryNode::Event {
                        event_id: "lua_script".into(),
                        payload: a.lua_script_id.clone(),
                        next: resolve(&a.next_node_id),
                    }
                }
                StoryNodeVariant::End(e) => {
                    if let Some(scene) = &e.target_scene_id {
                        StoryNode::Scene {
                            path: scene.clone(),
                            duration: 1.0,
                            next: None, // End of graph after scene change? Or new graph?
                        }
                    } else {
                        StoryNode::End
                    }
                }
                _ => StoryNode::End, // Unimplemented variants
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
    }, // Only send text to UI
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

pub struct StoryGraphPlugin;

impl Plugin for StoryGraphPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<StoryGraph>()
            .register_type::<StoryNode>()
            .register_type::<GraphChoice>()
            .register_type::<StoryFlags>()
            .register_type::<ExecutionStatus>()
            .register_type::<GraphExecutor>()
            .init_resource::<GraphExecutor>()
            .init_resource::<StoryFlags>()
            .add_message::<StoryEvent>()
            .add_message::<StoryFlowEvent>()
            .add_message::<StoryInputEvent>()
            .add_systems(Update, execute_graph);
    }
}

enum NodeAction {
    WaitInput,
    WaitTimer(f32),
    Advance,
    Jump(NodeId),
    End,
}

fn execute_graph(
    mut executor: ResMut<GraphExecutor>,
    mut flags: ResMut<StoryFlags>,
    mut audio_events: MessageWriter<AudioCommand>,
    mut scene_events: MessageWriter<ChangeSceneEvent>,
    mut flow_events: MessageWriter<StoryFlowEvent>,
    mut story_events: MessageWriter<StoryEvent>,
    mut input_events: MessageReader<StoryInputEvent>,
    time: Res<Time>,
) {
    // 1. Handle Input (if waiting)
    if executor.status == ExecutionStatus::WaitingForInput {
        for event in input_events.read() {
            match event {
                StoryInputEvent::Advance => {
                    executor.status = ExecutionStatus::Running;
                }
                StoryInputEvent::SelectChoice(index) => {
                    handle_choice_selection(&mut executor, *index);
                    // handle_choice_selection sets status to Running
                }
            }
        }
    }

    // 2. Handle Timer (if waiting)
    if executor.status == ExecutionStatus::WaitingForTimer {
        executor.wait_timer.tick(time.delta());
        if executor.wait_timer.is_finished() {
            executor.status = ExecutionStatus::Running;
            advance_node(&mut executor);
        }
    }

    // 3. Process Execution Loop
    // We loop to handle immediate transitions (Audio -> Scene -> Branch -> Dialogue) in one frame
    let mut loops = 0;
    while executor.status == ExecutionStatus::Running && loops < 100 {
        loops += 1;

        // In a real asset system we'd use Handle and Assets<StoryGraph>

        if let Some(graph) = &executor.active_graph {
            if let Some(node_id) = executor.current_node {
                if let Some(node) = graph.nodes.get(&node_id) {
                    let action = process_node(
                        node,
                        &mut flags,
                        &mut flow_events,
                        &mut audio_events,
                        &mut scene_events,
                        &mut story_events,
                    );

                    match action {
                        NodeAction::WaitInput => {
                            executor.status = ExecutionStatus::WaitingForInput;
                        }
                        NodeAction::WaitTimer(duration) => {
                            executor.status = ExecutionStatus::WaitingForTimer;
                            executor.wait_timer = Timer::from_seconds(duration, TimerMode::Once);
                        }
                        NodeAction::Advance => {
                            advance_node(&mut executor);
                        }
                        NodeAction::Jump(target_id) => {
                            executor.current_node = Some(target_id);
                        }
                        NodeAction::End => {
                            executor.status = ExecutionStatus::Idle;
                            flow_events.write(StoryFlowEvent::GraphComplete);
                        }
                    }
                } else {
                    executor.status = ExecutionStatus::Idle;
                }
            } else {
                executor.status = ExecutionStatus::Idle;
                flow_events.write(StoryFlowEvent::GraphComplete);
            }
        } else {
            executor.status = ExecutionStatus::Idle;
        }
    }
}

fn advance_node(executor: &mut GraphExecutor) {
    // Helper to move to the 'default next' of the current node
    // This duplicates logic inside process_node if we aren't careful,
    // but process_node returns 'Advance' meaning "Go to my .next field".

    // We need to peek at the current node to know its next.
    // This is slightly inefficient but safe.
    let next_id = if let Some(graph) = &executor.active_graph {
        if let Some(node_id) = executor.current_node {
            if let Some(node) = graph.nodes.get(&node_id) {
                match node {
                    StoryNode::Dialogue { next, .. } => *next,
                    StoryNode::Audio { next, .. } => *next,
                    StoryNode::Scene { next, .. } => *next,
                    StoryNode::Wait { next, .. } => *next,
                    StoryNode::SetFlag { next, .. } => *next,
                    StoryNode::Event { next, .. } => *next,
                    StoryNode::Start { next, .. } => *next,
                    _ => None,
                }
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    executor.current_node = next_id;
}

fn handle_choice_selection(executor: &mut GraphExecutor, index: usize) {
    let next_id = if let Some(graph) = &executor.active_graph {
        if let Some(node_id) = executor.current_node {
            if let StoryNode::Choice { options, .. } = &graph.nodes[&node_id] {
                options.get(index).and_then(|opt| opt.next)
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    executor.current_node = next_id;
    executor.status = ExecutionStatus::Running;
}

fn process_node(
    node: &StoryNode,
    flags: &mut StoryFlags,
    flow: &mut MessageWriter<StoryFlowEvent>,
    audio: &mut MessageWriter<AudioCommand>,
    scene: &mut MessageWriter<ChangeSceneEvent>,
    story: &mut MessageWriter<StoryEvent>,
) -> NodeAction {
    match node {
        StoryNode::Dialogue {
            speaker,
            text,
            portrait,
            ..
        } => {
            flow.write(StoryFlowEvent::ShowDialogue {
                speaker: speaker.clone(),
                text: text.clone(),
                portrait: portrait.clone(),
            });
            NodeAction::WaitInput
        }
        StoryNode::Choice {
            prompt, options, ..
        } => {
            let option_texts = options.iter().map(|o| o.text.clone()).collect();
            flow.write(StoryFlowEvent::ShowChoices {
                prompt: prompt.clone(),
                options: option_texts,
            });
            NodeAction::WaitInput
        }
        StoryNode::Audio { command, .. } => {
            audio.write(command.clone());
            NodeAction::Advance
        }
        StoryNode::Scene { path, duration, .. } => {
            scene.write(ChangeSceneEvent {
                background_path: path.clone(),
                duration: *duration,
            });
            NodeAction::Advance // Or WaitTimer if we want to block? For now Advance.
        }
        StoryNode::Wait { duration, .. } => NodeAction::WaitTimer(*duration),
        StoryNode::Branch {
            flag,
            if_true,
            if_false,
        } => {
            if flags.get(flag) {
                if let Some(id) = if_true {
                    NodeAction::Jump(*id)
                } else {
                    NodeAction::Advance
                }
            } else {
                if let Some(id) = if_false {
                    NodeAction::Jump(*id)
                } else {
                    NodeAction::Advance
                }
            }
        }
        StoryNode::SetFlag { flag, value, .. } => {
            flags.set(flag, *value);
            NodeAction::Advance
        }
        StoryNode::Event {
            event_id, payload, ..
        } => {
            story.write(StoryEvent {
                id: event_id.clone(),
                payload: payload.clone(),
            });
            NodeAction::Advance
        }
        StoryNode::End => NodeAction::End,
        StoryNode::Start { .. } => NodeAction::Advance,
    }
}
