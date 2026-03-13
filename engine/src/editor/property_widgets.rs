//! Recursive property editor widgets for custom document payloads.
//!
//! Renders structured editors for JSON values: text fields for strings/numbers,
//! checkboxes for bools, collapsible sections for objects/arrays. Edits are
//! collected as `(field_path, new_value)` pairs and applied after the UI pass.

use super::panels::{first_document_field_issue, validation_issue_color};
use crate::data::{DocumentRef, LoadedCustomDocuments, ValidationIssue};
use bevy_egui::egui::{self, Color32, RichText};

/// Preferred locale key for displaying localized string objects.
const DEFAULT_DISPLAY_LOCALE: &str = "en";

/// Maximum recursion depth for nested value editors.
const MAX_DEPTH: usize = 4;

/// A pending edit to apply after the UI pass.
pub(crate) struct PendingEdit {
    pub field_path: String,
    pub value: serde_json::Value,
}

/// Returns `true` if this object looks like a localized string map:
/// all values are strings and it contains the default locale key.
fn is_localized_string_object(obj: &serde_json::Map<String, serde_json::Value>) -> bool {
    obj.contains_key(DEFAULT_DISPLAY_LOCALE)
        && obj.values().all(|v| v.is_string())
        && obj.len() >= 2
}

/// Draw a property editor for a single JSON value, recursively.
///
/// Returns `true` if any edit was committed during this frame.
pub(crate) fn draw_value_editor(
    ui: &mut egui::Ui,
    loaded_documents: &LoadedCustomDocuments,
    document_ref: &DocumentRef,
    field_path: &str,
    key_label: &str,
    value: &serde_json::Value,
    depth: usize,
    edits: &mut Vec<PendingEdit>,
) -> bool {
    let issue = first_document_field_issue(loaded_documents, document_ref, field_path);
    let mut changed = false;

    match value {
        serde_json::Value::String(s) => {
            changed = draw_string_editor(ui, field_path, key_label, s, issue, edits);
        }
        serde_json::Value::Number(n) => {
            changed = draw_number_editor(ui, field_path, key_label, n, issue, edits);
        }
        serde_json::Value::Bool(b) => {
            changed = draw_bool_editor(ui, field_path, key_label, *b, issue, edits);
        }
        serde_json::Value::Null => {
            ui.horizontal(|ui| {
                label_with_issue(ui, key_label, issue);
                ui.colored_label(Color32::GRAY, "(null)");
            });
        }
        serde_json::Value::Object(obj) => {
            if depth >= MAX_DEPTH {
                draw_depth_limit_label(ui, key_label, obj.len(), "keys");
            } else if is_localized_string_object(obj) {
                changed = draw_localized_string_editor(
                    ui,
                    loaded_documents,
                    document_ref,
                    field_path,
                    key_label,
                    obj,
                    depth,
                    edits,
                );
            } else {
                changed = draw_object_editor(
                    ui,
                    loaded_documents,
                    document_ref,
                    field_path,
                    key_label,
                    obj,
                    depth,
                    edits,
                );
            }
        }
        serde_json::Value::Array(arr) => {
            if depth >= MAX_DEPTH {
                draw_depth_limit_label(ui, key_label, arr.len(), "items");
            } else {
                changed = draw_array_editor(
                    ui,
                    loaded_documents,
                    document_ref,
                    field_path,
                    key_label,
                    arr,
                    depth,
                    edits,
                );
            }
        }
    }

    changed
}

fn label_with_issue(ui: &mut egui::Ui, label: &str, issue: Option<&ValidationIssue>) {
    let mut text = RichText::new(label);
    if let Some(issue) = issue {
        text = text.color(validation_issue_color(issue.severity));
    }
    let response = ui.label(text);
    if let Some(issue) = issue {
        response.on_hover_text(format!("{}: {}", issue.code, issue.message));
    }
}

fn draw_depth_limit_label(ui: &mut egui::Ui, key_label: &str, count: usize, unit: &str) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(key_label).strong());
        ui.colored_label(Color32::GRAY, format!("({count} {unit}, depth limit)"));
    });
}

fn draw_string_editor(
    ui: &mut egui::Ui,
    field_path: &str,
    key_label: &str,
    current: &str,
    issue: Option<&ValidationIssue>,
    edits: &mut Vec<PendingEdit>,
) -> bool {
    let mut draft = current.to_string();
    let mut changed = false;

    ui.horizontal(|ui| {
        label_with_issue(ui, key_label, issue);
        let response = ui.add(
            egui::TextEdit::singleline(&mut draft)
                .desired_width(ui.available_width().min(300.0))
                .id_salt(format!("prop:{field_path}")),
        );
        if response.lost_focus() && draft != current {
            edits.push(PendingEdit {
                field_path: field_path.to_string(),
                value: serde_json::Value::String(draft),
            });
            changed = true;
        }
    });
    changed
}

fn draw_number_editor(
    ui: &mut egui::Ui,
    field_path: &str,
    key_label: &str,
    current: &serde_json::Number,
    issue: Option<&ValidationIssue>,
    edits: &mut Vec<PendingEdit>,
) -> bool {
    let current_text = current.to_string();
    let mut draft = current_text.clone();
    let mut changed = false;

    ui.horizontal(|ui| {
        label_with_issue(ui, key_label, issue);
        let response = ui.add(
            egui::TextEdit::singleline(&mut draft)
                .desired_width(80.0)
                .id_salt(format!("prop:{field_path}")),
        );
        if response.lost_focus() && draft != current_text {
            // Try to preserve the original number type.
            if let Ok(n) = draft.trim().parse::<i64>() {
                edits.push(PendingEdit {
                    field_path: field_path.to_string(),
                    value: serde_json::json!(n),
                });
                changed = true;
            } else if let Ok(n) = draft.trim().parse::<f64>() {
                if n.is_finite() {
                    if let Some(num) = serde_json::Number::from_f64(n) {
                        edits.push(PendingEdit {
                            field_path: field_path.to_string(),
                            value: serde_json::Value::Number(num),
                        });
                        changed = true;
                    }
                }
            }
            // If parsing fails, just ignore and keep the old value.
        }
    });
    changed
}

fn draw_bool_editor(
    ui: &mut egui::Ui,
    field_path: &str,
    key_label: &str,
    current: bool,
    issue: Option<&ValidationIssue>,
    edits: &mut Vec<PendingEdit>,
) -> bool {
    let mut draft = current;
    let mut changed = false;

    ui.horizontal(|ui| {
        label_with_issue(ui, key_label, issue);
        if ui.checkbox(&mut draft, "").changed() {
            edits.push(PendingEdit {
                field_path: field_path.to_string(),
                value: serde_json::Value::Bool(draft),
            });
            changed = true;
        }
    });
    changed
}

fn draw_localized_string_editor(
    ui: &mut egui::Ui,
    loaded_documents: &LoadedCustomDocuments,
    document_ref: &DocumentRef,
    field_path: &str,
    key_label: &str,
    obj: &serde_json::Map<String, serde_json::Value>,
    depth: usize,
    edits: &mut Vec<PendingEdit>,
) -> bool {
    let mut changed = false;

    let default_value = obj
        .get(DEFAULT_DISPLAY_LOCALE)
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let default_path = format!("{field_path}.{DEFAULT_DISPLAY_LOCALE}");
    let issue = first_document_field_issue(loaded_documents, document_ref, &default_path);

    let mut draft = default_value.to_string();
    ui.horizontal(|ui| {
        label_with_issue(ui, key_label, issue);
        let response = ui.add(
            egui::TextEdit::singleline(&mut draft)
                .desired_width(ui.available_width().min(300.0))
                .id_salt(format!("prop:{default_path}")),
        );
        if response.lost_focus() && draft != default_value {
            edits.push(PendingEdit {
                field_path: default_path,
                value: serde_json::Value::String(draft),
            });
            changed = true;
        }
    });

    // Show other locales in a collapsible section.
    let other_locales: Vec<_> = obj
        .keys()
        .filter(|k| k.as_str() != DEFAULT_DISPLAY_LOCALE)
        .collect();
    if !other_locales.is_empty() {
        ui.indent(format!("locales:{field_path}"), |ui| {
            for locale_key in other_locales {
                let locale_path = format!("{field_path}.{locale_key}");
                if let Some(locale_value) = obj.get(locale_key) {
                    draw_value_editor(
                        ui,
                        loaded_documents,
                        document_ref,
                        &locale_path,
                        locale_key,
                        locale_value,
                        depth + 1,
                        edits,
                    );
                }
            }
        });
    }

    changed
}

fn draw_object_editor(
    ui: &mut egui::Ui,
    loaded_documents: &LoadedCustomDocuments,
    document_ref: &DocumentRef,
    field_path: &str,
    key_label: &str,
    obj: &serde_json::Map<String, serde_json::Value>,
    depth: usize,
    edits: &mut Vec<PendingEdit>,
) -> bool {
    let mut changed = false;

    let header = format!("{key_label} ({} keys)", obj.len());
    egui::CollapsingHeader::new(header)
        .id_salt(format!("obj:{field_path}"))
        .show(ui, |ui| {
            let mut keys: Vec<&String> = obj.keys().collect();
            keys.sort();
            for key in keys {
                if let Some(child_value) = obj.get(key) {
                    let child_path = format!("{field_path}.{key}");
                    if draw_value_editor(
                        ui,
                        loaded_documents,
                        document_ref,
                        &child_path,
                        key,
                        child_value,
                        depth + 1,
                        edits,
                    ) {
                        changed = true;
                    }
                }
            }
        });

    changed
}

fn draw_array_editor(
    ui: &mut egui::Ui,
    loaded_documents: &LoadedCustomDocuments,
    document_ref: &DocumentRef,
    field_path: &str,
    key_label: &str,
    arr: &[serde_json::Value],
    depth: usize,
    edits: &mut Vec<PendingEdit>,
) -> bool {
    let mut changed = false;

    let header = format!("{key_label} [{} items]", arr.len());
    egui::CollapsingHeader::new(header)
        .id_salt(format!("arr:{field_path}"))
        .show(ui, |ui| {
            for (index, child_value) in arr.iter().enumerate() {
                let child_path = format!("{field_path}[{index}]");
                let child_label = format!("[{index}]");
                if draw_value_editor(
                    ui,
                    loaded_documents,
                    document_ref,
                    &child_path,
                    &child_label,
                    child_value,
                    depth + 1,
                    edits,
                ) {
                    changed = true;
                }
            }
        });

    changed
}
