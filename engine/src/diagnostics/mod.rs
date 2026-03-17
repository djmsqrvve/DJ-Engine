//! Engine diagnostics and performance monitoring.

use crate::{story_graph::GraphExecutor, types::DiagnosticConfig};
use bevy::{
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::*,
};

pub mod console;
pub mod inspector;

/// Plugin that provides diagnostic overlays and performance tracking.
pub struct DiagnosticsPlugin;

impl Plugin for DiagnosticsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<DiagnosticConfig>()
            .init_resource::<DiagnosticConfig>()
            .add_plugins(FrameTimeDiagnosticsPlugin::default())
            .add_systems(Startup, setup_diagnostics)
            .add_systems(
                Update,
                (
                    toggle_diagnostics_system,
                    update_diagnostics_system.run_if(resource_exists::<DiagnosticConfig>),
                    console_fps_logger_system.run_if(resource_exists::<DiagnosticConfig>),
                ),
            )
            // Disabled temporarily due to WSL2/LLVMpipe compatibility issues (incompatible window kind panic)
            // .add_plugins(inspector::InspectorPlugin)
            .add_plugins(console::ConsolePlugin);

        use crate::contracts::{AppContractExt, ContractEntry, PluginContract};
        app.register_contract(PluginContract {
            name: "DiagnosticsPlugin".into(),
            description: "FPS overlay, console logger, inspector".into(),
            resources: vec![ContractEntry::of::<DiagnosticConfig>(
                "Diagnostic overlay configuration",
            )],
            components: vec![],
            events: vec![],
            system_sets: vec![],
        });
    }
}

/// Marker for the diagnostic text UI.
#[derive(Component)]
struct DiagnosticText;

/// Marker for the diagnostic overlay root.
#[derive(Component)]
struct DiagnosticOverlay;

fn setup_diagnostics(mut commands: Commands, config: Res<DiagnosticConfig>) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(5.0),
                left: Val::Px(5.0),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            DiagnosticOverlay,
            Visibility::Inherited,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Diagnostics Initializing..."),
                TextFont {
                    font_size: config.font_size,
                    ..default()
                },
                TextColor(config.text_color),
                DiagnosticText,
            ));
        });
}

fn toggle_diagnostics_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut config: ResMut<DiagnosticConfig>,
    mut query: Query<&mut Visibility, With<DiagnosticOverlay>>,
) {
    if keys.just_pressed(KeyCode::F3) {
        config.enabled = !config.enabled;
        for mut vis in &mut query {
            *vis = if config.enabled {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            };
        }
        info!("Diagnostics toggled: {}", config.enabled);
    }
}

fn update_diagnostics_system(
    time: Res<Time>,
    diagnostics: Res<DiagnosticsStore>,
    mut config: ResMut<DiagnosticConfig>,
    mut query: Query<(&mut Text, &mut TextColor), With<DiagnosticText>>,
    entities: Query<Entity>,
    executor: Option<Res<GraphExecutor>>,
    windows: Query<&Window>,
) {
    if !config.enabled {
        return;
    }

    config.update_timer.tick(time.delta());
    if !config.update_timer.just_finished() {
        return;
    }

    let fps = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|diag| diag.smoothed());

    let frame_time = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FRAME_TIME)
        .and_then(|diag| diag.smoothed());

    let entity_count = entities.iter().count();

    let story_status = if let Some(exec) = executor {
        format!("{:?}", exec.status)
    } else {
        "No Executor".to_string()
    };

    let (width, height) = if let Ok(window) = windows.single() {
        (window.width() as i32, window.height() as i32)
    } else {
        (0, 0)
    };

    if let Ok((mut text, mut color)) = query.single_mut() {
        let fps_text = fps.map_or("N/A".to_string(), |v| format!("{:.1}", v));
        let ms_text = frame_time.map_or("N/A".to_string(), |v| format!("{:.2}", v));

        text.0 = format!(
            "FPS: {}\nFrame Time: {}ms\nEntities: {}\nStoryStatus: {}\nWindow: {}x{}",
            fps_text, ms_text, entity_count, story_status, width, height
        );

        // Color coding based on performance
        if let Some(v) = fps {
            let new_color = if v < 30.0 {
                Color::srgb(1.0, 0.0, 0.0) // Red
            } else if v < 55.0 {
                Color::srgb(1.0, 1.0, 0.0) // Yellow
            } else {
                config.text_color // Preferred green
            };
            color.0 = new_color;
        }
    }
}

/// Periodic console output for remote performance monitoring.
fn console_fps_logger_system(
    time: Res<Time>,
    diagnostics: Res<DiagnosticsStore>,
    mut last_log: Local<f32>,
    windows: Query<&Window>,
) {
    if time.elapsed_secs() - *last_log > 5.0 {
        let fps = diagnostics
            .get(&FrameTimeDiagnosticsPlugin::FPS)
            .and_then(|diag| diag.smoothed())
            .unwrap_or(0.0);

        let (w, h) = if let Ok(window) = windows.single() {
            (window.width() as i32, window.height() as i32)
        } else {
            (0, 0)
        };

        info!("Performance: {:.1} FPS | Window: {}x{}", fps, w, h);
        *last_log = time.elapsed_secs();
    }
}
