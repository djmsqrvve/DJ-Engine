use crate::data::loader::DataError;
use crate::data::project::Project;
use crate::project_mount::MountedProject;
use bevy::prelude::*;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::marker::PhantomData;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use thiserror::Error;

pub type DocumentKindId = String;
pub type DocumentId = String;

fn default_manifest_version() -> u32 {
    1
}

fn default_schema_version() -> u32 {
    1
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EditorDocumentRoute {
    #[default]
    Inspector,
    Table,
    Graph,
    CustomPanel,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocumentRef {
    pub kind: DocumentKindId,
    pub id: DocumentId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DocumentLinkTarget {
    Document {
        kind: DocumentKindId,
        id: DocumentId,
    },
    Scene {
        id: String,
    },
    StoryGraph {
        id: String,
    },
    Asset {
        path: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocumentLink {
    pub field_path: String,
    #[serde(flatten)]
    pub target: DocumentLinkTarget,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CustomDocument<T> {
    pub kind: DocumentKindId,
    pub id: DocumentId,
    #[serde(default = "default_schema_version")]
    pub schema_version: u32,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub references: Vec<DocumentLink>,
    pub payload: T,
}

pub type CustomDocumentEnvelope = CustomDocument<Value>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CustomDocumentEntry {
    pub kind: DocumentKindId,
    pub id: DocumentId,
    pub path: String,
    #[serde(default = "default_schema_version")]
    pub schema_version: u32,
    #[serde(default)]
    pub editor_route: EditorDocumentRoute,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CustomDataManifest {
    #[serde(default = "default_manifest_version")]
    pub version: u32,
    #[serde(default)]
    pub documents: Vec<CustomDocumentEntry>,
}

impl Default for CustomDataManifest {
    fn default() -> Self {
        Self {
            version: default_manifest_version(),
            documents: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValidationSeverity {
    #[default]
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidationIssue {
    pub severity: ValidationSeverity,
    pub code: String,
    #[serde(default)]
    pub source_kind: Option<DocumentKindId>,
    #[serde(default)]
    pub source_id: Option<DocumentId>,
    #[serde(default)]
    pub field_path: Option<String>,
    pub message: String,
    #[serde(default)]
    pub related_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LoadedCustomDocument {
    pub entry: CustomDocumentEntry,
    #[serde(default)]
    pub raw_json: String,
    #[serde(default)]
    pub document: Option<CustomDocumentEnvelope>,
    #[serde(default)]
    pub parse_error: Option<String>,
    #[serde(default)]
    pub resolved_route: EditorDocumentRoute,
}

impl LoadedCustomDocument {
    pub fn id(&self) -> &str {
        &self.entry.id
    }

    pub fn kind(&self) -> &str {
        &self.entry.kind
    }
}

#[derive(Resource, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LoadedCustomDocuments {
    #[serde(default)]
    pub manifest_path: Option<PathBuf>,
    #[serde(default)]
    pub manifest: Option<CustomDataManifest>,
    #[serde(default)]
    pub documents: Vec<LoadedCustomDocument>,
    #[serde(default)]
    pub issues: Vec<ValidationIssue>,
}

impl LoadedCustomDocuments {
    pub fn get(&self, kind: &str, id: &str) -> Option<&LoadedCustomDocument> {
        self.documents
            .iter()
            .find(|document| document.entry.kind == kind && document.entry.id == id)
    }

    pub fn get_mut(&mut self, kind: &str, id: &str) -> Option<&mut LoadedCustomDocument> {
        self.documents
            .iter_mut()
            .find(|document| document.entry.kind == kind && document.entry.id == id)
    }

    pub fn get_typed<T: DeserializeOwned>(
        &self,
        kind: &str,
        id: &str,
    ) -> Result<Option<CustomDocument<T>>, serde_json::Error> {
        let Some(document) = self
            .get(kind, id)
            .and_then(|document| document.document.clone())
        else {
            return Ok(None);
        };

        serde_json::from_value(serde_json::to_value(document)?).map(Some)
    }

    pub fn issues_for(&self, kind: &str, id: &str) -> Vec<&ValidationIssue> {
        self.issues
            .iter()
            .filter(|issue| {
                issue.source_kind.as_deref() == Some(kind) && issue.source_id.as_deref() == Some(id)
            })
            .collect()
    }

    pub fn available_kinds(&self) -> Vec<String> {
        let mut kinds: BTreeSet<String> = self
            .documents
            .iter()
            .map(|document| document.entry.kind.clone())
            .collect();
        kinds.insert("all".to_string());
        kinds.into_iter().collect()
    }

    pub fn has_blocking_errors(&self) -> bool {
        self.issues
            .iter()
            .any(|issue| issue.severity == ValidationSeverity::Error)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum CustomDocumentScalarValue {
    String(String),
    Number(serde_json::Number),
    Bool(bool),
}

impl CustomDocumentScalarValue {
    pub fn from_json_value(value: &Value) -> Option<Self> {
        match value {
            Value::String(value) => Some(Self::String(value.clone())),
            Value::Number(value) => Some(Self::Number(value.clone())),
            Value::Bool(value) => Some(Self::Bool(*value)),
            _ => None,
        }
    }

    pub fn kind_name(&self) -> &'static str {
        match self {
            Self::String(_) => "string",
            Self::Number(_) => "number",
            Self::Bool(_) => "bool",
        }
    }

    pub fn to_json_value(&self) -> Value {
        match self {
            Self::String(value) => Value::String(value.clone()),
            Self::Number(value) => Value::Number(value.clone()),
            Self::Bool(value) => Value::Bool(*value),
        }
    }
}

#[derive(Debug, Error)]
pub enum CustomDocumentUpdateError {
    #[error("Payload field path '{0}' must point to a top-level payload field.")]
    InvalidFieldPath(String),

    #[error("Custom document payload must remain a JSON object for table editing.")]
    PayloadNotObject,

    #[error("Payload field '{0}' was not found.")]
    MissingField(String),

    #[error("Payload field '{field}' is not a string, number, or bool scalar.")]
    NonScalarField { field: String },

    #[error("Payload field '{field}' expected a {expected} value but received {actual}.")]
    ScalarTypeMismatch {
        field: String,
        expected: &'static str,
        actual: &'static str,
    },

    #[error("Nested path '{path}' could not be resolved in the document payload.")]
    NestedPathNotFound { path: String },

    #[error("Index {index} is out of bounds (array length {length}) at path '{path}'.")]
    IndexOutOfBounds {
        path: String,
        index: usize,
        length: usize,
    },

    #[error("Value at '{path}' is {expected} but replacement is {actual}.")]
    NestedTypeMismatch {
        path: String,
        expected: &'static str,
        actual: &'static str,
    },

    #[error("String value exceeds maximum length of {max_length} characters for field '{field}'.")]
    StringTooLong { field: String, max_length: usize },

    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

pub type CustomDocumentValidator = Arc<
    dyn Fn(&LoadedCustomDocument, &LoadedCustomDocuments, &Project, &mut Vec<ValidationIssue>)
        + Send
        + Sync,
>;

#[derive(Clone)]
pub struct RegisteredCustomDocumentKind {
    pub kind: DocumentKindId,
    pub schema_version: u32,
    pub editor_route: EditorDocumentRoute,
    pub schema_json: String,
    pub schema_is_valid_json: bool,
    pub supports_runtime_preview: bool,
    pub rust_type_name: &'static str,
    pub validator: Option<CustomDocumentValidator>,
}

impl fmt::Debug for RegisteredCustomDocumentKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RegisteredCustomDocumentKind")
            .field("kind", &self.kind)
            .field("schema_version", &self.schema_version)
            .field("editor_route", &self.editor_route)
            .field("schema_is_valid_json", &self.schema_is_valid_json)
            .field("supports_runtime_preview", &self.supports_runtime_preview)
            .field("rust_type_name", &self.rust_type_name)
            .finish()
    }
}

#[derive(Resource, Default, Debug, Clone)]
pub struct CustomDocumentRegistry {
    kinds: BTreeMap<DocumentKindId, RegisteredCustomDocumentKind>,
}

impl CustomDocumentRegistry {
    pub fn register_kind(&mut self, registration: RegisteredCustomDocumentKind) {
        self.kinds.insert(registration.kind.clone(), registration);
    }

    pub fn get(&self, kind: &str) -> Option<&RegisteredCustomDocumentKind> {
        self.kinds.get(kind)
    }

    pub fn all(&self) -> impl Iterator<Item = &RegisteredCustomDocumentKind> {
        self.kinds.values()
    }
}

pub type TypedDocumentValidator<T> =
    fn(&CustomDocument<T>, &LoadedCustomDocuments, &Project, &mut Vec<ValidationIssue>);

pub struct CustomDocumentRegistration<T> {
    kind: &'static str,
    schema_version: u32,
    editor_route: EditorDocumentRoute,
    schema_json: &'static str,
    supports_runtime_preview: bool,
    validator: Option<TypedDocumentValidator<T>>,
    _marker: PhantomData<T>,
}

impl<T> CustomDocumentRegistration<T> {
    pub fn new(
        kind: &'static str,
        schema_version: u32,
        editor_route: EditorDocumentRoute,
        schema_json: &'static str,
    ) -> Self {
        Self {
            kind,
            schema_version,
            editor_route,
            schema_json,
            supports_runtime_preview: false,
            validator: None,
            _marker: PhantomData,
        }
    }

    pub fn with_runtime_preview(mut self, supports_runtime_preview: bool) -> Self {
        self.supports_runtime_preview = supports_runtime_preview;
        self
    }

    pub fn with_validator(mut self, validator: TypedDocumentValidator<T>) -> Self {
        self.validator = Some(validator);
        self
    }
}

pub trait AppCustomDocumentExt {
    fn register_custom_document<T>(
        &mut self,
        registration: CustomDocumentRegistration<T>,
    ) -> &mut Self
    where
        T: Serialize + DeserializeOwned + Send + Sync + 'static;
}

impl AppCustomDocumentExt for App {
    fn register_custom_document<T>(
        &mut self,
        registration: CustomDocumentRegistration<T>,
    ) -> &mut Self
    where
        T: Serialize + DeserializeOwned + Send + Sync + 'static,
    {
        self.init_resource::<CustomDocumentRegistry>();
        self.init_resource::<LoadedCustomDocuments>();

        let validator = registration.validator.map(|typed_validator| {
            Arc::new(
                move |document: &LoadedCustomDocument,
                      loaded: &LoadedCustomDocuments,
                      project: &Project,
                      issues: &mut Vec<ValidationIssue>| {
                    let Some(envelope) = document.document.clone() else {
                        return;
                    };

                    let value = match serde_json::to_value(envelope) {
                        Ok(value) => value,
                        Err(error) => {
                            issues.push(ValidationIssue {
                                severity: ValidationSeverity::Error,
                                code: "typed_validation_serialization_failed".into(),
                                source_kind: Some(document.entry.kind.clone()),
                                source_id: Some(document.entry.id.clone()),
                                field_path: None,
                                message: format!(
                                    "Failed to prepare '{}' for typed validation: {}",
                                    document.entry.id, error
                                ),
                                related_refs: Vec::new(),
                            });
                            return;
                        }
                    };

                    match serde_json::from_value::<CustomDocument<T>>(value) {
                        Ok(typed_document) => {
                            typed_validator(&typed_document, loaded, project, issues)
                        }
                        Err(error) => issues.push(ValidationIssue {
                            severity: ValidationSeverity::Error,
                            code: "typed_validation_deserialize_failed".into(),
                            source_kind: Some(document.entry.kind.clone()),
                            source_id: Some(document.entry.id.clone()),
                            field_path: None,
                            message: format!(
                                "Failed to deserialize '{}' into registered Rust type {}: {}",
                                document.entry.id,
                                std::any::type_name::<T>(),
                                error
                            ),
                            related_refs: Vec::new(),
                        }),
                    }
                },
            ) as CustomDocumentValidator
        });

        let schema_is_valid_json = serde_json::from_str::<Value>(registration.schema_json).is_ok();
        let registered = RegisteredCustomDocumentKind {
            kind: registration.kind.to_string(),
            schema_version: registration.schema_version,
            editor_route: registration.editor_route,
            schema_json: registration.schema_json.to_string(),
            schema_is_valid_json,
            supports_runtime_preview: registration.supports_runtime_preview,
            rust_type_name: std::any::type_name::<T>(),
            validator,
        };

        self.world_mut()
            .resource_mut::<CustomDocumentRegistry>()
            .register_kind(registered);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PreviewProfilePayload {
    #[serde(default)]
    pub scene_id: Option<String>,
    #[serde(default)]
    pub story_graph_id: Option<String>,
    #[serde(default)]
    pub document_refs: Vec<DocumentRef>,
}

const PREVIEW_PROFILE_SCHEMA_JSON: &str = r#"{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "DJ Engine Preview Profile",
  "type": "object",
  "required": ["kind", "id", "payload"],
  "properties": {
    "kind": { "const": "preview_profiles" },
    "id": { "type": "string", "pattern": "^[a-z][a-z0-9_]*$" },
    "schema_version": { "type": "integer", "minimum": 1 },
    "label": { "type": "string" },
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
      "properties": {
        "scene_id": { "type": ["string", "null"] },
        "story_graph_id": { "type": ["string", "null"] },
        "document_refs": {
          "type": "array",
          "items": {
            "type": "object",
            "required": ["kind", "id"],
            "properties": {
              "kind": { "type": "string" },
              "id": { "type": "string" }
            },
            "additionalProperties": false
          }
        }
      },
      "additionalProperties": false
    }
  },
  "additionalProperties": false
}"#;

fn preview_profile_validator(
    document: &CustomDocument<PreviewProfilePayload>,
    loaded: &LoadedCustomDocuments,
    project: &Project,
    issues: &mut Vec<ValidationIssue>,
) {
    if let Some(scene_id) = document.payload.scene_id.as_deref() {
        if project.find_scene(scene_id).is_none() {
            issues.push(ValidationIssue {
                severity: ValidationSeverity::Error,
                code: "preview_profile_missing_scene".into(),
                source_kind: Some(document.kind.clone()),
                source_id: Some(document.id.clone()),
                field_path: Some("payload.scene_id".into()),
                message: format!("Preview profile references unknown scene '{}'.", scene_id),
                related_refs: vec![format!("scene:{scene_id}")],
            });
        }
    }

    if let Some(story_graph_id) = document.payload.story_graph_id.as_deref() {
        if project.find_story_graph(story_graph_id).is_none() {
            issues.push(ValidationIssue {
                severity: ValidationSeverity::Error,
                code: "preview_profile_missing_story_graph".into(),
                source_kind: Some(document.kind.clone()),
                source_id: Some(document.id.clone()),
                field_path: Some("payload.story_graph_id".into()),
                message: format!(
                    "Preview profile references unknown story graph '{}'.",
                    story_graph_id
                ),
                related_refs: vec![format!("story_graph:{story_graph_id}")],
            });
        }
    }

    for document_ref in &document.payload.document_refs {
        if loaded.get(&document_ref.kind, &document_ref.id).is_none() {
            issues.push(ValidationIssue {
                severity: ValidationSeverity::Error,
                code: "preview_profile_missing_document".into(),
                source_kind: Some(document.kind.clone()),
                source_id: Some(document.id.clone()),
                field_path: Some("payload.document_refs".into()),
                message: format!(
                    "Preview profile references unknown custom document '{}:{}' .",
                    document_ref.kind, document_ref.id
                ),
                related_refs: vec![format!("{}:{}", document_ref.kind, document_ref.id)],
            });
        }
    }
}

pub struct DJDataRegistryPlugin;

impl Plugin for DJDataRegistryPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CustomDocumentRegistry>()
            .init_resource::<LoadedCustomDocuments>()
            .register_custom_document(
                CustomDocumentRegistration::<PreviewProfilePayload>::new(
                    "preview_profiles",
                    1,
                    EditorDocumentRoute::Inspector,
                    PREVIEW_PROFILE_SCHEMA_JSON,
                )
                .with_runtime_preview(true)
                .with_validator(preview_profile_validator),
            );
    }
}

pub fn default_custom_data_manifest_path(project_root: &Path, project: &Project) -> PathBuf {
    project_root
        .join(&project.settings.paths.data)
        .join("registry.json")
}

fn default_data_root(project_root: &Path, project: &Project) -> PathBuf {
    project_root.join(&project.settings.paths.data)
}

fn sort_loaded_documents(documents: &mut [LoadedCustomDocument]) {
    documents.sort_by(|left, right| {
        left.entry
            .kind
            .cmp(&right.entry.kind)
            .then_with(|| left.entry.id.cmp(&right.entry.id))
    });
}

pub fn resolve_default_preview_profile(
    loaded_documents: &LoadedCustomDocuments,
) -> Option<CustomDocument<PreviewProfilePayload>> {
    let default_profile = loaded_documents
        .get_typed::<PreviewProfilePayload>("preview_profiles", "default_preview")
        .ok()
        .flatten();
    if default_profile.is_some() {
        return default_profile;
    }

    let preview_ids: Vec<_> = loaded_documents
        .documents
        .iter()
        .filter(|document| document.entry.kind == "preview_profiles")
        .collect();
    if preview_ids.len() != 1 {
        return None;
    }

    loaded_documents
        .get_typed::<PreviewProfilePayload>(&preview_ids[0].entry.kind, &preview_ids[0].entry.id)
        .ok()
        .flatten()
}

pub fn resolve_preview_profile_by_id(
    loaded_documents: &LoadedCustomDocuments,
    profile_id: &str,
) -> Option<CustomDocument<PreviewProfilePayload>> {
    loaded_documents
        .get_typed::<PreviewProfilePayload>("preview_profiles", profile_id)
        .ok()
        .flatten()
}

pub fn load_custom_documents_from_project(
    mounted_project: &MountedProject,
    registry: &CustomDocumentRegistry,
) -> LoadedCustomDocuments {
    let Some(project_root) = mounted_project.root_path.as_ref() else {
        return LoadedCustomDocuments::default();
    };
    let Some(project) = mounted_project.project.as_ref() else {
        return LoadedCustomDocuments::default();
    };

    let manifest_path = default_custom_data_manifest_path(project_root, project);
    if !manifest_path.exists() {
        return LoadedCustomDocuments {
            manifest_path: Some(manifest_path),
            manifest: Some(CustomDataManifest::default()),
            documents: Vec::new(),
            issues: Vec::new(),
        };
    }

    let mut loaded = LoadedCustomDocuments {
        manifest_path: Some(manifest_path.clone()),
        manifest: None,
        documents: Vec::new(),
        issues: Vec::new(),
    };

    let manifest_source = match std::fs::read_to_string(&manifest_path) {
        Ok(source) => source,
        Err(error) => {
            loaded.issues.push(ValidationIssue {
                severity: ValidationSeverity::Error,
                code: "custom_manifest_read_failed".into(),
                source_kind: None,
                source_id: None,
                field_path: None,
                message: format!(
                    "Failed to read custom document manifest '{}': {}",
                    manifest_path.display(),
                    error
                ),
                related_refs: Vec::new(),
            });
            return loaded;
        }
    };

    let manifest = match serde_json::from_str::<CustomDataManifest>(&manifest_source) {
        Ok(manifest) => manifest,
        Err(error) => {
            loaded.issues.push(ValidationIssue {
                severity: ValidationSeverity::Error,
                code: "custom_manifest_parse_failed".into(),
                source_kind: None,
                source_id: None,
                field_path: None,
                message: format!(
                    "Failed to parse custom document manifest '{}': {}",
                    manifest_path.display(),
                    error
                ),
                related_refs: Vec::new(),
            });
            return loaded;
        }
    };

    let data_root = default_data_root(project_root, project);
    let mut seen_ids = BTreeSet::new();
    for entry in &manifest.documents {
        let key = (entry.kind.clone(), entry.id.clone());
        if !seen_ids.insert(key.clone()) {
            loaded.issues.push(ValidationIssue {
                severity: ValidationSeverity::Error,
                code: "duplicate_custom_document_id".into(),
                source_kind: Some(key.0),
                source_id: Some(key.1),
                field_path: None,
                message: "Duplicate custom document id found in registry.".into(),
                related_refs: Vec::new(),
            });
        }

        let registration = registry.get(&entry.kind);
        let resolved_route = registration
            .map(|registration| registration.editor_route)
            .unwrap_or(entry.editor_route);

        if let Some(registration) = registration {
            if !registration.schema_is_valid_json {
                loaded.issues.push(ValidationIssue {
                    severity: ValidationSeverity::Error,
                    code: "invalid_schema_artifact".into(),
                    source_kind: Some(entry.kind.clone()),
                    source_id: Some(entry.id.clone()),
                    field_path: None,
                    message: format!(
                        "Registered schema artifact for kind '{}' is not valid JSON.",
                        entry.kind
                    ),
                    related_refs: Vec::new(),
                });
            }

            if registration.schema_version != entry.schema_version {
                loaded.issues.push(ValidationIssue {
                    severity: ValidationSeverity::Warning,
                    code: "registry_schema_version_mismatch".into(),
                    source_kind: Some(entry.kind.clone()),
                    source_id: Some(entry.id.clone()),
                    field_path: Some("schema_version".into()),
                    message: format!(
                        "Registry entry schema version {} does not match registered version {}.",
                        entry.schema_version, registration.schema_version
                    ),
                    related_refs: Vec::new(),
                });
            }

            if registration.editor_route != entry.editor_route {
                loaded.issues.push(ValidationIssue {
                    severity: ValidationSeverity::Info,
                    code: "editor_route_override".into(),
                    source_kind: Some(entry.kind.clone()),
                    source_id: Some(entry.id.clone()),
                    field_path: Some("editor_route".into()),
                    message: format!(
                        "Registry route {:?} differs from registered default {:?}.",
                        entry.editor_route, registration.editor_route
                    ),
                    related_refs: Vec::new(),
                });
            }
        } else {
            loaded.issues.push(ValidationIssue {
                severity: ValidationSeverity::Error,
                code: "unknown_custom_document_kind".into(),
                source_kind: Some(entry.kind.clone()),
                source_id: Some(entry.id.clone()),
                field_path: Some("kind".into()),
                message: format!(
                    "Custom document kind '{}' is not registered with DJ Engine.",
                    entry.kind
                ),
                related_refs: Vec::new(),
            });
        }

        let document_path = data_root.join(&entry.path);
        if !document_path.exists() {
            loaded.documents.push(LoadedCustomDocument {
                entry: entry.clone(),
                raw_json: String::new(),
                document: None,
                parse_error: Some(format!(
                    "Missing custom document file '{}'.",
                    document_path.display()
                )),
                resolved_route,
            });
            loaded.issues.push(ValidationIssue {
                severity: ValidationSeverity::Error,
                code: "custom_document_missing_file".into(),
                source_kind: Some(entry.kind.clone()),
                source_id: Some(entry.id.clone()),
                field_path: Some("path".into()),
                message: format!(
                    "Custom document file '{}' could not be found.",
                    document_path.display()
                ),
                related_refs: Vec::new(),
            });
            continue;
        }

        let raw_json = match std::fs::read_to_string(&document_path) {
            Ok(raw_json) => raw_json,
            Err(error) => {
                loaded.documents.push(LoadedCustomDocument {
                    entry: entry.clone(),
                    raw_json: String::new(),
                    document: None,
                    parse_error: Some(error.to_string()),
                    resolved_route,
                });
                loaded.issues.push(ValidationIssue {
                    severity: ValidationSeverity::Error,
                    code: "custom_document_read_failed".into(),
                    source_kind: Some(entry.kind.clone()),
                    source_id: Some(entry.id.clone()),
                    field_path: Some("path".into()),
                    message: format!(
                        "Failed to read custom document '{}': {}",
                        document_path.display(),
                        error
                    ),
                    related_refs: Vec::new(),
                });
                continue;
            }
        };

        let document = serde_json::from_str::<CustomDocumentEnvelope>(&raw_json);
        match document {
            Ok(document) => {
                loaded.documents.push(LoadedCustomDocument {
                    entry: entry.clone(),
                    raw_json,
                    document: Some(document),
                    parse_error: None,
                    resolved_route,
                });
            }
            Err(error) => {
                loaded.documents.push(LoadedCustomDocument {
                    entry: entry.clone(),
                    raw_json,
                    document: None,
                    parse_error: Some(error.to_string()),
                    resolved_route,
                });
                loaded.issues.push(ValidationIssue {
                    severity: ValidationSeverity::Error,
                    code: "custom_document_parse_failed".into(),
                    source_kind: Some(entry.kind.clone()),
                    source_id: Some(entry.id.clone()),
                    field_path: None,
                    message: format!(
                        "Failed to parse custom document '{}': {}",
                        document_path.display(),
                        error
                    ),
                    related_refs: Vec::new(),
                });
            }
        }
    }

    loaded.manifest = Some(manifest);
    sort_loaded_documents(&mut loaded.documents);
    loaded
        .issues
        .extend(validate_loaded_custom_documents(&loaded, project, registry));
    loaded
}

pub fn validate_loaded_custom_documents(
    loaded_documents: &LoadedCustomDocuments,
    project: &Project,
    registry: &CustomDocumentRegistry,
) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();

    for document in &loaded_documents.documents {
        let Some(parsed) = document.document.as_ref() else {
            continue;
        };

        if parsed.kind != document.entry.kind {
            issues.push(ValidationIssue {
                severity: ValidationSeverity::Error,
                code: "custom_document_kind_mismatch".into(),
                source_kind: Some(document.entry.kind.clone()),
                source_id: Some(document.entry.id.clone()),
                field_path: Some("kind".into()),
                message: format!(
                    "Document kind '{}' does not match registry kind '{}'.",
                    parsed.kind, document.entry.kind
                ),
                related_refs: Vec::new(),
            });
        }

        if parsed.id != document.entry.id {
            issues.push(ValidationIssue {
                severity: ValidationSeverity::Error,
                code: "custom_document_id_mismatch".into(),
                source_kind: Some(document.entry.kind.clone()),
                source_id: Some(document.entry.id.clone()),
                field_path: Some("id".into()),
                message: format!(
                    "Document id '{}' does not match registry id '{}'.",
                    parsed.id, document.entry.id
                ),
                related_refs: Vec::new(),
            });
        }

        if parsed.schema_version != document.entry.schema_version {
            issues.push(ValidationIssue {
                severity: ValidationSeverity::Warning,
                code: "custom_document_schema_version_mismatch".into(),
                source_kind: Some(document.entry.kind.clone()),
                source_id: Some(document.entry.id.clone()),
                field_path: Some("schema_version".into()),
                message: format!(
                    "Document schema version {} does not match registry version {}.",
                    parsed.schema_version, document.entry.schema_version
                ),
                related_refs: Vec::new(),
            });
        }

        for link in &parsed.references {
            let related_refs = match &link.target {
                DocumentLinkTarget::Document { kind, id } => vec![format!("{kind}:{id}")],
                DocumentLinkTarget::Scene { id } => vec![format!("scene:{id}")],
                DocumentLinkTarget::StoryGraph { id } => vec![format!("story_graph:{id}")],
                DocumentLinkTarget::Asset { path } => vec![format!("asset:{path}")],
            };

            match &link.target {
                DocumentLinkTarget::Document { kind, id } => {
                    if loaded_documents.get(kind, id).is_none() {
                        issues.push(ValidationIssue {
                            severity: ValidationSeverity::Error,
                            code: "broken_document_ref".into(),
                            source_kind: Some(document.entry.kind.clone()),
                            source_id: Some(document.entry.id.clone()),
                            field_path: Some(link.field_path.clone()),
                            message: format!(
                                "Document reference '{}:{}' could not be resolved.",
                                kind, id
                            ),
                            related_refs,
                        });
                    }
                }
                DocumentLinkTarget::Scene { id } => {
                    if project.find_scene(id).is_none() {
                        issues.push(ValidationIssue {
                            severity: ValidationSeverity::Error,
                            code: "broken_scene_ref".into(),
                            source_kind: Some(document.entry.kind.clone()),
                            source_id: Some(document.entry.id.clone()),
                            field_path: Some(link.field_path.clone()),
                            message: format!("Scene reference '{}' could not be resolved.", id),
                            related_refs,
                        });
                    }
                }
                DocumentLinkTarget::StoryGraph { id } => {
                    if project.find_story_graph(id).is_none() {
                        issues.push(ValidationIssue {
                            severity: ValidationSeverity::Error,
                            code: "broken_story_graph_ref".into(),
                            source_kind: Some(document.entry.kind.clone()),
                            source_id: Some(document.entry.id.clone()),
                            field_path: Some(link.field_path.clone()),
                            message: format!(
                                "Story graph reference '{}' could not be resolved.",
                                id
                            ),
                            related_refs,
                        });
                    }
                }
                DocumentLinkTarget::Asset { path } => {
                    let asset_path = Path::new(&project.settings.paths.assets).join(path);
                    if !asset_path.is_absolute() && path.is_empty() {
                        issues.push(ValidationIssue {
                            severity: ValidationSeverity::Error,
                            code: "broken_asset_ref".into(),
                            source_kind: Some(document.entry.kind.clone()),
                            source_id: Some(document.entry.id.clone()),
                            field_path: Some(link.field_path.clone()),
                            message: "Asset reference path is empty.".into(),
                            related_refs,
                        });
                    }
                }
            }
        }
    }

    for document in &loaded_documents.documents {
        let Some(registration) = registry.get(document.kind()) else {
            continue;
        };
        let Some(validator) = registration.validator.as_ref() else {
            continue;
        };
        validator(document, loaded_documents, project, &mut issues);
    }

    issues
}

pub fn update_loaded_custom_document_raw_json(
    loaded_documents: &mut LoadedCustomDocuments,
    project: &Project,
    registry: &CustomDocumentRegistry,
    kind: &str,
    id: &str,
    raw_json: String,
) {
    if let Some(document) = loaded_documents.get_mut(kind, id) {
        document.raw_json = raw_json;
        match serde_json::from_str::<CustomDocumentEnvelope>(&document.raw_json) {
            Ok(parsed) => {
                document.document = Some(parsed);
                document.parse_error = None;
            }
            Err(error) => {
                document.document = None;
                document.parse_error = Some(error.to_string());
            }
        }
    }

    loaded_documents.issues = validate_loaded_custom_documents(loaded_documents, project, registry);
}

pub fn update_loaded_custom_document_envelope<F>(
    loaded_documents: &mut LoadedCustomDocuments,
    project: &Project,
    registry: &CustomDocumentRegistry,
    kind: &str,
    id: &str,
    update: F,
) -> Result<bool, serde_json::Error>
where
    F: FnOnce(&mut CustomDocumentEnvelope),
{
    let Some(mut document) = loaded_documents
        .get(kind, id)
        .and_then(|document| document.document.clone())
    else {
        return Ok(false);
    };

    update(&mut document);
    let raw_json = serde_json::to_string_pretty(&document)?;
    update_loaded_custom_document_raw_json(loaded_documents, project, registry, kind, id, raw_json);
    Ok(true)
}

pub fn update_loaded_custom_document_typed<T, F>(
    loaded_documents: &mut LoadedCustomDocuments,
    project: &Project,
    registry: &CustomDocumentRegistry,
    kind: &str,
    id: &str,
    update: F,
) -> Result<bool, serde_json::Error>
where
    T: Serialize + DeserializeOwned,
    F: FnOnce(&mut CustomDocument<T>),
{
    let Some(mut document) = loaded_documents.get_typed::<T>(kind, id)? else {
        return Ok(false);
    };

    update(&mut document);
    let raw_json = serde_json::to_string_pretty(&document)?;
    update_loaded_custom_document_raw_json(loaded_documents, project, registry, kind, id, raw_json);
    Ok(true)
}

pub fn update_loaded_custom_document_label(
    loaded_documents: &mut LoadedCustomDocuments,
    project: &Project,
    registry: &CustomDocumentRegistry,
    kind: &str,
    id: &str,
    label: Option<String>,
) -> Result<bool, CustomDocumentUpdateError> {
    let normalized = label.and_then(|label| {
        let trimmed = label.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    });

    update_loaded_custom_document_envelope(
        loaded_documents,
        project,
        registry,
        kind,
        id,
        move |document| {
            document.label = normalized.clone();
        },
    )
    .map_err(Into::into)
}

pub fn update_loaded_custom_document_top_level_scalar(
    loaded_documents: &mut LoadedCustomDocuments,
    project: &Project,
    registry: &CustomDocumentRegistry,
    kind: &str,
    id: &str,
    field_name: &str,
    value: CustomDocumentScalarValue,
) -> Result<bool, CustomDocumentUpdateError> {
    if field_name.trim().is_empty()
        || field_name.contains('.')
        || field_name.contains('[')
        || field_name.contains(']')
    {
        return Err(CustomDocumentUpdateError::InvalidFieldPath(
            field_name.to_string(),
        ));
    }

    let Some(document) = loaded_documents
        .get(kind, id)
        .and_then(|document| document.document.as_ref())
    else {
        return Ok(false);
    };

    let Some(payload) = document.payload.as_object() else {
        return Err(CustomDocumentUpdateError::PayloadNotObject);
    };

    let Some(existing_value) = payload.get(field_name) else {
        return Err(CustomDocumentUpdateError::MissingField(
            field_name.to_string(),
        ));
    };

    let Some(existing_scalar) = CustomDocumentScalarValue::from_json_value(existing_value) else {
        return Err(CustomDocumentUpdateError::NonScalarField {
            field: field_name.to_string(),
        });
    };

    if std::mem::discriminant(&existing_scalar) != std::mem::discriminant(&value) {
        return Err(CustomDocumentUpdateError::ScalarTypeMismatch {
            field: field_name.to_string(),
            expected: existing_scalar.kind_name(),
            actual: value.kind_name(),
        });
    }

    let field_name = field_name.to_string();
    update_loaded_custom_document_envelope(
        loaded_documents,
        project,
        registry,
        kind,
        id,
        move |document| {
            if let Some(payload) = document.payload.as_object_mut() {
                payload.insert(field_name.clone(), value.to_json_value());
            }
        },
    )
    .map_err(Into::into)
}

/// Maximum string length for field values set through the mutation helpers.
const MAX_STRING_VALUE_LENGTH: usize = 4096;

#[derive(Debug)]
enum PathSegment {
    Key(String),
    Index(usize),
}

fn parse_field_path(path: &str) -> Result<Vec<PathSegment>, CustomDocumentUpdateError> {
    if path.is_empty() {
        return Err(CustomDocumentUpdateError::NestedPathNotFound {
            path: path.to_string(),
        });
    }

    let mut segments = Vec::new();
    let mut remainder = path;

    while !remainder.is_empty() {
        if remainder.starts_with('[') {
            let close = remainder.find(']').ok_or_else(|| {
                CustomDocumentUpdateError::NestedPathNotFound {
                    path: path.to_string(),
                }
            })?;
            let index_str = &remainder[1..close];
            let index = index_str.parse::<usize>().map_err(|_| {
                CustomDocumentUpdateError::NestedPathNotFound {
                    path: path.to_string(),
                }
            })?;
            segments.push(PathSegment::Index(index));
            remainder = &remainder[close + 1..];
            if remainder.starts_with('.') {
                remainder = &remainder[1..];
            }
        } else {
            let end = remainder
                .find(|c: char| c == '.' || c == '[')
                .unwrap_or(remainder.len());
            let key = &remainder[..end];
            if key.is_empty() {
                return Err(CustomDocumentUpdateError::NestedPathNotFound {
                    path: path.to_string(),
                });
            }
            segments.push(PathSegment::Key(key.to_string()));
            remainder = &remainder[end..];
            if remainder.starts_with('.') {
                remainder = &remainder[1..];
            }
        }
    }

    if segments.is_empty() {
        return Err(CustomDocumentUpdateError::NestedPathNotFound {
            path: path.to_string(),
        });
    }

    Ok(segments)
}

fn resolve_mut_path<'a>(
    root: &'a mut Value,
    segments: &[PathSegment],
    full_path: &str,
) -> Result<&'a mut Value, CustomDocumentUpdateError> {
    let mut current = root;
    for segment in segments {
        match segment {
            PathSegment::Key(key) => {
                current = current
                    .as_object_mut()
                    .and_then(|obj| obj.get_mut(key))
                    .ok_or_else(|| CustomDocumentUpdateError::NestedPathNotFound {
                        path: full_path.to_string(),
                    })?;
            }
            PathSegment::Index(index) => {
                let arr = current.as_array_mut().ok_or_else(|| {
                    CustomDocumentUpdateError::NestedPathNotFound {
                        path: full_path.to_string(),
                    }
                })?;
                let length = arr.len();
                current =
                    arr.get_mut(*index)
                        .ok_or(CustomDocumentUpdateError::IndexOutOfBounds {
                            path: full_path.to_string(),
                            index: *index,
                            length,
                        })?;
            }
        }
    }
    Ok(current)
}

fn json_type_name(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "bool",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

/// Update a nested value within a custom document's payload using dot/bracket path notation.
///
/// Supports paths like `"stats.health"`, `"name.en"`, `"abilities[0]"`, `"loot[0].chance"`.
/// The replacement value's JSON type must match the existing value's type.
pub fn update_loaded_custom_document_nested_value(
    loaded_documents: &mut LoadedCustomDocuments,
    project: &Project,
    registry: &CustomDocumentRegistry,
    kind: &str,
    id: &str,
    field_path: &str,
    value: Value,
) -> Result<bool, CustomDocumentUpdateError> {
    let segments = parse_field_path(field_path)?;

    // Validate string length
    if let Value::String(s) = &value {
        if s.len() > MAX_STRING_VALUE_LENGTH {
            return Err(CustomDocumentUpdateError::StringTooLong {
                field: field_path.to_string(),
                max_length: MAX_STRING_VALUE_LENGTH,
            });
        }
    }

    // First pass: validate the path exists and type matches (read-only).
    {
        let Some(document) = loaded_documents
            .get(kind, id)
            .and_then(|d| d.document.as_ref())
        else {
            return Ok(false);
        };

        let mut payload_clone = document.payload.clone();
        let existing = resolve_mut_path(&mut payload_clone, &segments, field_path)?;
        let existing_type = json_type_name(existing);
        let new_type = json_type_name(&value);
        if existing_type != new_type {
            return Err(CustomDocumentUpdateError::NestedTypeMismatch {
                path: field_path.to_string(),
                expected: existing_type,
                actual: new_type,
            });
        }
    }

    // Second pass: apply the mutation.
    let segments = parse_field_path(field_path)?;
    update_loaded_custom_document_envelope(
        loaded_documents,
        project,
        registry,
        kind,
        id,
        move |document| {
            if let Ok(target) = resolve_mut_path(&mut document.payload, &segments, field_path) {
                *target = value.clone();
            }
        },
    )
    .map_err(Into::into)
}

pub fn save_loaded_custom_documents(
    loaded_documents: &LoadedCustomDocuments,
    root_path: &Path,
    project: &Project,
) -> Result<(), DataError> {
    let data_root = default_data_root(root_path, project);
    std::fs::create_dir_all(&data_root)?;

    let manifest = loaded_documents.manifest.clone().unwrap_or_default();
    let manifest_path = loaded_documents
        .manifest_path
        .clone()
        .unwrap_or_else(|| default_custom_data_manifest_path(root_path, project));

    if let Some(parent) = manifest_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&manifest_path, serde_json::to_string_pretty(&manifest)?)?;

    for document in &loaded_documents.documents {
        let path = data_root.join(&document.entry.path);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = if document.raw_json.trim().is_empty() {
            serde_json::to_string_pretty(&document.document)?
        } else {
            document.raw_json.clone()
        };
        std::fs::write(path, content)?;
    }

    Ok(())
}

pub fn filter_document_refs_by_kind(
    loaded_documents: &LoadedCustomDocuments,
    kind: &str,
    search_query: &str,
) -> Vec<DocumentRef> {
    let normalized_query = search_query.trim().to_lowercase();
    let mut refs: Vec<_> = loaded_documents
        .documents
        .iter()
        .filter(|document| document.entry.kind == kind)
        .filter(|document| {
            normalized_query.is_empty()
                || document.entry.id.to_lowercase().contains(&normalized_query)
        })
        .map(|document| DocumentRef {
            kind: document.entry.kind.clone(),
            id: document.entry.id.clone(),
        })
        .collect();
    refs.sort_by(|left, right| left.id.cmp(&right.id));
    refs
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::project::Project;

    const TEST_SCHEMA: &str = r#"{
      "$schema": "http://json-schema.org/draft-07/schema#",
      "type": "object"
    }"#;

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    struct AbilityPayload {
        power: u32,
    }

    fn write_test_project(
        temp_dir: &tempfile::TempDir,
        manifest: &CustomDataManifest,
        documents: &[(&str, &str)],
    ) -> (Project, MountedProject) {
        let mut project = Project::new("Custom Data Project");
        project.add_scene("arena", "scenes/arena.json");
        project.add_story_graph("opening", "story_graphs/opening.json");

        let root = temp_dir.path();
        std::fs::create_dir_all(root.join("data")).unwrap();
        std::fs::write(
            root.join("data/registry.json"),
            serde_json::to_string_pretty(manifest).unwrap(),
        )
        .unwrap();

        for (path, content) in documents {
            let file_path = root.join("data").join(path);
            if let Some(parent) = file_path.parent() {
                std::fs::create_dir_all(parent).unwrap();
            }
            std::fs::write(file_path, content).unwrap();
        }

        let mounted = MountedProject {
            root_path: Some(root.to_path_buf()),
            manifest_path: Some(root.join("project.json")),
            project: Some(project.clone()),
        };

        (project, mounted)
    }

    #[test]
    fn test_load_custom_documents_from_missing_manifest_returns_empty_resource() {
        let temp_dir = tempfile::tempdir().unwrap();
        let project = Project::new("Missing Manifest");
        let mounted = MountedProject {
            root_path: Some(temp_dir.path().to_path_buf()),
            manifest_path: Some(temp_dir.path().join("project.json")),
            project: Some(project),
        };
        let registry = CustomDocumentRegistry::default();

        let loaded = load_custom_documents_from_project(&mounted, &registry);

        assert!(loaded.documents.is_empty());
        assert!(loaded.issues.is_empty());
        assert!(loaded.manifest.is_some());
    }

    #[test]
    fn test_load_custom_documents_validates_duplicate_ids_and_broken_refs() {
        let temp_dir = tempfile::tempdir().unwrap();
        let manifest = CustomDataManifest {
            version: 1,
            documents: vec![
                CustomDocumentEntry {
                    kind: "abilities".into(),
                    id: "fireball".into(),
                    path: "abilities/fireball.json".into(),
                    schema_version: 1,
                    editor_route: EditorDocumentRoute::Table,
                    tags: Vec::new(),
                },
                CustomDocumentEntry {
                    kind: "abilities".into(),
                    id: "fireball".into(),
                    path: "abilities/fireball_dupe.json".into(),
                    schema_version: 1,
                    editor_route: EditorDocumentRoute::Table,
                    tags: Vec::new(),
                },
            ],
        };

        let fireball = r#"{
          "kind": "abilities",
          "id": "fireball",
          "schema_version": 1,
          "references": [
            { "field_path": "payload.upgrade", "type": "document", "kind": "abilities", "id": "missing" }
          ],
          "payload": { "power": 10 }
        }"#;
        let fireball_dupe = r#"{
          "kind": "abilities",
          "id": "fireball",
          "schema_version": 1,
          "payload": { "power": 20 }
        }"#;

        let (project, mounted) = write_test_project(
            &temp_dir,
            &manifest,
            &[
                ("abilities/fireball.json", fireball),
                ("abilities/fireball_dupe.json", fireball_dupe),
            ],
        );

        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(DJDataRegistryPlugin);
        app.register_custom_document(CustomDocumentRegistration::<AbilityPayload>::new(
            "abilities",
            1,
            EditorDocumentRoute::Table,
            TEST_SCHEMA,
        ));
        let registry = app.world().resource::<CustomDocumentRegistry>().clone();

        let loaded = load_custom_documents_from_project(&mounted, &registry);

        assert_eq!(project.settings.paths.data, "data");
        assert!(loaded
            .issues
            .iter()
            .any(|issue| issue.code == "duplicate_custom_document_id"));
        assert!(loaded
            .issues
            .iter()
            .any(|issue| issue.code == "broken_document_ref"));
    }

    #[test]
    fn test_resolve_default_preview_profile_prefers_default_id() {
        let loaded = LoadedCustomDocuments {
            manifest_path: None,
            manifest: Some(CustomDataManifest::default()),
            documents: vec![LoadedCustomDocument {
                entry: CustomDocumentEntry {
                    kind: "preview_profiles".into(),
                    id: "default_preview".into(),
                    path: "preview_profiles/default_preview.json".into(),
                    schema_version: 1,
                    editor_route: EditorDocumentRoute::Inspector,
                    tags: Vec::new(),
                },
                raw_json: r#"{"kind":"preview_profiles","id":"default_preview","schema_version":1,"payload":{"scene_id":"arena","story_graph_id":"opening","document_refs":[]}}"#.into(),
                document: Some(CustomDocumentEnvelope {
                    kind: "preview_profiles".into(),
                    id: "default_preview".into(),
                    schema_version: 1,
                    label: None,
                    tags: Vec::new(),
                    references: Vec::new(),
                    payload: serde_json::json!({
                        "scene_id": "arena",
                        "story_graph_id": "opening",
                        "document_refs": []
                    }),
                }),
                parse_error: None,
                resolved_route: EditorDocumentRoute::Inspector,
            }],
            issues: Vec::new(),
        };

        let profile = resolve_default_preview_profile(&loaded).unwrap();
        assert_eq!(profile.id, "default_preview");
        assert_eq!(profile.payload.scene_id.as_deref(), Some("arena"));
    }

    #[test]
    fn test_filter_document_refs_by_kind_filters_and_sorts() {
        let loaded = LoadedCustomDocuments {
            manifest_path: None,
            manifest: Some(CustomDataManifest::default()),
            documents: vec![
                LoadedCustomDocument {
                    entry: CustomDocumentEntry {
                        kind: "abilities".into(),
                        id: "zeta".into(),
                        path: "abilities/zeta.json".into(),
                        schema_version: 1,
                        editor_route: EditorDocumentRoute::Table,
                        tags: Vec::new(),
                    },
                    raw_json: String::new(),
                    document: None,
                    parse_error: None,
                    resolved_route: EditorDocumentRoute::Table,
                },
                LoadedCustomDocument {
                    entry: CustomDocumentEntry {
                        kind: "abilities".into(),
                        id: "alpha".into(),
                        path: "abilities/alpha.json".into(),
                        schema_version: 1,
                        editor_route: EditorDocumentRoute::Table,
                        tags: Vec::new(),
                    },
                    raw_json: String::new(),
                    document: None,
                    parse_error: None,
                    resolved_route: EditorDocumentRoute::Table,
                },
            ],
            issues: Vec::new(),
        };

        let refs = filter_document_refs_by_kind(&loaded, "abilities", "a");
        assert_eq!(refs.len(), 2);
        assert_eq!(refs[0].id, "alpha");
        assert_eq!(refs[1].id, "zeta");
    }

    #[test]
    fn test_update_loaded_custom_document_envelope_updates_pretty_json() {
        let project = Project::new("Envelope Update");
        let mut loaded = LoadedCustomDocuments {
            manifest_path: None,
            manifest: Some(CustomDataManifest::default()),
            documents: vec![LoadedCustomDocument {
                entry: CustomDocumentEntry {
                    kind: "abilities".into(),
                    id: "fireball".into(),
                    path: "abilities/fireball.json".into(),
                    schema_version: 1,
                    editor_route: EditorDocumentRoute::Table,
                    tags: Vec::new(),
                },
                raw_json: r#"{"kind":"abilities","id":"fireball","schema_version":1,"payload":{"power":10}}"#.into(),
                document: Some(CustomDocumentEnvelope {
                    kind: "abilities".into(),
                    id: "fireball".into(),
                    schema_version: 1,
                    label: None,
                    tags: Vec::new(),
                    references: Vec::new(),
                    payload: serde_json::json!({ "power": 10 }),
                }),
                parse_error: None,
                resolved_route: EditorDocumentRoute::Table,
            }],
            issues: Vec::new(),
        };

        let registry = CustomDocumentRegistry::default();
        let updated = update_loaded_custom_document_envelope(
            &mut loaded,
            &project,
            &registry,
            "abilities",
            "fireball",
            |document| {
                document.label = Some("Fireball".into());
                document.tags = vec!["starter".into(), "magic".into()];
            },
        )
        .unwrap();

        assert!(updated);
        let updated_document = loaded.get("abilities", "fireball").unwrap();
        let parsed = updated_document.document.as_ref().unwrap();
        assert_eq!(parsed.label.as_deref(), Some("Fireball"));
        assert_eq!(parsed.tags, vec!["starter", "magic"]);
        assert!(updated_document
            .raw_json
            .contains("\n  \"label\": \"Fireball\""));
        assert_eq!(updated_document.parse_error, None);
    }

    #[test]
    fn test_update_loaded_custom_document_typed_updates_preview_profile_payload() {
        let mut project = Project::new("Typed Update");
        project.add_scene("arena", "scenes/arena.json");
        project.add_story_graph("opening", "story_graphs/opening.json");
        project.add_story_graph("boss_intro", "story_graphs/boss_intro.json");

        let mut loaded = LoadedCustomDocuments {
            manifest_path: None,
            manifest: Some(CustomDataManifest::default()),
            documents: vec![LoadedCustomDocument {
                entry: CustomDocumentEntry {
                    kind: "preview_profiles".into(),
                    id: "default_preview".into(),
                    path: "preview_profiles/default_preview.json".into(),
                    schema_version: 1,
                    editor_route: EditorDocumentRoute::Inspector,
                    tags: Vec::new(),
                },
                raw_json: r#"{"kind":"preview_profiles","id":"default_preview","schema_version":1,"payload":{"scene_id":"arena","story_graph_id":"opening","document_refs":[]}}"#.into(),
                document: Some(CustomDocumentEnvelope {
                    kind: "preview_profiles".into(),
                    id: "default_preview".into(),
                    schema_version: 1,
                    label: None,
                    tags: Vec::new(),
                    references: Vec::new(),
                    payload: serde_json::json!({
                        "scene_id": "arena",
                        "story_graph_id": "opening",
                        "document_refs": []
                    }),
                }),
                parse_error: None,
                resolved_route: EditorDocumentRoute::Inspector,
            }],
            issues: Vec::new(),
        };

        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(DJDataRegistryPlugin);
        let registry = app.world().resource::<CustomDocumentRegistry>().clone();

        let updated = update_loaded_custom_document_typed::<PreviewProfilePayload, _>(
            &mut loaded,
            &project,
            &registry,
            "preview_profiles",
            "default_preview",
            |document| {
                document.payload.story_graph_id = Some("boss_intro".into());
                document.payload.document_refs.push(DocumentRef {
                    kind: "abilities".into(),
                    id: "dash".into(),
                });
            },
        )
        .unwrap();

        assert!(updated);
        let updated_document = loaded
            .get_typed::<PreviewProfilePayload>("preview_profiles", "default_preview")
            .unwrap()
            .unwrap();
        assert_eq!(
            updated_document.payload.story_graph_id.as_deref(),
            Some("boss_intro")
        );
        assert_eq!(updated_document.payload.document_refs.len(), 1);
        assert_eq!(updated_document.payload.document_refs[0].kind, "abilities");
        assert!(loaded
            .issues
            .iter()
            .any(|issue| issue.code == "preview_profile_missing_document"));
    }

    fn make_scalar_document(payload: serde_json::Value, raw_json: &str) -> LoadedCustomDocuments {
        LoadedCustomDocuments {
            manifest_path: None,
            manifest: Some(CustomDataManifest::default()),
            documents: vec![LoadedCustomDocument {
                entry: CustomDocumentEntry {
                    kind: "abilities".into(),
                    id: "fireball".into(),
                    path: "abilities/fireball.json".into(),
                    schema_version: 1,
                    editor_route: EditorDocumentRoute::Table,
                    tags: Vec::new(),
                },
                raw_json: raw_json.into(),
                document: Some(CustomDocument {
                    kind: "abilities".into(),
                    id: "fireball".into(),
                    schema_version: 1,
                    label: Some("Fireball".into()),
                    tags: Vec::new(),
                    references: Vec::new(),
                    payload,
                }),
                parse_error: None,
                resolved_route: EditorDocumentRoute::Table,
            }],
            issues: Vec::new(),
        }
    }

    #[test]
    fn test_update_loaded_custom_document_label_normalizes_and_refreshes_json() {
        let mut loaded = make_scalar_document(
            serde_json::json!({
                "power": 10,
                "name": "Fireball"
            }),
            r#"{"kind":"abilities","id":"fireball","schema_version":1,"label":"Fireball","payload":{"power":10,"name":"Fireball"}}"#,
        );
        let project = Project::new("Label Update");
        let registry = CustomDocumentRegistry::default();

        let updated = update_loaded_custom_document_label(
            &mut loaded,
            &project,
            &registry,
            "abilities",
            "fireball",
            Some("  Ember Burst  ".into()),
        )
        .unwrap();

        assert!(updated);
        let updated_document = loaded.get("abilities", "fireball").unwrap();
        assert_eq!(
            updated_document.document.as_ref().unwrap().label.as_deref(),
            Some("Ember Burst")
        );
        assert!(updated_document
            .raw_json
            .contains("\"label\": \"Ember Burst\""));
        assert!(updated_document.parse_error.is_none());
    }

    #[test]
    fn test_update_loaded_custom_document_top_level_scalar_updates_string_number_and_bool() {
        let mut loaded = make_scalar_document(
            serde_json::json!({
                "name": "Fireball",
                "power": 10,
                "enabled": true
            }),
            r#"{"kind":"abilities","id":"fireball","schema_version":1,"label":"Fireball","payload":{"name":"Fireball","power":10,"enabled":true}}"#,
        );
        let project = Project::new("Scalar Update");
        let registry = CustomDocumentRegistry::default();

        assert!(update_loaded_custom_document_top_level_scalar(
            &mut loaded,
            &project,
            &registry,
            "abilities",
            "fireball",
            "name",
            CustomDocumentScalarValue::String("Inferno".into()),
        )
        .unwrap());
        assert!(update_loaded_custom_document_top_level_scalar(
            &mut loaded,
            &project,
            &registry,
            "abilities",
            "fireball",
            "power",
            CustomDocumentScalarValue::Number(serde_json::Number::from(25)),
        )
        .unwrap());
        assert!(update_loaded_custom_document_top_level_scalar(
            &mut loaded,
            &project,
            &registry,
            "abilities",
            "fireball",
            "enabled",
            CustomDocumentScalarValue::Bool(false),
        )
        .unwrap());

        let updated = loaded.get("abilities", "fireball").unwrap();
        let payload = &updated.document.as_ref().unwrap().payload;
        assert_eq!(payload.get("name"), Some(&serde_json::json!("Inferno")));
        assert_eq!(payload.get("power"), Some(&serde_json::json!(25)));
        assert_eq!(payload.get("enabled"), Some(&serde_json::json!(false)));
        assert!(updated.raw_json.contains("\"power\": 25"));
    }

    #[test]
    fn test_update_loaded_custom_document_top_level_scalar_rejects_nested_or_non_scalar_fields() {
        let mut loaded = make_scalar_document(
            serde_json::json!({
                "name": "Fireball",
                "stats": { "power": 10 },
                "tags": ["fire"]
            }),
            r#"{"kind":"abilities","id":"fireball","schema_version":1,"label":"Fireball","payload":{"name":"Fireball","stats":{"power":10},"tags":["fire"]}}"#,
        );
        let project = Project::new("Scalar Rejection");
        let registry = CustomDocumentRegistry::default();

        let nested_error = update_loaded_custom_document_top_level_scalar(
            &mut loaded,
            &project,
            &registry,
            "abilities",
            "fireball",
            "stats.power",
            CustomDocumentScalarValue::Number(serde_json::Number::from(12)),
        )
        .unwrap_err();
        assert!(matches!(
            nested_error,
            CustomDocumentUpdateError::InvalidFieldPath(_)
        ));

        let object_error = update_loaded_custom_document_top_level_scalar(
            &mut loaded,
            &project,
            &registry,
            "abilities",
            "fireball",
            "stats",
            CustomDocumentScalarValue::String("bad".into()),
        )
        .unwrap_err();
        assert!(matches!(
            object_error,
            CustomDocumentUpdateError::NonScalarField { .. }
        ));

        let array_error = update_loaded_custom_document_top_level_scalar(
            &mut loaded,
            &project,
            &registry,
            "abilities",
            "fireball",
            "tags",
            CustomDocumentScalarValue::String("bad".into()),
        )
        .unwrap_err();
        assert!(matches!(
            array_error,
            CustomDocumentUpdateError::NonScalarField { .. }
        ));
    }

    fn make_nested_document() -> LoadedCustomDocuments {
        make_scalar_document(
            serde_json::json!({
                "name": { "en": "Fireball", "ja": "ファイアボール" },
                "stats": { "power": 10, "cost": 5 },
                "abilities": ["fire_blast", "ember"],
                "loot": [
                    { "item": "gem", "chance": 0.5 },
                    { "item": "potion", "chance": 1.0 }
                ],
                "enabled": true
            }),
            r#"{"kind":"abilities","id":"fireball","schema_version":1,"label":"Fireball","payload":{"name":{"en":"Fireball","ja":"ファイアボール"},"stats":{"power":10,"cost":5},"abilities":["fire_blast","ember"],"loot":[{"item":"gem","chance":0.5},{"item":"potion","chance":1.0}],"enabled":true}}"#,
        )
    }

    #[test]
    fn test_nested_update_object_field() {
        let mut loaded = make_nested_document();
        let project = Project::new("Nested");
        let registry = CustomDocumentRegistry::default();

        assert!(update_loaded_custom_document_nested_value(
            &mut loaded,
            &project,
            &registry,
            "abilities",
            "fireball",
            "stats.power",
            serde_json::json!(150),
        )
        .unwrap());

        let payload = &loaded
            .get("abilities", "fireball")
            .unwrap()
            .document
            .as_ref()
            .unwrap()
            .payload;
        assert_eq!(payload["stats"]["power"], serde_json::json!(150));
        assert_eq!(payload["stats"]["cost"], serde_json::json!(5));
    }

    #[test]
    fn test_nested_update_localized_string() {
        let mut loaded = make_nested_document();
        let project = Project::new("Nested");
        let registry = CustomDocumentRegistry::default();

        assert!(update_loaded_custom_document_nested_value(
            &mut loaded,
            &project,
            &registry,
            "abilities",
            "fireball",
            "name.en",
            serde_json::json!("Meteor"),
        )
        .unwrap());

        let payload = &loaded
            .get("abilities", "fireball")
            .unwrap()
            .document
            .as_ref()
            .unwrap()
            .payload;
        assert_eq!(payload["name"]["en"], serde_json::json!("Meteor"));
        assert_eq!(payload["name"]["ja"], serde_json::json!("ファイアボール"));
    }

    #[test]
    fn test_nested_update_array_element() {
        let mut loaded = make_nested_document();
        let project = Project::new("Nested");
        let registry = CustomDocumentRegistry::default();

        assert!(update_loaded_custom_document_nested_value(
            &mut loaded,
            &project,
            &registry,
            "abilities",
            "fireball",
            "abilities[0]",
            serde_json::json!("icebolt"),
        )
        .unwrap());

        let payload = &loaded
            .get("abilities", "fireball")
            .unwrap()
            .document
            .as_ref()
            .unwrap()
            .payload;
        assert_eq!(payload["abilities"][0], serde_json::json!("icebolt"));
        assert_eq!(payload["abilities"][1], serde_json::json!("ember"));
    }

    #[test]
    fn test_nested_update_array_of_objects_field() {
        let mut loaded = make_nested_document();
        let project = Project::new("Nested");
        let registry = CustomDocumentRegistry::default();

        assert!(update_loaded_custom_document_nested_value(
            &mut loaded,
            &project,
            &registry,
            "abilities",
            "fireball",
            "loot[0].chance",
            serde_json::json!(0.8),
        )
        .unwrap());

        let payload = &loaded
            .get("abilities", "fireball")
            .unwrap()
            .document
            .as_ref()
            .unwrap()
            .payload;
        assert_eq!(payload["loot"][0]["chance"], serde_json::json!(0.8));
        assert_eq!(payload["loot"][1]["chance"], serde_json::json!(1.0));
    }

    #[test]
    fn test_nested_update_rejects_path_not_found() {
        let mut loaded = make_nested_document();
        let project = Project::new("Nested");
        let registry = CustomDocumentRegistry::default();

        let err = update_loaded_custom_document_nested_value(
            &mut loaded,
            &project,
            &registry,
            "abilities",
            "fireball",
            "stats.nonexistent",
            serde_json::json!(1),
        )
        .unwrap_err();
        assert!(matches!(
            err,
            CustomDocumentUpdateError::NestedPathNotFound { .. }
        ));
    }

    #[test]
    fn test_nested_update_rejects_index_out_of_bounds() {
        let mut loaded = make_nested_document();
        let project = Project::new("Nested");
        let registry = CustomDocumentRegistry::default();

        let err = update_loaded_custom_document_nested_value(
            &mut loaded,
            &project,
            &registry,
            "abilities",
            "fireball",
            "abilities[99]",
            serde_json::json!("nope"),
        )
        .unwrap_err();
        assert!(matches!(
            err,
            CustomDocumentUpdateError::IndexOutOfBounds { .. }
        ));
    }

    #[test]
    fn test_nested_update_rejects_type_mismatch() {
        let mut loaded = make_nested_document();
        let project = Project::new("Nested");
        let registry = CustomDocumentRegistry::default();

        let err = update_loaded_custom_document_nested_value(
            &mut loaded,
            &project,
            &registry,
            "abilities",
            "fireball",
            "stats.power",
            serde_json::json!("not a number"),
        )
        .unwrap_err();
        assert!(matches!(
            err,
            CustomDocumentUpdateError::NestedTypeMismatch { .. }
        ));
    }
}
