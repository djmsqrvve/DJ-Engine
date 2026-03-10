//! Node-based story execution system.
//!
//! Replaces linear dialogue queues with a directed graph of nodes.
//! Supports branching logic, events, and complex narrative flow.

mod executor;
pub mod types;

pub use types::{
    ExecutionStatus, GraphChoice, GraphExecutor, NodeId, StoryEvent, StoryFlags, StoryFlowEvent,
    StoryGraph, StoryInputEvent, StoryNode,
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
            .add_message::<StoryEvent>()
            .add_message::<StoryFlowEvent>()
            .add_message::<StoryInputEvent>()
            .add_systems(Update, executor::execute_graph);
    }
}
