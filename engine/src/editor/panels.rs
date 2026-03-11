use super::scene_io::{load_mounted_project, resolve_asset_root, save_project_impl};
use super::types::{
    ActiveStoryGraph, BrowserTab, EditorState, EditorUiState, EditorView, LoadedProject,
    COLOR_PRIMARY, COLOR_SECONDARY,
};
use super::views::{draw_grid, draw_story_graph};
use crate::diagnostics::console::ConsoleLogStore;
use crate::story_graph::GraphExecutor;
use bevy::prelude::*;
use bevy_egui::egui::{self, Color32, RichText};
use bevy_inspector_egui::bevy_inspector;
use std::fs;
use std::path::Path;

pub(crate) fn draw_top_menu(ui: &mut egui::Ui, world: &mut World) {
    let has_mounted_project = world.resource::<LoadedProject>().manifest_path.is_some();
    let has_loaded_project = world.resource::<LoadedProject>().project.is_some();

    ui.horizontal(|ui| {
        // Logo with Cyberpunk colors
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

        // FILE MENU
        ui.menu_button("File", |ui| {
            if ui.button("💾 Save Project").clicked() {
                save_project_impl(world);
                ui.close();
            }
            if ui
                .add_enabled(
                    has_mounted_project,
                    egui::Button::new("📂 Load Mounted Project"),
                )
                .clicked()
            {
                if let Err(error) = load_mounted_project(world) {
                    error!("Failed to load mounted project: {}", error);
                }
                ui.close();
            }
            if ui
                .add_enabled(has_mounted_project, egui::Button::new("🔄 Reload Project"))
                .clicked()
            {
                if let Err(error) = load_mounted_project(world) {
                    error!("Failed to reload mounted project: {}", error);
                }
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

        // View Switcher Tabs
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

        ui.add_space(10.0);
        ui.separator();
        ui.add_space(10.0);

        // Play Controls
        let current_state = world.resource::<State<EditorState>>().get().clone();
        let is_playing = current_state == EditorState::Playing;

        if ui
            .add_enabled(
                !is_playing,
                egui::Button::new(RichText::new("▶ PLAY").color(COLOR_PRIMARY)),
            )
            .clicked()
        {
            // Launch logic
            world.resource_scope::<ActiveStoryGraph, _>(|world, graph| {
                // Clone data to avoid borrow issues when starting executor (which takes mut world usually, or system param)
                // But here we need to insert data into executor.
                if let Some(mut executor) = world.get_resource_mut::<GraphExecutor>() {
                    executor.load_from_data(&graph.0);
                    info!("Editor: Loaded Story Graph into Executor");
                }
            });
            world
                .resource_mut::<NextState<EditorState>>()
                .set(EditorState::Playing);
            info!("Editor: Play requested");
        }
        if ui
            .add_enabled(
                is_playing,
                egui::Button::new(RichText::new("⏹ STOP").color(COLOR_SECONDARY)),
            )
            .clicked()
        {
            world
                .resource_mut::<NextState<EditorState>>()
                .set(EditorState::Editor);
            info!("Editor: Stop requested");
        }

        ui.add_space(10.0);
        ui.separator();

        let project_name = world
            .resource::<LoadedProject>()
            .project
            .as_ref()
            .map(|project| project.name.clone())
            .unwrap_or_else(|| "No Project Mounted".to_string());

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let mut ui_state = world.resource_mut::<EditorUiState>();
            if ui
                .selectable_label(ui_state.console_open, "💻 Console")
                .clicked()
            {
                ui_state.console_open = !ui_state.console_open;
            }
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
                    let loaded_project = world.resource::<LoadedProject>().clone();
                    if let Some(asset_root) = resolve_asset_root(&loaded_project) {
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
                // Draw grid and handle interactions
                draw_grid(ui, world);
            } else {
                // Subtle overlay indicator
                ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
                    ui.label(RichText::new("● LIVE").color(Color32::RED).small());
                });
            }
        }
        EditorView::StoryGraph => {
            draw_story_graph(ui, world);
        }
    }
}
