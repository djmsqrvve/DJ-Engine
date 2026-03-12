use super::scene_io::{load_mounted_project, refresh_editor_dirty_state, save_project_impl};
use super::types::{
    BrowserTab, EditorDirtyState, EditorSnapshotBaseline, EditorState, EditorUiState, EditorView,
    PendingProjectAction, PendingProjectActionResolution, RuntimePreviewLaunchPhase,
    RuntimePreviewLaunchState, COLOR_BG, COLOR_PRIMARY,
};
use crate::diagnostics::console::ConsoleLogStore;
use crate::project_mount::{normalize_project_path, MountedProject};
use bevy::prelude::*;
use bevy_egui::{
    egui::{self, CornerRadius, Stroke},
    EguiContexts, EguiPlugin, EguiPrimaryContextPass,
};
use std::fmt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::sync::{Arc, Mutex};

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

#[derive(Debug, Clone, PartialEq, Eq)]
struct ResolvedRuntimePreviewCommand {
    program: PathBuf,
    args: Vec<String>,
    current_dir: Option<PathBuf>,
}

impl ResolvedRuntimePreviewCommand {
    fn spawn(&self) -> std::io::Result<Child> {
        let mut command = Command::new(&self.program);
        command.args(&self.args);
        if let Some(current_dir) = &self.current_dir {
            command.current_dir(current_dir);
        }
        command.spawn()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum RuntimePreviewLaunchError {
    CurrentExecutableUnavailable(String),
    RuntimePreviewExecutableNotFound(PathBuf),
}

impl fmt::Display for RuntimePreviewLaunchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CurrentExecutableUnavailable(message) => write!(f, "{message}"),
            Self::RuntimePreviewExecutableNotFound(path) => write!(
                f,
                "Runtime preview executable not found at {:?}, and dev fallback is unavailable.",
                path
            ),
        }
    }
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
            .init_resource::<EditorSnapshotBaseline>()
            .init_resource::<EditorDirtyState>()
            .init_resource::<RuntimePreviewLaunchState>()
            .add_systems(Startup, super::scene_io::load_initial_project_system)
            .add_systems(EguiPrimaryContextPass, configure_visuals_system)
            .add_systems(EguiPrimaryContextPass, super::editor_ui_system)
            .add_systems(
                Update,
                (
                    poll_runtime_preview_process_system,
                    refresh_editor_dirty_state,
                ),
            );

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

fn runtime_preview_sibling_path(current_exe: &Path) -> PathBuf {
    current_exe.with_file_name(format!("runtime_preview{}", std::env::consts::EXE_SUFFIX))
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("engine crate should live inside the workspace root")
        .to_path_buf()
}

fn resolve_runtime_preview_command(
    manifest_path: &Path,
) -> Result<ResolvedRuntimePreviewCommand, RuntimePreviewLaunchError> {
    let current_exe = std::env::current_exe().ok();
    resolve_runtime_preview_command_from_mode(
        current_exe.as_deref(),
        manifest_path,
        cfg!(debug_assertions),
    )
}

fn resolve_runtime_preview_command_from_mode(
    current_exe: Option<&Path>,
    manifest_path: &Path,
    allow_dev_fallback: bool,
) -> Result<ResolvedRuntimePreviewCommand, RuntimePreviewLaunchError> {
    if let Some(current_exe) = current_exe {
        let sibling_path = runtime_preview_sibling_path(current_exe);
        if sibling_path.is_file() {
            return Ok(ResolvedRuntimePreviewCommand {
                program: sibling_path,
                args: vec![
                    "--project".into(),
                    manifest_path.as_os_str().to_string_lossy().into_owned(),
                ],
                current_dir: None,
            });
        }

        if !allow_dev_fallback {
            return Err(RuntimePreviewLaunchError::RuntimePreviewExecutableNotFound(
                sibling_path,
            ));
        }
    } else if !allow_dev_fallback {
        return Err(RuntimePreviewLaunchError::CurrentExecutableUnavailable(
            "Unable to locate the editor executable to resolve a sibling runtime preview binary."
                .into(),
        ));
    }

    Ok(ResolvedRuntimePreviewCommand {
        program: PathBuf::from("cargo"),
        args: vec![
            "run".into(),
            "-p".into(),
            "dj_engine".into(),
            "--bin".into(),
            "runtime_preview".into(),
            "--".into(),
            "--project".into(),
            manifest_path.as_os_str().to_string_lossy().into_owned(),
        ],
        current_dir: Some(workspace_root()),
    })
}

fn log_preview_message(console: Option<&mut ConsoleLogStore>, message: &str) {
    if let Some(console) = console {
        console.log(message.to_string());
    }
}

fn set_launch_state_message(
    launch_state: &mut RuntimePreviewLaunchState,
    phase: RuntimePreviewLaunchPhase,
    message: String,
) -> String {
    launch_state.phase = phase;
    launch_state.status_message = Some(message.clone());
    message
}

fn execute_project_action(
    world: &mut World,
    action: PendingProjectAction,
) -> Result<(), crate::data::DataError> {
    match action {
        PendingProjectAction::LoadMountedProject | PendingProjectAction::ReloadProject => {
            load_mounted_project(world)
        }
    }
}

pub(crate) fn request_project_action(world: &mut World, action: PendingProjectAction) {
    let is_dirty = world.resource::<EditorDirtyState>().is_dirty;
    if is_dirty {
        world
            .resource_mut::<EditorDirtyState>()
            .pending_project_action = Some(action);
        return;
    }

    if let Err(error) = execute_project_action(world, action) {
        let message = format!("Editor action failed: {error}");
        log_preview_message(
            world.get_resource_mut::<ConsoleLogStore>().as_deref_mut(),
            &message,
        );
        error!("{message}");
    }
}

pub(crate) fn resolve_pending_project_action(
    world: &mut World,
    resolution: PendingProjectActionResolution,
) {
    let pending_action = world
        .resource::<EditorDirtyState>()
        .pending_project_action
        .clone();
    let Some(action) = pending_action else {
        return;
    };

    match resolution {
        PendingProjectActionResolution::Cancel => {
            world
                .resource_mut::<EditorDirtyState>()
                .pending_project_action = None;
        }
        PendingProjectActionResolution::DiscardChanges => {
            world
                .resource_mut::<EditorDirtyState>()
                .pending_project_action = None;
            if let Err(error) = execute_project_action(world, action) {
                let message = format!("Editor action failed: {error}");
                log_preview_message(
                    world.get_resource_mut::<ConsoleLogStore>().as_deref_mut(),
                    &message,
                );
                error!("{message}");
            }
        }
        PendingProjectActionResolution::SaveAndContinue => match save_project_impl(world) {
            Ok(()) => {
                world
                    .resource_mut::<EditorDirtyState>()
                    .pending_project_action = None;
                if let Err(error) = execute_project_action(world, action) {
                    let message = format!("Editor action failed: {error}");
                    log_preview_message(
                        world.get_resource_mut::<ConsoleLogStore>().as_deref_mut(),
                        &message,
                    );
                    error!("{message}");
                }
            }
            Err(error) => {
                let message =
                    format!("Editor action failed: could not save before continuing: {error}");
                log_preview_message(
                    world.get_resource_mut::<ConsoleLogStore>().as_deref_mut(),
                    &message,
                );
                error!("{message}");
            }
        },
    }
}

pub(crate) fn launch_runtime_preview_from_editor(world: &mut World) {
    let (manifest_path, has_loaded_project) = {
        let mounted_project = world.resource::<MountedProject>();
        (
            mounted_project.manifest_path.clone(),
            mounted_project.project.is_some(),
        )
    };

    let Some(manifest_path) = manifest_path else {
        let message = "Preview failed: no mounted project manifest is available.".to_string();
        {
            let mut launch_state = world.resource_mut::<RuntimePreviewLaunchState>();
            launch_state.manifest_path = None;
            launch_state.process = None;
            launch_state.last_exit = None;
            set_launch_state_message(
                &mut launch_state,
                RuntimePreviewLaunchPhase::Failed,
                message.clone(),
            );
        }
        log_preview_message(
            world.get_resource_mut::<ConsoleLogStore>().as_deref_mut(),
            &message,
        );
        error!("{message}");
        return;
    };

    if !has_loaded_project {
        let message = format!(
            "Preview failed: project {:?} is mounted but not loaded.",
            manifest_path
        );
        {
            let mut launch_state = world.resource_mut::<RuntimePreviewLaunchState>();
            launch_state.manifest_path = Some(manifest_path.clone());
            launch_state.process = None;
            launch_state.last_exit = None;
            set_launch_state_message(
                &mut launch_state,
                RuntimePreviewLaunchPhase::Failed,
                message.clone(),
            );
        }
        log_preview_message(
            world.get_resource_mut::<ConsoleLogStore>().as_deref_mut(),
            &message,
        );
        error!("{message}");
        return;
    }

    if world.resource::<RuntimePreviewLaunchState>().is_running() {
        let message = "Preview launch ignored: runtime preview is already running.".to_string();
        log_preview_message(
            world.get_resource_mut::<ConsoleLogStore>().as_deref_mut(),
            &message,
        );
        warn!("{message}");
        return;
    }

    if world.resource::<EditorDirtyState>().is_dirty {
        if let Err(error) = save_project_impl(world) {
            let message = format!("Preview failed: could not save project before launch: {error}");
            {
                let mut launch_state = world.resource_mut::<RuntimePreviewLaunchState>();
                launch_state.manifest_path = Some(manifest_path.clone());
                launch_state.process = None;
                set_launch_state_message(
                    &mut launch_state,
                    RuntimePreviewLaunchPhase::Failed,
                    message.clone(),
                );
            }
            log_preview_message(
                world.get_resource_mut::<ConsoleLogStore>().as_deref_mut(),
                &message,
            );
            error!("{message}");
            return;
        }
    }

    {
        let mut launch_state = world.resource_mut::<RuntimePreviewLaunchState>();
        launch_state.manifest_path = Some(manifest_path.clone());
        launch_state.last_exit = None;
        launch_state.process = None;
        set_launch_state_message(
            &mut launch_state,
            RuntimePreviewLaunchPhase::Launching,
            "Preview Launching".into(),
        );
    }

    let command = match resolve_runtime_preview_command(&manifest_path) {
        Ok(command) => command,
        Err(error) => {
            let message = format!("Preview failed: {error}");
            {
                let mut launch_state = world.resource_mut::<RuntimePreviewLaunchState>();
                launch_state.process = None;
                set_launch_state_message(
                    &mut launch_state,
                    RuntimePreviewLaunchPhase::Failed,
                    message.clone(),
                );
            }
            log_preview_message(
                world.get_resource_mut::<ConsoleLogStore>().as_deref_mut(),
                &message,
            );
            error!("{message}");
            return;
        }
    };

    match command.spawn() {
        Ok(child) => {
            let message = format!("Preview Running ({})", manifest_path.display());
            {
                let mut launch_state = world.resource_mut::<RuntimePreviewLaunchState>();
                launch_state.phase = RuntimePreviewLaunchPhase::Running;
                launch_state.status_message = Some(message.clone());
                launch_state.last_exit = None;
                launch_state.process = Some(Arc::new(Mutex::new(child)));
            }
            log_preview_message(
                world.get_resource_mut::<ConsoleLogStore>().as_deref_mut(),
                &message,
            );
            info!("{message}");
        }
        Err(error) => {
            let message = format!("Preview failed: could not launch runtime preview: {error}");
            {
                let mut launch_state = world.resource_mut::<RuntimePreviewLaunchState>();
                launch_state.process = None;
                set_launch_state_message(
                    &mut launch_state,
                    RuntimePreviewLaunchPhase::Failed,
                    message.clone(),
                );
            }
            log_preview_message(
                world.get_resource_mut::<ConsoleLogStore>().as_deref_mut(),
                &message,
            );
            error!("{message}");
        }
    }
}

pub(crate) fn stop_runtime_preview_from_editor(world: &mut World) {
    let process = world
        .resource::<RuntimePreviewLaunchState>()
        .process
        .clone();
    let Some(process) = process else {
        return;
    };

    let kill_result = match process.lock() {
        Ok(mut child) => child.kill(),
        Err(error) => {
            let message = format!("Preview failed: runtime preview lock poisoned: {error}");
            {
                let mut launch_state = world.resource_mut::<RuntimePreviewLaunchState>();
                launch_state.process = None;
                set_launch_state_message(
                    &mut launch_state,
                    RuntimePreviewLaunchPhase::Failed,
                    message.clone(),
                );
            }
            log_preview_message(
                world.get_resource_mut::<ConsoleLogStore>().as_deref_mut(),
                &message,
            );
            error!("{message}");
            return;
        }
    };

    match kill_result {
        Ok(()) => {
            let message = "Preview stopping...".to_string();
            {
                let mut launch_state = world.resource_mut::<RuntimePreviewLaunchState>();
                set_launch_state_message(
                    &mut launch_state,
                    RuntimePreviewLaunchPhase::Stopping,
                    message.clone(),
                );
            }
            log_preview_message(
                world.get_resource_mut::<ConsoleLogStore>().as_deref_mut(),
                &message,
            );
            info!("{message}");
        }
        Err(error) => {
            let message = format!("Preview failed: could not stop runtime preview: {error}");
            {
                let mut launch_state = world.resource_mut::<RuntimePreviewLaunchState>();
                launch_state.process = None;
                set_launch_state_message(
                    &mut launch_state,
                    RuntimePreviewLaunchPhase::Failed,
                    message.clone(),
                );
            }
            log_preview_message(
                world.get_resource_mut::<ConsoleLogStore>().as_deref_mut(),
                &message,
            );
            error!("{message}");
        }
    }
}

fn format_exit_status(exit_status: std::process::ExitStatus) -> String {
    match exit_status.code() {
        Some(code) => format!("exit code {code}"),
        None => "terminated by signal".into(),
    }
}

fn poll_runtime_preview_process_system(
    mut launch_state: ResMut<RuntimePreviewLaunchState>,
    mut console: Option<ResMut<ConsoleLogStore>>,
) {
    let Some(process) = launch_state.process.clone() else {
        return;
    };

    let wait_result = match process.lock() {
        Ok(mut child) => child.try_wait(),
        Err(error) => {
            let message = format!("Preview failed: runtime preview lock poisoned: {error}");
            launch_state.process = None;
            launch_state.phase = RuntimePreviewLaunchPhase::Failed;
            launch_state.status_message = Some(message.clone());
            log_preview_message(console.as_deref_mut(), &message);
            error!("{message}");
            return;
        }
    };

    match wait_result {
        Ok(Some(exit_status)) => {
            let exit_summary = format_exit_status(exit_status);
            launch_state.process = None;
            launch_state.last_exit = Some(exit_summary.clone());

            let message = if exit_status.success()
                || launch_state.phase == RuntimePreviewLaunchPhase::Stopping
            {
                launch_state.phase = RuntimePreviewLaunchPhase::Idle;
                "Preview Exited".to_string()
            } else {
                launch_state.phase = RuntimePreviewLaunchPhase::Failed;
                format!("Preview Failed ({exit_summary})")
            };

            launch_state.status_message = Some(message.clone());
            log_preview_message(console.as_deref_mut(), &message);
            info!("{message}");
        }
        Ok(None) => {
            if launch_state.phase == RuntimePreviewLaunchPhase::Launching {
                launch_state.phase = RuntimePreviewLaunchPhase::Running;
                launch_state.status_message = Some("Preview Running".into());
            }
        }
        Err(error) => {
            let message = format!("Preview failed while polling runtime preview: {error}");
            launch_state.process = None;
            launch_state.phase = RuntimePreviewLaunchPhase::Failed;
            launch_state.status_message = Some(message.clone());
            log_preview_message(console.as_deref_mut(), &message);
            error!("{message}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{loader, Project};
    use crate::editor::scene_io::sync_editor_snapshot_baseline;
    use crate::editor::{EditorDirtyState, EditorSnapshotBaseline};
    use std::fs;

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

    #[test]
    fn test_resolve_runtime_preview_command_prefers_sibling_binary() {
        let temp_dir = tempfile::tempdir().unwrap();
        let current_exe = temp_dir
            .path()
            .join(format!("dj_engine{}", std::env::consts::EXE_SUFFIX));
        let sibling = temp_dir
            .path()
            .join(format!("runtime_preview{}", std::env::consts::EXE_SUFFIX));
        fs::write(&current_exe, []).unwrap();
        fs::write(&sibling, []).unwrap();

        let command = resolve_runtime_preview_command_from_mode(
            Some(&current_exe),
            Path::new("/tmp/project.json"),
            false,
        )
        .unwrap();

        assert_eq!(command.program, sibling);
        assert_eq!(command.args, vec!["--project", "/tmp/project.json"]);
        assert_eq!(command.current_dir, None);
    }

    #[test]
    fn test_resolve_runtime_preview_command_uses_dev_cargo_fallback() {
        let command = resolve_runtime_preview_command_from_mode(
            Some(Path::new("/tmp/dj_engine")),
            Path::new("/tmp/project.json"),
            true,
        )
        .unwrap();

        assert_eq!(command.program, PathBuf::from("cargo"));
        assert_eq!(command.current_dir, Some(workspace_root()));
        assert_eq!(
            command.args,
            vec![
                "run",
                "-p",
                "dj_engine",
                "--bin",
                "runtime_preview",
                "--",
                "--project",
                "/tmp/project.json"
            ]
        );
    }

    #[test]
    fn test_resolve_runtime_preview_command_returns_structured_error_without_fallback() {
        let current_exe = Path::new("/tmp/dj_engine");
        let error = resolve_runtime_preview_command_from_mode(
            Some(current_exe),
            Path::new("/tmp/project.json"),
            false,
        )
        .unwrap_err();

        assert_eq!(
            error,
            RuntimePreviewLaunchError::RuntimePreviewExecutableNotFound(PathBuf::from(format!(
                "/tmp/runtime_preview{}",
                std::env::consts::EXE_SUFFIX
            )))
        );
    }

    fn build_project_action_test_world(temp_dir: &tempfile::TempDir) -> World {
        let mut project = Project::new("Disk Project");
        project.id = "project-action".into();
        let manifest_path = temp_dir.path().join("project.json");
        loader::save_project(&project, &manifest_path).unwrap();

        let mut world = World::new();
        world.insert_resource(MountedProject {
            root_path: Some(temp_dir.path().to_path_buf()),
            manifest_path: Some(manifest_path),
            project: Some(project),
        });
        world.init_resource::<ActiveStoryGraph>();
        world.init_resource::<EditorSnapshotBaseline>();
        world.init_resource::<EditorDirtyState>();
        sync_editor_snapshot_baseline(&mut world).unwrap();
        world
    }

    #[test]
    fn test_request_project_action_queues_when_dirty() {
        let temp_dir = tempfile::tempdir().unwrap();
        let mut world = build_project_action_test_world(&temp_dir);
        world.resource_mut::<EditorDirtyState>().is_dirty = true;

        request_project_action(&mut world, PendingProjectAction::ReloadProject);

        assert_eq!(
            world.resource::<EditorDirtyState>().pending_project_action,
            Some(PendingProjectAction::ReloadProject)
        );
    }

    #[test]
    fn test_request_project_action_reloads_immediately_when_clean() {
        let temp_dir = tempfile::tempdir().unwrap();
        let mut world = build_project_action_test_world(&temp_dir);
        world
            .resource_mut::<MountedProject>()
            .project
            .as_mut()
            .unwrap()
            .name = "Unsaved Name".into();

        request_project_action(&mut world, PendingProjectAction::ReloadProject);

        assert_eq!(
            world
                .resource::<MountedProject>()
                .project
                .as_ref()
                .unwrap()
                .name,
            "Disk Project"
        );
        assert_eq!(
            world.resource::<EditorDirtyState>().pending_project_action,
            None
        );
    }

    #[test]
    fn test_resolve_pending_project_action_discard_changes_restores_disk_state() {
        let temp_dir = tempfile::tempdir().unwrap();
        let mut world = build_project_action_test_world(&temp_dir);
        world
            .resource_mut::<MountedProject>()
            .project
            .as_mut()
            .unwrap()
            .name = "Unsaved Name".into();
        world.resource_mut::<EditorDirtyState>().is_dirty = true;
        world
            .resource_mut::<EditorDirtyState>()
            .pending_project_action = Some(PendingProjectAction::ReloadProject);

        resolve_pending_project_action(&mut world, PendingProjectActionResolution::DiscardChanges);

        assert_eq!(
            world
                .resource::<MountedProject>()
                .project
                .as_ref()
                .unwrap()
                .name,
            "Disk Project"
        );
        assert_eq!(
            world.resource::<EditorDirtyState>().pending_project_action,
            None
        );
    }

    #[test]
    fn test_resolve_pending_project_action_save_and_continue_persists_changes() {
        let temp_dir = tempfile::tempdir().unwrap();
        let mut world = build_project_action_test_world(&temp_dir);
        world
            .resource_mut::<MountedProject>()
            .project
            .as_mut()
            .unwrap()
            .name = "Saved Name".into();
        world.resource_mut::<EditorDirtyState>().is_dirty = true;
        world
            .resource_mut::<EditorDirtyState>()
            .pending_project_action = Some(PendingProjectAction::ReloadProject);

        resolve_pending_project_action(&mut world, PendingProjectActionResolution::SaveAndContinue);

        assert_eq!(
            world
                .resource::<MountedProject>()
                .project
                .as_ref()
                .unwrap()
                .name,
            "Saved Name"
        );
        assert_eq!(
            world.resource::<EditorDirtyState>().pending_project_action,
            None
        );
    }
}
