use super::types::{
    CameraCommand, ExecutionStatus, GraphExecutor, NodeId, StoryEvent, StoryFlags, StoryFlowEvent,
    StoryInputEvent, StoryNode, StoryVariables, TimeControlCommand,
};
use crate::audio::AudioCommand;
use crate::scene::ChangeSceneEvent;
use bevy::prelude::*;

enum NodeAction {
    WaitInput,
    WaitTimer(f32),
    Advance,
    Jump(NodeId),
    End,
}

pub(super) fn execute_graph(
    mut executor: ResMut<GraphExecutor>,
    mut flags: ResMut<StoryFlags>,
    mut variables: ResMut<StoryVariables>,
    mut audio_events: MessageWriter<AudioCommand>,
    mut scene_events: MessageWriter<ChangeSceneEvent>,
    mut flow_events: MessageWriter<StoryFlowEvent>,
    mut story_events: MessageWriter<StoryEvent>,
    mut camera_commands: MessageWriter<CameraCommand>,
    mut time_commands: MessageWriter<TimeControlCommand>,
    mut input_events: MessageReader<StoryInputEvent>,
    time: Res<Time>,
) {
    // 0. Seed initial variables from loaded graph data
    if !executor.initial_variables.is_empty() {
        for (k, v) in executor.initial_variables.drain() {
            variables.set(&k, v);
        }
    }

    // 1. Handle Input (if waiting)
    if executor.status == ExecutionStatus::WaitingForInput {
        for event in input_events.read() {
            match event {
                StoryInputEvent::Advance => {
                    executor.status = ExecutionStatus::Running;
                    advance_node(&mut executor);
                }
                StoryInputEvent::SelectChoice(index) => {
                    handle_choice_selection(&mut executor, *index);
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

        if let Some(graph) = &executor.active_graph {
            if let Some(node_id) = executor.current_node {
                if let Some(node) = graph.nodes.get(&node_id) {
                    let action = process_node(
                        node,
                        &mut flags,
                        &variables,
                        &mut flow_events,
                        &mut audio_events,
                        &mut scene_events,
                        &mut story_events,
                        &mut camera_commands,
                        &mut time_commands,
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
                    StoryNode::Camera { next, .. } => *next,
                    StoryNode::TimeControl { next, .. } => *next,
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

/// Resource controlling gameplay time scale, driven by TimeControlCommand.
#[derive(Resource, Debug, Clone)]
pub struct GameTimeScale {
    pub paused: bool,
    pub scale: f32,
}

impl Default for GameTimeScale {
    fn default() -> Self {
        Self {
            paused: false,
            scale: 1.0,
        }
    }
}

impl GameTimeScale {
    /// Effective multiplier: 0.0 when paused, otherwise the scale factor.
    pub fn effective(&self) -> f32 {
        if self.paused {
            0.0
        } else {
            self.scale
        }
    }
}

pub(super) fn handle_time_control(
    mut commands: MessageReader<TimeControlCommand>,
    mut time_scale: ResMut<GameTimeScale>,
) {
    for cmd in commands.read() {
        time_scale.paused = cmd.pause_gameplay;
        time_scale.scale = cmd.time_scale.max(0.0);
        info!(
            "TimeControl: paused={}, scale={:.2}",
            time_scale.paused, time_scale.scale
        );
    }
}

fn process_node(
    node: &StoryNode,
    flags: &mut StoryFlags,
    variables: &StoryVariables,
    flow: &mut MessageWriter<StoryFlowEvent>,
    audio: &mut MessageWriter<AudioCommand>,
    scene: &mut MessageWriter<ChangeSceneEvent>,
    story: &mut MessageWriter<StoryEvent>,
    camera: &mut MessageWriter<CameraCommand>,
    time_ctrl: &mut MessageWriter<TimeControlCommand>,
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
            NodeAction::Advance
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
            } else if let Some(id) = if_false {
                NodeAction::Jump(*id)
            } else {
                NodeAction::Advance
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
        StoryNode::Conditional {
            variable,
            operator,
            value,
            if_true,
            if_false,
        } => {
            let result = variables.evaluate(variable, *operator, value);
            let target = if result { if_true } else { if_false };
            if let Some(id) = target {
                NodeAction::Jump(*id)
            } else {
                NodeAction::Advance
            }
        }
        StoryNode::Camera {
            preset_id,
            position,
            zoom,
            angle,
            duration,
            easing,
            ..
        } => {
            camera.write(CameraCommand {
                preset_id: preset_id.clone(),
                position: *position,
                zoom: *zoom,
                angle: *angle,
                duration: *duration,
                easing: easing.clone(),
            });
            NodeAction::Advance
        }
        StoryNode::TimeControl {
            pause_gameplay,
            time_scale,
            ..
        } => {
            time_ctrl.write(TimeControlCommand {
                pause_gameplay: *pause_gameplay,
                time_scale: *time_scale,
            });
            NodeAction::Advance
        }
        StoryNode::End => NodeAction::End,
        StoryNode::Start { .. } => NodeAction::Advance,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::story_graph::{GraphExecutor, StoryGraph, StoryInputEvent, StoryNode};
    use bevy::ecs::message::Messages;

    #[test]
    fn test_time_control_command_updates_resource() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<GameTimeScale>();
        app.add_message::<TimeControlCommand>();
        app.add_systems(Update, handle_time_control);

        // Pause gameplay
        app.world_mut()
            .resource_mut::<Messages<TimeControlCommand>>()
            .write(TimeControlCommand {
                pause_gameplay: true,
                time_scale: 0.5,
            });
        app.update();

        let ts = app.world().resource::<GameTimeScale>();
        assert!(ts.paused);
        assert!((ts.scale - 0.5).abs() < f32::EPSILON);
        assert_eq!(ts.effective(), 0.0); // paused → 0

        // Resume
        app.world_mut()
            .resource_mut::<Messages<TimeControlCommand>>()
            .write(TimeControlCommand {
                pause_gameplay: false,
                time_scale: 2.0,
            });
        app.update();

        let ts = app.world().resource::<GameTimeScale>();
        assert!(!ts.paused);
        assert!((ts.effective() - 2.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_game_time_scale_default() {
        let ts = GameTimeScale::default();
        assert!(!ts.paused);
        assert!((ts.scale - 1.0).abs() < f32::EPSILON);
        assert!((ts.effective() - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_advance_input_moves_past_dialogue_node() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(crate::story_graph::StoryGraphPlugin);
        app.add_message::<AudioCommand>();
        app.add_message::<ChangeSceneEvent>();

        let mut graph = StoryGraph::new();
        let start = graph.add(StoryNode::Start { next: Some(1) });
        graph.add(StoryNode::Dialogue {
            speaker: "Guide".into(),
            text: "Welcome".into(),
            portrait: None,
            next: Some(2),
        });
        graph.add(StoryNode::End);
        graph.set_start(start);

        app.world_mut().resource_mut::<GraphExecutor>().start(graph);

        app.update();
        assert_eq!(
            app.world().resource::<GraphExecutor>().status,
            ExecutionStatus::WaitingForInput
        );

        app.world_mut()
            .resource_mut::<Messages<StoryInputEvent>>()
            .write(StoryInputEvent::Advance);
        app.update();

        assert_eq!(
            app.world().resource::<GraphExecutor>().status,
            ExecutionStatus::Idle
        );

        let flow_events = app.world().resource::<Messages<StoryFlowEvent>>();
        let mut cursor = flow_events.get_cursor();
        assert!(cursor
            .read(flow_events)
            .any(|event| matches!(event, StoryFlowEvent::GraphComplete)));
    }
}
