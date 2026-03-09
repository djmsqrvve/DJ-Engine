use bevy::prelude::*;
use bevy::window::WindowResolution;
use dj_engine::editor::EditorPlugin;
use dj_engine::prelude::*;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let project_path = args.get(1).map(std::path::PathBuf::from);

    let mut app = App::new();

    app.add_plugins(
        DefaultPlugins
            .set(ImagePlugin::default_nearest()) // Pixel art friendly
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "DJ Engine - Editor Mode".into(),
                    resolution: WindowResolution::new(1280, 720).with_scale_factor_override(1.0),
                    position: WindowPosition::Centered(MonitorSelection::Primary),
                    present_mode: bevy::window::PresentMode::AutoVsync,
                    ..default()
                }),
                ..default()
            }),
    )
    // Engine plugins (bundled)
    .add_plugins(DJEnginePlugin::default())
    // Editor UI
    .add_plugins(EditorPlugin);

    // If a project path was provided, mount it
    if let Some(path) = project_path {
        app.insert_resource(dj_engine::editor::ProjectMetadata {
            name: path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into(),
            path: Some(path),
        });
    }

    app.run();
}
