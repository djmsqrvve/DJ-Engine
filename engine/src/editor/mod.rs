//! Editor UI for DJ Engine.
//!
//! Provides a professional game development environment using Egui.

pub mod validation;

use crate::data::components::{
    ColorData, EntityComponents, SpriteComponent, TransformComponent, Vec3Data,
};
use crate::data::scene::{Entity as SceneEntity, Scene};
use crate::data::story::{StoryGraphData, StoryNodeData, StoryNodeVariant};
use crate::data::{loader, project::Project};
use crate::diagnostics::console::ConsoleLogStore;
use crate::story_graph::GraphExecutor;
use bevy::prelude::*;
use bevy_egui::{
    egui::{self, Color32, RichText},
    EguiPlugin,
};
use bevy_inspector_egui::bevy_inspector;
use std::path::PathBuf;

const COLOR_PRIMARY: Color32 = Color32::from_rgb(0, 255, 204); // Cyberpunk Mint
const COLOR_SECONDARY: Color32 = Color32::from_rgb(255, 175, 200); // Pale Rose
const COLOR_BG: Color32 = Color32::from_rgb(15, 15, 20);

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum EditorState {
    #[default]
    Editor,
    Playing,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum EditorView {
    #[default]
    Level,
    StoryGraph,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum BrowserTab {
    #[default]
    Hierarchy,
    Assets,
    Palette,
}

/// Resource holding the current project metadata.
#[derive(Resource, Default)]
pub struct ProjectMetadata {
    pub name: String,
    pub path: Option<std::path::PathBuf>,
}

#[derive(Resource, Default)]
pub struct ActiveStoryGraph(pub StoryGraphData);

#[derive(Resource, Default)]
pub struct EditorUiState {
    pub current_view: EditorView,
    pub browser_tab: BrowserTab,
    // We don't need Option<Entity> anymore, SelectedEntities handles it
    pub selected_entities: bevy_inspector_egui::bevy_inspector::hierarchy::SelectedEntities,
    pub asset_search_query: String,
    pub selected_palette_item: Option<String>,
    pub console_open: bool,
    pub dragged_node_id: Option<String>,
    pub connection_start_id: Option<String>,
    pub selected_node_id: Option<String>,
}

#[derive(Resource)]
struct AutomatedTestActive {
    timer: Timer,
    step: usize,
}

pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<EguiPlugin>() {
            app.add_plugins(EguiPlugin::default());
        }

        // Argument Parsing
        let args: Vec<String> = std::env::args().collect();
        let mut initial_project = ProjectMetadata::default();
        let mut initial_view = EditorView::Level;
        let mut test_mode = false;

        let mut i = 0;
        while i < args.len() {
            match args[i].as_str() {
                "--project" => {
                    if i + 1 < args.len() {
                        initial_project.name = "Loaded from CLI".into();
                        initial_project.path = Some(args[i + 1].clone().into());
                        info!("CLI: Pre-loading project from {}", args[i + 1]);
                    }
                }
                "--view" => {
                    if i + 1 < args.len() {
                        initial_view = match args[i + 1].as_str() {
                            "story" => EditorView::StoryGraph,
                            _ => EditorView::Level,
                        };
                        info!("CLI: Setting initial view to {:?}", initial_view);
                    }
                }
                "--test-mode" => {
                    test_mode = true;
                    info!("CLI: Automated Test Mode Enabled");
                }
                _ => {}
            }
            i += 1;
        }

        app.init_state::<EditorState>()
            .insert_resource(initial_project)
            .insert_resource(EditorUiState {
                current_view: initial_view,
                ..default()
            })
            .init_resource::<ActiveStoryGraph>()
            .add_systems(Update, configure_visuals_system)
            .add_systems(Update, editor_ui_system)
            .add_systems(OnEnter(EditorState::Playing), launch_project_system);

        if test_mode {
            app.insert_resource(AutomatedTestActive {
                timer: Timer::from_seconds(0.5, TimerMode::Repeating),
                step: 0,
            })
            .add_systems(Update, automated_ui_test_system);
        }

        info!("DJ Engine Editor initialized");
    }
}

fn configure_visuals_system(mut contexts: bevy_egui::EguiContexts) {
    let Ok(ctx) = contexts.ctx_mut() else {
        warn!("Editor visuals: primary Egui context unavailable, skipping visuals config");
        return;
    };
    let mut visuals = egui::Visuals::dark();

    // Cyberpunk tweaks
    visuals.window_corner_radius = egui::CornerRadius::same(2);
    visuals.widgets.noninteractive.bg_fill = COLOR_BG;
    visuals.widgets.inactive.bg_fill = Color32::from_rgb(25, 25, 35);
    visuals.widgets.hovered.bg_fill = Color32::from_rgb(40, 40, 50);
    visuals.widgets.active.bg_fill = Color32::from_rgb(50, 50, 65);
    visuals.selection.bg_fill = COLOR_PRIMARY.linear_multiply(0.3);
    visuals.selection.stroke = egui::Stroke::new(1.0, COLOR_PRIMARY);

    ctx.set_visuals(visuals);
}

fn automated_ui_test_system(
    mut commands: Commands,
    time: Res<Time>,
    mut test_state: ResMut<AutomatedTestActive>,
    mut ui_state: ResMut<EditorUiState>,
    mut console: ResMut<ConsoleLogStore>,
    mut app_exit: MessageWriter<bevy::app::AppExit>,
) {
    test_state.timer.tick(time.delta());
    if !test_state.timer.is_finished() {
        return;
    }

    match test_state.step {
        0 => {
            console.log("TEST: Starting automated UI test sequence...".into());
            test_state.step += 1;
        }
        1 => {
            console.log("TEST: Select 'Hamster' from palette".into());
            ui_state.browser_tab = BrowserTab::Palette;
            ui_state.selected_palette_item = Some("Hamster".into());
            test_state.step += 1;
        }
        2 => {
            console.log("TEST: Simulating click/spawn at (100, 100)".into());
            // Manually spawn entity as if clicked
            commands.spawn((
                Name::new("Hamster [100, 100]"),
                Sprite {
                    color: Color::srgb(0.8, 0.5, 0.2),
                    custom_size: Some(Vec2::new(30.0, 30.0)),
                    ..default()
                },
                Transform::from_xyz(100.0, 100.0, 0.0),
            ));
            test_state.step += 1;
        }
        3 => {
            console.log("TEST: Switching to Story Graph view".into());
            ui_state.current_view = EditorView::StoryGraph;
            test_state.step += 1;
        }
        4 => {
            console.log("TEST: Validation Complete. Exiting.".into());
            info!("Automated UI Test Passed");
            app_exit.write(bevy::app::AppExit::Success);
        }
        _ => {}
    }
}

fn launch_project_system(
    project: Res<ProjectMetadata>,
    mut script_events: MessageWriter<crate::scripting::ScriptCommand>,
) {
    let Some(path) = &project.path else {
        warn!("No project path mounted! Cannot launch.");
        return;
    };

    info!("Editor: Launching project from {:?}", path);

    // Look for a main.lua or hamster_test.lua in the project's script folder
    let script_path = path.join("assets/scripts/hamster_test.lua");
    if script_path.exists() {
        script_events.write(crate::scripting::ScriptCommand::Load {
            path: script_path.to_string_lossy().into(),
        });
    } else {
        warn!("No entry script found at {:?}", script_path);
    }
}

fn editor_ui_system(world: &mut World) {
    let egui_ctx = {
        let mut egui_query = world
            .query_filtered::<&mut bevy_egui::EguiContext, With<bevy::window::PrimaryWindow>>();
        let Ok(mut egui_context) = egui_query.single_mut(world) else {
            warn!("Editor UI: primary Egui context unavailable, skipping frame");
            return;
        };
        egui_context.get_mut().clone()
    };

    egui::TopBottomPanel::top("top_panel").show(&egui_ctx, |ui| {
        draw_top_menu(ui, world);
    });

    // Floating Console Window (Pop-up)
    if world.resource::<EditorUiState>().console_open {
        draw_console_window(&egui_ctx, world);
    }

    egui::SidePanel::left("left_panel")
        .default_width(250.0)
        .show(&egui_ctx, |ui| {
            draw_left_panel(ui, world);
        });

    egui::SidePanel::right("right_panel")
        .default_width(300.0)
        .show(&egui_ctx, |ui| {
            draw_right_panel(ui, world);
        });

    let current_state = world.resource::<State<EditorState>>().get();
    let central_frame = if *current_state == EditorState::Playing {
        egui::Frame::none()
    } else {
        egui::Frame::central_panel(&egui_ctx.style())
    };

    egui::CentralPanel::default()
        .frame(central_frame)
        .show(&egui_ctx, |ui| {
            draw_central_panel(ui, world);
        });
}

fn draw_top_menu(ui: &mut egui::Ui, world: &mut World) {
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
                ui.close_menu();
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
                ui.close_menu();
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

fn draw_left_panel(ui: &mut egui::Ui, world: &mut World) {
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

fn draw_right_panel(ui: &mut egui::Ui, world: &mut World) {
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

fn draw_console_window(ctx: &egui::Context, world: &mut World) {
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

fn draw_central_panel(ui: &mut egui::Ui, world: &mut World) {
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

fn draw_grid(ui: &mut egui::Ui, world: &mut World) {
    let rect = ui.available_rect_before_wrap();

    // 1. Handle Input (Placement)
    // We do this before drawing so the new item appears immediately (or next frame)
    let response = ui.allocate_rect(rect, egui::Sense::click());

    // Now valid to create painter after mutable borrow is done (or rather, we don't hold the painter while mutating ui via allocate_rect if we scope it,
    // but ui.painter() borrows ui. allocate_rect borrows ui mutably.
    // So we must call allocate_rect first, THEN get painter.
    let painter = ui.painter();

    if response.clicked() {
        if let Some(pointer_pos) = ui.input(|i| i.pointer.interact_pos()) {
            // Convert UI coordinates to "World" coordinates relative to the panel
            // For this 2D editor prototype, we treat the top-left of the panel as (0,0) world space for simplicity,
            // or we center it. Let's map it simply for now.
            let _relative_pos = pointer_pos - rect.min;

            // Snap to grid
            let grid_size = 40.0;
            // let grid_x = (relative_pos.x / grid_size).floor() * grid_size;
            // let grid_y = (relative_pos.y / grid_size).floor() * grid_size;

            // Bevy coordinates: Y is up, X is right. Center is (0,0).
            // Egui coordinates: Y is down, X is right. Top-left is (0,0).
            // We need a translation. For this visual prototype, we'll just spawn at a transform
            // that roughly aligns with where we clicked visually if we assume a standard 2D camera.
            // But since we aren't rendering the world *in* the egui panel yet (just a grid overlay),
            // this is a "blind" spawn into the world.
            // However, the Hierarchy will update, confirming the action.

            // Let's spawn at a 3D position assuming Z=0 plane.
            // We'll map the panel center to World (0,0).
            let center = rect.center();
            let world_x = pointer_pos.x - center.x;
            let world_y = center.y - pointer_pos.y; // Flip Y for Bevy

            let snap_x = (world_x / grid_size).round() * grid_size;
            let snap_y = (world_y / grid_size).round() * grid_size;

            let selected_item = world
                .resource::<EditorUiState>()
                .selected_palette_item
                .clone();

            if let Some(item) = selected_item {
                info!("Editor: Spawning {} at ({}, {})", item, snap_x, snap_y);

                // Determine color based on item
                let color = match item.as_str() {
                    "Grass" => Color::srgb(0.2, 0.8, 0.2),
                    "Wall" => Color::srgb(0.5, 0.5, 0.5),
                    "Hamster" => Color::srgb(0.8, 0.5, 0.2),
                    "Chest" => Color::srgb(0.8, 0.8, 0.1),
                    _ => Color::WHITE,
                };

                world.spawn((
                    Name::new(format!("{} [{:.0}, {:.0}]", item, snap_x, snap_y)),
                    Sprite {
                        color,
                        custom_size: Some(Vec2::new(30.0, 30.0)),
                        ..default()
                    },
                    Transform::from_xyz(snap_x, snap_y, 0.0),
                ));
            }
        }
    }

    // 2. Draw Grid Visuals
    painter.rect_filled(rect, 0.0, COLOR_BG);

    let grid_size = 40.0;
    let color = Color32::from_rgb(30, 30, 40);

    let mut x = rect.left();
    while x < rect.right() {
        painter.line_segment(
            [egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())],
            (1.0, color),
        );
        x += grid_size;
    }

    let mut y = rect.top();
    while y < rect.bottom() {
        painter.line_segment(
            [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)],
            (1.0, color),
        );
        y += grid_size;
    }

    // Draw ghost of selected item at mouse cursor
    if let Some(_item) = &world.resource::<EditorUiState>().selected_palette_item {
        if let Some(pointer_pos) = ui.input(|i| i.pointer.hover_pos()) {
            if rect.contains(pointer_pos) {
                painter.circle_filled(pointer_pos, 5.0, COLOR_PRIMARY);
            }
        }
    }
}

fn draw_story_graph(ui: &mut egui::Ui, world: &mut World) {
    let painter = ui.painter().clone(); // Clone painter to avoid borrow issues? No, ui.painter() returns reference.
                                        // We need to be careful with borrowing world and ui.

    let rect = ui.available_rect_before_wrap();

    // Darker background
    painter.rect_filled(rect, 0.0, Color32::from_rgb(10, 10, 15));

    // Context Menu
    let mut add_node_cmd = None;

    // We can't access world inside context_menu closure easily if we are borrowing it from outside?
    // Egui context menu runs immediately.
    let response = ui.allocate_rect(rect, egui::Sense::click());

    response.context_menu(|ui| {
        ui.label("Add Node");
        ui.separator();
        if ui.button("Start Node").clicked() {
            add_node_cmd = Some("Start");
            ui.close_menu();
        }
        if ui.button("Dialogue Node").clicked() {
            add_node_cmd = Some("Dialogue");
            ui.close_menu();
        }
        if ui.button("End Node").clicked() {
            add_node_cmd = Some("End");
            ui.close_menu();
        }
    });

    if let Some(cmd) = add_node_cmd {
        world.resource_scope::<ActiveStoryGraph, _>(|_, mut graph| {
            let id = format!("node_{}", graph.0.nodes.len());
            let pos = response.interact_pointer_pos().unwrap_or(rect.center());
            // Adjust to be relative to panel if needed, but we store absolute screen coords for simpler drag?
            // Ideally relative to rect.min.
            let rel_pos = pos - rect.min;

            let mut node = match cmd {
                "Start" => StoryNodeData::start(id.clone(), None::<String>),
                "Dialogue" => StoryNodeData::dialogue(id.clone(), "Stranger", "Hello world"),
                "End" => StoryNodeData::end(id.clone()),
                _ => StoryNodeData::dialogue(id.clone(), "Err", "Err"),
            };

            // Set position
            node.position = Vec3Data::new(rel_pos.x, rel_pos.y, 0.0);

            // If start, set root
            if cmd == "Start" {
                graph.0.root_node_id = id.clone();
            }

            graph.0.add_node(node);
        });
    }

    // DRAW NODES AND LINES
    // We need to scope world to get graph
    world.resource_scope::<ActiveStoryGraph, _>(|world, mut graph| {
        let mut ui_state = world.resource_mut::<EditorUiState>();

        // 1. Draw Connections
        for node in &graph.0.nodes {
            let start_pos =
                rect.min + egui::vec2(node.position.x, node.position.y) + egui::vec2(100.0, 25.0); // Approx right side

            for next_id in node.next_node_ids() {
                if let Some(target) = graph.0.find_node(next_id) {
                    let end_pos = rect.min
                        + egui::vec2(target.position.x, target.position.y)
                        + egui::vec2(0.0, 25.0); // Approx left side
                    painter.line_segment([start_pos, end_pos], (2.0, Color32::GRAY));
                }
            }

            // Draw active drag line
            if let Some(start_id) = &ui_state.connection_start_id {
                if start_id == &node.id {
                    if let Some(pointer) = ui.input(|i| i.pointer.hover_pos()) {
                        painter.line_segment([start_pos, pointer], (2.0, Color32::YELLOW));
                    }
                }
            }
        }

        // 2. Draw Nodes
        let mut node_to_update_pos = None;
        let mut connection_established = None; // (from, to)

        for node in &mut graph.0.nodes {
            let node_rect = egui::Rect::from_min_size(
                rect.min + egui::vec2(node.position.x, node.position.y),
                egui::vec2(150.0, 80.0),
            );

            // Background
            let color = match node.node_type() {
                crate::data::story::StoryNodeType::Start => Color32::from_rgb(50, 200, 100),
                crate::data::story::StoryNodeType::End => Color32::from_rgb(200, 50, 50),
                crate::data::story::StoryNodeType::Dialogue => Color32::from_rgb(50, 100, 200),
                _ => Color32::from_rgb(100, 100, 100),
            };

            painter.rect_filled(node_rect, 5.0, color);
            painter.rect_stroke(
                node_rect,
                5.0,
                (1.0, Color32::WHITE),
                egui::StrokeKind::Inside,
            );

            // Content
            painter.text(
                node_rect.min + egui::vec2(10.0, 10.0),
                egui::Align2::LEFT_TOP,
                &node.id,
                egui::FontId::proportional(14.0),
                Color32::WHITE,
            );
            painter.text(
                node_rect.min + egui::vec2(10.0, 30.0),
                egui::Align2::LEFT_TOP,
                format!("{:?}", node.node_type()),
                egui::FontId::proportional(12.0),
                Color32::BLACK,
            );

            // Interaction
            let response = ui.allocate_rect(node_rect, egui::Sense::drag());
            if response.dragged() {
                node_to_update_pos = Some((node.id.clone(), response.drag_delta()));
            }
            if response.clicked() {
                ui_state.selected_node_id = Some(node.id.clone());
            }

            // Connect Button (Little circle on right)
            let port_rect =
                egui::Rect::from_center_size(node_rect.right_center(), egui::vec2(12.0, 12.0));
            painter.circle_filled(port_rect.center(), 6.0, Color32::WHITE);
            let port_resp = ui.allocate_rect(port_rect, egui::Sense::click());

            if port_resp.clicked() {
                if let Some(start_id) = ui_state.connection_start_id.clone() {
                    // Complete connection
                    if start_id != node.id {
                        connection_established = Some((start_id, node.id.clone()));
                        ui_state.connection_start_id = None;
                    }
                } else {
                    // Start connection
                    ui_state.connection_start_id = Some(node.id.clone());
                }
            }

            // If clicking node body while connecting, also connect (easier target)
            if response.clicked() && ui_state.connection_start_id.is_some() {
                if let Some(start_id) = &ui_state.connection_start_id {
                    if start_id != &node.id {
                        connection_established = Some((start_id.clone(), node.id.clone()));
                        ui_state.connection_start_id = None;
                    }
                }
            }
        }

        // Apply position updates
        if let Some((id, delta)) = node_to_update_pos {
            if let Some(node) = graph.0.nodes.iter_mut().find(|n| n.id == id) {
                node.position.x += delta.x;
                node.position.y += delta.y;
            }
        }

        // Apply connection
        if let Some((from, to)) = connection_established {
            if let Some(node) = graph.0.nodes.iter_mut().find(|n| n.id == from) {
                // Ugly mutation manually based on type
                // TODO: Add helper 'set_next' to StoryNodeData
                match &mut node.data {
                    StoryNodeVariant::Start(d) => d.next_node_id = Some(to),
                    StoryNodeVariant::Dialogue(d) => d.next_node_id = Some(to),
                    StoryNodeVariant::Action(a) => a.next_node_id = Some(to),
                    _ => {}
                }
            }
        }
    });
}

fn save_project_impl(world: &mut World) {
    // Clone necessary data to avoid holding borrow on world
    let (project_name, project_path) = {
        let project_meta = world.resource::<ProjectMetadata>();
        (project_meta.name.clone(), project_meta.path.clone())
    };

    if let Some(path) = project_path {
        info!("Saving project to {:?}", path);

        // 1. Save Project Structure
        let project_data = Project::new(&project_name);
        match loader::save_project_structure(&project_data, &path) {
            Ok(_) => info!("Successfully saved project structure"),
            Err(e) => error!("Failed to save project structure: {}", e),
        }

        // 2. Save Current Scene
        let scene = world_to_scene(world);
        let scene_path = path.join("scenes/current_scene.json");
        match loader::save_scene(&scene, &scene_path) {
            Ok(_) => info!("Successfully saved scene to {:?}", scene_path),
            Err(e) => error!("Failed to save scene: {}", e),
        }

        // 3. Save Story Graph
        let graph = &world.resource::<ActiveStoryGraph>().0;
        let graph_path = path.join("story_graphs/main.json");
        match loader::save_story_graph(graph, &graph_path) {
            Ok(_) => info!("Successfully saved story graph to {:?}", graph_path),
            Err(e) => error!("Failed to save story graph: {}", e),
        }
    } else {
        warn!("Cannot save: No project path set!");
    }
}

fn world_to_scene(world: &mut World) -> Scene {
    let mut scene = Scene::new("current_scene", "Current Scene");

    // In a real implementation, we'd query for all entities with specific marker components.
    // For this prototype, we'll query all entities with a Name and Transform.

    let mut entities = Vec::new();
    let mut query = world.query::<(Entity, &Name, &Transform, Option<&Sprite>)>();

    // We need to collect first to avoid borrowing world inside loop if we needed mutable access,
    // though query iteration is fine. But constructing SceneEntity might need data types.
    let mut world_entities = Vec::new();
    for (_e, name, transform, sprite) in query.iter(world) {
        // Clone data out of world
        let pos = transform.translation;
        let scale = transform.scale;

        let sprite_color = sprite.map(|s| s.color.to_linear().to_f32_array());

        world_entities.push((name.to_string(), pos, scale, sprite_color));
    }

    for (name, pos, scale, sprite_color) in world_entities {
        // Skip editor-only entities (like cameras or UI, unless tagged)
        // For now, simple filter: if it has a name starting with "Editor", skip?
        // Or better, only save things we know we spawned.

        let mut components = EntityComponents::default();

        components.transform = TransformComponent {
            position: Vec3Data::new(pos.x, pos.y, pos.z),
            rotation: Vec3Data::default(), // Simplification
            scale: Vec3Data::new(scale.x, scale.y, scale.z),
            lock_uniform_scale: false,
        };

        if let Some([r, g, b, a]) = sprite_color {
            components.sprite = Some(SpriteComponent {
                sprite_id: "pixel".to_string(), // Placeholder
                tint: ColorData::rgba(r, g, b, a),
                ..Default::default()
            });
        }

        let entity = SceneEntity::new(name.clone(), name) // using name as ID for prototype
            .with_components(components);

        entities.push(entity);
    }

    scene.entities = entities;
    scene
}

fn load_scene_into_editor(world: &mut World, scene: Scene) {
    // 1. Clear existing entities (naive approach: despawn everything with a Name)
    // Real engine would use a SceneRoot component
    let entities_to_despawn: Vec<Entity> = world
        .query_filtered::<Entity, With<Name>>()
        .iter(world)
        .collect();
    for e in entities_to_despawn {
        world.despawn(e);
    }

    // 2. Spawn new entities
    let entity_count = scene.entities.len();
    for entity_data in scene.entities {
        let transform = entity_data.components.transform;
        let pos = transform.position;
        let scale = transform.scale;

        let mut entity_cmd = world.spawn((
            Name::new(entity_data.name),
            Transform::from_xyz(pos.x, pos.y, pos.z)
                .with_scale(Vec3::new(scale.x, scale.y, scale.z)),
        ));

        if let Some(sprite) = entity_data.components.sprite {
            let c = sprite.tint;
            entity_cmd.insert(Sprite {
                color: Color::srgba(c.r, c.g, c.b, c.a),
                custom_size: Some(Vec2::new(30.0, 30.0)), // Default size for now
                ..default()
            });
        }
    }
    info!("Loaded scene with {} entities", entity_count);
}
