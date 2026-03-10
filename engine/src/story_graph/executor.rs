use super::types::{
    ExecutionStatus, GraphExecutor, NodeId, StoryEvent, StoryFlags, StoryFlowEvent,
    StoryInputEvent, StoryNode,
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
        StoryNode::End => NodeAction::End,
        StoryNode::Start { .. } => NodeAction::Advance,
    }
}
