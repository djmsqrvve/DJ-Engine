//! Bidirectional TOML exporter for helix3d data.
//!
//! Reads `LoadedCustomDocuments` and writes one TOML file per helix kind
//! back to a `dist/helix3d/`-compatible directory.  This closes the
//! round-trip: TOML -> engine editor -> TOML.

use dj_engine::data::LoadedCustomDocuments;
use std::collections::BTreeMap;
use std::path::Path;
use thiserror::Error;

/// Prefix that identifies helix document kinds.
const HELIX_PREFIX: &str = "helix_";

/// Summary returned after a successful export.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportSummary {
    pub files_written: usize,
    pub entities_exported: usize,
}

/// Errors that can occur during export.
#[derive(Debug, Error)]
pub enum ExportError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("TOML serialization error: {0}")]
    Toml(String),

    #[error("JSON-to-TOML conversion error for {kind}/{id}: {message}")]
    Conversion {
        kind: String,
        id: String,
        message: String,
    },
}

/// Export edited custom documents back to helix3d TOML format.
///
/// Groups documents by kind, writes one TOML file per kind to `output_dir`.
/// Only exports `helix_*` document kinds.  Each entity appears as a named
/// table whose key is the entity ID.
///
/// ```toml
/// [fireball]
/// name = { en = "Fireball" }
/// ability_type = "offensive"
/// cooldown = 8.0
/// ```
pub fn export_to_helix3d(
    documents: &LoadedCustomDocuments,
    output_dir: &Path,
) -> Result<ExportSummary, ExportError> {
    // Group payloads by kind -> BTreeMap<entity_id, toml::Value>.
    let mut grouped: BTreeMap<String, BTreeMap<String, toml::Value>> = BTreeMap::new();

    for loaded in &documents.documents {
        if !loaded.entry.kind.starts_with(HELIX_PREFIX) {
            continue;
        }

        let envelope = match &loaded.document {
            Some(env) => env,
            None => continue,
        };

        let toml_value =
            json_to_toml(&envelope.payload).map_err(|message| ExportError::Conversion {
                kind: loaded.entry.kind.clone(),
                id: loaded.entry.id.clone(),
                message,
            })?;

        grouped
            .entry(loaded.entry.kind.clone())
            .or_default()
            .insert(loaded.entry.id.clone(), toml_value);
    }

    std::fs::create_dir_all(output_dir)?;

    let mut files_written = 0usize;
    let mut entities_exported = 0usize;

    for (kind, entities) in &grouped {
        let filename = kind_to_filename(kind);
        let path = output_dir.join(&filename);

        // Build a top-level TOML table with each entity as a named sub-table.
        let top_level = toml::Value::Table(entities.clone().into_iter().collect());
        let toml_string =
            toml::to_string_pretty(&top_level).map_err(|e| ExportError::Toml(e.to_string()))?;

        std::fs::write(&path, toml_string)?;

        files_written += 1;
        entities_exported += entities.len();
    }

    Ok(ExportSummary {
        files_written,
        entities_exported,
    })
}

/// Strip the `helix_` prefix from a kind string to produce a TOML filename.
///
/// `"helix_abilities"` -> `"abilities.toml"`
fn kind_to_filename(kind: &str) -> String {
    let stem = kind.strip_prefix(HELIX_PREFIX).unwrap_or(kind);
    format!("{stem}.toml")
}

/// Convert a `serde_json::Value` into a `toml::Value`.
///
/// TOML cannot represent `null`, so null values are omitted (mapped to
/// empty strings for leaf nulls inside objects, or skipped entirely).
fn json_to_toml(json: &serde_json::Value) -> Result<toml::Value, String> {
    match json {
        serde_json::Value::Null => Ok(toml::Value::String(String::new())),
        serde_json::Value::Bool(b) => Ok(toml::Value::Boolean(*b)),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(toml::Value::Integer(i))
            } else if let Some(f) = n.as_f64() {
                Ok(toml::Value::Float(f))
            } else {
                Err(format!("unsupported JSON number: {n}"))
            }
        }
        serde_json::Value::String(s) => Ok(toml::Value::String(s.clone())),
        serde_json::Value::Array(arr) => {
            let items: Result<Vec<toml::Value>, String> = arr.iter().map(json_to_toml).collect();
            Ok(toml::Value::Array(items?))
        }
        serde_json::Value::Object(map) => {
            let mut table = toml::map::Map::new();
            for (key, value) in map {
                // Skip null values in objects — TOML has no null.
                if value.is_null() {
                    continue;
                }
                table.insert(key.clone(), json_to_toml(value)?);
            }
            Ok(toml::Value::Table(table))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dj_engine::data::{
        CustomDocument, CustomDocumentEntry, EditorDocumentRoute, LoadedCustomDocument,
    };
    use serde_json::json;
    use tempfile::tempdir;

    /// Helper: build a `LoadedCustomDocument` with a given kind, id, and payload.
    fn make_loaded(kind: &str, id: &str, payload: serde_json::Value) -> LoadedCustomDocument {
        let envelope = CustomDocument {
            kind: kind.to_string(),
            id: id.to_string(),
            schema_version: 1,
            label: None,
            tags: vec!["source:toml_registry".to_string()],
            references: Vec::new(),
            payload,
        };
        let raw_json = serde_json::to_string_pretty(&envelope).unwrap_or_default();

        LoadedCustomDocument {
            entry: CustomDocumentEntry {
                kind: kind.to_string(),
                id: id.to_string(),
                path: format!("{kind}/{id}.toml"),
                schema_version: 1,
                editor_route: EditorDocumentRoute::Table,
                tags: vec!["source:toml_registry".to_string()],
            },
            raw_json,
            document: Some(envelope),
            parse_error: None,
            resolved_route: EditorDocumentRoute::Table,
        }
    }

    #[test]
    fn test_export_creates_files() {
        let dir = tempdir().unwrap();
        let mut loaded = LoadedCustomDocuments::default();
        loaded.documents.push(make_loaded(
            "helix_abilities",
            "fireball",
            json!({
                "name": { "en": "Fireball" },
                "ability_type": "offensive",
                "cooldown": 8.0
            }),
        ));
        loaded.documents.push(make_loaded(
            "helix_abilities",
            "ice_bolt",
            json!({
                "name": { "en": "Ice Bolt" },
                "ability_type": "offensive",
                "cooldown": 6.0
            }),
        ));

        let summary = export_to_helix3d(&loaded, dir.path()).unwrap();

        assert_eq!(summary.files_written, 1);
        assert_eq!(summary.entities_exported, 2);
        assert!(dir.path().join("abilities.toml").exists());
    }

    #[test]
    fn test_export_toml_format() {
        let dir = tempdir().unwrap();
        let mut loaded = LoadedCustomDocuments::default();
        loaded.documents.push(make_loaded(
            "helix_abilities",
            "fireball",
            json!({
                "name": { "en": "Fireball" },
                "ability_type": "offensive",
                "cooldown": 8.0
            }),
        ));

        export_to_helix3d(&loaded, dir.path()).unwrap();

        let content = std::fs::read_to_string(dir.path().join("abilities.toml")).unwrap();

        // Verify named table structure.
        assert!(
            content.contains("[fireball]"),
            "Expected [fireball] table header, got:\n{content}"
        );
        assert!(
            content.contains("ability_type = \"offensive\""),
            "Expected ability_type field, got:\n{content}"
        );
        assert!(
            content.contains("cooldown = 8.0"),
            "Expected cooldown field, got:\n{content}"
        );

        // Verify the name sub-table is present.
        assert!(
            content.contains("[fireball.name]") || content.contains("name = { en"),
            "Expected name sub-table, got:\n{content}"
        );
    }

    #[test]
    fn test_export_strips_helix_prefix() {
        let dir = tempdir().unwrap();
        let mut loaded = LoadedCustomDocuments::default();
        loaded.documents.push(make_loaded(
            "helix_abilities",
            "fireball",
            json!({ "name": { "en": "Fireball" } }),
        ));
        loaded.documents.push(make_loaded(
            "helix_items",
            "dagger",
            json!({ "name": { "en": "Dagger" } }),
        ));

        let summary = export_to_helix3d(&loaded, dir.path()).unwrap();

        assert_eq!(summary.files_written, 2);
        assert!(
            dir.path().join("abilities.toml").exists(),
            "Expected abilities.toml"
        );
        assert!(
            dir.path().join("items.toml").exists(),
            "Expected items.toml"
        );
        // Should NOT have the helix_ prefix in filenames.
        assert!(!dir.path().join("helix_abilities.toml").exists());
        assert!(!dir.path().join("helix_items.toml").exists());
    }

    #[test]
    fn test_export_skips_non_helix_kinds() {
        let dir = tempdir().unwrap();
        let mut loaded = LoadedCustomDocuments::default();
        loaded
            .documents
            .push(make_loaded("custom_foo", "bar", json!({ "name": "Bar" })));
        loaded.documents.push(make_loaded(
            "preview_profiles",
            "default",
            json!({ "scene_id": "main" }),
        ));

        let summary = export_to_helix3d(&loaded, dir.path()).unwrap();

        assert_eq!(summary.files_written, 0);
        assert_eq!(summary.entities_exported, 0);

        // No TOML files should be written.
        let entries: Vec<_> = std::fs::read_dir(dir.path())
            .unwrap()
            .filter_map(Result::ok)
            .collect();
        assert!(entries.is_empty(), "Expected no files, found {entries:?}");
    }

    #[test]
    fn test_export_null_values_omitted() {
        let dir = tempdir().unwrap();
        let mut loaded = LoadedCustomDocuments::default();
        loaded.documents.push(make_loaded(
            "helix_items",
            "mystery_box",
            json!({
                "name": { "en": "Mystery Box" },
                "description": null,
                "sell_price": 100
            }),
        ));

        export_to_helix3d(&loaded, dir.path()).unwrap();

        let content = std::fs::read_to_string(dir.path().join("items.toml")).unwrap();
        assert!(
            !content.contains("description"),
            "Null fields should be omitted, got:\n{content}"
        );
        assert!(content.contains("sell_price = 100"));
    }

    #[test]
    fn test_export_deterministic_ordering() {
        let dir = tempdir().unwrap();
        let mut loaded = LoadedCustomDocuments::default();

        // Insert in non-alphabetical order.  Use flat payloads so the
        // TOML serializer emits `[id]` headers we can search for reliably.
        for id in ["zephyr", "alpha", "mid"] {
            loaded.documents.push(make_loaded(
                "helix_abilities",
                id,
                json!({ "ability_type": "offensive" }),
            ));
        }

        export_to_helix3d(&loaded, dir.path()).unwrap();

        let content = std::fs::read_to_string(dir.path().join("abilities.toml")).unwrap();
        let alpha_pos = content.find("[alpha]").expect("missing [alpha]");
        let mid_pos = content.find("[mid]").expect("missing [mid]");
        let zephyr_pos = content.find("[zephyr]").expect("missing [zephyr]");

        assert!(
            alpha_pos < mid_pos && mid_pos < zephyr_pos,
            "Expected alphabetical table ordering, got:\n{content}"
        );
    }

    #[test]
    fn test_export_skips_documents_without_envelope() {
        let dir = tempdir().unwrap();
        let mut loaded = LoadedCustomDocuments::default();

        // Document with no parsed envelope (e.g. parse error).
        loaded.documents.push(LoadedCustomDocument {
            entry: CustomDocumentEntry {
                kind: "helix_abilities".to_string(),
                id: "broken".to_string(),
                path: "helix_abilities/broken.toml".to_string(),
                schema_version: 1,
                editor_route: EditorDocumentRoute::Table,
                tags: Vec::new(),
            },
            raw_json: String::new(),
            document: None,
            parse_error: Some("bad data".to_string()),
            resolved_route: EditorDocumentRoute::Table,
        });

        let summary = export_to_helix3d(&loaded, dir.path()).unwrap();

        assert_eq!(summary.files_written, 0);
        assert_eq!(summary.entities_exported, 0);
    }

    #[test]
    fn test_json_to_toml_basic_types() {
        assert_eq!(
            json_to_toml(&json!(true)).unwrap(),
            toml::Value::Boolean(true)
        );
        assert_eq!(json_to_toml(&json!(42)).unwrap(), toml::Value::Integer(42));
        assert_eq!(
            json_to_toml(&json!(3.14)).unwrap(),
            toml::Value::Float(3.14)
        );
        assert_eq!(
            json_to_toml(&json!("hello")).unwrap(),
            toml::Value::String("hello".into())
        );
    }

    #[test]
    fn test_kind_to_filename() {
        assert_eq!(kind_to_filename("helix_abilities"), "abilities.toml");
        assert_eq!(kind_to_filename("helix_class_data"), "class_data.toml");
        assert_eq!(
            kind_to_filename("helix_weapon_skills"),
            "weapon_skills.toml"
        );
        // Edge case: no prefix (should still work, just doesn't strip).
        assert_eq!(kind_to_filename("custom_foo"), "custom_foo.toml");
    }
}
