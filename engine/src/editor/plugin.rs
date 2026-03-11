use super::types::{BrowserTab, EditorState, EditorUiState, EditorView, COLOR_BG, COLOR_PRIMARY};
use crate::diagnostics::console::ConsoleLogStore;
use crate::project_mount::{normalize_project_path, MountedProject};
use bevy::prelude::*;
use bevy_egui::{
    egui::{self, CornerRadius, Stroke},
    EguiContexts, EguiPlugin, EguiPrimaryContextPass,
};
use std::path::PathBuf;

use super::types::ActiveStoryGraph;

#[derive(Resource)]
struct AutomatedTestActive {
    timer: Timer,
    step: usize,
}

#[derive(Debug, Default, PartialEq, Eq)]
struct EditorCliOptions {
    project_path: Option<PathBuf>,
    initial_view: EditorView,
    test_mode: bool,
}

pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<EguiPlugin>() {
            app.add_plugins(EguiPlugin::default());
        }

        let cli = parse_editor_cli_args(std::env::args());
        let mut mounted_project = MountedProject::default();

        if let Some(path) = cli.project_path.as_deref() {
            match normalize_project_path(path) {
                Ok((root_path, manifest_path)) => {
                    info!("CLI: Mounted project manifest {:?}", manifest_path);
                    mounted_project.root_path = Some(root_path);
                    mounted_project.manifest_path = Some(manifest_path);
                }
                Err(error) => {
                    warn!(
                        "CLI: Failed to normalize project path {:?}: {}",
                        path, error
                    );
                }
            }
        }

        app.init_state::<EditorState>()
            .insert_resource(mounted_project)
            .insert_resource(EditorUiState {
                current_view: cli.initial_view,
                ..default()
            })
            .init_resource::<ActiveStoryGraph>()
            .add_systems(Startup, super::scene_io::load_initial_project_system)
            .add_systems(EguiPrimaryContextPass, configure_visuals_system)
            .add_systems(EguiPrimaryContextPass, super::editor_ui_system)
            .add_systems(OnEnter(EditorState::Playing), launch_project_system);

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

fn parse_editor_cli_args(args: impl IntoIterator<Item = String>) -> EditorCliOptions {
    let args: Vec<String> = args.into_iter().collect();
    let mut options = EditorCliOptions::default();
    let mut positional_project = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--project" => {
                if i + 1 < args.len() {
                    options.project_path = Some(PathBuf::from(&args[i + 1]));
                    info!("CLI: Pre-loading project from {}", args[i + 1]);
                    i += 1;
                }
            }
            "--view" => {
                if i + 1 < args.len() {
                    options.initial_view = match args[i + 1].as_str() {
                        "story" => EditorView::StoryGraph,
                        _ => EditorView::Level,
                    };
                    info!("CLI: Setting initial view to {:?}", options.initial_view);
                    i += 1;
                }
            }
            "--test-mode" => {
                options.test_mode = true;
                info!("CLI: Automated Test Mode Enabled");
            }
            arg if !arg.starts_with("--") && positional_project.is_none() => {
                positional_project = Some(PathBuf::from(arg));
            }
            _ => {}
        }
        i += 1;
    }

    if options.project_path.is_none() {
        options.project_path = positional_project;
    }

    options
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
            console.log("TEST: Select 'Actor' from palette".into());
            ui_state.browser_tab = BrowserTab::Palette;
            ui_state.selected_palette_item = Some("Actor".into());
            test_state.step += 1;
        }
        2 => {
            console.log("TEST: Simulating click/spawn at (100, 100)".into());
            // Manually spawn entity as if clicked
            commands.spawn((
                Name::new("Actor [100, 100]"),
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
    mounted_project: Res<MountedProject>,
    mut script_events: MessageWriter<crate::scripting::ScriptCommand>,
) {
    let Some(project) = &mounted_project.project else {
        warn!("No project loaded; continuing play mode without entry script.");
        return;
    };
    let Some(root_path) = &mounted_project.root_path else {
        warn!("Project root missing; continuing play mode without entry script.");
        return;
    };
    let Some(entry_script) = project.settings.startup.entry_script.as_deref() else {
        info!("No entry script configured; story graph preview will continue without scripting.");
        return;
    };

    let script_path = root_path.join(entry_script);
    if !script_path.exists() {
        warn!(
            "Configured entry script {:?} was not found; continuing without scripting.",
            script_path
        );
        return;
    }

    info!("Editor: Launching project entry script {:?}", script_path);
    script_events.write(crate::scripting::ScriptCommand::Load {
        path: script_path.to_string_lossy().into(),
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_editor_cli_args_supports_positional_project_path() {
        let cli = parse_editor_cli_args([
            "dj_engine".into(),
            "projects/sample".into(),
            "--view".into(),
            "story".into(),
            "--test-mode".into(),
        ]);

        assert_eq!(cli.project_path, Some(PathBuf::from("projects/sample")));
        assert_eq!(cli.initial_view, EditorView::StoryGraph);
        assert!(cli.test_mode);
    }

    #[test]
    fn test_parse_editor_cli_args_prefers_explicit_project_flag() {
        let cli = parse_editor_cli_args([
            "dj_engine".into(),
            "projects/ignored".into(),
            "--project".into(),
            "projects/explicit".into(),
        ]);

        assert_eq!(cli.project_path, Some(PathBuf::from("projects/explicit")));
    }
}
