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
pub mod custom;
pub mod database;
pub mod grid;
pub mod loader;
pub mod project;
pub mod scene;
pub mod spawner;
pub mod story;

// Re-export commonly used types
pub use assets::{AssetIndex, Prefab};
pub use grid::Grid;
pub use components::*;
pub use custom::{
    default_custom_data_manifest_path, filter_document_refs_by_kind,
    load_custom_documents_from_project, resolve_default_preview_profile,
    resolve_preview_profile_by_id, save_loaded_custom_documents,
    update_loaded_custom_document_envelope, update_loaded_custom_document_label,
    update_loaded_custom_document_nested_value, update_loaded_custom_document_raw_json,
    update_loaded_custom_document_top_level_scalar, update_loaded_custom_document_typed,
    validate_loaded_custom_documents, AppCustomDocumentExt, CustomDataManifest, CustomDocument,
    CustomDocumentEntry, CustomDocumentRegistration, CustomDocumentRegistry,
    CustomDocumentScalarValue, CustomDocumentUpdateError, DJDataRegistryPlugin, DocumentId,
    DocumentKindId, DocumentLink, DocumentLinkTarget, DocumentRef, EditorDocumentRoute,
    LoadedCustomDocument, LoadedCustomDocuments, PreviewProfilePayload, ValidationIssue,
    ValidationSeverity,
};
pub use database::{
    AbilityRow, Database, EnemyRow, ItemRow, LootTableRow, NpcRow, QuestRow, TowerRow, ZoneRow,
};
pub use loader::{load_database, load_project, load_scene, load_story_graph, DataError};
pub use project::{EditorPreferences, Project, ProjectSettings};
pub use scene::{Entity, EntityType, Layer, Scene, SceneType};
pub use story::{StoryGraphData, StoryNodeData, StoryNodeType};

use bevy::prelude::*;

pub struct DataPlugin;

impl Plugin for DataPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<custom::DJDataRegistryPlugin>() {
            app.add_plugins(custom::DJDataRegistryPlugin);
        }
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

        use crate::contracts::{AppContractExt, ContractEntry, PluginContract};
        app.register_contract(PluginContract {
            name: "DataPlugin".into(),
            description: "Data models, custom documents, scene/story serialization".into(),
            resources: vec![
                ContractEntry::of::<custom::CustomDocumentRegistry>("Custom document kind registry"),
                ContractEntry::of::<custom::LoadedCustomDocuments>("All loaded custom documents"),
            ],
            components: vec![],
            events: vec![],
            system_sets: vec![],
        });
    }
}
