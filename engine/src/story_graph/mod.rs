//! Node-based story execution system.
//!
//! Replaces linear dialogue queues with a directed graph of nodes.
//! Supports branching logic, events, and complex narrative flow.

mod executor;
pub mod types;

pub use types::{
    CameraCommand, ExecutionStatus, GraphChoice, GraphExecutor, NodeId, StoryEvent, StoryFlags,
    StoryFlowEvent, StoryGraph, StoryInputEvent, StoryNode, StoryVariables, TimeControlCommand,
};

use bevy::prelude::*;

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
            .init_resource::<StoryVariables>()
            .register_type::<StoryVariables>()
            .add_message::<StoryEvent>()
            .add_message::<StoryFlowEvent>()
            .add_message::<StoryInputEvent>()
            .add_message::<CameraCommand>()
            .add_message::<TimeControlCommand>()
            .add_systems(
                Update,
                (executor::execute_graph, executor::handle_time_control),
            );

        use crate::contracts::{AppContractExt, ContractEntry, PluginContract};
        app.register_contract(PluginContract {
            name: "StoryGraphPlugin".into(),
            description: "Node-based narrative branching and execution".into(),
            resources: vec![
                ContractEntry::of::<GraphExecutor>("Story graph executor state"),
                ContractEntry::of::<StoryFlags>("Boolean story state flags"),
                ContractEntry::of::<StoryVariables>("JSON story state variables"),
            ],
            components: vec![
                ContractEntry::of::<StoryGraph>("Story graph data on entity"),
                ContractEntry::of::<StoryNode>("Individual story node"),
                ContractEntry::of::<GraphChoice>("Choice option in story node"),
            ],
            events: vec![
                ContractEntry::of::<StoryEvent>("Generic story event with payload"),
                ContractEntry::of::<StoryFlowEvent>("Executor -> UI flow events"),
                ContractEntry::of::<StoryInputEvent>("UI -> Executor input events"),
                ContractEntry::of::<CameraCommand>("Camera control from story"),
                ContractEntry::of::<TimeControlCommand>("Time control from story"),
            ],
            system_sets: vec![],
        });
    }
}
