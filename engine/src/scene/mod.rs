//! Scene management for visual novel style backgrounds and transitions.
//!
//! Provides components for static backgrounds and systems for handling
//! cross-fade transitions between scenes.

use bevy::prelude::*;

/// Component marking an entity as a background image.
#[derive(Component)]
pub struct SceneBackground;

/// State of the transition effect.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TransitionState {
    #[default]
    Idle,
    FadingOut,
    FadingIn,
}

/// Resource managing scene transitions.
#[derive(Resource, Default)]
pub struct SceneManager {
    /// Current state of the transition
    pub state: TransitionState,
    /// Current alpha value of the overlay (0.0 = transparent, 1.0 = black)
    pub alpha: f32,
    /// Speed of the transition fade
    pub speed: f32,
    /// Path to the next background image to load
    pub next_background: Option<String>,
}

/// Pure transition tick logic, extracted for testability.
///
/// Given the current state, alpha, speed, and delta time, returns the next (state, alpha).
/// Does not handle background swapping — that remains in the system.
pub fn tick_transition(
    state: TransitionState,
    alpha: f32,
    speed: f32,
    dt: f32,
) -> (TransitionState, f32) {
    match state {
        TransitionState::FadingOut => {
            let new_alpha = (alpha + speed * dt).min(1.0);
            if new_alpha >= 1.0 {
                (TransitionState::FadingIn, new_alpha)
            } else {
                (TransitionState::FadingOut, new_alpha)
            }
        }
        TransitionState::FadingIn => {
            let new_alpha = (alpha - speed * dt).max(0.0);
            if new_alpha <= 0.0 {
                (TransitionState::Idle, new_alpha)
            } else {
                (TransitionState::FadingIn, new_alpha)
            }
        }
        other => (other, alpha),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transition_state_default() {
        assert_eq!(TransitionState::default(), TransitionState::Idle);
    }

    #[test]
    fn test_scene_manager_default() {
        let m = SceneManager::default();
        assert_eq!(m.state, TransitionState::Idle);
        assert_eq!(m.alpha, 0.0);
        assert_eq!(m.speed, 0.0);
        assert!(m.next_background.is_none());
    }

    #[test]
    fn test_tick_fading_out_advances_alpha() {
        let (state, alpha) = tick_transition(TransitionState::FadingOut, 0.0, 1.0, 0.3);
        assert_eq!(state, TransitionState::FadingOut);
        assert!((alpha - 0.3).abs() < f32::EPSILON);
    }

    #[test]
    fn test_tick_fading_out_transitions_to_fading_in_at_full() {
        let (state, alpha) = tick_transition(TransitionState::FadingOut, 0.9, 1.0, 0.5);
        assert_eq!(state, TransitionState::FadingIn);
        assert_eq!(alpha, 1.0);
    }

    #[test]
    fn test_tick_fading_in_decreases_alpha() {
        let (state, alpha) = tick_transition(TransitionState::FadingIn, 1.0, 1.0, 0.4);
        assert_eq!(state, TransitionState::FadingIn);
        assert!((alpha - 0.6).abs() < f32::EPSILON);
    }

    #[test]
    fn test_tick_fading_in_transitions_to_idle_at_zero() {
        let (state, alpha) = tick_transition(TransitionState::FadingIn, 0.1, 1.0, 0.5);
        assert_eq!(state, TransitionState::Idle);
        assert_eq!(alpha, 0.0);
    }

    #[test]
    fn test_tick_idle_is_noop() {
        let (state, alpha) = tick_transition(TransitionState::Idle, 0.5, 1.0, 1.0);
        assert_eq!(state, TransitionState::Idle);
        assert_eq!(alpha, 0.5);
    }
}

/// Message to trigger a scene change.
#[derive(Message)]
pub struct ChangeSceneEvent {
    /// Path to the new background image
    pub background_path: String,
    /// Duration of the fade transition in seconds
    pub duration: f32,
}

/// Plugin providing scene management.
pub struct DJScenePlugin;

impl Plugin for DJScenePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SceneManager>()
            .add_message::<ChangeSceneEvent>()
            .add_systems(Startup, setup_transition_overlay)
            .add_systems(Update, (handle_scene_change, update_transition));

        info!("DJ Scene Plugin initialized");
    }
}

/// Setup the UI overlay for transitions.
fn setup_transition_overlay(mut commands: Commands) {
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.0)),
        ZIndex(100), // Ensure it's on top of everything
        TransitionOverlay,
    ));
}

/// Marker component for the black overlay.
#[derive(Component)]
pub struct TransitionOverlay;

/// System to handle scene change events.
fn handle_scene_change(
    mut events: MessageReader<ChangeSceneEvent>,
    mut manager: ResMut<SceneManager>,
) {
    for event in events.read() {
        if manager.state != TransitionState::Idle {
            warn!("Ignored scene change request while transition already active");
            continue;
        }

        info!("Starting scene transition to: {}", event.background_path);

        // Start fade out to black
        manager.state = TransitionState::FadingOut;
        manager.alpha = 0.0;
        manager.speed = 1.0 / event.duration.max(0.1);
        manager.next_background = Some(event.background_path.clone());
    }
}

/// System to update the transition fade effect.
fn update_transition(
    mut commands: Commands,
    mut manager: ResMut<SceneManager>,
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    mut overlay_query: Query<&mut BackgroundColor, With<TransitionOverlay>>,
    bg_query: Query<Entity, With<SceneBackground>>,
) {
    if manager.state == TransitionState::Idle {
        return;
    }

    let dt = time.delta_secs();

    let prev_state = manager.state;
    let (next_state, next_alpha) = tick_transition(manager.state, manager.alpha, manager.speed, dt);
    manager.state = next_state;
    manager.alpha = next_alpha;

    // When FadingOut just reached full black, swap the background
    if prev_state == TransitionState::FadingOut && next_state == TransitionState::FadingIn {
        for entity in bg_query.iter() {
            commands.entity(entity).despawn();
        }
        if let Some(path) = manager.next_background.take() {
            let texture = asset_server.load(path);
            commands.spawn((
                Sprite {
                    image: texture,
                    custom_size: Some(Vec2::new(320.0, 240.0)),
                    ..default()
                },
                Transform::from_translation(Vec3::new(0.0, 0.0, -10.0)),
                SceneBackground,
            ));
        }
    }

    if prev_state == TransitionState::FadingIn && next_state == TransitionState::Idle {
        info!("Scene transition complete");
    }

    // Apply alpha to overlay
    for mut bg_color in overlay_query.iter_mut() {
        bg_color.0 = Color::srgba(0.0, 0.0, 0.0, manager.alpha);
    }
}
