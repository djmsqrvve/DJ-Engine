use super::scene_io::{resolve_asset_root, save_project_impl};
use super::types::{
    ActiveStoryGraph, BrowserTab, EditorDirtyState, EditorState, EditorUiState, EditorView,
    PendingProjectAction, PendingProjectActionResolution, RuntimePreviewLaunchPhase,
    RuntimePreviewLaunchState, COLOR_PRIMARY, COLOR_SECONDARY,
};
use super::views::{draw_grid, draw_story_graph};
use crate::data::{
    filter_document_refs_by_kind, update_loaded_custom_document_envelope,
    update_loaded_custom_document_raw_json, update_loaded_custom_document_typed,
    CustomDocumentRegistry, DocumentLink, DocumentLinkTarget, DocumentRef, EditorDocumentRoute,
    LoadedCustomDocuments, PreviewProfilePayload, Project,
};
use crate::diagnostics::console::ConsoleLogStore;
use crate::editor::extensions::EditorExtensionRegistry;
use crate::editor::plugin::{
    launch_runtime_preview_from_editor, request_project_action, resolve_pending_project_action,
    stop_runtime_preview_from_editor,
};
use crate::project_mount::MountedProject;
use crate::story_graph::GraphExecutor;
use bevy::prelude::*;
use bevy_egui::egui::{self, Color32, RichText};
use bevy_inspector_egui::bevy_inspector;
use std::fs;
use std::path::Path;

fn log_console(world: &mut World, message: impl Into<String>) {
    if let Some(mut store) = world.get_resource_mut::<ConsoleLogStore>() {
        store.log(message.into());
    }
}

fn save_project_with_feedback(world: &mut World) -> bool {
    match save_project_impl(world) {
        Ok(()) => {
            let message = "Project saved successfully.".to_string();
            log_console(world, &message);
            info!("{message}");
            true
        }
        Err(error) => {
            let message = format!("Failed to save project: {error}");
            log_console(world, &message);
            error!("{message}");
            false
        }
    }
}

fn available_document_kinds(loaded_documents: &LoadedCustomDocuments) -> Vec<String> {
    loaded_documents
        .available_kinds()
        .into_iter()
        .filter(|kind| kind != "all")
        .collect()
}

fn first_document_id_for_kind(loaded_documents: &LoadedCustomDocuments, kind: &str) -> String {
    filter_document_refs_by_kind(loaded_documents, kind, "")
        .into_iter()
        .next()
        .map(|document_ref| document_ref.id)
        .unwrap_or_default()
}

fn default_document_link(
    loaded_documents: &LoadedCustomDocuments,
    preferred_kind: Option<&str>,
) -> DocumentLink {
    let available_kinds = available_document_kinds(loaded_documents);
    let kind = preferred_kind
        .filter(|kind| available_kinds.iter().any(|candidate| candidate == *kind))
        .map(str::to_string)
        .or_else(|| available_kinds.first().cloned())
        .unwrap_or_default();
    let id = if kind.is_empty() {
        String::new()
    } else {
        first_document_id_for_kind(loaded_documents, &kind)
    };

    DocumentLink {
        field_path: "payload.ref".into(),
        target: DocumentLinkTarget::Document { kind, id },
    }
}

fn default_preview_profile_ref(
    loaded_documents: &LoadedCustomDocuments,
    preferred_kind: Option<&str>,
) -> DocumentRef {
    let available_kinds = available_document_kinds(loaded_documents);
    let kind = preferred_kind
        .filter(|kind| available_kinds.iter().any(|candidate| candidate == *kind))
        .map(str::to_string)
        .or_else(|| available_kinds.first().cloned())
        .unwrap_or_default();
    let id = if kind.is_empty() {
        String::new()
    } else {
        first_document_id_for_kind(loaded_documents, &kind)
    };

    DocumentRef { kind, id }
}

fn apply_envelope_update<F>(
    loaded_documents: &mut LoadedCustomDocuments,
    project: &Project,
    registry: &CustomDocumentRegistry,
    selected: &DocumentRef,
    structured_error: &mut Option<String>,
    update: F,
) where
    F: FnOnce(&mut crate::data::CustomDocument<serde_json::Value>),
{
    if let Err(error) = update_loaded_custom_document_envelope(
        loaded_documents,
        project,
        registry,
        &selected.kind,
        &selected.id,
        update,
    ) {
        *structured_error = Some(format!(
            "Failed to apply structured document update: {error}"
        ));
    }
}

fn apply_typed_update<T, F>(
    loaded_documents: &mut LoadedCustomDocuments,
    project: &Project,
    registry: &CustomDocumentRegistry,
    selected: &DocumentRef,
    structured_error: &mut Option<String>,
    update: F,
) where
    T: serde::Serialize + serde::de::DeserializeOwned,
    F: FnOnce(&mut crate::data::CustomDocument<T>),
{
    if let Err(error) = update_loaded_custom_document_typed::<T, _>(
        loaded_documents,
        project,
        registry,
        &selected.kind,
        &selected.id,
        update,
    ) {
        *structured_error = Some(format!("Failed to apply typed document update: {error}"));
    }
}

fn draw_generic_document_metadata_editor(
    ui: &mut egui::Ui,
    loaded_documents: &mut LoadedCustomDocuments,
    project: &Project,
    registry: &CustomDocumentRegistry,
    selected: &DocumentRef,
    document: &crate::data::LoadedCustomDocument,
    structured_error: &mut Option<String>,
) {
    let Some(parsed) = document.document.as_ref() else {
        return;
    };

    ui.separator();
    ui.label(RichText::new("Structured Metadata").strong());

    let mut label = parsed.label.clone().unwrap_or_default();
    ui.label("Label");
    if ui.text_edit_singleline(&mut label).changed() {
        let normalized = label.trim();
        let value = if normalized.is_empty() {
            None
        } else {
            Some(normalized.to_string())
        };
        apply_envelope_update(
            loaded_documents,
            project,
            registry,
            selected,
            structured_error,
            move |document| {
                document.label = value;
            },
        );
    }

    let mut tags = parsed.tags.join(", ");
    ui.label("Tags (comma separated)");
    if ui.text_edit_singleline(&mut tags).changed() {
        let normalized_tags: Vec<String> = tags
            .split(',')
            .map(str::trim)
            .filter(|tag| !tag.is_empty())
            .map(str::to_string)
            .collect();
        apply_envelope_update(
            loaded_documents,
            project,
            registry,
            selected,
            structured_error,
            move |document| {
                document.tags = normalized_tags;
            },
        );
    }
}

fn draw_document_links_editor(
    ui: &mut egui::Ui,
    loaded_documents: &mut LoadedCustomDocuments,
    project: &Project,
    registry: &CustomDocumentRegistry,
    selected: &DocumentRef,
    document: &crate::data::LoadedCustomDocument,
    structured_error: &mut Option<String>,
) {
    let Some(parsed) = document.document.as_ref() else {
        return;
    };

    ui.separator();
    ui.label(RichText::new("Reference Links").strong());

    let available_kinds = available_document_kinds(loaded_documents);
    let mut links = parsed.references.clone();
    let mut links_changed = false;
    let mut remove_index = None;

    for (index, link) in links.iter_mut().enumerate() {
        let before_link = link.clone();
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new(format!("Link {}", index + 1)).strong());
                if ui.small_button("Remove").clicked() {
                    remove_index = Some(index);
                }
            });

            ui.label("Field Path");
            if ui.text_edit_singleline(&mut link.field_path).changed() {
                links_changed = true;
            }

            let mut target_kind_label = match &link.target {
                DocumentLinkTarget::Document { .. } => "document",
                DocumentLinkTarget::Scene { .. } => "scene",
                DocumentLinkTarget::StoryGraph { .. } => "story_graph",
                DocumentLinkTarget::Asset { .. } => "asset",
            }
            .to_string();

            egui::ComboBox::from_id_salt(format!("doc_link_target_type_{index}"))
                .selected_text(target_kind_label.clone())
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut target_kind_label, "document".to_string(), "document");
                    ui.selectable_value(&mut target_kind_label, "scene".to_string(), "scene");
                    ui.selectable_value(
                        &mut target_kind_label,
                        "story_graph".to_string(),
                        "story_graph",
                    );
                    ui.selectable_value(&mut target_kind_label, "asset".to_string(), "asset");
                });

            let current_target_label = match &link.target {
                DocumentLinkTarget::Document { .. } => "document",
                DocumentLinkTarget::Scene { .. } => "scene",
                DocumentLinkTarget::StoryGraph { .. } => "story_graph",
                DocumentLinkTarget::Asset { .. } => "asset",
            };

            if target_kind_label != current_target_label {
                link.target = match target_kind_label.as_str() {
                    "scene" => DocumentLinkTarget::Scene {
                        id: project
                            .scenes
                            .first()
                            .map(|scene_ref| scene_ref.id.clone())
                            .unwrap_or_default(),
                    },
                    "story_graph" => DocumentLinkTarget::StoryGraph {
                        id: project
                            .story_graphs
                            .first()
                            .map(|graph_ref| graph_ref.id.clone())
                            .unwrap_or_default(),
                    },
                    "asset" => DocumentLinkTarget::Asset {
                        path: String::new(),
                    },
                    _ => default_document_link(loaded_documents, Some(&selected.kind)).target,
                };
                links_changed = true;
            }

            match &mut link.target {
                DocumentLinkTarget::Document { kind, id } => {
                    egui::ComboBox::from_id_salt(format!("doc_link_kind_{index}"))
                        .selected_text(if kind.is_empty() {
                            "<kind>".to_string()
                        } else {
                            kind.clone()
                        })
                        .show_ui(ui, |ui| {
                            for available_kind in &available_kinds {
                                ui.selectable_value(kind, available_kind.clone(), available_kind);
                            }
                        });

                    let id_options = if kind.is_empty() {
                        Vec::new()
                    } else {
                        filter_document_refs_by_kind(loaded_documents, kind, "")
                    };
                    if id_options.iter().all(|candidate| candidate.id != *id) {
                        *id = id_options
                            .first()
                            .map(|candidate| candidate.id.clone())
                            .unwrap_or_default();
                        links_changed = true;
                    }

                    egui::ComboBox::from_id_salt(format!("doc_link_id_{index}"))
                        .selected_text(if id.is_empty() {
                            "<id>".to_string()
                        } else {
                            id.clone()
                        })
                        .show_ui(ui, |ui| {
                            for option in id_options {
                                ui.selectable_value(id, option.id.clone(), option.id);
                            }
                        });
                }
                DocumentLinkTarget::Scene { id } => {
                    egui::ComboBox::from_id_salt(format!("doc_link_scene_{index}"))
                        .selected_text(if id.is_empty() {
                            "<scene>".to_string()
                        } else {
                            id.clone()
                        })
                        .show_ui(ui, |ui| {
                            for scene_ref in &project.scenes {
                                ui.selectable_value(id, scene_ref.id.clone(), scene_ref.id.clone());
                            }
                        });
                }
                DocumentLinkTarget::StoryGraph { id } => {
                    egui::ComboBox::from_id_salt(format!("doc_link_story_graph_{index}"))
                        .selected_text(if id.is_empty() {
                            "<story graph>".to_string()
                        } else {
                            id.clone()
                        })
                        .show_ui(ui, |ui| {
                            for graph_ref in &project.story_graphs {
                                ui.selectable_value(id, graph_ref.id.clone(), graph_ref.id.clone());
                            }
                        });
                }
                DocumentLinkTarget::Asset { path } => {
                    ui.label("Asset Path");
                    if ui.text_edit_singleline(path).changed() {
                        links_changed = true;
                    }
                }
            }
        });
        if *link != before_link {
            links_changed = true;
        }
        ui.add_space(4.0);
    }

    if let Some(index) = remove_index {
        links.remove(index);
        links_changed = true;
    }

    ui.horizontal(|ui| {
        if ui.button("Add Document Link").clicked() {
            links.push(default_document_link(
                loaded_documents,
                Some(&selected.kind),
            ));
            links_changed = true;
        }
        if ui.button("Add Scene Link").clicked() {
            links.push(DocumentLink {
                field_path: "payload.scene_id".into(),
                target: DocumentLinkTarget::Scene {
                    id: project
                        .scenes
                        .first()
                        .map(|scene_ref| scene_ref.id.clone())
                        .unwrap_or_default(),
                },
            });
            links_changed = true;
        }
        if ui.button("Add Story Graph Link").clicked() {
            links.push(DocumentLink {
                field_path: "payload.story_graph_id".into(),
                target: DocumentLinkTarget::StoryGraph {
                    id: project
                        .story_graphs
                        .first()
                        .map(|graph_ref| graph_ref.id.clone())
                        .unwrap_or_default(),
                },
            });
            links_changed = true;
        }
        if ui.button("Add Asset Link").clicked() {
            links.push(DocumentLink {
                field_path: "payload.asset".into(),
                target: DocumentLinkTarget::Asset {
                    path: String::new(),
                },
            });
            links_changed = true;
        }
    });

    if links_changed {
        apply_envelope_update(
            loaded_documents,
            project,
            registry,
            selected,
            structured_error,
            move |document| {
                document.references = links;
            },
        );
    }
}

fn draw_preview_profile_editor(
    ui: &mut egui::Ui,
    loaded_documents: &mut LoadedCustomDocuments,
    project: &Project,
    registry: &CustomDocumentRegistry,
    selected: &DocumentRef,
    structured_error: &mut Option<String>,
) {
    let Ok(Some(document)) =
        loaded_documents.get_typed::<PreviewProfilePayload>(&selected.kind, &selected.id)
    else {
        return;
    };

    ui.separator();
    ui.label(RichText::new("Preview Profile").strong());

    let mut scene_id = document.payload.scene_id.clone().unwrap_or_default();
    ui.horizontal(|ui| {
        ui.label("Startup Scene");
        egui::ComboBox::from_id_salt("preview_profile_scene_id")
            .selected_text(if scene_id.is_empty() {
                "<none>".to_string()
            } else {
                scene_id.clone()
            })
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut scene_id, String::new(), "<none>");
                for scene_ref in &project.scenes {
                    ui.selectable_value(&mut scene_id, scene_ref.id.clone(), scene_ref.id.clone());
                }
            });
    });
    if scene_id != document.payload.scene_id.clone().unwrap_or_default() {
        let scene_value = if scene_id.is_empty() {
            None
        } else {
            Some(scene_id)
        };
        apply_typed_update::<PreviewProfilePayload, _>(
            loaded_documents,
            project,
            registry,
            selected,
            structured_error,
            move |document| {
                document.payload.scene_id = scene_value;
            },
        );
    }

    let mut story_graph_id = document.payload.story_graph_id.clone().unwrap_or_default();
    ui.horizontal(|ui| {
        ui.label("Startup Story Graph");
        egui::ComboBox::from_id_salt("preview_profile_story_graph_id")
            .selected_text(if story_graph_id.is_empty() {
                "<none>".to_string()
            } else {
                story_graph_id.clone()
            })
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut story_graph_id, String::new(), "<none>");
                for graph_ref in &project.story_graphs {
                    ui.selectable_value(
                        &mut story_graph_id,
                        graph_ref.id.clone(),
                        graph_ref.id.clone(),
                    );
                }
            });
    });
    if story_graph_id != document.payload.story_graph_id.clone().unwrap_or_default() {
        let story_graph_value = if story_graph_id.is_empty() {
            None
        } else {
            Some(story_graph_id)
        };
        apply_typed_update::<PreviewProfilePayload, _>(
            loaded_documents,
            project,
            registry,
            selected,
            structured_error,
            move |document| {
                document.payload.story_graph_id = story_graph_value;
            },
        );
    }

    ui.label(RichText::new("Custom Document Bundle").strong());
    let available_kinds = available_document_kinds(loaded_documents);
    let mut refs = document.payload.document_refs.clone();
    let mut refs_changed = false;
    let mut remove_index = None;

    for (index, document_ref) in refs.iter_mut().enumerate() {
        let before_ref = document_ref.clone();
        ui.horizontal(|ui| {
            egui::ComboBox::from_id_salt(format!("preview_profile_doc_kind_{index}"))
                .selected_text(if document_ref.kind.is_empty() {
                    "<kind>".to_string()
                } else {
                    document_ref.kind.clone()
                })
                .show_ui(ui, |ui| {
                    for kind in &available_kinds {
                        ui.selectable_value(&mut document_ref.kind, kind.clone(), kind);
                    }
                });

            let id_options = if document_ref.kind.is_empty() {
                Vec::new()
            } else {
                filter_document_refs_by_kind(loaded_documents, &document_ref.kind, "")
            };
            if id_options
                .iter()
                .all(|candidate| candidate.id != document_ref.id)
            {
                document_ref.id = id_options
                    .first()
                    .map(|candidate| candidate.id.clone())
                    .unwrap_or_default();
                refs_changed = true;
            }

            egui::ComboBox::from_id_salt(format!("preview_profile_doc_id_{index}"))
                .selected_text(if document_ref.id.is_empty() {
                    "<id>".to_string()
                } else {
                    document_ref.id.clone()
                })
                .show_ui(ui, |ui| {
                    for option in id_options {
                        ui.selectable_value(
                            &mut document_ref.id,
                            option.id.clone(),
                            option.id.clone(),
                        );
                    }
                });

            if ui.small_button("Remove").clicked() {
                remove_index = Some(index);
            }
        });
        if *document_ref != before_ref {
            refs_changed = true;
        }
    }

    if let Some(index) = remove_index {
        refs.remove(index);
        refs_changed = true;
    }

    if ui.button("Add Document To Bundle").clicked() {
        refs.push(default_preview_profile_ref(loaded_documents, None));
        refs_changed = true;
    }

    if refs_changed {
        apply_typed_update::<PreviewProfilePayload, _>(
            loaded_documents,
            project,
            registry,
            selected,
            structured_error,
            move |document| {
                document.payload.document_refs = refs;
            },
        );
    }
}

pub(crate) fn draw_top_menu(ui: &mut egui::Ui, world: &mut World) {
    let has_mounted_project = world.resource::<MountedProject>().manifest_path.is_some();
    let has_loaded_project = world.resource::<MountedProject>().project.is_some();
    let current_state = world.resource::<State<EditorState>>().get().clone();
    let dirty_state = world.resource::<EditorDirtyState>();
    let is_dirty = dirty_state.is_dirty;
    let snapshot_error = dirty_state.snapshot_error.clone();
    let runtime_preview = world.resource::<RuntimePreviewLaunchState>();
    let preview_is_running = runtime_preview.is_running();
    let preview_status = runtime_preview.status_message.clone();
    let preview_phase = runtime_preview.phase.clone();
    let preview_last_exit = runtime_preview.last_exit.clone();

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 2.0;
        ui.label(
            RichText::new("DJ")
                .color(COLOR_PRIMARY)
                .strong()
                .size(20.0)
                .italics(),
        );
        ui.label(
            RichText::new("ENGINE")
                .color(COLOR_SECONDARY)
                .strong()
                .size(20.0),
        );

        ui.add_space(10.0);
        ui.separator();
        ui.add_space(10.0);

        ui.menu_button("File", |ui| {
            if ui.button("💾 Save Project").clicked() {
                save_project_with_feedback(world);
                ui.close();
            }
            if ui
                .add_enabled(
                    has_mounted_project,
                    egui::Button::new("📂 Load Mounted Project"),
                )
                .clicked()
            {
                request_project_action(world, PendingProjectAction::LoadMountedProject);
                ui.close();
            }
            if ui
                .add_enabled(has_mounted_project, egui::Button::new("🔄 Reload Project"))
                .clicked()
            {
                request_project_action(world, PendingProjectAction::ReloadProject);
                ui.close();
            }
            if !has_mounted_project {
                ui.separator();
                ui.label("Mount a project with --project <dir|project.json>.");
            }
        });

        ui.add_space(10.0);
        ui.separator();
        ui.add_space(10.0);

        {
            let mut ui_state = world.resource_mut::<EditorUiState>();
            ui.selectable_value(
                &mut ui_state.current_view,
                EditorView::Level,
                RichText::new("🌍 Level Editor").strong(),
            );
            ui.selectable_value(
                &mut ui_state.current_view,
                EditorView::StoryGraph,
                RichText::new("🕸 Story Graph").strong(),
            );
        }

        if current_state == EditorState::GraphPreview
            && world.resource::<EditorUiState>().current_view != EditorView::StoryGraph
        {
            world
                .resource_mut::<NextState<EditorState>>()
                .set(EditorState::Editor);
            let message = "Graph preview stopped after leaving Story Graph view.".to_string();
            log_console(world, &message);
            info!("{message}");
        }

        ui.add_space(10.0);
        ui.separator();
        ui.add_space(10.0);

        if ui
            .add_enabled(
                has_loaded_project && !preview_is_running,
                egui::Button::new(RichText::new("▶ RUN PROJECT").color(COLOR_PRIMARY)),
            )
            .clicked()
        {
            launch_runtime_preview_from_editor(world);
        }
        if ui
            .add_enabled(
                preview_is_running,
                egui::Button::new(RichText::new("⏹ STOP PREVIEW").color(COLOR_SECONDARY)),
            )
            .clicked()
        {
            stop_runtime_preview_from_editor(world);
        }

        if world.resource::<EditorUiState>().current_view == EditorView::StoryGraph {
            ui.add_space(10.0);
            ui.separator();
            ui.add_space(10.0);

            let graph_preview_active = current_state == EditorState::GraphPreview;
            if ui
                .add_enabled(
                    !graph_preview_active,
                    egui::Button::new(RichText::new("Preview Graph").color(COLOR_PRIMARY)),
                )
                .clicked()
            {
                world.resource_scope::<ActiveStoryGraph, _>(|world, graph| {
                    if let Some(mut executor) = world.get_resource_mut::<GraphExecutor>() {
                        executor.load_from_data(&graph.0);
                        info!("Editor: Loaded Story Graph into GraphExecutor");
                    }
                });
                world
                    .resource_mut::<NextState<EditorState>>()
                    .set(EditorState::GraphPreview);
                let message = "Graph preview started.".to_string();
                log_console(world, &message);
                info!("{message}");
            }
            if ui
                .add_enabled(
                    graph_preview_active,
                    egui::Button::new(RichText::new("Stop Graph Preview").color(COLOR_SECONDARY)),
                )
                .clicked()
            {
                world
                    .resource_mut::<NextState<EditorState>>()
                    .set(EditorState::Editor);
                let message = "Graph preview stopped.".to_string();
                log_console(world, &message);
                info!("{message}");
            }
        }

        ui.add_space(10.0);
        ui.separator();

        let mounted_project = world.resource::<MountedProject>();
        let project_name = mounted_project
            .project
            .as_ref()
            .map(|project| project.name.clone())
            .unwrap_or_else(|| "No Project Mounted".to_string());
        let manifest_label = mounted_project
            .manifest_path
            .as_ref()
            .map(|path| path.display().to_string())
            .unwrap_or_else(|| "No manifest mounted".to_string());

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let mut ui_state = world.resource_mut::<EditorUiState>();
            if ui
                .selectable_label(ui_state.console_open, "💻 Console")
                .clicked()
            {
                ui_state.console_open = !ui_state.console_open;
            }
            ui.separator();
            if is_dirty {
                ui.label(RichText::new("DIRTY").color(COLOR_SECONDARY).strong());
                ui.separator();
            }
            if let Some(last_exit) = &preview_last_exit {
                ui.label(
                    RichText::new(format!("Last Exit: {last_exit}"))
                        .italics()
                        .color(Color32::GRAY),
                );
                ui.separator();
            }
            if let Some(status) = &preview_status {
                let status_color = match preview_phase {
                    RuntimePreviewLaunchPhase::Running => COLOR_PRIMARY,
                    RuntimePreviewLaunchPhase::Failed => Color32::RED,
                    RuntimePreviewLaunchPhase::Stopping => COLOR_SECONDARY,
                    _ => Color32::GRAY,
                };
                ui.label(RichText::new(status.clone()).italics().color(status_color));
                ui.separator();
            }
            if let Some(snapshot_error) = &snapshot_error {
                ui.label(
                    RichText::new(snapshot_error.clone())
                        .italics()
                        .color(Color32::RED),
                );
                ui.separator();
            }
            ui.label(RichText::new(manifest_label).italics().color(Color32::GRAY));
            ui.separator();
            ui.label(
                RichText::new(format!("Active: {}", project_name))
                    .italics()
                    .color(if has_loaded_project {
                        Color32::GRAY
                    } else {
                        COLOR_SECONDARY
                    }),
            );
        });
    });
}

pub(crate) fn draw_pending_project_action_window(ctx: &egui::Context, world: &mut World) {
    let pending_action = world
        .resource::<EditorDirtyState>()
        .pending_project_action
        .clone();
    let Some(pending_action) = pending_action else {
        return;
    };

    let action_label = match pending_action {
        PendingProjectAction::LoadMountedProject => "load the mounted project",
        PendingProjectAction::ReloadProject => "reload the mounted project",
    };

    egui::Window::new("Unsaved Changes")
        .collapsible(false)
        .resizable(false)
        .default_size(egui::vec2(420.0, 0.0))
        .show(ctx, |ui| {
            ui.label(format!(
                "You have unsaved changes. Do you want to save before I {action_label}?"
            ));
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                if ui.button("Save and Continue").clicked() {
                    resolve_pending_project_action(
                        world,
                        PendingProjectActionResolution::SaveAndContinue,
                    );
                }
                if ui.button("Discard Changes").clicked() {
                    resolve_pending_project_action(
                        world,
                        PendingProjectActionResolution::DiscardChanges,
                    );
                }
                if ui.button("Cancel").clicked() {
                    resolve_pending_project_action(world, PendingProjectActionResolution::Cancel);
                }
            });
        });
}

pub(crate) fn draw_left_panel(ui: &mut egui::Ui, world: &mut World) {
    world.resource_scope::<EditorUiState, _>(|world, mut ui_state| {
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.selectable_value(&mut ui_state.browser_tab, BrowserTab::Palette, "Palette");
            ui.selectable_value(
                &mut ui_state.browser_tab,
                BrowserTab::Hierarchy,
                "Hierarchy",
            );
            ui.selectable_value(&mut ui_state.browser_tab, BrowserTab::Assets, "Files");
            ui.selectable_value(&mut ui_state.browser_tab, BrowserTab::Documents, "Docs");
        });
        ui.add_space(4.0);
        ui.separator();

        match ui_state.browser_tab {
            BrowserTab::Hierarchy => {
                ui.add_space(5.0);
                ui.label(
                    RichText::new("SCENE HIERARCHY")
                        .strong()
                        .color(COLOR_PRIMARY),
                );
                ui.add_space(5.0);
                bevy_inspector::hierarchy::hierarchy_ui(world, ui, &mut ui_state.selected_entities);
            }
            BrowserTab::Assets => {
                ui.add_space(5.0);
                ui.label(RichText::new("ASSET BROWSER").strong().color(COLOR_PRIMARY));
                ui.add_space(5.0);
                let mut query = ui_state.asset_search_query.clone();
                ui.text_edit_singleline(&mut query);
                ui_state.asset_search_query = query;

                ui.separator();
                egui::ScrollArea::vertical().show(ui, |ui| {
                    let mounted_project = world.resource::<MountedProject>().clone();
                    if let Some(asset_root) = resolve_asset_root(&mounted_project) {
                        let entries =
                            collect_asset_entries(&asset_root, &ui_state.asset_search_query);
                        if entries.is_empty() {
                            ui.label(
                                RichText::new("No matching assets found.")
                                    .italics()
                                    .color(Color32::GRAY),
                            );
                        } else {
                            for entry in entries {
                                ui.label(entry);
                            }
                        }
                    } else {
                        ui.label(
                            RichText::new("No mounted project assets available.")
                                .italics()
                                .color(Color32::GRAY),
                        );
                    }
                });
            }
            BrowserTab::Palette => {
                ui.add_space(5.0);
                ui.label(RichText::new("TOOL PALETTE").strong().color(COLOR_PRIMARY));
                ui.add_space(5.0);
                ui.label(RichText::new("Select item to paint:").italics());

                let mut selected = ui_state.selected_palette_item.clone();

                ui.add_space(5.0);
                ui.selectable_value(
                    &mut selected,
                    Some("Terrain".to_string()),
                    "🌿 Terrain Tile",
                );
                ui.selectable_value(
                    &mut selected,
                    Some("Blocker".to_string()),
                    "🧱 Collision Block",
                );
                ui.selectable_value(&mut selected, Some("Actor".to_string()), "🙂 Actor Marker");
                ui.selectable_value(&mut selected, Some("Prop".to_string()), "📦 Prop Marker");

                ui.add_space(10.0);
                if ui
                    .button(RichText::new("❌ Clear Selection").color(COLOR_SECONDARY))
                    .clicked()
                {
                    selected = None;
                }

                ui_state.selected_palette_item = selected;
            }
            BrowserTab::Documents => {
                ui.add_space(5.0);
                ui.label(
                    RichText::new("CUSTOM DOCUMENTS")
                        .strong()
                        .color(COLOR_PRIMARY),
                );
                ui.add_space(5.0);
                ui.label(RichText::new("Search").italics());
                ui.text_edit_singleline(&mut ui_state.custom_document_search_query);

                let loaded_documents = world.resource::<LoadedCustomDocuments>().clone();
                let available_kinds = loaded_documents.available_kinds();
                if ui_state.custom_document_kind_filter.is_empty() {
                    ui_state.custom_document_kind_filter = "all".into();
                }

                egui::ComboBox::from_label("Kind")
                    .selected_text(ui_state.custom_document_kind_filter.clone())
                    .show_ui(ui, |ui| {
                        for kind in available_kinds {
                            ui.selectable_value(
                                &mut ui_state.custom_document_kind_filter,
                                kind.clone(),
                                kind,
                            );
                        }
                    });

                ui.separator();
                egui::ScrollArea::vertical().show(ui, |ui| {
                    let search_query = ui_state.custom_document_search_query.to_lowercase();
                    let selected_kind = ui_state.custom_document_kind_filter.clone();

                    if loaded_documents.documents.is_empty() {
                        ui.label(
                            RichText::new("No custom documents discovered.")
                                .italics()
                                .color(Color32::GRAY),
                        );
                        return;
                    }

                    for document in &loaded_documents.documents {
                        let kind_matches =
                            selected_kind == "all" || document.entry.kind == selected_kind;
                        let search_matches = search_query.is_empty()
                            || document.entry.id.to_lowercase().contains(&search_query)
                            || document.entry.kind.to_lowercase().contains(&search_query);
                        if !kind_matches || !search_matches {
                            continue;
                        }

                        let issue_count = loaded_documents
                            .issues_for(&document.entry.kind, &document.entry.id)
                            .len();
                        let selected = ui_state.selected_custom_document.as_ref()
                            == Some(&DocumentRef {
                                kind: document.entry.kind.clone(),
                                id: document.entry.id.clone(),
                            });
                        let label = format!(
                            "{}:{}  [{:?}]{}",
                            document.entry.kind,
                            document.entry.id,
                            document.resolved_route,
                            if issue_count > 0 {
                                format!("  ({} issues)", issue_count)
                            } else {
                                String::new()
                            }
                        );

                        if ui.selectable_label(selected, label).clicked() {
                            ui_state.selected_custom_document = Some(DocumentRef {
                                kind: document.entry.kind.clone(),
                                id: document.entry.id.clone(),
                            });
                        }
                    }
                });
            }
        }
    });
}

pub(crate) fn draw_right_panel(ui: &mut egui::Ui, world: &mut World) {
    ui.add_space(5.0);
    ui.label(RichText::new("INSPECTOR").strong().color(COLOR_PRIMARY));
    ui.add_space(5.0);
    ui.separator();

    // Check if we are in Story Graph mode and have a selected node
    let story_node_selected = {
        let state = world.resource::<EditorUiState>();
        if state.current_view == EditorView::StoryGraph {
            state.selected_node_id.clone()
        } else {
            None
        }
    };

    if let Some(node_id) = story_node_selected {
        // Edit Story Node
        world.resource_scope::<ActiveStoryGraph, _>(|_, mut graph| {
            if let Some(node) = graph.0.nodes.iter_mut().find(|n| n.id == node_id) {
                ui.label(RichText::new(format!("Node: {}", node.id)).strong());
                ui.separator();

                ui.label("Position");
                ui.horizontal(|ui| {
                    ui.label("X:");
                    ui.add(egui::DragValue::new(&mut node.position.x));
                    ui.label("Y:");
                    ui.add(egui::DragValue::new(&mut node.position.y));
                });

                ui.separator();
                ui.label("Properties");

                use crate::data::story::nodes::StoryNodeVariant;
                match &mut node.data {
                    StoryNodeVariant::Start(_) => {
                        ui.label("Start Node (Entry Point)");
                    }
                    StoryNodeVariant::Dialogue(d) => {
                        ui.label("Speaker:");
                        ui.text_edit_singleline(&mut d.speaker_id);
                        ui.label("Text (EN):");
                        let mut text = d.text.get("en").cloned().unwrap_or_default();
                        if ui.text_edit_multiline(&mut text).changed() {
                            d.text.insert("en".to_string(), text);
                        }
                    }
                    StoryNodeVariant::End(e) => {
                        ui.label("Target Scene ID:");
                        let mut scene = e.target_scene_id.clone().unwrap_or_default();
                        if ui.text_edit_singleline(&mut scene).changed() {
                            e.target_scene_id = if scene.is_empty() { None } else { Some(scene) };
                        }
                    }
                    _ => {
                        ui.label("Not implemented in inspector yet.");
                    }
                }
            }
        });
        return;
    }

    let selected_custom_document = world
        .resource::<EditorUiState>()
        .selected_custom_document
        .clone();
    if let Some(selected_custom_document) = selected_custom_document {
        let mounted_project = world.resource::<MountedProject>().clone();
        let project = mounted_project.project.clone();
        let registry = world
            .get_resource::<CustomDocumentRegistry>()
            .cloned()
            .unwrap_or_default();
        let extensions = world
            .get_resource::<EditorExtensionRegistry>()
            .cloned()
            .unwrap_or_default();

        world.resource_scope::<LoadedCustomDocuments, _>(|_, mut loaded_documents| {
            let Some(document) = loaded_documents
                .get(&selected_custom_document.kind, &selected_custom_document.id)
                .cloned()
            else {
                ui.label(
                    RichText::new("Selected custom document could not be found.")
                        .italics()
                        .color(Color32::RED),
                );
                return;
            };

            ui.label(
                RichText::new(format!(
                    "{}:{}",
                    selected_custom_document.kind, selected_custom_document.id
                ))
                .strong(),
            );
            ui.separator();
            ui.label(format!("Route: {:?}", document.resolved_route));
            ui.label(format!("Registry Path: {}", document.entry.path));
            ui.label(format!("Schema Version: {}", document.entry.schema_version));
            ui.label(format!(
                "Issue Count: {}",
                loaded_documents
                    .issues_for(&selected_custom_document.kind, &selected_custom_document.id)
                    .len()
            ));

            if let Some(parse_error) = &document.parse_error {
                ui.colored_label(Color32::RED, format!("Parse Error: {}", parse_error));
            }

            if !document.entry.tags.is_empty() {
                ui.label(format!("Tags: {}", document.entry.tags.join(", ")));
            }

            let mut structured_error = None;
            if let Some(project) = project.as_ref() {
                draw_generic_document_metadata_editor(
                    ui,
                    &mut loaded_documents,
                    project,
                    &registry,
                    &selected_custom_document,
                    &document,
                    &mut structured_error,
                );
                draw_document_links_editor(
                    ui,
                    &mut loaded_documents,
                    project,
                    &registry,
                    &selected_custom_document,
                    &document,
                    &mut structured_error,
                );
                if document.kind() == "preview_profiles" {
                    draw_preview_profile_editor(
                        ui,
                        &mut loaded_documents,
                        project,
                        &registry,
                        &selected_custom_document,
                        &mut structured_error,
                    );
                }
            }

            if let Some(structured_error) = structured_error {
                ui.colored_label(Color32::RED, structured_error);
            }

            if document.resolved_route == EditorDocumentRoute::CustomPanel {
                let matching_panels: Vec<_> = extensions
                    .custom_panels
                    .iter()
                    .filter(|panel| panel.kind == selected_custom_document.kind)
                    .collect();
                ui.separator();
                ui.label(RichText::new("Registered Custom Panels").strong());
                if matching_panels.is_empty() {
                    ui.label(
                        RichText::new("No custom panels registered for this kind yet.")
                            .italics()
                            .color(Color32::GRAY),
                    );
                } else {
                    for panel in matching_panels {
                        ui.label(format!("• {} ({})", panel.title, panel.panel_id));
                    }
                }
            }

            if let Some(parsed) = &document.document {
                if !parsed.references.is_empty() {
                    ui.separator();
                    ui.label(RichText::new("References").strong());
                    for link in &parsed.references {
                        let target = match &link.target {
                            DocumentLinkTarget::Document { kind, id } => {
                                format!("document {}:{}", kind, id)
                            }
                            DocumentLinkTarget::Scene { id } => format!("scene {}", id),
                            DocumentLinkTarget::StoryGraph { id } => {
                                format!("story graph {}", id)
                            }
                            DocumentLinkTarget::Asset { path } => format!("asset {}", path),
                        };
                        ui.label(format!("{} -> {}", link.field_path, target));
                    }
                }
            }

            let reference_suggestions =
                filter_document_refs_by_kind(&loaded_documents, &selected_custom_document.kind, "");
            if !reference_suggestions.is_empty() {
                ui.separator();
                ui.label(RichText::new("Same-Kind Reference Suggestions").strong());
                for document_ref in reference_suggestions.into_iter().take(6) {
                    ui.label(format!("{}:{}", document_ref.kind, document_ref.id));
                }
            }

            ui.separator();
            ui.label(RichText::new("Raw Document").strong());
            let mut raw_json = document.raw_json.clone();
            let response = ui.add(
                egui::TextEdit::multiline(&mut raw_json)
                    .desired_width(f32::INFINITY)
                    .desired_rows(20),
            );
            if response.changed() {
                if let Some(project) = project.as_ref() {
                    update_loaded_custom_document_raw_json(
                        &mut loaded_documents,
                        project,
                        &registry,
                        &selected_custom_document.kind,
                        &selected_custom_document.id,
                        raw_json,
                    );
                } else if let Some(selected) = loaded_documents
                    .get_mut(&selected_custom_document.kind, &selected_custom_document.id)
                {
                    selected.raw_json = raw_json;
                }
            }

            let issues = loaded_documents
                .issues_for(&selected_custom_document.kind, &selected_custom_document.id);
            if !issues.is_empty() {
                ui.separator();
                ui.label(RichText::new("Validation Issues").strong());
                for issue in issues {
                    let color = match issue.severity {
                        crate::data::ValidationSeverity::Error => Color32::RED,
                        crate::data::ValidationSeverity::Warning => COLOR_SECONDARY,
                        crate::data::ValidationSeverity::Info => Color32::GRAY,
                    };
                    ui.colored_label(color, format!("{}: {}", issue.code, issue.message));
                }
            }
        });
        return;
    }

    world.resource_scope::<EditorUiState, _>(|world, ui_state| {
        if ui_state.selected_entities.is_empty() {
            ui.add_space(10.0);
            ui.label(
                RichText::new("No entity selected.")
                    .italics()
                    .color(Color32::GRAY),
            );
            ui.add_space(10.0);
            ui.separator();
            ui.collapsing("Global Resources", |ui| {
                bevy_inspector::ui_for_resources(world, ui);
            });
        } else {
            bevy_inspector::ui_for_entities_shared_components(
                world,
                ui_state.selected_entities.as_slice(),
                ui,
            );
        }
    });
}

pub(crate) fn draw_console_window(ctx: &egui::Context, world: &mut World) {
    let mut open = true;
    egui::Window::new(RichText::new("CONSOLE").color(COLOR_PRIMARY))
        .open(&mut open)
        .default_size(egui::vec2(600.0, 300.0))
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("System Logs").strong());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .button(RichText::new("Clear").color(Color32::GRAY))
                        .clicked()
                    {
                        if let Some(mut store) = world.get_resource_mut::<ConsoleLogStore>() {
                            store.logs.clear();
                        }
                    }
                });
            });
            ui.separator();

            egui::ScrollArea::vertical()
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    if let Some(store) = world.get_resource::<ConsoleLogStore>() {
                        for log in &store.logs {
                            let color = if log.contains("TEST:") || log.contains("Passed") {
                                COLOR_PRIMARY
                            } else if log.contains("WARN") {
                                COLOR_SECONDARY
                            } else if log.contains("ERROR") {
                                Color32::RED
                            } else {
                                Color32::LIGHT_GRAY
                            };
                            ui.label(RichText::new(log).color(color).monospace());
                        }
                    } else {
                        ui.label("ConsoleLogStore resource missing.");
                    }
                });
        });

    // Update state if window closed
    if !open {
        world.resource_mut::<EditorUiState>().console_open = false;
    }
}

fn collect_asset_entries(asset_root: &Path, search_query: &str) -> Vec<String> {
    if !asset_root.exists() {
        return Vec::new();
    }

    let mut entries = Vec::new();
    let normalized_query = search_query.trim().to_lowercase();
    collect_asset_entries_recursive(asset_root, asset_root, &normalized_query, &mut entries);
    entries.sort();
    entries
}

fn collect_asset_entries_recursive(
    asset_root: &Path,
    current_path: &Path,
    search_query: &str,
    entries: &mut Vec<String>,
) {
    let Ok(read_dir) = fs::read_dir(current_path) else {
        return;
    };

    let mut children: Vec<_> = read_dir.filter_map(Result::ok).collect();
    children.sort_by_key(|entry| entry.path());

    for child in children {
        let path = child.path();
        let Ok(relative_path) = path.strip_prefix(asset_root) else {
            continue;
        };
        let relative_string = relative_path.display().to_string();
        let matches_query =
            search_query.is_empty() || relative_string.to_lowercase().contains(search_query);

        if matches_query {
            let prefix = if path.is_dir() { "📁" } else { "📄" };
            entries.push(format!("{prefix} {relative_string}"));
        }

        if path.is_dir() {
            collect_asset_entries_recursive(asset_root, &path, search_query, entries);
        }
    }
}

pub(crate) fn draw_central_panel(ui: &mut egui::Ui, world: &mut World) {
    let ui_state = world.resource::<EditorUiState>();

    match ui_state.current_view {
        EditorView::Level => {
            let state = world.resource::<State<EditorState>>().get();
            if *state == EditorState::Editor {
                draw_grid(ui, world);
            } else {
                ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
                    ui.label(RichText::new("● GRAPH PREVIEW").color(Color32::RED).small());
                });
            }
        }
        EditorView::StoryGraph => {
            draw_story_graph(ui, world);
        }
    }
}
