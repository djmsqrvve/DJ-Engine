//! Table editor for record-heavy custom document kinds.
//!
//! Provides a filterable, sortable table view with inline editing for
//! label and top-level scalar payload fields.

use super::panels::{first_document_field_issue, validation_issue_color};
use super::types::{COLOR_PRIMARY, COLOR_SECONDARY};
use crate::data::{
    update_loaded_custom_document_label, update_loaded_custom_document_top_level_scalar,
    CustomDocumentRegistry, CustomDocumentScalarValue, DocumentRef, LoadedCustomDocument,
    LoadedCustomDocuments, Project, ValidationIssue,
};
use bevy::prelude::*;
use bevy_egui::egui::{self, Color32, RichText};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TableSortColumn {
    Id,
    Label,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TableSortDirection {
    Ascending,
    Descending,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum TableCellField {
    Label,
    Payload(String),
}

impl TableCellField {
    fn display_name(&self) -> String {
        match self {
            Self::Label => "label".into(),
            Self::Payload(field_name) => field_name.clone(),
        }
    }

    fn field_path(&self) -> String {
        match self {
            Self::Label => "label".into(),
            Self::Payload(field_name) => format!("payload.{field_name}"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TableCellRef {
    row: DocumentRef,
    field: TableCellField,
}

#[derive(Debug, Clone, PartialEq)]
struct ActiveTableCellEdit {
    cell: TableCellRef,
    draft_text: String,
    scalar: Option<CustomDocumentScalarValue>,
    error_message: Option<String>,
    focus_requested: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TableCommitFeedback {
    row: DocumentRef,
    field_label: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TableKindState {
    filter_text: String,
    sort_column: TableSortColumn,
    sort_direction: TableSortDirection,
    pub(crate) selected_row: Option<DocumentRef>,
    last_commit: Option<TableCommitFeedback>,
}

impl Default for TableKindState {
    fn default() -> Self {
        Self {
            filter_text: String::new(),
            sort_column: TableSortColumn::Id,
            sort_direction: TableSortDirection::Ascending,
            selected_row: None,
            last_commit: None,
        }
    }
}

#[derive(Resource, Debug, Clone, Default)]
pub struct TableEditorState {
    pub(crate) kind_states: BTreeMap<String, TableKindState>,
    active_edit: Option<ActiveTableCellEdit>,
}

impl TableEditorState {
    pub(crate) fn active_field_path_for(&self, selected: &DocumentRef) -> Option<String> {
        self.active_edit.as_ref().and_then(|edit| {
            if edit.cell.row == *selected {
                Some(edit.cell.field.field_path())
            } else {
                None
            }
        })
    }
}

fn loaded_document_exists(
    loaded_documents: &LoadedCustomDocuments,
    document_ref: &DocumentRef,
) -> bool {
    loaded_documents
        .get(&document_ref.kind, &document_ref.id)
        .is_some()
}

fn reconcile_table_kind_state(
    kind_state: &mut TableKindState,
    loaded_documents: &LoadedCustomDocuments,
    kind: &str,
    preferred_row: Option<&DocumentRef>,
) {
    if kind_state
        .selected_row
        .as_ref()
        .is_some_and(|selected_row| {
            selected_row.kind != kind || !loaded_document_exists(loaded_documents, selected_row)
        })
    {
        kind_state.selected_row = None;
    }

    if kind_state.selected_row.is_none() {
        if let Some(preferred_row) = preferred_row.filter(|preferred_row| {
            preferred_row.kind == kind && loaded_document_exists(loaded_documents, preferred_row)
        }) {
            kind_state.selected_row = Some(preferred_row.clone());
        }
    }
}

fn active_edit_is_valid_for_kind(
    active_edit: &Option<ActiveTableCellEdit>,
    loaded_documents: &LoadedCustomDocuments,
    kind: &str,
) -> bool {
    active_edit.as_ref().is_some_and(|edit| {
        edit.cell.row.kind == kind && loaded_document_exists(loaded_documents, &edit.cell.row)
    })
}

fn discover_payload_columns(documents: &[&LoadedCustomDocument]) -> Vec<String> {
    let mut columns = BTreeSet::new();
    for document in documents.iter() {
        if let Some(parsed) = &document.document {
            if let Some(object) = parsed.payload.as_object() {
                for key in object.keys() {
                    columns.insert(key.clone());
                }
            }
        }
    }
    columns.into_iter().collect()
}

fn payload_cell_text(document: &LoadedCustomDocument, column: &str) -> String {
    document
        .document
        .as_ref()
        .and_then(|parsed| parsed.payload.get(column))
        .map(|value| match value {
            serde_json::Value::String(s) => s.clone(),
            serde_json::Value::Number(n) => n.to_string(),
            serde_json::Value::Bool(b) => b.to_string(),
            serde_json::Value::Null => String::new(),
            serde_json::Value::Object(map) => map
                .get("en")
                .and_then(serde_json::Value::as_str)
                .map(str::to_string)
                .unwrap_or_else(|| format!("{{...{} keys}}", map.len())),
            serde_json::Value::Array(arr) => format!("[...{} items]", arr.len()),
        })
        .unwrap_or_default()
}

fn payload_scalar_value(
    document: &LoadedCustomDocument,
    column: &str,
) -> Option<CustomDocumentScalarValue> {
    document
        .document
        .as_ref()
        .and_then(|parsed| parsed.payload.get(column))
        .and_then(CustomDocumentScalarValue::from_json_value)
}

fn scalar_value_text(value: &CustomDocumentScalarValue) -> String {
    match value {
        CustomDocumentScalarValue::String(value) => value.clone(),
        CustomDocumentScalarValue::Number(value) => value.to_string(),
        CustomDocumentScalarValue::Bool(value) => value.to_string(),
    }
}

fn parse_scalar_draft(
    draft_text: &str,
    original: &CustomDocumentScalarValue,
) -> Result<CustomDocumentScalarValue, String> {
    match original {
        CustomDocumentScalarValue::String(_) => {
            Ok(CustomDocumentScalarValue::String(draft_text.to_string()))
        }
        CustomDocumentScalarValue::Number(original_number) => {
            let trimmed = draft_text.trim();
            if trimmed.is_empty() {
                return Err("Number fields cannot be blank.".into());
            }

            if original_number.is_i64() {
                trimmed
                    .parse::<i64>()
                    .map(serde_json::Number::from)
                    .map(CustomDocumentScalarValue::Number)
                    .map_err(|error| format!("Invalid integer: {error}"))
            } else if original_number.is_u64() {
                trimmed
                    .parse::<u64>()
                    .map(serde_json::Number::from)
                    .map(CustomDocumentScalarValue::Number)
                    .map_err(|error| format!("Invalid unsigned integer: {error}"))
            } else {
                let parsed = trimmed
                    .parse::<f64>()
                    .map_err(|error| format!("Invalid decimal number: {error}"))?;
                serde_json::Number::from_f64(parsed)
                    .map(CustomDocumentScalarValue::Number)
                    .ok_or_else(|| "Number must be finite.".into())
            }
        }
        CustomDocumentScalarValue::Bool(_) => match draft_text.trim().to_lowercase().as_str() {
            "true" => Ok(CustomDocumentScalarValue::Bool(true)),
            "false" => Ok(CustomDocumentScalarValue::Bool(false)),
            _ => Err("Bool fields must be 'true' or 'false'.".into()),
        },
    }
}

fn begin_table_cell_edit(
    active_edit: &mut Option<ActiveTableCellEdit>,
    row: &DocumentRef,
    field: TableCellField,
    draft_text: String,
    scalar: Option<CustomDocumentScalarValue>,
) {
    *active_edit = Some(ActiveTableCellEdit {
        cell: TableCellRef {
            row: row.clone(),
            field,
        },
        draft_text,
        scalar,
        error_message: None,
        focus_requested: true,
    });
}

fn draw_table_display_cell(
    ui: &mut egui::Ui,
    text: impl Into<String>,
    selected: bool,
    issue: Option<&ValidationIssue>,
    hover_hint: Option<&str>,
) -> egui::Response {
    let mut label = RichText::new(text.into());
    if let Some(issue) = issue {
        label = label.color(validation_issue_color(issue.severity));
    }

    let response = ui.selectable_label(selected, label);
    let mut hover_lines = Vec::new();
    if let Some(issue) = issue {
        hover_lines.push(format!("{}: {}", issue.code, issue.message));
    }
    if let Some(hover_hint) = hover_hint {
        hover_lines.push(hover_hint.to_string());
    }

    if hover_lines.is_empty() {
        response
    } else {
        response.on_hover_text(hover_lines.join("\n"))
    }
}

fn draw_table_active_editor(
    ui: &mut egui::Ui,
    loaded_documents: &mut LoadedCustomDocuments,
    project: Option<&Project>,
    registry: &CustomDocumentRegistry,
    row_ref: &DocumentRef,
    field: &TableCellField,
    active_edit: &mut Option<ActiveTableCellEdit>,
    last_commit: &mut Option<TableCommitFeedback>,
) {
    let Some(edit) = active_edit.as_mut() else {
        return;
    };
    if edit.cell.row != *row_ref || edit.cell.field != *field {
        return;
    }

    let field_path = field.field_path();
    let field_issue = first_document_field_issue(loaded_documents, row_ref, &field_path).cloned();

    let editor_response = ui.scope(|ui| {
        if let Some(issue) = &field_issue {
            ui.visuals_mut().override_text_color = Some(validation_issue_color(issue.severity));
        }
        ui.add(
            egui::TextEdit::singleline(&mut edit.draft_text)
                .desired_width(f32::INFINITY)
                .id_salt(format!(
                    "table_edit:{}:{}:{}",
                    row_ref.kind,
                    row_ref.id,
                    field.display_name()
                )),
        )
    });

    let mut response = editor_response.inner;
    if let Some(issue) = &field_issue {
        response = response.on_hover_text(format!("{}: {}", issue.code, issue.message));
    }
    if edit.focus_requested {
        response.request_focus();
        edit.focus_requested = false;
    }

    let escape_pressed =
        response.has_focus() && ui.input(|input| input.key_pressed(egui::Key::Escape));
    if escape_pressed {
        *active_edit = None;
        return;
    }

    let enter_pressed =
        response.has_focus() && ui.input(|input| input.key_pressed(egui::Key::Enter));
    if !(enter_pressed || response.lost_focus()) {
        return;
    }

    let Some(project) = project else {
        edit.error_message = Some("Mounted project is required for table edits.".into());
        return;
    };

    let result: Result<(), String> = match field {
        TableCellField::Label => update_loaded_custom_document_label(
            loaded_documents,
            project,
            registry,
            &row_ref.kind,
            &row_ref.id,
            Some(edit.draft_text.clone()),
        )
        .map(|_| ())
        .map_err(|error| error.to_string()),
        TableCellField::Payload(field_name) => {
            let Some(original_scalar) = edit.scalar.as_ref() else {
                edit.error_message =
                    Some("Only scalar payload fields are editable in the table.".into());
                return;
            };
            match parse_scalar_draft(&edit.draft_text, original_scalar) {
                Ok(parsed_scalar) => update_loaded_custom_document_top_level_scalar(
                    loaded_documents,
                    project,
                    registry,
                    &row_ref.kind,
                    &row_ref.id,
                    field_name,
                    parsed_scalar,
                )
                .map(|_| ())
                .map_err(|error| error.to_string()),
                Err(error) => Err(error),
            }
        }
    };

    match result {
        Ok(()) => {
            *last_commit = Some(TableCommitFeedback {
                row: row_ref.clone(),
                field_label: field.display_name(),
            });
            *active_edit = None;
        }
        Err(error) => {
            edit.error_message = Some(error);
        }
    }
}

pub(crate) fn draw_table_editor(
    ui: &mut egui::Ui,
    loaded_documents: &mut LoadedCustomDocuments,
    project: Option<&Project>,
    registry: &CustomDocumentRegistry,
    kind: &str,
    preferred_row: Option<&DocumentRef>,
    table_state: &mut TableEditorState,
) {
    let mut kind_state = table_state
        .kind_states
        .get(kind)
        .cloned()
        .unwrap_or_default();
    let mut active_edit = table_state.active_edit.clone();

    reconcile_table_kind_state(&mut kind_state, loaded_documents, kind, preferred_row);

    if !active_edit_is_valid_for_kind(&active_edit, loaded_documents, kind) {
        active_edit = None;
    }

    let issue_count = loaded_documents
        .issues
        .iter()
        .filter(|issue| issue.source_kind.as_deref() == Some(kind))
        .count();

    ui.label(
        RichText::new(format!("{kind} (table)"))
            .strong()
            .color(COLOR_PRIMARY),
    );

    if issue_count > 0 {
        ui.colored_label(
            COLOR_SECONDARY,
            format!("{issue_count} validation issue(s)"),
        );
    }

    if let Some(last_commit) = &kind_state.last_commit {
        ui.colored_label(
            COLOR_PRIMARY,
            format!(
                "Updated {} ({})",
                last_commit.row.id, last_commit.field_label
            ),
        );
    }

    if let Some(error_message) = active_edit
        .as_ref()
        .filter(|edit| edit.cell.row.kind == kind)
        .and_then(|edit| edit.error_message.as_deref())
    {
        ui.colored_label(Color32::RED, error_message);
    }

    ui.horizontal(|ui| {
        ui.label("Filter:");
        ui.text_edit_singleline(&mut kind_state.filter_text);
    });

    let normalized_filter = kind_state.filter_text.trim().to_lowercase();

    let mut rows: Vec<LoadedCustomDocument> = loaded_documents
        .documents
        .iter()
        .filter(|document| document.entry.kind == kind)
        .filter(|document| {
            if normalized_filter.is_empty() {
                return true;
            }
            if document
                .entry
                .id
                .to_lowercase()
                .contains(&normalized_filter)
            {
                return true;
            }
            if let Some(parsed) = &document.document {
                if let Some(label) = &parsed.label {
                    if label.to_lowercase().contains(&normalized_filter) {
                        return true;
                    }
                }
                for tag in &parsed.tags {
                    if tag.to_lowercase().contains(&normalized_filter) {
                        return true;
                    }
                }
            }
            false
        })
        .cloned()
        .collect();

    match (kind_state.sort_column, kind_state.sort_direction) {
        (TableSortColumn::Id, TableSortDirection::Ascending) => {
            rows.sort_by(|a, b| a.entry.id.cmp(&b.entry.id));
        }
        (TableSortColumn::Id, TableSortDirection::Descending) => {
            rows.sort_by(|a, b| b.entry.id.cmp(&a.entry.id));
        }
        (TableSortColumn::Label, TableSortDirection::Ascending) => {
            rows.sort_by(|a, b| {
                let label_a = a
                    .document
                    .as_ref()
                    .and_then(|d| d.label.as_deref())
                    .unwrap_or("");
                let label_b = b
                    .document
                    .as_ref()
                    .and_then(|d| d.label.as_deref())
                    .unwrap_or("");
                label_a.cmp(label_b)
            });
        }
        (TableSortColumn::Label, TableSortDirection::Descending) => {
            rows.sort_by(|a, b| {
                let label_a = a
                    .document
                    .as_ref()
                    .and_then(|d| d.label.as_deref())
                    .unwrap_or("");
                let label_b = b
                    .document
                    .as_ref()
                    .and_then(|d| d.label.as_deref())
                    .unwrap_or("");
                label_b.cmp(label_a)
            });
        }
    }

    let total = loaded_documents
        .documents
        .iter()
        .filter(|d| d.entry.kind == kind)
        .count();
    ui.label(format!("{} of {} documents", rows.len(), total,));
    ui.add_space(4.0);

    let sampled_rows: Vec<_> = rows.iter().collect();
    let payload_columns = discover_payload_columns(&sampled_rows);

    fn sort_arrow(state: &TableKindState, col: TableSortColumn) -> &'static str {
        if state.sort_column == col {
            match state.sort_direction {
                TableSortDirection::Ascending => " ^",
                TableSortDirection::Descending => " v",
            }
        } else {
            ""
        }
    }

    let id_header = format!("id{}", sort_arrow(&kind_state, TableSortColumn::Id));
    let label_header = format!("label{}", sort_arrow(&kind_state, TableSortColumn::Label));

    // Fixed columns: id, label. Then up to 6 payload columns.
    let visible_payload_columns: Vec<&str> = payload_columns
        .iter()
        .filter(|c| *c != "id" && *c != "name")
        .take(6)
        .map(String::as_str)
        .collect();

    let total_columns = 3 + visible_payload_columns.len();

    let mut clicked_sort: Option<TableSortColumn> = None;

    egui::ScrollArea::both()
        .max_height(ui.available_height() * 0.6)
        .show(ui, |ui| {
            egui::Grid::new("table_editor_grid")
                .num_columns(total_columns)
                .striped(true)
                .min_col_width(60.0)
                .show(ui, |ui| {
                    // Header row
                    if ui.selectable_label(false, &id_header).clicked() {
                        clicked_sort = Some(TableSortColumn::Id);
                    }
                    if ui.selectable_label(false, &label_header).clicked() {
                        clicked_sort = Some(TableSortColumn::Label);
                    }
                    for col in &visible_payload_columns {
                        ui.label(RichText::new(*col).strong());
                    }
                    ui.label(RichText::new("status").strong());
                    ui.end_row();

                    // Data rows
                    for document in &rows {
                        let row_ref = DocumentRef {
                            kind: document.entry.kind.clone(),
                            id: document.entry.id.clone(),
                        };
                        let is_selected = kind_state.selected_row.as_ref() == Some(&row_ref);

                        let id_response = draw_table_display_cell(
                            ui,
                            &document.entry.id,
                            is_selected,
                            None,
                            None,
                        );
                        if id_response.clicked() {
                            if kind_state.selected_row.as_ref() != Some(&row_ref) {
                                active_edit = None;
                            }
                            kind_state.selected_row = Some(row_ref.clone());
                        }

                        let label_field = TableCellField::Label;
                        let label_path = label_field.field_path();
                        let label_issue =
                            first_document_field_issue(loaded_documents, &row_ref, &label_path);
                        let label_text = document
                            .document
                            .as_ref()
                            .and_then(|d| d.label.clone())
                            .unwrap_or_default();
                        let label_is_active = active_edit
                            .as_ref()
                            .is_some_and(|edit| edit.cell.row == row_ref && edit.cell.field == label_field);

                        if is_selected && label_is_active {
                            draw_table_active_editor(
                                ui,
                                loaded_documents,
                                project,
                                registry,
                                &row_ref,
                                &label_field,
                                &mut active_edit,
                                &mut kind_state.last_commit,
                            );
                        } else {
                            let label_response = draw_table_display_cell(
                                ui,
                                if label_text.is_empty() {
                                    "<label>".to_string()
                                } else {
                                    label_text
                                },
                                is_selected,
                                label_issue,
                                is_selected.then_some("Click again to edit this field."),
                            );
                            if label_response.clicked() {
                                let was_selected = is_selected;
                                if kind_state.selected_row.as_ref() != Some(&row_ref) {
                                    active_edit = None;
                                }
                                kind_state.selected_row = Some(row_ref.clone());
                                if was_selected && project.is_some() {
                                    begin_table_cell_edit(
                                        &mut active_edit,
                                        &row_ref,
                                        label_field.clone(),
                                        document
                                            .document
                                            .as_ref()
                                            .and_then(|parsed| parsed.label.clone())
                                            .unwrap_or_default(),
                                        None,
                                    );
                                }
                            }
                        }

                        for col in &visible_payload_columns {
                            let payload_field = TableCellField::Payload((*col).to_string());
                            let payload_path = payload_field.field_path();
                            let payload_issue = first_document_field_issue(
                                loaded_documents,
                                &row_ref,
                                &payload_path,
                            );
                            let payload_text = payload_cell_text(document, col);
                            let truncated = if payload_text.len() > 32 {
                                format!("{}...", &payload_text[..29])
                            } else {
                                payload_text
                            };
                            let editable_scalar = payload_scalar_value(document, col);
                            let payload_is_active = active_edit.as_ref().is_some_and(|edit| {
                                edit.cell.row == row_ref && edit.cell.field == payload_field
                            });

                            if is_selected && payload_is_active {
                                draw_table_active_editor(
                                    ui,
                                    loaded_documents,
                                    project,
                                    registry,
                                    &row_ref,
                                    &payload_field,
                                    &mut active_edit,
                                    &mut kind_state.last_commit,
                                );
                            } else {
                                let hover_hint = if editable_scalar.is_some() && is_selected {
                                    Some("Click again to edit this field.")
                                } else if editable_scalar.is_none() {
                                    Some("Read-only in the table. Use the inspector for nested fields.")
                                } else {
                                    None
                                };
                                let payload_response = draw_table_display_cell(
                                    ui,
                                    truncated,
                                    is_selected,
                                    payload_issue,
                                    hover_hint,
                                );
                                if payload_response.clicked() {
                                    let was_selected = is_selected;
                                    if kind_state.selected_row.as_ref() != Some(&row_ref) {
                                        active_edit = None;
                                    }
                                    kind_state.selected_row = Some(row_ref.clone());
                                    if was_selected {
                                        if let Some(scalar) = editable_scalar {
                                            begin_table_cell_edit(
                                                &mut active_edit,
                                                &row_ref,
                                                payload_field.clone(),
                                                scalar_value_text(&scalar),
                                                Some(scalar),
                                            );
                                        }
                                    }
                                }
                            }
                        }

                        let status_text = kind_state
                            .last_commit
                            .as_ref()
                            .filter(|last_commit| last_commit.row == row_ref)
                            .map(|last_commit| format!("updated {}", last_commit.field_label))
                            .unwrap_or_default();
                        ui.label(
                            RichText::new(status_text)
                                .color(COLOR_PRIMARY)
                                .small(),
                        );
                        ui.end_row();
                    }
                });
        });

    if let Some(col) = clicked_sort {
        if kind_state.sort_column == col {
            kind_state.sort_direction = match kind_state.sort_direction {
                TableSortDirection::Ascending => TableSortDirection::Descending,
                TableSortDirection::Descending => TableSortDirection::Ascending,
            };
        } else {
            kind_state.sort_column = col;
            kind_state.sort_direction = TableSortDirection::Ascending;
        }
    }

    table_state.kind_states.insert(kind.to_string(), kind_state);
    table_state.active_edit = active_edit;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{
        CustomDocument, CustomDocumentEntry, CustomDocumentScalarValue, EditorDocumentRoute,
        LoadedCustomDocument, ValidationSeverity,
    };
    use crate::editor::panels::field_path_matches;
    use serde_json::json;

    fn make_loaded_document(id: &str, payload: serde_json::Value) -> LoadedCustomDocument {
        LoadedCustomDocument {
            entry: CustomDocumentEntry {
                kind: "abilities".into(),
                id: id.into(),
                path: format!("abilities/{id}.json"),
                schema_version: 1,
                editor_route: EditorDocumentRoute::Table,
                tags: Vec::new(),
            },
            raw_json: String::new(),
            document: Some(CustomDocument {
                kind: "abilities".into(),
                id: id.into(),
                schema_version: 1,
                label: Some(id.into()),
                tags: Vec::new(),
                references: Vec::new(),
                payload,
            }),
            parse_error: None,
            resolved_route: EditorDocumentRoute::Table,
        }
    }

    #[test]
    fn test_payload_scalar_value_only_allows_string_number_and_bool() {
        let scalar_document = make_loaded_document(
            "fireball",
            json!({
                "name": "Fireball",
                "power": 10,
                "enabled": true
            }),
        );
        let nested_document = make_loaded_document(
            "icewall",
            json!({
                "stats": { "power": 5 },
                "tags": ["ice"]
            }),
        );

        assert_eq!(
            payload_scalar_value(&scalar_document, "name"),
            Some(CustomDocumentScalarValue::String("Fireball".into()))
        );
        assert_eq!(
            payload_scalar_value(&scalar_document, "power"),
            Some(CustomDocumentScalarValue::Number(serde_json::Number::from(
                10
            )))
        );
        assert_eq!(
            payload_scalar_value(&scalar_document, "enabled"),
            Some(CustomDocumentScalarValue::Bool(true))
        );
        assert!(payload_scalar_value(&nested_document, "stats").is_none());
        assert!(payload_scalar_value(&nested_document, "tags").is_none());
    }

    #[test]
    fn test_field_path_matches_exact_and_nested_payload_paths() {
        let direct_issue = ValidationIssue {
            severity: ValidationSeverity::Error,
            code: "bad_label".into(),
            source_kind: Some("abilities".into()),
            source_id: Some("fireball".into()),
            field_path: Some("label".into()),
            message: "Label mismatch".into(),
            related_refs: Vec::new(),
        };
        let nested_issue = ValidationIssue {
            severity: ValidationSeverity::Error,
            code: "broken_ref".into(),
            source_kind: Some("abilities".into()),
            source_id: Some("fireball".into()),
            field_path: Some("payload.loot[0].item".into()),
            message: "Broken item ref".into(),
            related_refs: Vec::new(),
        };

        assert!(field_path_matches(&direct_issue, "label"));
        assert!(field_path_matches(&nested_issue, "payload.loot"));
        assert!(!field_path_matches(&nested_issue, "payload.stats"));
    }

    #[test]
    fn test_reconcile_table_kind_state_preserves_selection_filter_and_sort_on_reload() {
        let selected_row = DocumentRef {
            kind: "abilities".into(),
            id: "fireball".into(),
        };
        let mut kind_state = TableKindState {
            filter_text: "fire".into(),
            sort_column: TableSortColumn::Label,
            sort_direction: TableSortDirection::Descending,
            selected_row: Some(selected_row.clone()),
            last_commit: None,
        };
        let loaded_documents = LoadedCustomDocuments {
            manifest_path: None,
            manifest: None,
            documents: vec![
                make_loaded_document("fireball", json!({ "power": 10 })),
                make_loaded_document("flare", json!({ "power": 4 })),
            ],
            issues: Vec::new(),
        };

        reconcile_table_kind_state(&mut kind_state, &loaded_documents, "abilities", None);

        assert_eq!(kind_state.filter_text, "fire");
        assert_eq!(kind_state.sort_column, TableSortColumn::Label);
        assert_eq!(kind_state.sort_direction, TableSortDirection::Descending);
        assert_eq!(kind_state.selected_row, Some(selected_row.clone()));

        let reloaded_documents = LoadedCustomDocuments {
            manifest_path: None,
            manifest: None,
            documents: vec![
                make_loaded_document("fireball", json!({ "power": 12 })),
                make_loaded_document("flare", json!({ "power": 6 })),
            ],
            issues: Vec::new(),
        };
        reconcile_table_kind_state(&mut kind_state, &reloaded_documents, "abilities", None);
        assert_eq!(kind_state.selected_row, Some(selected_row));
        assert_eq!(kind_state.filter_text, "fire");
    }
}
