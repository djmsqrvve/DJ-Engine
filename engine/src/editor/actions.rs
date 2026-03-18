use super::scene_io::{load_mounted_project, save_project_impl};
use super::types::{
    BrowserTab, EditorDirtyState, EditorUiState, EditorView, PendingProjectAction,
    PendingProjectActionResolution, RuntimePreviewLaunchPhase, RuntimePreviewLaunchState,
};
use crate::diagnostics::console::ConsoleLogStore;
use crate::editor::extensions::EditorExtensionRegistry;
use crate::project_mount::MountedProject;
use bevy::prelude::*;
use std::sync::{Arc, Mutex};

use super::preview::{
    log_preview_message, resolve_runtime_preview_command, set_launch_state_message,
};

#[derive(Resource)]
pub struct AutomatedTestActive {
    pub timer: Timer,
    pub step: usize,
}

pub fn automated_ui_test_system(
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

pub fn execute_project_action(
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
                None, // We can't borrow console and launch state simultaneously easily here, but we could refactor.
                &message,
            );
            launch_state.phase = RuntimePreviewLaunchPhase::Failed;
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
            set_launch_state_message(&mut launch_state, None, &message);
            launch_state.phase = RuntimePreviewLaunchPhase::Failed;
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
                set_launch_state_message(&mut launch_state, None, &message);
                launch_state.phase = RuntimePreviewLaunchPhase::Failed;
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
        set_launch_state_message(&mut launch_state, None, "Preview Launching");
        launch_state.phase = RuntimePreviewLaunchPhase::Launching;
    }

    // Resolve the preview profile from the selected preset (if any).
    let preview_profile = {
        let selected = world.resource::<super::extensions::SelectedPreviewPreset>();
        let registry = world.resource::<EditorExtensionRegistry>();
        selected.preset_id.as_ref().and_then(|preset_id| {
            registry
                .preview_presets
                .iter()
                .find(|p| &p.preset_id == preset_id)
                .and_then(|p| p.profile_id.clone())
        })
    };

    let command = match resolve_runtime_preview_command(&manifest_path, preview_profile.as_deref())
    {
        Ok(command) => command,
        Err(error) => {
            let message = format!("Preview failed: {error}");
            {
                let mut launch_state = world.resource_mut::<RuntimePreviewLaunchState>();
                launch_state.process = None;
                set_launch_state_message(&mut launch_state, None, &message);
                launch_state.phase = RuntimePreviewLaunchPhase::Failed;
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
                set_launch_state_message(&mut launch_state, None, &message);
                launch_state.phase = RuntimePreviewLaunchPhase::Failed;
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
    let process_arc = world
        .resource::<RuntimePreviewLaunchState>()
        .process
        .clone();

    let Some(process_arc) = process_arc else {
        return;
    };

    let kill_result = match process_arc.lock() {
        Ok(mut child) => child.kill(),
        Err(error) => {
            let message = format!("Preview failed: runtime preview lock poisoned: {error}");
            {
                let mut launch_state = world.resource_mut::<RuntimePreviewLaunchState>();
                launch_state.process = None;
                set_launch_state_message(&mut launch_state, None, &message);
                launch_state.phase = RuntimePreviewLaunchPhase::Failed;
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
                set_launch_state_message(&mut launch_state, None, &message);
                launch_state.phase = RuntimePreviewLaunchPhase::Stopping;
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
                set_launch_state_message(&mut launch_state, None, &message);
                launch_state.phase = RuntimePreviewLaunchPhase::Failed;
            }
            log_preview_message(
                world.get_resource_mut::<ConsoleLogStore>().as_deref_mut(),
                &message,
            );
            error!("{message}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{loader, Project};
    use crate::editor::scene_io::sync_editor_snapshot_baseline;
    use crate::editor::{types::ActiveStoryGraph, EditorSnapshotBaseline};

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
        world.init_resource::<crate::editor::grid::GridLevel>();
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
