#![allow(
    clippy::too_many_arguments,
    clippy::type_complexity,
    clippy::upper_case_acronyms,
    clippy::module_inception,
    clippy::drop_non_drop
)]
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

pub mod ability;
pub mod animation;
pub mod assets;
pub mod audio;
pub mod character;
pub mod collision;
pub mod combat;
pub mod contracts;
pub mod core;
pub mod data;
pub mod debug_console;
pub mod diagnostics;
pub mod economy;
pub mod input;
pub mod interaction;
pub mod inventory;
pub mod loot;
pub mod midi;
pub mod project_mount;
pub mod quest;
pub mod rendering;
pub mod save;
pub mod scene;
pub mod scripting;
pub mod status;
pub mod story_graph;
pub mod types;
pub mod zone;

pub mod editor;
pub mod runtime_preview;

/// Prelude module for convenient imports
pub mod prelude {
    // Core engine plugin
    pub use crate::contracts::{
        AppContractExt, ContractEntry, ContractRegistry, ContractSystemSet, PluginContract,
    };
    pub use crate::core::DJEnginePlugin;

    // Individual plugins (for fine-grained control)
    pub use crate::ability::{AbilityPlugin, AbilityUsedEvent, UseAbilityRequest};
    pub use crate::animation::DJAnimationPlugin;
    pub use crate::assets::DJAssetPlugin;
    pub use crate::audio::{AudioCommand, AudioState, BgmSource, DJAudioPlugin, SfxSource};
    pub use crate::character::{
        CharacterPlugin, PlayerTitle, TitleEvent, WeaponProficiencies, WeaponSkillGainEvent,
    };
    pub use crate::collision::{
        CollisionPlugin, CollisionSet, MovementIntent, RuntimeCollider, RuntimeColliderShape,
        SpatialHash, TriggerContactEvent, TriggerContacts,
    };
    pub use crate::combat::{
        calculate_damage, CombatConfig, CombatEvent, CombatPlugin, DamageEvent,
    };
    pub use crate::diagnostics::DiagnosticsPlugin;
    pub use crate::economy::{
        ConsumableUsedEvent, EconomyPlugin, EquipItemRequest, EquipmentEvent, UnequipItemRequest,
        UseConsumableRequest, VendorBuyRequest, VendorEvent, VendorSellRequest,
    };
    pub use crate::input::{ActionState, DJInputPlugin, InputAction, InputConfig};
    pub use crate::interaction::{
        InteractionEvent, InteractionPlugin, InteractionSource, InteractionTarget,
    };
    pub use crate::inventory::{Inventory, InventoryEvent, InventoryPlugin, ItemStack};
    pub use crate::loot::{LootDropEvent, LootPlugin};
    pub use crate::midi::AutoLoadMidi;
    pub use crate::project_mount::MountedProject;
    pub use crate::quest::{
        ObjectiveProgress, QuestEvent, QuestJournal, QuestPlugin, QuestState, QuestStatus,
    };
    pub use crate::rendering::RenderingPlugin;
    pub use crate::runtime_preview::{
        bootstrap_mounted_project, parse_runtime_preview_cli_args, PreviewPlayer,
        PreviewPlayerController, PreviewState, RuntimePreviewCliOptions, RuntimePreviewPlugin,
        RuntimePreviewProfileOverride,
    };
    pub use crate::save::{
        has_save, has_save_scoped, load_game, load_game_scoped, save_game, save_game_scoped,
        LoadCommand, LoadedSave, SaveCommand, SaveData, SavePlugin, SaveScope,
    };
    pub use crate::scene::*;
    pub use crate::scripting::*;
    pub use crate::status::{
        apply_effect, is_on_cooldown, remove_effect, start_cooldown, AbilityReady,
        StatusEffectExpired, StatusPlugin,
    };
    pub use crate::story_graph::*;
    pub use crate::zone::{ActiveZone, PortalComponent, ZonePlugin, ZoneTransitionEvent};

    // Engine types
    pub use crate::types::*;

    // Data model types (for editor and runtime)
    pub use crate::data::spawner::{LoadedScene, SceneDataPlugin};
    pub use crate::data::{
        load_database, load_project, load_scene, load_story_graph, AchievementRow, AssetIndex,
        AuraRow, ClassDataRow, DataError, Database, EditorPreferences, EnemyRow, Entity,
        EntityType, GuildRow, ItemRow, Layer, LootTableRow, MountRow, NpcRow, Prefab,
        ProfessionRow, Project, ProjectSettings, PvpRow, QuestRow, RaidRow, Scene, SceneType,
        StoryGraphData, StoryNodeData, StoryNodeType, TalentRow, TowerRow, ZoneRow,
    };

    // Re-export commonly used rendering items
    pub use crate::data::{
        update_loaded_custom_document_envelope, update_loaded_custom_document_label,
        update_loaded_custom_document_nested_value, update_loaded_custom_document_top_level_scalar,
        update_loaded_custom_document_typed, AppCustomDocumentExt, CustomDataManifest,
        CustomDocument, CustomDocumentEntry, CustomDocumentRegistration, CustomDocumentRegistry,
        CustomDocumentScalarValue, CustomDocumentUpdateError, DJDataRegistryPlugin, DocumentRef,
        EditorDocumentRoute, LoadedCustomDocuments, PreviewProfilePayload, ValidationIssue,
        ValidationSeverity,
    };
    pub use crate::editor::{
        AppEditorExtensionExt, SelectedPreviewPreset, ToolbarActionFired, ToolbarActionQueue,
    };
    pub use crate::rendering::{
        CrtConfig, DisplayCamera, MainCamera, OffscreenTarget, GAME_HEIGHT, GAME_WIDTH,
    };
}

/// Returns the current engine version from Cargo.toml
pub fn engine_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
