use bevy::prelude::*;
use bevy::window::WindowResolution;
use dj_engine::editor::EditorPlugin;
use dj_engine::prelude::*;
use dj_engine_helix::{HelixDataPlugin, HelixImportConfig};
use std::path::PathBuf;

fn main() {
    let helix_dist = parse_helix_editor_dist_arg(std::env::args());

    let mut app = App::new();

    app.add_plugins(
        DefaultPlugins
            .set(ImagePlugin::default_nearest())
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "DJ Engine - Helix Editor".into(),
                    resolution: WindowResolution::new(1280, 720).with_scale_factor_override(1.0),
                    position: WindowPosition::Centered(MonitorSelection::Primary),
                    present_mode: bevy::window::PresentMode::AutoVsync,
                    ..default()
                }),
                ..default()
            }),
    )
    .add_plugins(DJEnginePlugin::default())
    .add_plugins(HelixDataPlugin)
    .add_plugins(EditorPlugin);

    if let Some(dist_path) = helix_dist {
        app.insert_resource(HelixImportConfig {
            helix_dist_path: Some(dist_path),
        });
    }

    app.run();
}

fn parse_helix_editor_dist_arg(args: impl IntoIterator<Item = String>) -> Option<PathBuf> {
    let args: Vec<String> = args.into_iter().collect();
    let mut i = 1;
    while i < args.len() {
        if args[i] == "--helix-dist" && i + 1 < args.len() {
            return Some(PathBuf::from(&args[i + 1]));
        }
        i += 1;
    }
    None
}
