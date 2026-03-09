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
        app.register_type::<components::Vec3Data>()
            .register_type::<components::ColorData>()
            .register_type::<components::TransformComponent>()
            .register_type::<components::SpriteComponent>()
            .register_type::<components::CollisionComponent>()
            .register_type::<components::InteractivityComponent>()
            .register_type::<components::NpcComponent>()
            .register_type::<components::EnemyComponent>()
            .register_type::<components::CombatStatsComponent>()
            .register_type::<components::TowerComponent>()
            .register_type::<components::SpawnerComponent>()
            .register_type::<components::SpawnerWave>()
            .register_type::<components::AudioSourceComponent>()
            .register_type::<components::CameraAnchorComponent>()
            .register_type::<components::AnimationData>()
            .register_type::<components::BodyType>()
            .register_type::<components::CollisionShape>()
            .register_type::<components::TriggerType>()
            .register_type::<components::TargetingMode>()
            .register_type::<story::StoryGraphData>()
            .register_type::<story::StoryNodeData>()
            .register_type::<story::StoryNodeVariant>()
            .register_type::<story::StoryGraphType>()
            .register_type::<story::StoryNodeType>()
            .register_type::<story::ConditionOperator>()
            .register_type::<story::EndType>()
            .register_type::<story::EffectType>()
            .register_type::<story::RequiredEntity>()
            .register_type::<story::RequiredItem>()
            .register_type::<story::StoryCondition>()
            .register_type::<story::StoryEffect>()
            .register_type::<story::DialogueNodeData>()
            .register_type::<story::ChoiceNodeData>()
            .register_type::<story::ChoiceOption>()
            .register_type::<story::ActionNodeData>()
            .register_type::<story::ConditionalNodeData>()
            .register_type::<story::CameraNodeData>()
            .register_type::<story::TimeControlNodeData>()
            .register_type::<story::EndNodeData>()
            .register_type::<scene::SceneType>()
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
