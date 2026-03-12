//! DJ Engine - Core shared library for cursed narrative games
//!
//! This crate provides the foundational systems for building procedural
//! 2D character animation games with Lua scripting and palette-driven effects.
//!
//! # Example
//!
//! ```ignore
//! use dj_engine::prelude::*;
//!
//! App::new()
//!     .add_plugins(DefaultPlugins)
//!     .add_plugins(DJEnginePlugin::default())
//!     .run();
//! ```

pub mod animation;
pub mod assets;
pub mod audio;
pub mod collision;
pub mod core;
pub mod data;
pub mod diagnostics;
pub mod input;
pub mod midi;
pub mod project_mount;
pub mod rendering;
pub mod save;
pub mod scene;
pub mod scripting;
pub mod story_graph;
pub mod types;

pub mod editor;
pub mod runtime_preview;

/// Prelude module for convenient imports
pub mod prelude {
    // Core engine plugin
    pub use crate::core::DJEnginePlugin;

    // Individual plugins (for fine-grained control)
    pub use crate::animation::DJAnimationPlugin;
    pub use crate::assets::DJAssetPlugin;
    pub use crate::audio::{AudioCommand, AudioState, BgmSource, DJAudioPlugin, SfxSource};
    pub use crate::collision::{
        CollisionPlugin, CollisionSet, MovementIntent, RuntimeCollider, RuntimeColliderShape,
        TriggerContactEvent, TriggerContacts,
    };
    pub use crate::diagnostics::DiagnosticsPlugin;
    pub use crate::input::{ActionState, DJInputPlugin, InputAction, InputConfig};
    pub use crate::midi::AutoLoadMidi;
    pub use crate::project_mount::MountedProject;
    pub use crate::rendering::RenderingPlugin;
    pub use crate::runtime_preview::{
        bootstrap_mounted_project, parse_runtime_preview_cli_args, PreviewPlayer,
        PreviewPlayerController, PreviewState, RuntimePreviewCliOptions, RuntimePreviewPlugin,
    };
    pub use crate::save::{
        has_save, has_save_scoped, load_game, load_game_scoped, save_game, save_game_scoped,
        LoadCommand, LoadedSave, SaveCommand, SaveData, SavePlugin, SaveScope,
    };
    pub use crate::scene::*;
    pub use crate::scripting::*;
    pub use crate::story_graph::*;

    // Engine types
    pub use crate::types::*;

    // Data model types (for editor and runtime)
    pub use crate::data::spawner::{LoadedScene, SceneDataPlugin};
    pub use crate::data::{
        load_database, load_project, load_scene, load_story_graph, AssetIndex, DataError, Database,
        EditorPreferences, EnemyRow, Entity, EntityType, ItemRow, Layer, LootTableRow, NpcRow,
        Prefab, Project, ProjectSettings, QuestRow, Scene, SceneType, StoryGraphData,
        StoryNodeData, StoryNodeType, TowerRow,
    };

    // Re-export commonly used rendering items
    pub use crate::data::{
        update_loaded_custom_document_envelope, update_loaded_custom_document_typed,
        AppCustomDocumentExt, CustomDataManifest, CustomDocument, CustomDocumentEntry,
        CustomDocumentRegistration, CustomDocumentRegistry, DJDataRegistryPlugin, DocumentRef,
        EditorDocumentRoute, LoadedCustomDocuments, PreviewProfilePayload, ValidationIssue,
        ValidationSeverity,
    };
    pub use crate::editor::AppEditorExtensionExt;
    pub use crate::rendering::{
        CrtConfig, DisplayCamera, MainCamera, OffscreenTarget, GAME_HEIGHT, GAME_WIDTH,
    };
}

/// Returns the current engine version from Cargo.toml
pub fn engine_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
