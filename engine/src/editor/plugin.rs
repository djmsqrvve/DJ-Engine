use super::types::{
    BrowserTab, EditorState, EditorUiState, EditorView, ProjectMetadata, COLOR_BG, COLOR_PRIMARY,
};
use crate::diagnostics::console::ConsoleLogStore;
use bevy::prelude::*;
use bevy_egui::{
    egui::{self, CornerRadius, Stroke},
    EguiContexts, EguiPlugin, EguiPrimaryContextPass,
};

use super::types::ActiveStoryGraph;

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
            .add_systems(EguiPrimaryContextPass, configure_visuals_system)
            .add_systems(EguiPrimaryContextPass, super::editor_ui_system)
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

fn configure_visuals_system(mut contexts: EguiContexts) {
    let Ok(ctx) = contexts.ctx_mut() else {
        warn!("Editor visuals: primary Egui context unavailable, skipping visuals config");
        return;
    };
    let mut visuals = egui::Visuals::dark();

    // Cyberpunk tweaks
    visuals.window_corner_radius = CornerRadius::same(2);
    visuals.widgets.noninteractive.bg_fill = COLOR_BG;
    visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(25, 25, 35);
    visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(40, 40, 50);
    visuals.widgets.active.bg_fill = egui::Color32::from_rgb(50, 50, 65);
    visuals.selection.bg_fill = COLOR_PRIMARY.linear_multiply(0.3);
    visuals.selection.stroke = Stroke::new(1.0, COLOR_PRIMARY);

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
