use super::scene_io::{load_scene_into_editor, save_project_impl};
use super::types::{
    ActiveStoryGraph, BrowserTab, EditorState, EditorUiState, EditorView, ProjectMetadata,
    COLOR_PRIMARY, COLOR_SECONDARY,
};
use super::views::{draw_grid, draw_story_graph};
use crate::data::loader;
use crate::diagnostics::console::ConsoleLogStore;
use crate::story_graph::GraphExecutor;
use bevy::prelude::*;
use bevy_egui::egui::{self, Color32, RichText};
use bevy_inspector_egui::bevy_inspector;
use std::path::PathBuf;

pub(crate) fn draw_top_menu(ui: &mut egui::Ui, world: &mut World) {
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
            if ui.button("📂 Load Project").clicked() {
                // For now, load default dev path
                let mut project = world.resource_mut::<ProjectMetadata>();
                project.name = "DoomExe".into();
                let path = PathBuf::from("games/dev/doomexe");
                project.path = Some(path.clone());

                // Try load scene
                let scene_path = path.join("scenes/current_scene.json");
                if scene_path.exists() {
                    match loader::load_scene(&scene_path) {
                        Ok(scene) => load_scene_into_editor(world, scene),
                        Err(e) => error!("Failed to load scene: {}", e),
                    }
                } else {
                    warn!("No scene found at {:?}", scene_path);
                }

                // Try load story graph
                let graph_path = path.join("story_graphs/main.json");
                if graph_path.exists() {
                    match loader::load_story_graph(&graph_path) {
                        Ok(graph) => {
                            world.insert_resource(ActiveStoryGraph(graph));
                            info!("Loaded story graph");
                        }
                        Err(e) => error!("Failed to load story graph: {}", e),
                    }
                }

                info!("Editor: Loaded project path 'games/dev/doomexe'");
                ui.close();
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

        let project_name = world.resource::<ProjectMetadata>().name.clone();

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
                    .color(Color32::GRAY),
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
                    ui.label("📁 music");
                    ui.label("📁 sprites");
                    ui.label("📁 scripts");
                    ui.label("  📄 hamster_test.lua");
                });
            }
            BrowserTab::Palette => {
                ui.add_space(5.0);
                ui.label(RichText::new("TOOL PALETTE").strong().color(COLOR_PRIMARY));
                ui.add_space(5.0);
                ui.label(RichText::new("Select item to paint:").italics());

                let mut selected = ui_state.selected_palette_item.clone();

                ui.add_space(5.0);
                ui.selectable_value(&mut selected, Some("Grass".to_string()), "🌿 Grass Tile");
                ui.selectable_value(&mut selected, Some("Wall".to_string()), "🧱 Stone Wall");
                ui.selectable_value(
                    &mut selected,
                    Some("Hamster".to_string()),
                    "🐹 Hamster Unit",
                );
                ui.selectable_value(&mut selected, Some("Chest".to_string()), "📦 Loot Chest");

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
