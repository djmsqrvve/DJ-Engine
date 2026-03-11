use crate::audio::AudioCommand;
use crate::data::story::graph::StoryGraphData;
use crate::data::story::nodes::StoryNodeVariant;
use crate::data::story::types::ConditionOperator;
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
    /// Conditional branch based on a variable condition.
    Conditional {
        variable: String,
        operator: ConditionOperator,
        #[reflect(ignore)]
        value: serde_json::Value,
        if_true: Option<NodeId>,
        if_false: Option<NodeId>,
    },
    /// Control the camera position, zoom, and angle.
    Camera {
        preset_id: Option<String>,
        position: Vec3,
        zoom: f32,
        angle: f32,
        duration: f32,
        easing: String,
        next: Option<NodeId>,
    },
    /// Control time scale or pause gameplay.
    TimeControl {
        pause_gameplay: bool,
        time_scale: f32,
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

#[derive(Resource, Default, Debug, Clone, Reflect)]
#[reflect(Resource)]
pub struct StoryVariables(#[reflect(ignore)] pub HashMap<String, serde_json::Value>);

impl StoryVariables {
    pub fn set(&mut self, key: &str, value: serde_json::Value) {
        self.0.insert(key.to_string(), value);
    }

    pub fn get(&self, key: &str) -> Option<&serde_json::Value> {
        self.0.get(key)
    }

    pub fn evaluate(
        &self,
        variable: &str,
        operator: ConditionOperator,
        expected: &serde_json::Value,
    ) -> bool {
        let Some(actual) = self.0.get(variable) else {
            return false;
        };

        match (actual, expected) {
            (serde_json::Value::Bool(a), serde_json::Value::Bool(b)) => match operator {
                ConditionOperator::Equals => a == b,
                ConditionOperator::NotEquals => a != b,
                _ => false,
            },
            (serde_json::Value::Number(a), serde_json::Value::Number(b)) => {
                let (Some(a), Some(b)) = (a.as_f64(), b.as_f64()) else {
                    return false;
                };
                match operator {
                    ConditionOperator::Equals => (a - b).abs() < f64::EPSILON,
                    ConditionOperator::NotEquals => (a - b).abs() >= f64::EPSILON,
                    ConditionOperator::LessThan => a < b,
                    ConditionOperator::LessThanOrEquals => a <= b,
                    ConditionOperator::GreaterThan => a > b,
                    ConditionOperator::GreaterThanOrEquals => a >= b,
                    ConditionOperator::Contains => false,
                }
            }
            (serde_json::Value::String(a), serde_json::Value::String(b)) => match operator {
                ConditionOperator::Equals => a == b,
                ConditionOperator::NotEquals => a != b,
                ConditionOperator::Contains => a.contains(b.as_str()),
                _ => false,
            },
            _ => false,
        }
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
    #[reflect(ignore)]
    pub initial_variables: HashMap<String, serde_json::Value>,
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
                StoryNodeVariant::Conditional(c) => StoryNode::Conditional {
                    variable: c.condition.variable.clone(),
                    operator: c.condition.operator,
                    value: c.condition.value.clone(),
                    if_true: id_map.get(&c.true_target_node_id).cloned(),
                    if_false: id_map.get(&c.false_target_node_id).cloned(),
                },
                StoryNodeVariant::Camera(c) => StoryNode::Camera {
                    preset_id: c.preset_id.clone(),
                    position: Vec3::new(c.position.x, c.position.y, c.position.z),
                    zoom: c.zoom,
                    angle: c.angle,
                    duration: c.duration,
                    easing: c.easing.clone(),
                    next: resolve(&c.next_node_id),
                },
                StoryNodeVariant::TimeControl(t) => StoryNode::TimeControl {
                    pause_gameplay: t.pause_gameplay,
                    time_scale: t.time_scale,
                    next: resolve(&t.next_node_id),
                },
            };

            graph.nodes.insert(runtime_id, node);
        }

        if let Some(start_id) = id_map.get(&data.root_node_id) {
            graph.set_start(*start_id);
        }

        self.initial_variables = data.variables.clone();
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

#[derive(Message, Debug, Clone)]
pub struct CameraCommand {
    pub preset_id: Option<String>,
    pub position: Vec3,
    pub zoom: f32,
    pub angle: f32,
    pub duration: f32,
    pub easing: String,
}

#[derive(Message, Debug, Clone)]
pub struct TimeControlCommand {
    pub pause_gameplay: bool,
    pub time_scale: f32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::Vec3Data;

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
        flags.set("met_guide", true);
        assert!(flags.get("met_guide"));
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

    #[test]
    fn test_story_variables_set_get() {
        let mut vars = StoryVariables::default();
        vars.set("health", serde_json::json!(50));
        assert_eq!(vars.get("health"), Some(&serde_json::json!(50)));
        assert_eq!(vars.get("missing"), None);
    }

    #[test]
    fn test_story_variables_evaluate_bool() {
        let mut vars = StoryVariables::default();
        vars.set("flag", serde_json::json!(true));
        assert!(vars.evaluate("flag", ConditionOperator::Equals, &serde_json::json!(true)));
        assert!(!vars.evaluate("flag", ConditionOperator::Equals, &serde_json::json!(false)));
        assert!(vars.evaluate(
            "flag",
            ConditionOperator::NotEquals,
            &serde_json::json!(false)
        ));
    }

    #[test]
    fn test_story_variables_evaluate_number() {
        let mut vars = StoryVariables::default();
        vars.set("health", serde_json::json!(50.0));
        assert!(vars.evaluate(
            "health",
            ConditionOperator::LessThan,
            &serde_json::json!(75.0)
        ));
        assert!(!vars.evaluate(
            "health",
            ConditionOperator::LessThan,
            &serde_json::json!(25.0)
        ));
        assert!(vars.evaluate(
            "health",
            ConditionOperator::GreaterThan,
            &serde_json::json!(25.0)
        ));
        assert!(vars.evaluate(
            "health",
            ConditionOperator::Equals,
            &serde_json::json!(50.0)
        ));
        assert!(vars.evaluate(
            "health",
            ConditionOperator::GreaterThanOrEquals,
            &serde_json::json!(50.0)
        ));
        assert!(vars.evaluate(
            "health",
            ConditionOperator::LessThanOrEquals,
            &serde_json::json!(50.0)
        ));
    }

    #[test]
    fn test_story_variables_evaluate_string() {
        let mut vars = StoryVariables::default();
        vars.set("name", serde_json::json!("guide"));
        assert!(vars.evaluate(
            "name",
            ConditionOperator::Equals,
            &serde_json::json!("guide")
        ));
        assert!(vars.evaluate(
            "name",
            ConditionOperator::Contains,
            &serde_json::json!("gui")
        ));
        assert!(!vars.evaluate(
            "name",
            ConditionOperator::Contains,
            &serde_json::json!("cat")
        ));
    }

    #[test]
    fn test_story_variables_evaluate_missing() {
        let vars = StoryVariables::default();
        assert!(!vars.evaluate(
            "missing",
            ConditionOperator::Equals,
            &serde_json::json!(true)
        ));
    }

    #[test]
    fn test_load_from_data_conditional() {
        use crate::data::story::*;

        let mut data = StoryGraphData::new("test", "Test");
        data.add_node(StoryNodeData::start("start", Some("cond")));
        data.add_node(StoryNodeData {
            id: "cond".into(),
            position: Vec3Data::default(),
            data: StoryNodeVariant::Conditional(ConditionalNodeData {
                condition: StoryCondition {
                    variable: "health".into(),
                    operator: ConditionOperator::LessThan,
                    value: serde_json::json!(50),
                },
                true_target_node_id: "yes".into(),
                false_target_node_id: "no".into(),
            }),
            required_entities: vec![],
            required_items: vec![],
        });
        data.add_node(StoryNodeData::end("yes"));
        data.add_node(StoryNodeData::end("no"));
        data.root_node_id = "start".into();

        let mut executor = GraphExecutor::default();
        executor.load_from_data(&data);

        let graph = executor.active_graph.as_ref().unwrap();
        let cond_node = &graph.nodes[&1];
        assert!(matches!(cond_node, StoryNode::Conditional { .. }));
    }

    #[test]
    fn test_load_from_data_camera() {
        use crate::data::story::*;

        let mut data = StoryGraphData::new("test", "Test");
        data.add_node(StoryNodeData::start("start", Some("cam")));
        data.add_node(StoryNodeData {
            id: "cam".into(),
            position: Vec3Data::default(),
            data: StoryNodeVariant::Camera(CameraNodeData {
                zoom: 2.0,
                duration: 0.5,
                ..Default::default()
            }),
            required_entities: vec![],
            required_items: vec![],
        });
        data.add_node(StoryNodeData::end("end"));
        data.root_node_id = "start".into();

        let mut executor = GraphExecutor::default();
        executor.load_from_data(&data);

        let graph = executor.active_graph.as_ref().unwrap();
        let cam_node = &graph.nodes[&1];
        assert!(
            matches!(cam_node, StoryNode::Camera { zoom, .. } if (*zoom - 2.0).abs() < f32::EPSILON)
        );
    }

    #[test]
    fn test_load_from_data_time_control() {
        use crate::data::story::*;

        let mut data = StoryGraphData::new("test", "Test");
        data.add_node(StoryNodeData::start("start", Some("tc")));
        data.add_node(StoryNodeData {
            id: "tc".into(),
            position: Vec3Data::default(),
            data: StoryNodeVariant::TimeControl(TimeControlNodeData {
                pause_gameplay: true,
                time_scale: 0.5,
                next_node_id: None,
            }),
            required_entities: vec![],
            required_items: vec![],
        });
        data.root_node_id = "start".into();

        let mut executor = GraphExecutor::default();
        executor.load_from_data(&data);

        let graph = executor.active_graph.as_ref().unwrap();
        let tc_node = &graph.nodes[&1];
        assert!(matches!(
            tc_node,
            StoryNode::TimeControl {
                pause_gameplay: true,
                ..
            }
        ));
    }
}
