use bevy::prelude::*;
use bevy::window::WindowResolution;
use dj_engine::editor::EditorPlugin;
use dj_engine::prelude::*;
use dj_engine_helix::HelixDataPlugin;

fn main() {
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

    app.run();
}
