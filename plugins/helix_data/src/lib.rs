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

pub mod importer;

pub use importer::{
    import_helix_project, parse_helix_import_cli_args, HelixImportCliOptions, HelixImportError,
    HelixImportSummary,
};

pub const HELIX_ABILITY_KIND: &str = "helix_abilities";
pub const HELIX_ITEM_KIND: &str = "helix_items";
pub const HELIX_MOB_KIND: &str = "helix_mobs";
pub const HELIX_IMPORT_PREVIEW_ID: &str = "helix_import_preview";

/// Configuration for the Helix import pipeline. Set `helix_dist_path` to enable
/// in-editor re-import via the "Re-import Helix Data" toolbar action.
#[derive(Resource, Default, Debug, Clone)]
pub struct HelixImportConfig {
    pub helix_dist_path: Option<PathBuf>,
}

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

#[derive(Resource, Default, Debug, Clone, PartialEq)]
pub struct HelixDocumentIndex {
    pub abilities: BTreeMap<String, CustomDocument<Value>>,
    pub items: BTreeMap<String, CustomDocument<Value>>,
    pub mobs: BTreeMap<String, CustomDocument<Value>>,
}

impl HelixDocumentIndex {
    pub fn ability(&self, id: &str) -> Option<&CustomDocument<Value>> {
        self.abilities.get(id)
    }

    pub fn item(&self, id: &str) -> Option<&CustomDocument<Value>> {
        self.items.get(id)
    }

    pub fn mob(&self, id: &str) -> Option<&CustomDocument<Value>> {
        self.mobs.get(id)
    }

    fn rebuild_from_loaded_documents(&mut self, loaded_documents: &LoadedCustomDocuments) {
        self.abilities.clear();
        self.items.clear();
        self.mobs.clear();

        for document in &loaded_documents.documents {
            let Some(parsed) = document.document.clone() else {
                continue;
            };

            match document.entry.kind.as_str() {
                HELIX_ABILITY_KIND => {
                    self.abilities.insert(document.entry.id.clone(), parsed);
                }
                HELIX_ITEM_KIND => {
                    self.items.insert(document.entry.id.clone(), parsed);
                }
                HELIX_MOB_KIND => {
                    self.mobs.insert(document.entry.id.clone(), parsed);
                }
                _ => {}
            }
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
            .init_resource::<HelixImportConfig>()
            .add_systems(
                Update,
                (
                    refresh_helix_document_index_system,
                    handle_helix_toolbar_actions_system,
                ),
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
    }
}
