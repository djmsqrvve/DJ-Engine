//! Data model types for dj_engine editor and runtime.
//!
//! This module contains serializable data structures used for:
//! - Project configuration and settings
//! - Scene composition (layers, entities, components)
//! - Story graphs and dialogue systems
//! - Game databases (items, NPCs, towers, enemies, etc.)
//! - Asset indexing and prefabs
//!
//! These types are designed to be stored as JSON and loaded by both
//! the editor and runtime. They are intentionally separate from Bevy
//! ECS components to maintain a clean data transfer layer.

pub mod assets;
pub mod components;
pub mod database;
pub mod loader;
pub mod project;
pub mod scene;
pub mod spawner;
pub mod story;

// Re-export commonly used types
pub use assets::{AssetIndex, Prefab};
pub use components::*;
pub use database::{Database, EnemyRow, ItemRow, LootTableRow, NpcRow, QuestRow, TowerRow};
pub use loader::{load_database, load_project, load_scene, load_story_graph, DataError};
pub use project::{EditorPreferences, Project, ProjectSettings};
pub use scene::{Entity, EntityType, Layer, Scene, SceneType};
pub use story::{StoryGraphData, StoryNodeData, StoryNodeType};

use bevy::prelude::*;

pub struct DataPlugin;

impl Plugin for DataPlugin {
    fn build(&self, app: &mut App) {
        components::register_types(app);
        story::register_types(app);
        app.register_type::<scene::SceneType>()
            .register_type::<scene::EntityType>()
            .register_type::<scene::TileSize>()
            .register_type::<scene::DefaultSpawn>()
            .register_type::<scene::SceneAudio>()
            .register_type::<scene::SceneScripts>()
            .register_type::<scene::Layer>()
            .register_type::<scene::PathfindingCell>()
            .register_type::<scene::PathfindingGrid>()
            .register_type::<scene::ScenePathfinding>()
            .register_type::<scene::Entity>()
            .register_type::<scene::Scene>();
    }
}
