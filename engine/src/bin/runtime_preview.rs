use bevy::prelude::*;
use bevy::window::WindowResolution;
use dj_engine::prelude::*;
use dj_engine::runtime_preview::{
    bootstrap_mounted_project, parse_runtime_preview_cli_args, RuntimePreviewPlugin,
    RuntimePreviewProfileOverride,
};

fn main() {
    let cli = parse_runtime_preview_cli_args(std::env::args());
    let mounted_project = cli
        .project_path
        .as_deref()
        .map(|path| {
            bootstrap_mounted_project(path).unwrap_or_else(|error| {
                eprintln!("Runtime Preview: Failed to mount project: {error}");
                MountedProject::default()
            })
        })
        .unwrap_or_default();

    let (window_title, resolution) = mounted_project
        .project
        .as_ref()
        .map(|project| {
            (
                format!("{} - Runtime Preview", project.name),
                WindowResolution::new(
                    project.settings.default_resolution.width,
                    project.settings.default_resolution.height,
                )
                .with_scale_factor_override(1.0),
            )
        })
        .unwrap_or_else(|| {
            (
                "DJ Engine - Runtime Preview".to_string(),
                WindowResolution::new(1280, 720).with_scale_factor_override(1.0),
            )
        });

    let mut app = App::new();
    app.insert_resource(mounted_project);
    app.add_plugins(
        DefaultPlugins
            .set(ImagePlugin::default_nearest())
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: window_title,
                    resolution,
                    position: WindowPosition::Centered(MonitorSelection::Primary),
                    present_mode: bevy::window::PresentMode::AutoVsync,
                    ..default()
                }),
                ..default()
            }),
    )
    .add_plugins(DJEnginePlugin::default())
    .add_plugins(RuntimePreviewPlugin::new(cli.test_mode));

    if let Some(profile_id) = cli.preview_profile {
        app.insert_resource(RuntimePreviewProfileOverride {
            profile_id: Some(profile_id),
        });
    }

    app.run();
}
