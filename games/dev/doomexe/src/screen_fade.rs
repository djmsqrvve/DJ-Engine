//! Screen fade transitions — fade to/from black between areas.
//!
//! Fire a [`FadeOutEvent`] to start a fade-to-black. When the fade
//! completes, a [`FadeComplete`] event is emitted. Use [`FadeInEvent`]
//! on area entry to fade from black.

use bevy::prelude::*;

use crate::state::GameState;

/// Marker for the fullscreen fade overlay.
#[derive(Component)]
pub struct FadeOverlay;

/// Request a fade-to-black. Duration in seconds.
#[derive(Message, Debug, Clone)]
pub struct FadeOutEvent {
    pub duration: f32,
}

/// Request a fade-from-black. Duration in seconds.
#[derive(Message, Debug, Clone)]
pub struct FadeInEvent {
    pub duration: f32,
}

/// Emitted when a fade-out completes (screen fully black).
#[derive(Message, Debug, Clone)]
pub struct FadeComplete;

/// Active fade animation state.
#[derive(Resource, Default)]
pub struct FadeState {
    pub active: bool,
    pub fading_out: bool,
    pub timer: f32,
    pub duration: f32,
}

impl FadeState {
    pub fn progress(&self) -> f32 {
        if self.duration <= 0.0 {
            return 1.0;
        }
        (self.timer / self.duration).clamp(0.0, 1.0)
    }
}

pub struct ScreenFadePlugin;

impl Plugin for ScreenFadePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FadeState>()
            .add_systems(Startup, setup_fade_overlay)
            .add_systems(Update, (handle_fade_events, animate_fade).chain());
    }
}

fn setup_fade_overlay(mut commands: Commands) {
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(0.0),
            left: Val::Px(0.0),
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.0)),
        FadeOverlay,
        GlobalZIndex(100), // above everything
    ));
}

fn handle_fade_events(
    mut fade_out_events: MessageReader<FadeOutEvent>,
    mut fade_in_events: MessageReader<FadeInEvent>,
    mut state: ResMut<FadeState>,
) {
    for event in fade_out_events.read() {
        state.active = true;
        state.fading_out = true;
        state.timer = 0.0;
        state.duration = event.duration;
    }
    for event in fade_in_events.read() {
        state.active = true;
        state.fading_out = false;
        state.timer = 0.0;
        state.duration = event.duration;
    }
}

fn animate_fade(
    time: Res<Time>,
    mut state: ResMut<FadeState>,
    mut overlay_query: Query<&mut BackgroundColor, With<FadeOverlay>>,
    mut complete_events: MessageWriter<FadeComplete>,
) {
    if !state.active {
        return;
    }

    state.timer += time.delta_secs();
    let progress = state.progress();

    let alpha = if state.fading_out {
        progress // 0 → 1 (transparent → black)
    } else {
        1.0 - progress // 1 → 0 (black → transparent)
    };

    for mut bg in overlay_query.iter_mut() {
        bg.0 = Color::srgba(0.0, 0.0, 0.0, alpha);
    }

    if progress >= 1.0 {
        state.active = false;
        if state.fading_out {
            complete_events.write(FadeComplete);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fade_state_default() {
        let state = FadeState::default();
        assert!(!state.active);
        assert!(!state.fading_out);
        assert_eq!(state.timer, 0.0);
        assert_eq!(state.duration, 0.0);
    }

    #[test]
    fn test_fade_progress_zero_duration() {
        let state = FadeState {
            active: true,
            duration: 0.0,
            timer: 0.0,
            fading_out: true,
        };
        assert_eq!(state.progress(), 1.0);
    }

    #[test]
    fn test_fade_progress_midway() {
        let state = FadeState {
            active: true,
            duration: 1.0,
            timer: 0.5,
            fading_out: true,
        };
        assert!((state.progress() - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_fade_progress_clamped() {
        let state = FadeState {
            active: true,
            duration: 1.0,
            timer: 2.0, // over duration
            fading_out: true,
        };
        assert_eq!(state.progress(), 1.0);
    }

    #[test]
    fn test_fade_out_alpha_progression() {
        // Fade out: alpha goes 0 → 1
        let state = FadeState {
            active: true,
            duration: 1.0,
            timer: 0.5,
            fading_out: true,
        };
        let alpha = state.progress(); // 0.5
        assert!((alpha - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_fade_in_alpha_progression() {
        // Fade in: alpha goes 1 → 0
        let state = FadeState {
            active: true,
            duration: 1.0,
            timer: 0.5,
            fading_out: false,
        };
        let alpha = 1.0 - state.progress(); // 0.5
        assert!((alpha - 0.5).abs() < f32::EPSILON);
    }
}
