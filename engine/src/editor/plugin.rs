use super::scene_io::refresh_editor_dirty_state;
use super::types::{
    EditorDirtyState, EditorSnapshotBaseline, EditorState, EditorUiState,
    RuntimePreviewLaunchState, COLOR_BG, COLOR_PRIMARY,
};
use crate::data::{DJDataRegistryPlugin, LoadedCustomDocuments};
use crate::editor::extensions::EditorExtensionRegistry;
use crate::project_mount::{
    auto_discover_or_create_project, normalize_project_path, MountedProject,
};
use bevy::prelude::*;
use bevy_egui::{
    egui::{self, CornerRadius, Stroke},
    EguiContexts, EguiPlugin, EguiPrimaryContextPass,
};
use std::path::Path;

use super::actions::{automated_ui_test_system, AutomatedTestActive};
use super::cli::parse_editor_cli_args;
use super::preview::poll_runtime_preview_process_system;
use super::types::ActiveStoryGraph;

pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<EguiPlugin>() {
            app.add_plugins(EguiPlugin::default());
        }
        if !app.is_plugin_added::<DJDataRegistryPlugin>() {
            app.add_plugins(DJDataRegistryPlugin);
        }

        let cli = parse_editor_cli_args(std::env::args());
        let mounted_project = if let Some(path) = cli.project_path.as_deref() {
            match normalize_project_path(path) {
                Ok((root_path, manifest_path)) => {
                    info!("CLI: Mounted project manifest {:?}", manifest_path);
                    MountedProject {
                        root_path: Some(root_path),
                        manifest_path: Some(manifest_path),
                        project: None,
                    }
                }
                Err(error) => {
                    warn!(
                        "CLI: Failed to normalize project path {:?}: {}",
                        path, error
                    );
                    MountedProject::default()
                }
            }
        } else {
            // No CLI project: auto-discover or create a default.
            match auto_discover_or_create_project() {
                Ok(mount) => {
                    info!(
                        "Auto-mount: {:?}",
                        mount.manifest_path.as_deref().unwrap_or(Path::new(""))
                    );
                    mount
                }
                Err(error) => {
                    warn!("Auto-mount failed: {}", error);
                    MountedProject::default()
                }
            }
        };

        app.init_state::<EditorState>()
            .insert_resource(mounted_project)
            .insert_resource(EditorUiState {
                current_view: cli.initial_view,
                console_open: true,
                ..default()
            })
            .init_resource::<ActiveStoryGraph>()
            .init_resource::<EditorSnapshotBaseline>()
            .init_resource::<EditorDirtyState>()
            .init_resource::<RuntimePreviewLaunchState>()
            .init_resource::<super::types::Helix3DViewerState>()
            .init_resource::<super::types::Helix3DLaunchConfig>()
            .init_resource::<LoadedCustomDocuments>()
            .init_resource::<EditorExtensionRegistry>()
            .init_resource::<super::extensions::SelectedPreviewPreset>()
            .init_resource::<super::extensions::ToolbarActionQueue>()
            .init_resource::<super::graph_editor::GraphEditorState>()
            .init_resource::<super::tutorial::TutorialState>()
            .insert_resource(super::tutorial::build_catalog())
            .init_resource::<super::panel_export::PanelExportRequest>()
            .init_resource::<super::panel_export::PanelExportResult>()
            .add_systems(Startup, super::scene_io::load_initial_project_system)
            .add_systems(EguiPrimaryContextPass, configure_visuals_system)
            .add_systems(EguiPrimaryContextPass, super::editor_ui_system)
            .add_systems(
                Update,
                (
                    poll_runtime_preview_process_system,
                    refresh_editor_dirty_state,
                    super::panel_export::process_panel_export_system,
                    super::panels::screenshot_hotkey_system,
                ),
            );

        if cli.start_tutorial {
            app.add_systems(
                Startup,
                |mut tut_state: ResMut<super::tutorial::TutorialState>| {
                    super::tutorial::start_first_game_tutorial(&mut tut_state);
                },
            );
        }

        if cli.test_mode {
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

    visuals.window_corner_radius = CornerRadius::same(2);
    visuals.widgets.noninteractive.bg_fill = COLOR_BG;
    visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(25, 25, 35);
    visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(40, 40, 50);
    visuals.widgets.active.bg_fill = egui::Color32::from_rgb(50, 50, 65);
    visuals.selection.bg_fill = COLOR_PRIMARY.linear_multiply(0.3);
    visuals.selection.stroke = Stroke::new(1.0, COLOR_PRIMARY);

    ctx.set_visuals(visuals);
}
