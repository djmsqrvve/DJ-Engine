//! Panel data export — each panel can serialize its visible content to disk.
//!
//! Exports go to `<project_root>/exports/` with timestamped filenames.

use crate::data::LoadedCustomDocuments;
use crate::diagnostics::console::ConsoleLogStore;
use crate::project_mount::MountedProject;
use bevy::prelude::*;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

/// What kind of panel content is being exported.
#[derive(Debug, Clone)]
pub enum PanelExportKind {
    /// All documents of a specific kind (or all kinds if None).
    Documents { kind_filter: Option<String> },
    /// A single document by kind + id.
    DocumentInspector { kind: String, id: String },
    /// The full story graph.
    StoryGraph,
    /// The current scene.
    Scene,
    /// Console log text.
    Console,
    /// Asset file listing.
    AssetListing,
}

/// Pending export request, consumed by the export system.
#[derive(Resource, Default)]
pub struct PanelExportRequest {
    pub pending: Option<PanelExportKind>,
}

/// Result of the last export, shown as feedback.
#[derive(Resource, Default, Debug, Clone)]
pub struct PanelExportResult {
    pub message: Option<String>,
    pub is_error: bool,
}

fn export_dir(mounted_project: &MountedProject) -> Option<PathBuf> {
    Some(mounted_project.root_path.as_ref()?.join("exports"))
}

fn timestamped_filename(prefix: &str, extension: &str) -> String {
    let ts = chrono::Local::now().format("%Y%m%d_%H%M%S");
    format!("{prefix}_{ts}.{extension}")
}

fn write_export(dir: &Path, filename: &str, content: &str) -> Result<PathBuf, String> {
    fs::create_dir_all(dir).map_err(|e| format!("Failed to create exports dir: {e}"))?;
    let path = dir.join(filename);
    fs::write(&path, content).map_err(|e| format!("Failed to write {}: {e}", path.display()))?;
    Ok(path)
}

/// Export documents matching the kind filter as a JSON array.
pub fn export_documents(
    loaded_documents: &LoadedCustomDocuments,
    kind_filter: Option<&str>,
) -> String {
    #[derive(Serialize)]
    struct ExportDoc {
        kind: String,
        id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        label: Option<String>,
        tags: Vec<String>,
        payload: serde_json::Value,
        #[serde(skip_serializing_if = "Vec::is_empty")]
        issues: Vec<ExportIssue>,
    }

    #[derive(Serialize)]
    struct ExportIssue {
        severity: String,
        code: String,
        message: String,
    }

    let docs: Vec<ExportDoc> = loaded_documents
        .documents
        .iter()
        .filter(|doc| {
            kind_filter
                .map(|k| k == "all" || doc.entry.kind == k)
                .unwrap_or(true)
        })
        .filter_map(|doc| {
            let parsed = doc.document.as_ref()?;
            let issues: Vec<ExportIssue> = loaded_documents
                .issues_for(&doc.entry.kind, &doc.entry.id)
                .iter()
                .map(|i| ExportIssue {
                    severity: format!("{:?}", i.severity),
                    code: i.code.clone(),
                    message: i.message.clone(),
                })
                .collect();
            Some(ExportDoc {
                kind: parsed.kind.clone(),
                id: parsed.id.clone(),
                label: parsed.label.clone(),
                tags: parsed.tags.clone(),
                payload: parsed.payload.clone(),
                issues,
            })
        })
        .collect();

    serde_json::to_string_pretty(&docs).unwrap_or_else(|e| format!("{{\"error\": \"{e}\"}}"))
}

/// Export a single document as pretty JSON.
pub fn export_single_document(
    loaded_documents: &LoadedCustomDocuments,
    kind: &str,
    id: &str,
) -> Option<String> {
    let doc = loaded_documents
        .documents
        .iter()
        .find(|d| d.entry.kind == kind && d.entry.id == id)?;
    let parsed = doc.document.as_ref()?;
    Some(serde_json::to_string_pretty(parsed).unwrap_or_else(|e| format!("{{\"error\": \"{e}\"}}")))
}

/// Export console logs as plain text.
pub fn export_console(console: &ConsoleLogStore) -> String {
    console.logs.join("\n")
}

/// Export an asset listing as plain text.
pub fn export_asset_listing(mounted_project: &MountedProject) -> String {
    let Some(root) = mounted_project.root_path.as_ref() else {
        return "No project mounted.".into();
    };
    let asset_dir = root.join(
        mounted_project
            .project
            .as_ref()
            .map(|p| p.settings.paths.assets.as_str())
            .unwrap_or("assets"),
    );
    let mut lines = vec![format!("# Asset listing: {}", asset_dir.display())];
    collect_file_listing(&asset_dir, &asset_dir, &mut lines);
    if lines.len() == 1 {
        lines.push("(empty)".into());
    }
    lines.join("\n")
}

fn collect_file_listing(root: &Path, dir: &Path, lines: &mut Vec<String>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    let mut entries: Vec<_> = entries.filter_map(Result::ok).collect();
    entries.sort_by_key(|e| e.path());
    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            collect_file_listing(root, &path, lines);
        } else if let Ok(rel) = path.strip_prefix(root) {
            lines.push(rel.display().to_string());
        }
    }
}

/// System that processes pending export requests.
pub fn process_panel_export_system(world: &mut World) {
    let pending = world
        .get_resource_mut::<PanelExportRequest>()
        .and_then(|mut req| req.pending.take());

    let Some(kind) = pending else {
        return;
    };

    let mounted_project = world.resource::<MountedProject>().clone();
    let Some(dir) = export_dir(&mounted_project) else {
        set_export_result(world, "No project mounted — cannot export.", true);
        return;
    };

    let result = match &kind {
        PanelExportKind::Documents { kind_filter } => {
            let loaded = world.resource::<LoadedCustomDocuments>();
            let content = export_documents(loaded, kind_filter.as_deref());
            let suffix = kind_filter
                .as_deref()
                .filter(|k| *k != "all")
                .unwrap_or("all");
            let filename = timestamped_filename(&format!("documents_{suffix}"), "json");
            write_export(&dir, &filename, &content)
        }
        PanelExportKind::DocumentInspector { kind, id } => {
            let loaded = world.resource::<LoadedCustomDocuments>();
            match export_single_document(loaded, kind, id) {
                Some(content) => {
                    let filename = timestamped_filename(&format!("{kind}_{id}"), "json");
                    write_export(&dir, &filename, &content)
                }
                None => Err(format!("Document {kind}:{id} not found.")),
            }
        }
        PanelExportKind::StoryGraph => {
            let graph = world.resource::<super::types::ActiveStoryGraph>();
            let content = serde_json::to_string_pretty(&graph.0)
                .unwrap_or_else(|e| format!("{{\"error\": \"{e}\"}}"));
            let filename = timestamped_filename("story_graph", "json");
            write_export(&dir, &filename, &content)
        }
        PanelExportKind::Scene => match super::scene_io::capture_editor_snapshot(world) {
            Ok(snapshot) => {
                let content = snapshot.scene_json.unwrap_or_else(|| "{}".into());
                let filename = timestamped_filename("scene", "json");
                write_export(&dir, &filename, &content)
            }
            Err(e) => Err(format!("Failed to capture scene: {e}")),
        },
        PanelExportKind::Console => {
            let console = world.resource::<ConsoleLogStore>();
            let content = export_console(console);
            let filename = timestamped_filename("console", "txt");
            write_export(&dir, &filename, &content)
        }
        PanelExportKind::AssetListing => {
            let content = export_asset_listing(&mounted_project);
            let filename = timestamped_filename("assets", "txt");
            write_export(&dir, &filename, &content)
        }
    };

    match result {
        Ok(path) => set_export_result(world, &format!("Exported to {}", path.display()), false),
        Err(e) => set_export_result(world, &e, true),
    }
}

fn set_export_result(world: &mut World, message: &str, is_error: bool) {
    if let Some(mut result) = world.get_resource_mut::<PanelExportResult>() {
        result.message = Some(message.to_string());
        result.is_error = is_error;
    }

    // Also log to console.
    if let Some(mut console) = world.get_resource_mut::<ConsoleLogStore>() {
        console.log(message.to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{
        CustomDocument, CustomDocumentEntry, EditorDocumentRoute, LoadedCustomDocument,
    };
    use serde_json::json;

    #[test]
    fn test_export_documents_serializes_matching_docs() {
        let loaded = LoadedCustomDocuments {
            manifest_path: None,
            manifest: None,
            documents: vec![
                LoadedCustomDocument {
                    entry: CustomDocumentEntry {
                        kind: "items".into(),
                        id: "sword".into(),
                        path: "items/sword.json".into(),
                        schema_version: 1,
                        editor_route: EditorDocumentRoute::Inspector,
                        tags: Vec::new(),
                    },
                    raw_json: String::new(),
                    document: Some(CustomDocument {
                        kind: "items".into(),
                        id: "sword".into(),
                        schema_version: 1,
                        label: Some("Iron Sword".into()),
                        tags: vec!["weapon".into()],
                        references: Vec::new(),
                        payload: json!({"damage": 10}),
                    }),
                    parse_error: None,
                    resolved_route: EditorDocumentRoute::Inspector,
                },
                LoadedCustomDocument {
                    entry: CustomDocumentEntry {
                        kind: "mobs".into(),
                        id: "goblin".into(),
                        path: "mobs/goblin.json".into(),
                        schema_version: 1,
                        editor_route: EditorDocumentRoute::Inspector,
                        tags: Vec::new(),
                    },
                    raw_json: String::new(),
                    document: Some(CustomDocument {
                        kind: "mobs".into(),
                        id: "goblin".into(),
                        schema_version: 1,
                        label: None,
                        tags: Vec::new(),
                        references: Vec::new(),
                        payload: json!({"hp": 30}),
                    }),
                    parse_error: None,
                    resolved_route: EditorDocumentRoute::Inspector,
                },
            ],
            issues: Vec::new(),
        };

        // Filter to items only.
        let json_str = export_documents(&loaded, Some("items"));
        let parsed: Vec<serde_json::Value> = serde_json::from_str(&json_str).unwrap();
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0]["id"], "sword");
        assert_eq!(parsed[0]["label"], "Iron Sword");

        // All documents.
        let json_str = export_documents(&loaded, None);
        let parsed: Vec<serde_json::Value> = serde_json::from_str(&json_str).unwrap();
        assert_eq!(parsed.len(), 2);
    }

    #[test]
    fn test_export_single_document_returns_pretty_json() {
        let loaded = LoadedCustomDocuments {
            manifest_path: None,
            manifest: None,
            documents: vec![LoadedCustomDocument {
                entry: CustomDocumentEntry {
                    kind: "items".into(),
                    id: "shield".into(),
                    path: "items/shield.json".into(),
                    schema_version: 1,
                    editor_route: EditorDocumentRoute::Inspector,
                    tags: Vec::new(),
                },
                raw_json: String::new(),
                document: Some(CustomDocument {
                    kind: "items".into(),
                    id: "shield".into(),
                    schema_version: 1,
                    label: Some("Tower Shield".into()),
                    tags: Vec::new(),
                    references: Vec::new(),
                    payload: json!({"defense": 5}),
                }),
                parse_error: None,
                resolved_route: EditorDocumentRoute::Inspector,
            }],
            issues: Vec::new(),
        };

        let result = export_single_document(&loaded, "items", "shield");
        assert!(result.is_some());
        let json_str = result.unwrap();
        assert!(json_str.contains("Tower Shield"));
        assert!(json_str.contains("defense"));
    }

    #[test]
    fn test_export_single_document_returns_none_for_missing() {
        let loaded = LoadedCustomDocuments::default();
        assert!(export_single_document(&loaded, "items", "nope").is_none());
    }

    #[test]
    fn test_export_console_joins_logs() {
        let mut console = ConsoleLogStore::default();
        console.logs.push("[00:00:01] hello".into());
        console.logs.push("[00:00:02] world".into());
        let result = export_console(&console);
        assert_eq!(result, "[00:00:01] hello\n[00:00:02] world");
    }

    #[test]
    fn test_timestamped_filename_has_expected_shape() {
        let name = timestamped_filename("docs_items", "json");
        assert!(name.starts_with("docs_items_"));
        assert!(name.ends_with(".json"));
    }

    #[test]
    fn test_write_export_creates_file() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("exports");
        let path = write_export(&dir, "test.json", "{\"ok\":true}").unwrap();
        assert!(path.is_file());
        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("ok"));
    }
}
