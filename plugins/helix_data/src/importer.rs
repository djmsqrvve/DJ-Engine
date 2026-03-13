use crate::{
    safe_document_reference, HELIX_ABILITY_KIND, HELIX_IMPORT_PREVIEW_ID, HELIX_ITEM_KIND,
    HELIX_MOB_KIND,
};
use dj_engine::data::loader::{
    save_custom_data_manifest, save_project, save_project_structure, DataError,
};
use dj_engine::data::{
    load_project, CustomDataManifest, CustomDocument, CustomDocumentEntry, DocumentLink,
    DocumentRef, EditorDocumentRoute, PreviewProfilePayload, Project,
};
use dj_engine::project_mount::normalize_project_path;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

const ABILITY_BUCKETS: &[&str] = &[
    "ability/abilitys",
    "Uncategorized/spells",
    "Uncategorized/abilitys",
    "GameData/abilitys",
];

const ITEM_BUCKETS: &[&str] = &[
    "weapon/items",
    "GameData/armors",
    "GameData/consumables",
    "consumable_potion/potions",
    "item/items",
];

const MOB_BUCKETS: &[&str] = &[
    "GameData/beasts",
    "GameData/undeads",
    "GameData/demons",
    "GameData/dragonkins",
    "GameData/elementals",
    "GameData/aberrations",
    "GameData/humanoids",
];

const TEMPLATE_PROJECT_JSON: &str = include_str!("../template/project.json");
const TEMPLATE_SCENE_JSON: &str = include_str!("../template/scenes/helix_preview.json");
const TEMPLATE_STORY_GRAPH_JSON: &str = include_str!("../template/story_graphs/helix_intro.json");
const TEMPLATE_REGISTRY_JSON: &str = include_str!("../template/data/registry.json");
const TEMPLATE_PREVIEW_PROFILE_JSON: &str =
    include_str!("../template/data/preview_profiles/helix_import_preview.json");

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HelixImportCliOptions {
    pub helix_dist: PathBuf,
    pub project_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HelixImportSummary {
    pub project_root: PathBuf,
    pub project_manifest_path: PathBuf,
    pub registry_path: PathBuf,
    pub abilities: usize,
    pub items: usize,
    pub mobs: usize,
    pub skipped_files: usize,
    pub preview_profile_id: String,
}

#[derive(Debug, Error)]
pub enum HelixImportError {
    #[error("Missing required CLI option {0}.")]
    MissingCliOption(&'static str),

    #[error("Helix dist path '{0}' was not found or is not a directory.")]
    InvalidHelixDist(String),

    #[error("Duplicate Helix id '{id}' found for kind '{kind}' in '{first}' and '{second}'.")]
    DuplicateId {
        kind: String,
        id: String,
        first: String,
        second: String,
    },

    #[error(transparent)]
    Data(#[from] DataError),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum ImportedKind {
    Ability,
    Item,
    Mob,
}

impl ImportedKind {
    fn document_kind(self) -> &'static str {
        match self {
            Self::Ability => HELIX_ABILITY_KIND,
            Self::Item => HELIX_ITEM_KIND,
            Self::Mob => HELIX_MOB_KIND,
        }
    }

    fn output_root(self) -> &'static str {
        self.document_kind()
    }

    fn source_buckets(self) -> &'static [&'static str] {
        match self {
            Self::Ability => ABILITY_BUCKETS,
            Self::Item => ITEM_BUCKETS,
            Self::Mob => MOB_BUCKETS,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct DiscoveredDocument {
    kind: ImportedKind,
    id: String,
    payload: Value,
    source_bucket: String,
    source_path: PathBuf,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
struct ImportedIdSets {
    abilities: BTreeSet<String>,
    items: BTreeSet<String>,
    mobs: BTreeSet<String>,
}

#[derive(Debug, Clone, PartialEq)]
struct GeneratedDocument {
    entry: CustomDocumentEntry,
    envelope: CustomDocument<Value>,
}

pub fn parse_helix_import_cli_args(
    args: impl IntoIterator<Item = String>,
) -> Result<HelixImportCliOptions, HelixImportError> {
    let args: Vec<String> = args.into_iter().collect();
    let mut helix_dist = None;
    let mut project_path = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--helix-dist" => {
                if i + 1 < args.len() {
                    helix_dist = Some(PathBuf::from(&args[i + 1]));
                    i += 1;
                }
            }
            "--project" => {
                if i + 1 < args.len() {
                    project_path = Some(PathBuf::from(&args[i + 1]));
                    i += 1;
                }
            }
            _ => {}
        }
        i += 1;
    }

    Ok(HelixImportCliOptions {
        helix_dist: helix_dist.ok_or(HelixImportError::MissingCliOption("--helix-dist"))?,
        project_path: project_path.ok_or(HelixImportError::MissingCliOption("--project"))?,
    })
}

pub fn import_helix_project(
    helix_dist: &Path,
    project_path: &Path,
) -> Result<HelixImportSummary, HelixImportError> {
    if !helix_dist.is_dir() {
        return Err(HelixImportError::InvalidHelixDist(
            helix_dist.display().to_string(),
        ));
    }

    let (project_root, manifest_path) = normalize_project_path(project_path)?;
    let mut template_applied = false;
    if !manifest_path.exists() {
        initialize_project_from_template(&project_root, &manifest_path)?;
        template_applied = true;
    }

    let mut project = load_project(&manifest_path)?;
    if template_applied {
        project.id = Project::new(project.name.clone()).id;
        if let Some(name) = project_root.file_name().and_then(OsStr::to_str) {
            project.name = name.to_string();
        }
        save_project_structure(&project, &project_root)?;
        save_project(&project, &manifest_path)?;
    }

    let (mut discovered, skipped_files) = discover_source_documents(helix_dist)?;
    discovered.sort_by(|left, right| {
        left.kind
            .cmp(&right.kind)
            .then_with(|| left.id.cmp(&right.id))
            .then_with(|| left.source_bucket.cmp(&right.source_bucket))
    });
    ensure_no_duplicate_ids(&discovered)?;

    let imported_ids = imported_id_sets(&discovered);
    let mut generated = discovered
        .iter()
        .map(|document| build_generated_document(document, &imported_ids))
        .collect::<Result<Vec<_>, HelixImportError>>()?;

    let preview_profile = build_preview_profile_document(&project, &imported_ids);
    generated.push(preview_profile);
    generated.sort_by(|left, right| {
        left.entry
            .kind
            .cmp(&right.entry.kind)
            .then_with(|| left.entry.id.cmp(&right.entry.id))
    });

    write_generated_project_data(&project_root, &project, &generated)?;

    let registry_path = project_root
        .join(&project.settings.paths.data)
        .join("registry.json");

    Ok(HelixImportSummary {
        project_root,
        project_manifest_path: manifest_path,
        registry_path,
        abilities: imported_ids.abilities.len(),
        items: imported_ids.items.len(),
        mobs: imported_ids.mobs.len(),
        skipped_files,
        preview_profile_id: HELIX_IMPORT_PREVIEW_ID.to_string(),
    })
}

fn discover_source_documents(
    helix_dist: &Path,
) -> Result<(Vec<DiscoveredDocument>, usize), HelixImportError> {
    let mut documents = Vec::new();
    let mut skipped_files = 0;

    for kind in [ImportedKind::Ability, ImportedKind::Item, ImportedKind::Mob] {
        for bucket in kind.source_buckets() {
            let bucket_path = helix_dist.join(bucket);
            if !bucket_path.is_dir() {
                continue;
            }

            let mut entries: Vec<_> = fs::read_dir(&bucket_path)?
                .filter_map(Result::ok)
                .map(|entry| entry.path())
                .filter(|path| path.extension().and_then(OsStr::to_str) == Some("json"))
                .collect();
            entries.sort();

            for path in entries {
                let payload: Value = serde_json::from_str(&fs::read_to_string(&path)?)?;
                let Some(object) = payload.as_object() else {
                    skipped_files += 1;
                    continue;
                };
                let Some(id) = object.get("id").and_then(Value::as_str) else {
                    skipped_files += 1;
                    continue;
                };

                documents.push(DiscoveredDocument {
                    kind,
                    id: id.to_string(),
                    payload,
                    source_bucket: (*bucket).to_string(),
                    source_path: path,
                });
            }
        }
    }

    Ok((documents, skipped_files))
}

fn ensure_no_duplicate_ids(documents: &[DiscoveredDocument]) -> Result<(), HelixImportError> {
    let mut seen = BTreeMap::<(&'static str, &str), &Path>::new();
    for document in documents {
        let key = (document.kind.document_kind(), document.id.as_str());
        if let Some(existing) = seen.insert(key, document.source_path.as_path()) {
            return Err(HelixImportError::DuplicateId {
                kind: document.kind.document_kind().to_string(),
                id: document.id.clone(),
                first: existing.display().to_string(),
                second: document.source_path.display().to_string(),
            });
        }
    }

    Ok(())
}

fn imported_id_sets(documents: &[DiscoveredDocument]) -> ImportedIdSets {
    let mut ids = ImportedIdSets::default();
    for document in documents {
        match document.kind {
            ImportedKind::Ability => {
                ids.abilities.insert(document.id.clone());
            }
            ImportedKind::Item => {
                ids.items.insert(document.id.clone());
            }
            ImportedKind::Mob => {
                ids.mobs.insert(document.id.clone());
            }
        }
    }
    ids
}

fn build_generated_document(
    document: &DiscoveredDocument,
    imported_ids: &ImportedIdSets,
) -> Result<GeneratedDocument, HelixImportError> {
    let output_path =
        output_path_for_document(document.kind, &document.source_bucket, &document.id);
    let tags = collect_envelope_tags(&document.payload, &document.source_bucket);
    let label = extract_label(&document.payload);
    let references = derive_safe_references(document.kind, &document.payload, imported_ids);

    Ok(GeneratedDocument {
        entry: CustomDocumentEntry {
            kind: document.kind.document_kind().to_string(),
            id: document.id.clone(),
            path: output_path.to_string_lossy().into_owned(),
            schema_version: 1,
            editor_route: EditorDocumentRoute::Inspector,
            tags: tags.clone(),
        },
        envelope: CustomDocument {
            kind: document.kind.document_kind().to_string(),
            id: document.id.clone(),
            schema_version: 1,
            label,
            tags,
            references,
            payload: document.payload.clone(),
        },
    })
}

fn build_preview_profile_document(
    project: &Project,
    imported_ids: &ImportedIdSets,
) -> GeneratedDocument {
    let mut document_refs = Vec::new();
    let mut references = Vec::new();

    for (index, (kind, id)) in [
        (HELIX_ABILITY_KIND, imported_ids.abilities.iter().next()),
        (HELIX_ITEM_KIND, imported_ids.items.iter().next()),
        (HELIX_MOB_KIND, imported_ids.mobs.iter().next()),
    ]
    .into_iter()
    .enumerate()
    {
        let Some(id) = id else {
            continue;
        };
        document_refs.push(DocumentRef {
            kind: kind.to_string(),
            id: id.clone(),
        });
        references.push(safe_document_reference(
            format!("payload.document_refs[{index}]"),
            kind,
            id,
        ));
    }

    let scene_id = project
        .settings
        .startup
        .default_scene_id
        .clone()
        .or_else(|| project.scenes.first().map(|scene| scene.id.clone()));
    let story_graph_id = project
        .settings
        .startup
        .default_story_graph_id
        .clone()
        .or_else(|| project.story_graphs.first().map(|graph| graph.id.clone()));

    GeneratedDocument {
        entry: CustomDocumentEntry {
            kind: "preview_profiles".into(),
            id: HELIX_IMPORT_PREVIEW_ID.into(),
            path: format!("preview_profiles/{HELIX_IMPORT_PREVIEW_ID}.json"),
            schema_version: 1,
            editor_route: EditorDocumentRoute::Inspector,
            tags: vec!["source:helix_import".into()],
        },
        envelope: CustomDocument {
            kind: "preview_profiles".into(),
            id: HELIX_IMPORT_PREVIEW_ID.into(),
            schema_version: 1,
            label: Some("Helix Import Preview".into()),
            tags: vec!["source:helix_import".into()],
            references,
            payload: serde_json::to_value(PreviewProfilePayload {
                scene_id,
                story_graph_id,
                document_refs,
            })
            .expect("preview profile payload should serialize"),
        },
    }
}

fn write_generated_project_data(
    project_root: &Path,
    project: &Project,
    generated: &[GeneratedDocument],
) -> Result<(), HelixImportError> {
    let data_root = project_root.join(&project.settings.paths.data);
    fs::create_dir_all(&data_root)?;

    for generated_root in [HELIX_ABILITY_KIND, HELIX_ITEM_KIND, HELIX_MOB_KIND] {
        let root = data_root.join(generated_root);
        if root.exists() {
            fs::remove_dir_all(&root)?;
        }
    }

    let preview_profile_path = data_root
        .join("preview_profiles")
        .join(format!("{HELIX_IMPORT_PREVIEW_ID}.json"));
    if preview_profile_path.exists() {
        fs::remove_file(&preview_profile_path)?;
    }

    let manifest = CustomDataManifest {
        version: 1,
        documents: generated
            .iter()
            .map(|document| document.entry.clone())
            .collect(),
    };

    for document in generated {
        let path = data_root.join(&document.entry.path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, serde_json::to_string_pretty(&document.envelope)?)?;
    }

    save_custom_data_manifest(&manifest, &data_root.join("registry.json"))?;
    Ok(())
}

fn initialize_project_from_template(
    project_root: &Path,
    manifest_path: &Path,
) -> Result<(), HelixImportError> {
    let template_files = [
        ("project.json", TEMPLATE_PROJECT_JSON),
        ("scenes/helix_preview.json", TEMPLATE_SCENE_JSON),
        ("story_graphs/helix_intro.json", TEMPLATE_STORY_GRAPH_JSON),
        ("data/registry.json", TEMPLATE_REGISTRY_JSON),
        (
            "data/preview_profiles/helix_import_preview.json",
            TEMPLATE_PREVIEW_PROFILE_JSON,
        ),
    ];

    fs::create_dir_all(project_root)?;
    for (relative_path, contents) in template_files {
        let path = project_root.join(relative_path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, contents)?;
    }

    let project = load_project(manifest_path)?;
    save_project_structure(&project, project_root)?;
    Ok(())
}

fn output_path_for_document(kind: ImportedKind, source_bucket: &str, id: &str) -> PathBuf {
    PathBuf::from(kind.output_root())
        .join(bucket_slug(source_bucket))
        .join(format!("{id}.json"))
}

fn bucket_slug(source_bucket: &str) -> String {
    let mut slug = String::new();
    let mut previous_was_separator = false;

    for ch in source_bucket.chars().flat_map(char::to_lowercase) {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch);
            previous_was_separator = false;
        } else if !previous_was_separator {
            slug.push('_');
            previous_was_separator = true;
        }
    }

    slug.trim_matches('_').to_string()
}

fn collect_envelope_tags(payload: &Value, source_bucket: &str) -> Vec<String> {
    let mut tags = BTreeSet::new();
    if let Some(payload_tags) = payload.get("tags").and_then(Value::as_array) {
        for tag in payload_tags.iter().filter_map(Value::as_str) {
            tags.insert(tag.to_string());
        }
    }
    tags.insert(format!("source_bucket:{source_bucket}"));
    tags.into_iter().collect()
}

fn extract_label(payload: &Value) -> Option<String> {
    match payload.get("name") {
        Some(Value::String(label)) if !label.trim().is_empty() => Some(label.clone()),
        Some(Value::Object(map)) => map
            .get("en")
            .and_then(Value::as_str)
            .map(str::to_string)
            .or_else(|| map.values().find_map(Value::as_str).map(str::to_string)),
        _ => None,
    }
}

fn derive_safe_references(
    kind: ImportedKind,
    payload: &Value,
    imported_ids: &ImportedIdSets,
) -> Vec<DocumentLink> {
    if kind != ImportedKind::Mob {
        return Vec::new();
    }

    let mut references = Vec::new();

    if let Some(abilities) = payload.get("abilities").and_then(Value::as_array) {
        for (index, ability_id) in abilities.iter().enumerate() {
            let Some(ability_id) = ability_id.as_str() else {
                continue;
            };
            if imported_ids.abilities.contains(ability_id) {
                references.push(safe_document_reference(
                    format!("payload.abilities[{index}]"),
                    HELIX_ABILITY_KIND,
                    ability_id,
                ));
            }
        }
    }

    if let Some(loot) = payload.get("loot").and_then(Value::as_array) {
        for (index, loot_entry) in loot.iter().enumerate() {
            let Some(item_id) = loot_entry.get("item").and_then(Value::as_str) else {
                continue;
            };
            if imported_ids.items.contains(item_id) {
                references.push(safe_document_reference(
                    format!("payload.loot[{index}].item"),
                    HELIX_ITEM_KIND,
                    item_id,
                ));
            }
        }
    }

    references
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tempfile::tempdir;

    #[test]
    fn test_bucket_slug_normalizes_paths() {
        assert_eq!(bucket_slug("Uncategorized/spells"), "uncategorized_spells");
        assert_eq!(bucket_slug("ability/abilitys"), "ability_abilitys");
    }

    #[test]
    fn test_collect_envelope_tags_merges_payload_tags_with_source_bucket() {
        let tags = collect_envelope_tags(
            &json!({
                "id": "fireball",
                "tags": ["fire", "spell"]
            }),
            "Uncategorized/spells",
        );

        assert!(tags.contains(&"fire".to_string()));
        assert!(tags.contains(&"spell".to_string()));
        assert!(tags.contains(&"source_bucket:Uncategorized/spells".to_string()));
    }

    #[test]
    fn test_derive_safe_references_only_links_imported_targets() {
        let refs = derive_safe_references(
            ImportedKind::Mob,
            &json!({
                "id": "felguard",
                "abilities": ["fireball", "missing_spell"],
                "loot": [
                    { "item": "dagger" },
                    { "item": "missing_item" }
                ]
            }),
            &ImportedIdSets {
                abilities: BTreeSet::from(["fireball".to_string()]),
                items: BTreeSet::from(["dagger".to_string()]),
                mobs: BTreeSet::new(),
            },
        );

        assert_eq!(refs.len(), 2);
        assert_eq!(refs[0].field_path, "payload.abilities[0]");
        assert_eq!(refs[1].field_path, "payload.loot[0].item");
    }

    #[test]
    fn test_duplicate_ids_are_rejected() {
        let duplicate = vec![
            DiscoveredDocument {
                kind: ImportedKind::Ability,
                id: "fireball".into(),
                payload: json!({ "id": "fireball" }),
                source_bucket: "ability/abilitys".into(),
                source_path: PathBuf::from("/tmp/a.json"),
            },
            DiscoveredDocument {
                kind: ImportedKind::Ability,
                id: "fireball".into(),
                payload: json!({ "id": "fireball" }),
                source_bucket: "Uncategorized/spells".into(),
                source_path: PathBuf::from("/tmp/b.json"),
            },
        ];

        let error = ensure_no_duplicate_ids(&duplicate).unwrap_err();
        assert!(matches!(error, HelixImportError::DuplicateId { .. }));
    }

    #[test]
    fn test_discover_source_documents_skips_metadata_json_without_ids() {
        let temp_dir = tempdir().unwrap();
        let bucket = temp_dir.path().join("GameData/armors");
        fs::create_dir_all(&bucket).unwrap();
        fs::write(
            bucket.join("armor.json"),
            serde_json::to_string_pretty(
                &json!({ "id": "linen_tunic", "name": { "en": "Linen Tunic" } }),
            )
            .unwrap(),
        )
        .unwrap();
        fs::write(
            bucket.join("stats.json"),
            serde_json::to_string_pretty(&json!({ "armor": 4 })).unwrap(),
        )
        .unwrap();

        let (documents, skipped) = discover_source_documents(temp_dir.path()).unwrap();
        assert_eq!(documents.len(), 1);
        assert_eq!(documents[0].id, "linen_tunic");
        assert_eq!(skipped, 1);
    }

    #[test]
    fn test_output_path_preserves_kind_and_source_bucket() {
        assert_eq!(
            output_path_for_document(ImportedKind::Ability, "Uncategorized/spells", "fireball"),
            PathBuf::from("helix_abilities/uncategorized_spells/fireball.json")
        );
    }
}
