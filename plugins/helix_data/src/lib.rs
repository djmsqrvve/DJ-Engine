#![allow(clippy::too_many_arguments, clippy::type_complexity)]
use bevy::prelude::*;
use dj_engine::data::{
    AppCustomDocumentExt, CustomDocument, CustomDocumentRegistration, DJDataRegistryPlugin,
    DocumentLink, EditorDocumentRoute, LoadedCustomDocuments, ValidationIssue, ValidationSeverity,
};
use dj_engine::editor::{
    AppEditorExtensionExt, RegisteredPreviewPreset, RegisteredToolbarAction, ToolbarActionQueue,
};
use dj_engine::project_mount::MountedProject;
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::PathBuf;

pub mod balance;
pub mod bridge;
pub mod dashboard;
pub mod exporter;
pub mod importer;
pub mod registries;
pub mod toml_loader;

pub use balance::{BalanceOverlay, BalanceOverlays};
pub use bridge::populate_database_from_helix;
pub use exporter::{export_to_helix3d, ExportError, ExportSummary};
pub use importer::{
    import_helix_project, parse_helix_import_cli_args, HelixImportCliOptions, HelixImportError,
    HelixImportSummary,
};
pub use registries::{load_helix_registries, load_helix_registries_lenient, HelixRegistries};
pub use toml_loader::HelixLoadError;

pub const HELIX_ABILITY_KIND: &str = "helix_abilities";
pub const HELIX_ACHIEVEMENT_KIND: &str = "helix_achievements";
pub const HELIX_AURA_KIND: &str = "helix_auras";
pub const HELIX_CLASS_DATA_KIND: &str = "helix_class_data";
pub const HELIX_CONSUMABLE_KIND: &str = "helix_consumables";
pub const HELIX_CURRENCY_KIND: &str = "helix_currencies";
pub const HELIX_EQUIPMENT_KIND: &str = "helix_equipment";
pub const HELIX_GUILD_KIND: &str = "helix_guilds";
pub const HELIX_INVENTORY_KIND: &str = "helix_inventory";
pub const HELIX_ITEM_KIND: &str = "helix_items";
pub const HELIX_MOB_KIND: &str = "helix_mobs";
pub const HELIX_MOUNT_KIND: &str = "helix_mounts";
pub const HELIX_NPC_KIND: &str = "helix_npcs";
pub const HELIX_PROFESSION_KIND: &str = "helix_professions";
pub const HELIX_PVP_KIND: &str = "helix_pvp";
pub const HELIX_QUEST_KIND: &str = "helix_quests";
pub const HELIX_RAID_KIND: &str = "helix_raids";
pub const HELIX_TALENT_KIND: &str = "helix_talents";
pub const HELIX_TITLE_KIND: &str = "helix_titles";
pub const HELIX_TRADE_GOOD_KIND: &str = "helix_trade_goods";
pub const HELIX_WEAPON_SKILL_KIND: &str = "helix_weapon_skills";
pub const HELIX_ZONE_KIND: &str = "helix_zones";
pub const HELIX_IMPORT_PREVIEW_ID: &str = "helix_import_preview";

/// All 22 helix document kind constants for programmatic iteration.
pub const ALL_HELIX_KINDS: &[&str] = &[
    HELIX_ABILITY_KIND,
    HELIX_ACHIEVEMENT_KIND,
    HELIX_AURA_KIND,
    HELIX_CLASS_DATA_KIND,
    HELIX_CONSUMABLE_KIND,
    HELIX_CURRENCY_KIND,
    HELIX_EQUIPMENT_KIND,
    HELIX_GUILD_KIND,
    HELIX_INVENTORY_KIND,
    HELIX_ITEM_KIND,
    HELIX_MOB_KIND,
    HELIX_MOUNT_KIND,
    HELIX_NPC_KIND,
    HELIX_PROFESSION_KIND,
    HELIX_PVP_KIND,
    HELIX_QUEST_KIND,
    HELIX_RAID_KIND,
    HELIX_TALENT_KIND,
    HELIX_TITLE_KIND,
    HELIX_TRADE_GOOD_KIND,
    HELIX_WEAPON_SKILL_KIND,
    HELIX_ZONE_KIND,
];

/// Configuration for the Helix import pipeline.
///
/// - `helix_dist_path`: Legacy JSON bucket path for the old import pipeline.
/// - `helix3d_path`: Path to `dist/helix3d/` for the typed TOML pipeline.
/// - `balance_dir`: Optional directory containing per-kind TOML balance overlay files.
///
/// When `helix3d_path` is set, the plugin loads typed registries at startup.
#[derive(Resource, Default, Debug, Clone)]
pub struct HelixImportConfig {
    pub helix_dist_path: Option<PathBuf>,
    pub helix3d_path: Option<PathBuf>,
    pub balance_dir: Option<PathBuf>,
}

/// Wrapper resource so `Database` (a pure-data struct) can be used as a Bevy Resource.
#[derive(Resource, Default, Debug, Clone)]
pub struct HelixDatabase(pub dj_engine::data::Database);

const HELIX_ABILITY_SCHEMA_JSON: &str = r#"{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "DJ Engine Helix Ability Document",
  "type": "object",
  "required": ["kind", "id", "payload"],
  "properties": {
    "kind": { "const": "helix_abilities" },
    "id": { "type": "string", "minLength": 1 },
    "schema_version": { "type": "integer", "minimum": 1 },
    "label": { "type": ["string", "null"] },
    "tags": {
      "type": "array",
      "items": { "type": "string" }
    },
    "references": {
      "type": "array",
      "items": { "type": "object" }
    },
    "payload": {
      "type": "object",
      "required": ["id"],
      "properties": {
        "id": { "type": "string", "minLength": 1 }
      },
      "additionalProperties": true
    }
  },
  "additionalProperties": false
}"#;

const HELIX_ITEM_SCHEMA_JSON: &str = r#"{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "DJ Engine Helix Item Document",
  "type": "object",
  "required": ["kind", "id", "payload"],
  "properties": {
    "kind": { "const": "helix_items" },
    "id": { "type": "string", "minLength": 1 },
    "schema_version": { "type": "integer", "minimum": 1 },
    "label": { "type": ["string", "null"] },
    "tags": {
      "type": "array",
      "items": { "type": "string" }
    },
    "references": {
      "type": "array",
      "items": { "type": "object" }
    },
    "payload": {
      "type": "object",
      "required": ["id"],
      "properties": {
        "id": { "type": "string", "minLength": 1 }
      },
      "additionalProperties": true
    }
  },
  "additionalProperties": false
}"#;

const HELIX_MOB_SCHEMA_JSON: &str = r#"{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "DJ Engine Helix Mob Document",
  "type": "object",
  "required": ["kind", "id", "payload"],
  "properties": {
    "kind": { "const": "helix_mobs" },
    "id": { "type": "string", "minLength": 1 },
    "schema_version": { "type": "integer", "minimum": 1 },
    "label": { "type": ["string", "null"] },
    "tags": {
      "type": "array",
      "items": { "type": "string" }
    },
    "references": {
      "type": "array",
      "items": { "type": "object" }
    },
    "payload": {
      "type": "object",
      "required": ["id"],
      "properties": {
        "id": { "type": "string", "minLength": 1 }
      },
      "additionalProperties": true
    }
  },
  "additionalProperties": false
}"#;

/// Generic schema for typed helix document kinds that are loaded via TOML
/// and only need minimal JSON envelope validation in the CustomDocument system.
const HELIX_GENERIC_SCHEMA_JSON: &str = r#"{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "DJ Engine Helix Document",
  "type": "object",
  "required": ["kind", "id", "payload"],
  "properties": {
    "kind": { "type": "string" },
    "id": { "type": "string", "minLength": 1 },
    "schema_version": { "type": "integer", "minimum": 1 },
    "payload": { "type": "object", "additionalProperties": true }
  },
  "additionalProperties": true
}"#;

#[derive(Resource, Default, Debug, Clone, PartialEq)]
pub struct HelixDocumentIndex {
    kinds: BTreeMap<String, BTreeMap<String, CustomDocument<Value>>>,
}

impl HelixDocumentIndex {
    /// Generic lookup: any kind + id.
    pub fn get(&self, kind: &str, id: &str) -> Option<&CustomDocument<Value>> {
        self.kinds.get(kind).and_then(|map| map.get(id))
    }

    /// Convenience: lookup an ability by id.
    pub fn ability(&self, id: &str) -> Option<&CustomDocument<Value>> {
        self.get(HELIX_ABILITY_KIND, id)
    }

    /// Convenience: lookup an item by id.
    pub fn item(&self, id: &str) -> Option<&CustomDocument<Value>> {
        self.get(HELIX_ITEM_KIND, id)
    }

    /// Convenience: lookup a mob by id.
    pub fn mob(&self, id: &str) -> Option<&CustomDocument<Value>> {
        self.get(HELIX_MOB_KIND, id)
    }

    /// Number of indexed entities across all kinds.
    pub fn total(&self) -> usize {
        self.kinds.values().map(|m| m.len()).sum()
    }

    fn rebuild_from_loaded_documents(&mut self, loaded_documents: &LoadedCustomDocuments) {
        self.kinds.clear();

        for document in &loaded_documents.documents {
            let kind = document.entry.kind.as_str();
            if !ALL_HELIX_KINDS.contains(&kind) {
                continue;
            }

            let Some(parsed) = document.document.clone() else {
                continue;
            };

            self.kinds
                .entry(kind.to_string())
                .or_default()
                .insert(document.entry.id.clone(), parsed);
        }
    }
}

pub struct HelixDataPlugin;

impl Plugin for HelixDataPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<DJDataRegistryPlugin>() {
            app.add_plugins(DJDataRegistryPlugin);
        }

        app.init_resource::<HelixDocumentIndex>()
            .init_resource::<HelixRegistries>()
            .register_custom_document(
                CustomDocumentRegistration::<Value>::new(
                    HELIX_ABILITY_KIND,
                    1,
                    EditorDocumentRoute::Table,
                    HELIX_ABILITY_SCHEMA_JSON,
                )
                .with_validator(validate_helix_ability_document),
            )
            .register_custom_document(
                CustomDocumentRegistration::<Value>::new(
                    HELIX_ITEM_KIND,
                    1,
                    EditorDocumentRoute::Table,
                    HELIX_ITEM_SCHEMA_JSON,
                )
                .with_validator(validate_helix_item_document),
            )
            .register_custom_document(
                CustomDocumentRegistration::<Value>::new(
                    HELIX_MOB_KIND,
                    1,
                    EditorDocumentRoute::Table,
                    HELIX_MOB_SCHEMA_JSON,
                )
                .with_validator(validate_helix_mob_document),
            )
            // Register remaining 19 helix document kinds for the editor
            .register_custom_document(
                CustomDocumentRegistration::<Value>::new(
                    HELIX_ACHIEVEMENT_KIND,
                    1,
                    EditorDocumentRoute::Table,
                    HELIX_GENERIC_SCHEMA_JSON,
                )
                .with_validator(validate_helix_achievement_document),
            )
            .register_custom_document(
                CustomDocumentRegistration::<Value>::new(
                    HELIX_AURA_KIND,
                    1,
                    EditorDocumentRoute::Table,
                    HELIX_GENERIC_SCHEMA_JSON,
                )
                .with_validator(validate_helix_aura_document),
            )
            .register_custom_document(
                CustomDocumentRegistration::<Value>::new(
                    HELIX_CLASS_DATA_KIND,
                    1,
                    EditorDocumentRoute::Graph,
                    HELIX_GENERIC_SCHEMA_JSON,
                )
                .with_validator(validate_helix_class_data_document),
            )
            .register_custom_document(
                CustomDocumentRegistration::<Value>::new(
                    HELIX_CONSUMABLE_KIND,
                    1,
                    EditorDocumentRoute::Table,
                    HELIX_GENERIC_SCHEMA_JSON,
                )
                .with_validator(validate_helix_consumable_document),
            )
            .register_custom_document(
                CustomDocumentRegistration::<Value>::new(
                    HELIX_CURRENCY_KIND,
                    1,
                    EditorDocumentRoute::Table,
                    HELIX_GENERIC_SCHEMA_JSON,
                )
                .with_validator(validate_helix_currency_document),
            )
            .register_custom_document(
                CustomDocumentRegistration::<Value>::new(
                    HELIX_EQUIPMENT_KIND,
                    1,
                    EditorDocumentRoute::Table,
                    HELIX_GENERIC_SCHEMA_JSON,
                )
                .with_validator(validate_helix_equipment_document),
            )
            .register_custom_document(
                CustomDocumentRegistration::<Value>::new(
                    HELIX_GUILD_KIND,
                    1,
                    EditorDocumentRoute::Table,
                    HELIX_GENERIC_SCHEMA_JSON,
                )
                .with_validator(validate_helix_guild_document),
            )
            .register_custom_document(
                CustomDocumentRegistration::<Value>::new(
                    HELIX_INVENTORY_KIND,
                    1,
                    EditorDocumentRoute::Table,
                    HELIX_GENERIC_SCHEMA_JSON,
                )
                .with_validator(validate_helix_inventory_document),
            )
            .register_custom_document(
                CustomDocumentRegistration::<Value>::new(
                    HELIX_MOUNT_KIND,
                    1,
                    EditorDocumentRoute::Table,
                    HELIX_GENERIC_SCHEMA_JSON,
                )
                .with_validator(validate_helix_mount_document),
            )
            .register_custom_document(
                CustomDocumentRegistration::<Value>::new(
                    HELIX_NPC_KIND,
                    1,
                    EditorDocumentRoute::Table,
                    HELIX_GENERIC_SCHEMA_JSON,
                )
                .with_validator(validate_helix_npc_document),
            )
            .register_custom_document(
                CustomDocumentRegistration::<Value>::new(
                    HELIX_PROFESSION_KIND,
                    1,
                    EditorDocumentRoute::Table,
                    HELIX_GENERIC_SCHEMA_JSON,
                )
                .with_validator(validate_helix_profession_document),
            )
            .register_custom_document(
                CustomDocumentRegistration::<Value>::new(
                    HELIX_PVP_KIND,
                    1,
                    EditorDocumentRoute::Table,
                    HELIX_GENERIC_SCHEMA_JSON,
                )
                .with_validator(validate_helix_pvp_document),
            )
            .register_custom_document(
                CustomDocumentRegistration::<Value>::new(
                    HELIX_QUEST_KIND,
                    1,
                    EditorDocumentRoute::Graph,
                    HELIX_GENERIC_SCHEMA_JSON,
                )
                .with_validator(validate_helix_quest_document),
            )
            .register_custom_document(
                CustomDocumentRegistration::<Value>::new(
                    HELIX_RAID_KIND,
                    1,
                    EditorDocumentRoute::Table,
                    HELIX_GENERIC_SCHEMA_JSON,
                )
                .with_validator(validate_helix_raid_document),
            )
            .register_custom_document(
                CustomDocumentRegistration::<Value>::new(
                    HELIX_TALENT_KIND,
                    1,
                    EditorDocumentRoute::Graph,
                    HELIX_GENERIC_SCHEMA_JSON,
                )
                .with_validator(validate_helix_talent_document),
            )
            .register_custom_document(
                CustomDocumentRegistration::<Value>::new(
                    HELIX_TITLE_KIND,
                    1,
                    EditorDocumentRoute::Table,
                    HELIX_GENERIC_SCHEMA_JSON,
                )
                .with_validator(validate_helix_title_document),
            )
            .register_custom_document(
                CustomDocumentRegistration::<Value>::new(
                    HELIX_TRADE_GOOD_KIND,
                    1,
                    EditorDocumentRoute::Table,
                    HELIX_GENERIC_SCHEMA_JSON,
                )
                .with_validator(validate_helix_trade_good_document),
            )
            .register_custom_document(
                CustomDocumentRegistration::<Value>::new(
                    HELIX_WEAPON_SKILL_KIND,
                    1,
                    EditorDocumentRoute::Table,
                    HELIX_GENERIC_SCHEMA_JSON,
                )
                .with_validator(validate_helix_weapon_skill_document),
            )
            .register_custom_document(
                CustomDocumentRegistration::<Value>::new(
                    HELIX_ZONE_KIND,
                    1,
                    EditorDocumentRoute::Table,
                    HELIX_GENERIC_SCHEMA_JSON,
                )
                .with_validator(validate_helix_zone_document),
            )
            .init_resource::<HelixImportConfig>()
            .init_resource::<HelixDashboardRan>()
            .init_resource::<HelixDatabase>()
            .init_resource::<balance::BalanceOverlays>()
            .add_systems(
                Startup,
                (
                    load_helix_registries_startup_system,
                    load_balance_overlays_startup_system,
                )
                    .chain(),
            )
            .add_systems(
                Startup,
                populate_database_startup_system
                    .after(load_helix_registries_startup_system)
                    .after(load_balance_overlays_startup_system),
            )
            .add_systems(
                Update,
                (
                    sync_registries_to_custom_documents_system,
                    refresh_helix_document_index_system,
                    handle_helix_toolbar_actions_system,
                    run_dashboard_validation_system,
                )
                    .chain(),
            )
            .register_toolbar_action(RegisteredToolbarAction {
                action_id: "helix_reimport".into(),
                title: "Re-import Helix Data".into(),
                kind_filter: None,
            })
            .register_preview_preset(RegisteredPreviewPreset {
                preset_id: "helix_default".into(),
                title: "Helix Default".into(),
                profile_id: Some(HELIX_IMPORT_PREVIEW_ID.into()),
            });
    }
}

/// Tracks whether dashboard validation has run (runs once after registries load).
#[derive(Resource, Default)]
struct HelixDashboardRan(bool);

/// Run dashboard validation once after registries are loaded, feeding issues
/// into LoadedCustomDocuments so they appear in the editor's validation view.
fn run_dashboard_validation_system(
    registries: Res<HelixRegistries>,
    config: Res<HelixImportConfig>,
    mut dashboard_ran: ResMut<HelixDashboardRan>,
    mut loaded_documents: ResMut<LoadedCustomDocuments>,
) {
    if dashboard_ran.0 || registries.total_entities() == 0 {
        return;
    }
    dashboard_ran.0 = true;

    let mut issues = Vec::new();
    dashboard::validate_helix_registries(&registries, config.helix3d_path.as_deref(), &mut issues);

    if !issues.is_empty() {
        info!(
            "Helix dashboard: {} validation issue(s) found",
            issues.len()
        );
        loaded_documents.issues.extend(issues);
    }
}

fn load_helix_registries_startup_system(
    config: Res<HelixImportConfig>,
    mut registries: ResMut<HelixRegistries>,
) {
    let Some(helix3d_path) = config.helix3d_path.as_ref() else {
        return;
    };

    info!("Loading typed Helix registries from {:?}...", helix3d_path);
    match registries::load_helix_registries_lenient(helix3d_path) {
        Ok(loaded) => {
            let total = loaded.total_entities();
            *registries = loaded;
            info!("Loaded {} typed Helix entities across 22 registries", total);
        }
        Err(error) => {
            error!("Failed to load Helix registries: {error}");
        }
    }
}

fn load_balance_overlays_startup_system(
    config: Res<HelixImportConfig>,
    mut overlays: ResMut<balance::BalanceOverlays>,
) {
    let Some(balance_dir) = config.balance_dir.as_ref() else {
        return;
    };

    info!("Loading balance overlays from {:?}...", balance_dir);
    match balance::load_balance_overlays(balance_dir) {
        Ok(loaded) => {
            let layer_count: usize = loaded.layers.values().map(|m| m.len()).sum();
            *overlays = loaded;
            info!("Loaded {} balance overlay entries", layer_count);
        }
        Err(error) => {
            error!("Failed to load balance overlays: {error}");
        }
    }
}

fn populate_database_startup_system(
    registries: Res<HelixRegistries>,
    overlays: Res<balance::BalanceOverlays>,
    mut database: ResMut<HelixDatabase>,
) {
    if registries.total_entities() == 0 {
        return;
    }

    info!("Populating engine database from Helix registries...");
    let db = bridge::populate_database_from_helix(&registries, Some(&overlays));
    let total = db.items.len() + db.enemies.len() + db.npcs.len() + db.quests.len();
    database.0 = db;
    info!(
        "Engine database populated: {} items, {} enemies, {} npcs, {} quests ({} total)",
        database.0.items.len(),
        database.0.enemies.len(),
        database.0.npcs.len(),
        database.0.quests.len(),
        total,
    );
}

fn sync_registries_to_custom_documents_system(
    registries: Res<HelixRegistries>,
    registry: Res<dj_engine::data::CustomDocumentRegistry>,
    mut loaded_documents: ResMut<LoadedCustomDocuments>,
) {
    if registries.total_entities() == 0 {
        return;
    }

    // Only sync once — if we already have TOML-sourced documents, skip.
    let already_synced = loaded_documents
        .documents
        .iter()
        .any(|d| d.entry.tags.iter().any(|t| t == "source:toml_registry"));
    if already_synced {
        return;
    }

    let mut count = 0usize;
    registries.for_each_as_json(|kind, id, payload| {
        let resolved_route = registry
            .get(kind)
            .map(|reg| reg.editor_route)
            .unwrap_or_default();

        let envelope = dj_engine::data::CustomDocument {
            kind: kind.to_string(),
            id: id.to_string(),
            schema_version: 1,
            label: payload
                .get("name")
                .and_then(|n| n.get("en"))
                .and_then(serde_json::Value::as_str)
                .map(str::to_string),
            tags: vec!["source:toml_registry".to_string()],
            references: Vec::new(),
            payload: payload.clone(),
        };

        let raw_json = serde_json::to_string_pretty(&envelope).unwrap_or_default();

        loaded_documents
            .documents
            .push(dj_engine::data::LoadedCustomDocument {
                entry: dj_engine::data::CustomDocumentEntry {
                    kind: kind.to_string(),
                    id: id.to_string(),
                    path: format!("{kind}/{id}.toml"),
                    schema_version: 1,
                    editor_route: resolved_route,
                    tags: vec!["source:toml_registry".to_string()],
                },
                raw_json,
                document: Some(envelope),
                parse_error: None,
                resolved_route,
            });

        count += 1;
    });

    if count > 0 {
        info!(
            "Synced {} typed TOML entities into LoadedCustomDocuments",
            count
        );
    }
}

fn refresh_helix_document_index_system(
    loaded_documents: Res<LoadedCustomDocuments>,
    mut index: ResMut<HelixDocumentIndex>,
) {
    if !loaded_documents.is_changed() && !index.is_added() {
        return;
    }

    index.rebuild_from_loaded_documents(&loaded_documents);
}

fn handle_helix_toolbar_actions_system(
    action_queue: Option<ResMut<ToolbarActionQueue>>,
    config: Option<Res<HelixImportConfig>>,
    mounted_project: Option<Res<MountedProject>>,
    registry: Option<Res<dj_engine::data::CustomDocumentRegistry>>,
    mut loaded_documents: ResMut<LoadedCustomDocuments>,
    mut index: ResMut<HelixDocumentIndex>,
) {
    let Some(mut action_queue) = action_queue else {
        return;
    };
    let Some(config) = config else {
        return;
    };
    let Some(mounted_project) = mounted_project else {
        return;
    };
    let Some(registry) = registry else {
        return;
    };

    let had_reimport = action_queue
        .pending
        .iter()
        .any(|a| a.action_id == "helix_reimport");
    action_queue
        .pending
        .retain(|a| a.action_id != "helix_reimport");

    if !had_reimport {
        return;
    }

    let Some(helix_dist) = config.helix_dist_path.as_ref() else {
        warn!(
            "Helix re-import requested but no helix_dist_path configured. \
             Set HelixImportConfig.helix_dist_path or pass --helix-dist to the editor."
        );
        return;
    };

    let Some(manifest_path) = mounted_project.manifest_path.as_ref() else {
        warn!("Helix re-import requested but no project is mounted.");
        return;
    };

    info!("Re-importing Helix data from {:?}...", helix_dist);
    match importer::import_helix_project(helix_dist, manifest_path) {
        Ok(summary) => {
            info!(
                "Helix re-import complete: {} abilities, {} items, {} mobs ({} skipped)",
                summary.abilities, summary.items, summary.mobs, summary.skipped_files
            );
            // Reload custom documents so the editor picks up the new data.
            let fresh =
                dj_engine::data::load_custom_documents_from_project(&mounted_project, &registry);
            *loaded_documents = fresh;
            index.rebuild_from_loaded_documents(&loaded_documents);
        }
        Err(error) => {
            error!("Helix re-import failed: {error}");
        }
    }
}

fn validate_helix_ability_document(
    document: &CustomDocument<Value>,
    _loaded: &LoadedCustomDocuments,
    _project: &dj_engine::data::Project,
    issues: &mut Vec<ValidationIssue>,
) {
    validate_helix_document_payload(HELIX_ABILITY_KIND, document, issues);
}

fn validate_helix_item_document(
    document: &CustomDocument<Value>,
    _loaded: &LoadedCustomDocuments,
    _project: &dj_engine::data::Project,
    issues: &mut Vec<ValidationIssue>,
) {
    validate_helix_document_payload(HELIX_ITEM_KIND, document, issues);
}

fn validate_helix_mob_document(
    document: &CustomDocument<Value>,
    _loaded: &LoadedCustomDocuments,
    _project: &dj_engine::data::Project,
    issues: &mut Vec<ValidationIssue>,
) {
    validate_helix_document_payload(HELIX_MOB_KIND, document, issues);
}

fn validate_helix_document_payload(
    kind: &str,
    document: &CustomDocument<Value>,
    issues: &mut Vec<ValidationIssue>,
) {
    let Some(payload) = document.payload.as_object() else {
        issues.push(ValidationIssue {
            severity: ValidationSeverity::Error,
            code: "helix_payload_not_object".into(),
            source_kind: Some(kind.to_string()),
            source_id: Some(document.id.clone()),
            field_path: Some("payload".into()),
            message: "Helix payload must remain a JSON object.".into(),
            related_refs: Vec::<String>::new(),
        });
        return;
    };

    let Some(payload_id) = payload.get("id").and_then(Value::as_str) else {
        issues.push(ValidationIssue {
            severity: ValidationSeverity::Error,
            code: "helix_payload_missing_id".into(),
            source_kind: Some(kind.to_string()),
            source_id: Some(document.id.clone()),
            field_path: Some("payload.id".into()),
            message: "Helix payload must include a string id field.".into(),
            related_refs: Vec::<String>::new(),
        });
        return;
    };

    if payload_id != document.id {
        issues.push(ValidationIssue {
            severity: ValidationSeverity::Error,
            code: "helix_payload_id_mismatch".into(),
            source_kind: Some(kind.to_string()),
            source_id: Some(document.id.clone()),
            field_path: Some("payload.id".into()),
            message: format!(
                "Helix payload id '{}' does not match envelope id '{}'.",
                payload_id, document.id
            ),
            related_refs: Vec::<String>::new(),
        });
    }
}

fn validate_helix_zone_document(
    document: &CustomDocument<Value>,
    _loaded: &LoadedCustomDocuments,
    _project: &dj_engine::data::Project,
    issues: &mut Vec<ValidationIssue>,
) {
    validate_helix_localized_name(HELIX_ZONE_KIND, document, issues);
}

fn validate_helix_quest_document(
    document: &CustomDocument<Value>,
    loaded: &LoadedCustomDocuments,
    _project: &dj_engine::data::Project,
    issues: &mut Vec<ValidationIssue>,
) {
    validate_helix_localized_name(HELIX_QUEST_KIND, document, issues);

    if let Some(prereqs) = document
        .payload
        .get("prerequisite_quests")
        .and_then(Value::as_array)
    {
        for (i, prereq) in prereqs.iter().enumerate() {
            if let Some(quest_id) = prereq.as_str() {
                if loaded.get(HELIX_QUEST_KIND, quest_id).is_none() {
                    issues.push(ValidationIssue {
                        severity: ValidationSeverity::Warning,
                        code: "helix_broken_quest_prereq".into(),
                        source_kind: Some(HELIX_QUEST_KIND.to_string()),
                        source_id: Some(document.id.clone()),
                        field_path: Some(format!("payload.prerequisite_quests[{i}]")),
                        message: format!(
                            "Prerequisite quest '{}' not found in loaded documents.",
                            quest_id
                        ),
                        related_refs: vec![format!("{HELIX_QUEST_KIND}:{quest_id}")],
                    });
                }
            }
        }
    }
}

fn validate_helix_npc_document(
    document: &CustomDocument<Value>,
    loaded: &LoadedCustomDocuments,
    _project: &dj_engine::data::Project,
    issues: &mut Vec<ValidationIssue>,
) {
    validate_helix_localized_name(HELIX_NPC_KIND, document, issues);

    if let Some(quests) = document.payload.get("quests").and_then(Value::as_array) {
        for (i, quest) in quests.iter().enumerate() {
            if let Some(quest_id) = quest.as_str() {
                if loaded.get(HELIX_QUEST_KIND, quest_id).is_none() {
                    issues.push(ValidationIssue {
                        severity: ValidationSeverity::Warning,
                        code: "helix_broken_npc_quest_ref".into(),
                        source_kind: Some(HELIX_NPC_KIND.to_string()),
                        source_id: Some(document.id.clone()),
                        field_path: Some(format!("payload.quests[{i}]")),
                        message: format!(
                            "Referenced quest '{}' not found in loaded documents.",
                            quest_id
                        ),
                        related_refs: vec![format!("{HELIX_QUEST_KIND}:{quest_id}")],
                    });
                }
            }
        }
    }
}

fn validate_helix_achievement_document(
    document: &CustomDocument<Value>,
    _loaded: &LoadedCustomDocuments,
    _project: &dj_engine::data::Project,
    issues: &mut Vec<ValidationIssue>,
) {
    validate_helix_localized_name(HELIX_ACHIEVEMENT_KIND, document, issues);
}

fn validate_helix_equipment_document(
    document: &CustomDocument<Value>,
    _loaded: &LoadedCustomDocuments,
    _project: &dj_engine::data::Project,
    issues: &mut Vec<ValidationIssue>,
) {
    if let Some(slot) = document.payload.get("equip_slot").and_then(Value::as_str) {
        const VALID_SLOTS: &[&str] = &[
            "head",
            "neck",
            "shoulder",
            "chest",
            "waist",
            "legs",
            "feet",
            "wrist",
            "hands",
            "finger",
            "trinket",
            "back",
            "main_hand",
            "off_hand",
            "ranged",
            "two_hand",
            "one_hand",
            "shirt",
            "tabard",
        ];
        if !VALID_SLOTS.contains(&slot) {
            issues.push(ValidationIssue {
                severity: ValidationSeverity::Warning,
                code: "helix_unknown_equip_slot".into(),
                source_kind: Some(HELIX_EQUIPMENT_KIND.to_string()),
                source_id: Some(document.id.clone()),
                field_path: Some("payload.equip_slot".into()),
                message: format!("Unrecognized equip_slot '{slot}'."),
                related_refs: Vec::new(),
            });
        }
    }
}

fn validate_helix_aura_document(
    document: &CustomDocument<Value>,
    _loaded: &LoadedCustomDocuments,
    _project: &dj_engine::data::Project,
    issues: &mut Vec<ValidationIssue>,
) {
    validate_helix_document_payload(HELIX_AURA_KIND, document, issues);
    validate_helix_localized_name(HELIX_AURA_KIND, document, issues);
}

fn validate_helix_class_data_document(
    document: &CustomDocument<Value>,
    _loaded: &LoadedCustomDocuments,
    _project: &dj_engine::data::Project,
    issues: &mut Vec<ValidationIssue>,
) {
    validate_helix_document_payload(HELIX_CLASS_DATA_KIND, document, issues);
    validate_helix_localized_name(HELIX_CLASS_DATA_KIND, document, issues);
}

fn validate_helix_consumable_document(
    document: &CustomDocument<Value>,
    _loaded: &LoadedCustomDocuments,
    _project: &dj_engine::data::Project,
    issues: &mut Vec<ValidationIssue>,
) {
    validate_helix_document_payload(HELIX_CONSUMABLE_KIND, document, issues);
    validate_helix_localized_name(HELIX_CONSUMABLE_KIND, document, issues);

    // Validate consumable_type if present.
    if let Some(ct) = document
        .payload
        .get("consumable_type")
        .and_then(Value::as_str)
    {
        const VALID_TYPES: &[&str] = &[
            "potion",
            "food",
            "drink",
            "elixir",
            "flask",
            "bandage",
            "scroll",
            "consumable",
        ];
        if !VALID_TYPES.contains(&ct) {
            issues.push(ValidationIssue {
                severity: ValidationSeverity::Warning,
                code: "helix_unknown_consumable_type".into(),
                source_kind: Some(HELIX_CONSUMABLE_KIND.to_string()),
                source_id: Some(document.id.clone()),
                field_path: Some("payload.consumable_type".into()),
                message: format!("Unrecognized consumable_type '{ct}'."),
                related_refs: Vec::new(),
            });
        }
    }
}

fn validate_helix_currency_document(
    document: &CustomDocument<Value>,
    _loaded: &LoadedCustomDocuments,
    _project: &dj_engine::data::Project,
    issues: &mut Vec<ValidationIssue>,
) {
    validate_helix_document_payload(HELIX_CURRENCY_KIND, document, issues);
    validate_helix_localized_name(HELIX_CURRENCY_KIND, document, issues);
    if let Some(max) = document.payload.get("max_amount").and_then(Value::as_u64) {
        if max == 0 {
            issues.push(ValidationIssue {
                severity: ValidationSeverity::Warning,
                code: "helix_currency_zero_max".into(),
                source_kind: Some(HELIX_CURRENCY_KIND.to_string()),
                source_id: Some(document.id.clone()),
                field_path: Some("max_amount".into()),
                message: "Currency max_amount is 0 — players cannot hold any.".into(),
                related_refs: Vec::<String>::new(),
            });
        }
    }
}

fn validate_helix_guild_document(
    document: &CustomDocument<Value>,
    _loaded: &LoadedCustomDocuments,
    _project: &dj_engine::data::Project,
    issues: &mut Vec<ValidationIssue>,
) {
    validate_helix_document_payload(HELIX_GUILD_KIND, document, issues);
    validate_helix_localized_name(HELIX_GUILD_KIND, document, issues);
    if let Some(max) = document.payload.get("max_members").and_then(Value::as_u64) {
        if max == 0 {
            issues.push(ValidationIssue {
                severity: ValidationSeverity::Warning,
                code: "helix_guild_zero_members".into(),
                source_kind: Some(HELIX_GUILD_KIND.to_string()),
                source_id: Some(document.id.clone()),
                field_path: Some("max_members".into()),
                message: "Guild max_members is 0 — no one can join.".into(),
                related_refs: Vec::<String>::new(),
            });
        }
    }
}

fn validate_helix_inventory_document(
    document: &CustomDocument<Value>,
    _loaded: &LoadedCustomDocuments,
    _project: &dj_engine::data::Project,
    issues: &mut Vec<ValidationIssue>,
) {
    validate_helix_document_payload(HELIX_INVENTORY_KIND, document, issues);
    validate_helix_localized_name(HELIX_INVENTORY_KIND, document, issues);
    if let Some(cap) = document.payload.get("capacity").and_then(Value::as_u64) {
        if cap == 0 {
            issues.push(ValidationIssue {
                severity: ValidationSeverity::Warning,
                code: "helix_inventory_zero_capacity".into(),
                source_kind: Some(HELIX_INVENTORY_KIND.to_string()),
                source_id: Some(document.id.clone()),
                field_path: Some("capacity".into()),
                message: "Inventory capacity is 0 — cannot hold items.".into(),
                related_refs: Vec::<String>::new(),
            });
        }
    }
}

fn validate_helix_mount_document(
    document: &CustomDocument<Value>,
    _loaded: &LoadedCustomDocuments,
    _project: &dj_engine::data::Project,
    issues: &mut Vec<ValidationIssue>,
) {
    validate_helix_document_payload(HELIX_MOUNT_KIND, document, issues);
    validate_helix_localized_name(HELIX_MOUNT_KIND, document, issues);
}

fn validate_helix_profession_document(
    document: &CustomDocument<Value>,
    _loaded: &LoadedCustomDocuments,
    _project: &dj_engine::data::Project,
    issues: &mut Vec<ValidationIssue>,
) {
    validate_helix_document_payload(HELIX_PROFESSION_KIND, document, issues);
    validate_helix_localized_name(HELIX_PROFESSION_KIND, document, issues);
}

fn validate_helix_pvp_document(
    document: &CustomDocument<Value>,
    _loaded: &LoadedCustomDocuments,
    _project: &dj_engine::data::Project,
    issues: &mut Vec<ValidationIssue>,
) {
    validate_helix_document_payload(HELIX_PVP_KIND, document, issues);
    validate_helix_localized_name(HELIX_PVP_KIND, document, issues);
}

fn validate_helix_raid_document(
    document: &CustomDocument<Value>,
    loaded: &LoadedCustomDocuments,
    _project: &dj_engine::data::Project,
    issues: &mut Vec<ValidationIssue>,
) {
    validate_helix_document_payload(HELIX_RAID_KIND, document, issues);
    validate_helix_localized_name(HELIX_RAID_KIND, document, issues);

    // Cross-reference: zone_id must exist in helix_zones.
    if let Some(zone_id) = document.payload.get("zone_id").and_then(Value::as_str) {
        if loaded.get(HELIX_ZONE_KIND, zone_id).is_none() {
            issues.push(ValidationIssue {
                severity: ValidationSeverity::Warning,
                code: "helix_broken_raid_zone_ref".into(),
                source_kind: Some(HELIX_RAID_KIND.to_string()),
                source_id: Some(document.id.clone()),
                field_path: Some("payload.zone_id".into()),
                message: format!(
                    "Referenced zone '{}' not found in loaded documents.",
                    zone_id
                ),
                related_refs: vec![format!("{HELIX_ZONE_KIND}:{zone_id}")],
            });
        }
    }
}

fn validate_helix_talent_document(
    document: &CustomDocument<Value>,
    loaded: &LoadedCustomDocuments,
    _project: &dj_engine::data::Project,
    issues: &mut Vec<ValidationIssue>,
) {
    validate_helix_document_payload(HELIX_TALENT_KIND, document, issues);
    validate_helix_localized_name(HELIX_TALENT_KIND, document, issues);

    // Cross-reference: class_id must exist in helix_class_data.
    if let Some(class_id) = document.payload.get("class_id").and_then(Value::as_str) {
        if loaded.get(HELIX_CLASS_DATA_KIND, class_id).is_none() {
            issues.push(ValidationIssue {
                severity: ValidationSeverity::Warning,
                code: "helix_broken_talent_class_ref".into(),
                source_kind: Some(HELIX_TALENT_KIND.to_string()),
                source_id: Some(document.id.clone()),
                field_path: Some("payload.class_id".into()),
                message: format!(
                    "Referenced class '{}' not found in loaded documents.",
                    class_id
                ),
                related_refs: vec![format!("{HELIX_CLASS_DATA_KIND}:{class_id}")],
            });
        }
    }
}

fn validate_helix_title_document(
    document: &CustomDocument<Value>,
    _loaded: &LoadedCustomDocuments,
    _project: &dj_engine::data::Project,
    issues: &mut Vec<ValidationIssue>,
) {
    validate_helix_document_payload(HELIX_TITLE_KIND, document, issues);
    validate_helix_localized_name(HELIX_TITLE_KIND, document, issues);
    if let Some(style) = document.payload.get("style").and_then(Value::as_str) {
        if style != "prefix" && style != "suffix" {
            issues.push(ValidationIssue {
                severity: ValidationSeverity::Warning,
                code: "helix_title_invalid_style".into(),
                source_kind: Some(HELIX_TITLE_KIND.to_string()),
                source_id: Some(document.id.clone()),
                field_path: Some("style".into()),
                message: format!("Title style '{}' is not 'prefix' or 'suffix'.", style),
                related_refs: Vec::<String>::new(),
            });
        }
    }
}

fn validate_helix_trade_good_document(
    document: &CustomDocument<Value>,
    _loaded: &LoadedCustomDocuments,
    _project: &dj_engine::data::Project,
    issues: &mut Vec<ValidationIssue>,
) {
    validate_helix_document_payload(HELIX_TRADE_GOOD_KIND, document, issues);
    validate_helix_localized_name(HELIX_TRADE_GOOD_KIND, document, issues);
    if let Some(stack) = document.payload.get("stack_size").and_then(Value::as_u64) {
        if stack == 0 {
            issues.push(ValidationIssue {
                severity: ValidationSeverity::Warning,
                code: "helix_trade_good_zero_stack".into(),
                source_kind: Some(HELIX_TRADE_GOOD_KIND.to_string()),
                source_id: Some(document.id.clone()),
                field_path: Some("stack_size".into()),
                message: "Trade good stack_size is 0.".into(),
                related_refs: Vec::<String>::new(),
            });
        }
    }
}

fn validate_helix_weapon_skill_document(
    document: &CustomDocument<Value>,
    loaded: &LoadedCustomDocuments,
    _project: &dj_engine::data::Project,
    issues: &mut Vec<ValidationIssue>,
) {
    validate_helix_document_payload(HELIX_WEAPON_SKILL_KIND, document, issues);
    validate_helix_localized_name(HELIX_WEAPON_SKILL_KIND, document, issues);
    // Cross-ref: each class in classes[] should exist in helix_class_data.
    if let Some(classes) = document.payload.get("classes").and_then(Value::as_array) {
        for class_val in classes {
            if let Some(class_id) = class_val.as_str() {
                if loaded.get(HELIX_CLASS_DATA_KIND, class_id).is_none() {
                    issues.push(ValidationIssue {
                        severity: ValidationSeverity::Warning,
                        code: "helix_weapon_skill_broken_class_ref".into(),
                        source_kind: Some(HELIX_WEAPON_SKILL_KIND.to_string()),
                        source_id: Some(document.id.clone()),
                        field_path: Some("classes".into()),
                        message: format!(
                            "Referenced class '{}' not found in loaded documents.",
                            class_id
                        ),
                        related_refs: vec![format!("{HELIX_CLASS_DATA_KIND}:{class_id}")],
                    });
                }
            }
        }
    }
}

fn validate_helix_localized_name(
    kind: &str,
    document: &CustomDocument<Value>,
    issues: &mut Vec<ValidationIssue>,
) {
    let has_en_name = document
        .payload
        .get("name")
        .and_then(|n| n.get("en"))
        .and_then(Value::as_str)
        .map(|s| !s.trim().is_empty())
        .unwrap_or(false);

    if !has_en_name {
        issues.push(ValidationIssue {
            severity: ValidationSeverity::Warning,
            code: "helix_missing_en_name".into(),
            source_kind: Some(kind.to_string()),
            source_id: Some(document.id.clone()),
            field_path: Some("payload.name.en".into()),
            message: "Missing or empty English name (name.en).".into(),
            related_refs: Vec::new(),
        });
    }
}

pub fn safe_document_reference(
    field_path: impl Into<String>,
    kind: &str,
    id: &str,
) -> DocumentLink {
    DocumentLink {
        field_path: field_path.into(),
        target: dj_engine::data::DocumentLinkTarget::Document {
            kind: kind.to_string(),
            id: id.to_string(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dj_engine::data::{CustomDocument, LoadedCustomDocuments};
    use serde_json::json;

    #[test]
    fn test_validate_helix_document_payload_rejects_non_object_payload() {
        let document = CustomDocument {
            kind: HELIX_ABILITY_KIND.into(),
            id: "fireball".into(),
            schema_version: 1,
            label: None,
            tags: Vec::new(),
            references: Vec::new(),
            payload: json!(["bad"]),
        };

        let mut issues = Vec::new();
        validate_helix_ability_document(
            &document,
            &LoadedCustomDocuments::default(),
            &dj_engine::data::Project::new("Test"),
            &mut issues,
        );

        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].code, "helix_payload_not_object");
    }

    #[test]
    fn test_index_rebuild_collects_parsed_documents_by_kind() {
        let mut index = HelixDocumentIndex::default();
        let loaded = LoadedCustomDocuments {
            manifest_path: None,
            manifest: None,
            documents: vec![dj_engine::data::LoadedCustomDocument {
                entry: dj_engine::data::CustomDocumentEntry {
                    kind: HELIX_ITEM_KIND.into(),
                    id: "dagger".into(),
                    path: "helix_items/weapon_items/dagger.json".into(),
                    schema_version: 1,
                    editor_route: EditorDocumentRoute::Inspector,
                    tags: Vec::new(),
                },
                raw_json: String::new(),
                document: Some(CustomDocument {
                    kind: HELIX_ITEM_KIND.into(),
                    id: "dagger".into(),
                    schema_version: 1,
                    label: Some("Dagger".into()),
                    tags: Vec::new(),
                    references: Vec::new(),
                    payload: json!({ "id": "dagger" }),
                }),
                parse_error: None,
                resolved_route: EditorDocumentRoute::Inspector,
            }],
            issues: Vec::new(),
        };

        index.rebuild_from_loaded_documents(&loaded);
        assert!(index.item("dagger").is_some());
        assert!(index.get(HELIX_ITEM_KIND, "dagger").is_some());
        assert_eq!(index.total(), 1);
    }

    #[test]
    fn test_validate_helix_localized_name_catches_missing_en() {
        let document = CustomDocument {
            kind: HELIX_ZONE_KIND.into(),
            id: "dark_forest".into(),
            schema_version: 1,
            label: None,
            tags: Vec::new(),
            references: Vec::new(),
            payload: json!({ "name": { "ja": "暗い森" } }),
        };

        let mut issues = Vec::new();
        validate_helix_zone_document(
            &document,
            &LoadedCustomDocuments::default(),
            &dj_engine::data::Project::new("Test"),
            &mut issues,
        );

        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].code, "helix_missing_en_name");
    }

    #[test]
    fn test_validate_helix_equipment_catches_invalid_slot() {
        let document = CustomDocument {
            kind: HELIX_EQUIPMENT_KIND.into(),
            id: "weird_gear".into(),
            schema_version: 1,
            label: None,
            tags: Vec::new(),
            references: Vec::new(),
            payload: json!({ "equip_slot": "nostril" }),
        };

        let mut issues = Vec::new();
        validate_helix_equipment_document(
            &document,
            &LoadedCustomDocuments::default(),
            &dj_engine::data::Project::new("Test"),
            &mut issues,
        );

        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].code, "helix_unknown_equip_slot");
    }

    /// Helper: make a minimal document for a given kind with the specified payload.
    fn make_doc(kind: &str, id: &str, payload: Value) -> CustomDocument<Value> {
        CustomDocument {
            kind: kind.into(),
            id: id.into(),
            schema_version: 1,
            label: None,
            tags: Vec::new(),
            references: Vec::new(),
            payload,
        }
    }

    /// Helper: make a LoadedCustomDocument entry for cross-ref lookups.
    fn make_loaded_entry(kind: &str, id: &str) -> dj_engine::data::LoadedCustomDocument {
        dj_engine::data::LoadedCustomDocument {
            entry: dj_engine::data::CustomDocumentEntry {
                kind: kind.into(),
                id: id.into(),
                path: format!("{kind}/{id}.toml"),
                schema_version: 1,
                editor_route: EditorDocumentRoute::Table,
                tags: Vec::new(),
            },
            raw_json: String::new(),
            document: Some(CustomDocument {
                kind: kind.into(),
                id: id.into(),
                schema_version: 1,
                label: None,
                tags: Vec::new(),
                references: Vec::new(),
                payload: json!({ "id": id }),
            }),
            parse_error: None,
            resolved_route: EditorDocumentRoute::Table,
        }
    }

    #[test]
    fn test_new_validators_no_panic_on_empty_payload() {
        let project = dj_engine::data::Project::new("Test");
        let loaded = LoadedCustomDocuments::default();

        // Each new validator should handle an empty object payload without panicking.
        let validators: Vec<(
            &str,
            fn(
                &CustomDocument<Value>,
                &LoadedCustomDocuments,
                &dj_engine::data::Project,
                &mut Vec<ValidationIssue>,
            ),
        )> = vec![
            (HELIX_AURA_KIND, validate_helix_aura_document),
            (HELIX_CLASS_DATA_KIND, validate_helix_class_data_document),
            (HELIX_CONSUMABLE_KIND, validate_helix_consumable_document),
            (HELIX_CURRENCY_KIND, validate_helix_currency_document),
            (HELIX_GUILD_KIND, validate_helix_guild_document),
            (HELIX_INVENTORY_KIND, validate_helix_inventory_document),
            (HELIX_MOUNT_KIND, validate_helix_mount_document),
            (HELIX_PROFESSION_KIND, validate_helix_profession_document),
            (HELIX_PVP_KIND, validate_helix_pvp_document),
            (HELIX_RAID_KIND, validate_helix_raid_document),
            (HELIX_TALENT_KIND, validate_helix_talent_document),
            (HELIX_TITLE_KIND, validate_helix_title_document),
            (HELIX_TRADE_GOOD_KIND, validate_helix_trade_good_document),
            (
                HELIX_WEAPON_SKILL_KIND,
                validate_helix_weapon_skill_document,
            ),
        ];

        for (kind, validator) in &validators {
            let doc = make_doc(kind, "test_id", json!({}));
            let mut issues = Vec::new();
            validator(&doc, &loaded, &project, &mut issues);
            // Should produce issues (missing id, missing name) but never panic.
            assert!(
                !issues.is_empty(),
                "{kind} validator produced no issues on empty payload"
            );
        }
    }

    #[test]
    fn test_new_validators_no_panic_on_non_object_payload() {
        let project = dj_engine::data::Project::new("Test");
        let loaded = LoadedCustomDocuments::default();

        let validators: Vec<(
            &str,
            fn(
                &CustomDocument<Value>,
                &LoadedCustomDocuments,
                &dj_engine::data::Project,
                &mut Vec<ValidationIssue>,
            ),
        )> = vec![
            (HELIX_AURA_KIND, validate_helix_aura_document),
            (HELIX_CLASS_DATA_KIND, validate_helix_class_data_document),
            (HELIX_CONSUMABLE_KIND, validate_helix_consumable_document),
            (HELIX_CURRENCY_KIND, validate_helix_currency_document),
            (HELIX_GUILD_KIND, validate_helix_guild_document),
            (HELIX_INVENTORY_KIND, validate_helix_inventory_document),
            (HELIX_MOUNT_KIND, validate_helix_mount_document),
            (HELIX_PROFESSION_KIND, validate_helix_profession_document),
            (HELIX_PVP_KIND, validate_helix_pvp_document),
            (HELIX_RAID_KIND, validate_helix_raid_document),
            (HELIX_TALENT_KIND, validate_helix_talent_document),
            (HELIX_TITLE_KIND, validate_helix_title_document),
            (HELIX_TRADE_GOOD_KIND, validate_helix_trade_good_document),
            (
                HELIX_WEAPON_SKILL_KIND,
                validate_helix_weapon_skill_document,
            ),
        ];

        for (kind, validator) in &validators {
            let doc = make_doc(kind, "test_id", json!("not_an_object"));
            let mut issues = Vec::new();
            validator(&doc, &loaded, &project, &mut issues);
            assert!(
                issues.iter().any(|i| i.code == "helix_payload_not_object"),
                "{kind} validator did not catch non-object payload"
            );
        }
    }

    #[test]
    fn test_new_validators_clean_on_valid_document() {
        let project = dj_engine::data::Project::new("Test");
        let loaded = LoadedCustomDocuments::default();

        // A valid document with matching id and name.en should produce no issues
        // for simple validators (those without cross-ref checks).
        let simple_kinds: &[(
            &str,
            fn(
                &CustomDocument<Value>,
                &LoadedCustomDocuments,
                &dj_engine::data::Project,
                &mut Vec<ValidationIssue>,
            ),
        )] = &[
            (HELIX_AURA_KIND, validate_helix_aura_document),
            (HELIX_CLASS_DATA_KIND, validate_helix_class_data_document),
            (HELIX_CURRENCY_KIND, validate_helix_currency_document),
            (HELIX_GUILD_KIND, validate_helix_guild_document),
            (HELIX_INVENTORY_KIND, validate_helix_inventory_document),
            (HELIX_MOUNT_KIND, validate_helix_mount_document),
            (HELIX_PROFESSION_KIND, validate_helix_profession_document),
            (HELIX_PVP_KIND, validate_helix_pvp_document),
            (HELIX_TITLE_KIND, validate_helix_title_document),
            (HELIX_TRADE_GOOD_KIND, validate_helix_trade_good_document),
            (
                HELIX_WEAPON_SKILL_KIND,
                validate_helix_weapon_skill_document,
            ),
        ];

        for (kind, validator) in simple_kinds {
            let doc = make_doc(
                kind,
                "valid_id",
                json!({ "id": "valid_id", "name": { "en": "Valid Name" } }),
            );
            let mut issues = Vec::new();
            validator(&doc, &loaded, &project, &mut issues);
            assert!(
                issues.is_empty(),
                "{kind} validator produced issues on valid document: {:?}",
                issues
            );
        }
    }

    #[test]
    fn test_consumable_validator_catches_invalid_type() {
        let project = dj_engine::data::Project::new("Test");
        let loaded = LoadedCustomDocuments::default();

        let doc = make_doc(
            HELIX_CONSUMABLE_KIND,
            "bad_potion",
            json!({
                "id": "bad_potion",
                "name": { "en": "Bad Potion" },
                "consumable_type": "nasal_spray"
            }),
        );
        let mut issues = Vec::new();
        validate_helix_consumable_document(&doc, &loaded, &project, &mut issues);
        assert!(issues
            .iter()
            .any(|i| i.code == "helix_unknown_consumable_type"));
    }

    #[test]
    fn test_consumable_validator_accepts_valid_type() {
        let project = dj_engine::data::Project::new("Test");
        let loaded = LoadedCustomDocuments::default();

        let doc = make_doc(
            HELIX_CONSUMABLE_KIND,
            "health_pot",
            json!({
                "id": "health_pot",
                "name": { "en": "Health Potion" },
                "consumable_type": "potion"
            }),
        );
        let mut issues = Vec::new();
        validate_helix_consumable_document(&doc, &loaded, &project, &mut issues);
        assert!(
            issues.is_empty(),
            "valid consumable produced issues: {:?}",
            issues
        );
    }

    #[test]
    fn test_talent_validator_catches_missing_class_ref() {
        let project = dj_engine::data::Project::new("Test");
        let loaded = LoadedCustomDocuments::default();

        let doc = make_doc(
            HELIX_TALENT_KIND,
            "mortal_strike",
            json!({
                "id": "mortal_strike",
                "name": { "en": "Mortal Strike" },
                "class_id": "nonexistent_class"
            }),
        );
        let mut issues = Vec::new();
        validate_helix_talent_document(&doc, &loaded, &project, &mut issues);
        assert!(issues
            .iter()
            .any(|i| i.code == "helix_broken_talent_class_ref"));
    }

    #[test]
    fn test_talent_validator_passes_with_valid_class_ref() {
        let project = dj_engine::data::Project::new("Test");
        let mut loaded = LoadedCustomDocuments::default();
        loaded
            .documents
            .push(make_loaded_entry(HELIX_CLASS_DATA_KIND, "warrior"));

        let doc = make_doc(
            HELIX_TALENT_KIND,
            "mortal_strike",
            json!({
                "id": "mortal_strike",
                "name": { "en": "Mortal Strike" },
                "class_id": "warrior"
            }),
        );
        let mut issues = Vec::new();
        validate_helix_talent_document(&doc, &loaded, &project, &mut issues);
        assert!(
            issues.is_empty(),
            "talent with valid class_id produced issues: {:?}",
            issues
        );
    }

    #[test]
    fn test_raid_validator_catches_missing_zone_ref() {
        let project = dj_engine::data::Project::new("Test");
        let loaded = LoadedCustomDocuments::default();

        let doc = make_doc(
            HELIX_RAID_KIND,
            "molten_core",
            json!({
                "id": "molten_core",
                "name": { "en": "Molten Core" },
                "zone_id": "nonexistent_zone"
            }),
        );
        let mut issues = Vec::new();
        validate_helix_raid_document(&doc, &loaded, &project, &mut issues);
        assert!(issues
            .iter()
            .any(|i| i.code == "helix_broken_raid_zone_ref"));
    }

    #[test]
    fn test_raid_validator_passes_with_valid_zone_ref() {
        let project = dj_engine::data::Project::new("Test");
        let mut loaded = LoadedCustomDocuments::default();
        loaded
            .documents
            .push(make_loaded_entry(HELIX_ZONE_KIND, "mc_zone"));

        let doc = make_doc(
            HELIX_RAID_KIND,
            "molten_core",
            json!({
                "id": "molten_core",
                "name": { "en": "Molten Core" },
                "zone_id": "mc_zone"
            }),
        );
        let mut issues = Vec::new();
        validate_helix_raid_document(&doc, &loaded, &project, &mut issues);
        assert!(
            issues.is_empty(),
            "raid with valid zone_id produced issues: {:?}",
            issues
        );
    }

    #[test]
    fn test_populate_database_from_real_helix_registries() {
        let helix3d_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../helix/helix_standardization/dist/helix3d");
        if !helix3d_dir.is_dir() {
            // Skip if helix3d data isn't available (CI, etc.)
            return;
        }

        let registries = registries::load_helix_registries_lenient(&helix3d_dir).unwrap();
        assert!(registries.total_entities() > 0);

        let db = bridge::populate_database_from_helix(&registries, None);
        assert!(!db.items.is_empty(), "expected items from helix registries");
        assert!(
            !db.enemies.is_empty(),
            "expected enemies from helix registries"
        );
    }

    #[test]
    fn test_helix_database_resource_populated_via_plugin_startup() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(HelixDataPlugin);

        // Verify resources exist after plugin init.
        assert!(app.world().contains_resource::<HelixDatabase>());
        assert!(app.world().contains_resource::<balance::BalanceOverlays>());

        // Without a helix3d_path, database stays empty (no panic).
        let db = app.world().resource::<HelixDatabase>();
        assert!(db.0.items.is_empty());
    }
}
