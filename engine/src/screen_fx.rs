//! Screen-level visual effects for DJ Engine.
//!
//! Provides screen shake, screen flash, and low-health vignette.
//! These are engine-level systems available to all game crates.
//!
//! Trigger effects via [`ScreenShakeEvent`] and [`ScreenFlashEvent`],
//! or use [`LowHealthVignette`] for automatic HP-based edge darkening.

use bevy::prelude::*;

use crate::rendering::MainCamera;

// ---------------------------------------------------------------------------
// Events
// ---------------------------------------------------------------------------

/// Trigger a screen shake. Camera returns to origin after duration.
#[derive(Message, Debug, Clone)]
pub struct ScreenShakeEvent {
    pub intensity: f32,
    pub duration: f32,
}

impl ScreenShakeEvent {
    /// Light shake — minor hit.
    pub fn light() -> Self {
        Self {
            intensity: 2.0,
            duration: 0.15,
        }
    }

    /// Medium shake — significant damage.
    pub fn medium() -> Self {
        Self {
            intensity: 4.0,
            duration: 0.25,
        }
    }

    /// Heavy shake — boss hit, explosion.
    pub fn heavy() -> Self {
        Self {
            intensity: 8.0,
            duration: 0.4,
        }
    }
}

/// Trigger a fullscreen color flash that fades out.
#[derive(Message, Debug, Clone)]
pub struct ScreenFlashEvent {
    pub color: Color,
    pub duration: f32,
    pub intensity: f32,
}

impl ScreenFlashEvent {
    /// Gold flash — loot, level up.
    pub fn gold() -> Self {
        Self {
            color: Color::srgba(1.0, 0.85, 0.2, 0.4),
            duration: 0.5,
            intensity: 1.0,
        }
    }

    /// Red flash — damage taken.
    pub fn damage() -> Self {
        Self {
            color: Color::srgba(0.8, 0.0, 0.0, 0.3),
            duration: 0.2,
            intensity: 1.0,
        }
    }

    /// White flash — critical hit, parry.
    pub fn white() -> Self {
        Self {
            color: Color::srgba(1.0, 1.0, 1.0, 0.5),
            duration: 0.15,
            intensity: 1.0,
        }
    }

    /// Green flash — heal received.
    pub fn heal() -> Self {
        Self {
            color: Color::srgba(0.2, 0.9, 0.3, 0.25),
            duration: 0.3,
            intensity: 1.0,
        }
    }
}

// ---------------------------------------------------------------------------
// Resources
// ---------------------------------------------------------------------------

/// Active screen shake state. Managed by the system; not user-facing.
#[derive(Resource, Default)]
pub struct ScreenShakeState {
    pub timer: Option<Timer>,
    pub intensity: f32,
    pub camera_home: Vec3,
}

/// Global screen FX config.
#[derive(Resource)]
pub struct ScreenFxConfig {
    pub shake_enabled: bool,
    pub flash_enabled: bool,
    pub vignette_enabled: bool,
}

impl Default for ScreenFxConfig {
    fn default() -> Self {
        Self {
            shake_enabled: true,
            flash_enabled: true,
            vignette_enabled: true,
        }
    }
}

// ---------------------------------------------------------------------------
// Components
// ---------------------------------------------------------------------------

/// Marks the fullscreen flash overlay entity.
#[derive(Component)]
pub struct ScreenFlashOverlay;

/// Attach to a UI node to create an HP-based vignette overlay.
/// The system updates the alpha based on `hp_fraction` each frame.
#[derive(Component)]
pub struct LowHealthVignette {
    pub hp_fraction: f32,
    pub threshold: f32,
}

impl Default for LowHealthVignette {
    fn default() -> Self {
        Self {
            hp_fraction: 1.0,
            threshold: 0.3,
        }
    }
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

fn setup_flash_overlay(mut commands: Commands) {
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
        ScreenFlashOverlay,
    ));
}

fn handle_screen_shake(
    mut shake: ResMut<ScreenShakeState>,
    events: Option<MessageReader<ScreenShakeEvent>>,
    config: Res<ScreenFxConfig>,
    time: Res<Time>,
    mut camera_query: Query<&mut Transform, With<MainCamera>>,
) {
    if !config.shake_enabled {
        return;
    }

    // Read new shake events
    if let Some(mut events) = events {
        for event in events.read() {
            if shake.timer.is_none() {
                if let Ok(cam) = camera_query.single_mut() {
                    shake.camera_home = cam.translation;
                }
            }
            shake.intensity = event.intensity;
            shake.timer = Some(Timer::from_seconds(event.duration, TimerMode::Once));
        }
    }

    // Animate active shake
    let mut clear_timer = false;
    if let Some(ref mut timer) = shake.timer {
        timer.tick(time.delta());
        let finished = timer.is_finished();
        let fraction = timer.fraction();
        let elapsed = timer.elapsed_secs();

        if let Ok(mut cam) = camera_query.single_mut() {
            if finished {
                cam.translation = shake.camera_home;
                clear_timer = true;
            } else {
                let progress = 1.0 - fraction;
                let decay = progress * shake.intensity;
                let offset_x = (elapsed * 37.0).sin() * decay;
                let offset_y = (elapsed * 53.0).cos() * decay;
                cam.translation.x = shake.camera_home.x + offset_x;
                cam.translation.y = shake.camera_home.y + offset_y;
            }
        }
    }
    if clear_timer {
        shake.timer = None;
    }
}

fn handle_screen_flash(
    events: Option<MessageReader<ScreenFlashEvent>>,
    config: Res<ScreenFxConfig>,
    time: Res<Time>,
    mut flash_query: Query<&mut BackgroundColor, With<ScreenFlashOverlay>>,
) {
    if !config.flash_enabled {
        return;
    }

    let Ok(mut bg) = flash_query.single_mut() else {
        return;
    };

    // Read new flash events — start a new flash
    if let Some(mut events) = events {
        for event in events.read() {
            bg.0 = event
                .color
                .with_alpha(event.intensity * event.color.alpha());
        }
    }

    // Fade existing flash toward transparent
    let current_alpha = bg.0.alpha();
    if current_alpha > 0.001 {
        let new_alpha = (current_alpha - time.delta_secs() * 3.0).max(0.0);
        bg.0 = bg.0.with_alpha(new_alpha);
    }
}

fn update_low_health_vignette(
    mut vignette_query: Query<(&LowHealthVignette, &mut BackgroundColor)>,
) {
    for (vignette, mut bg) in vignette_query.iter_mut() {
        if vignette.hp_fraction < vignette.threshold {
            let severity = 1.0 - (vignette.hp_fraction / vignette.threshold);
            let alpha = severity * 0.3;
            bg.0 = Color::srgba(0.5, 0.0, 0.0, alpha);
        } else {
            bg.0 = Color::srgba(0.0, 0.0, 0.0, 0.0);
        }
    }
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct ScreenFxPlugin;

impl Plugin for ScreenFxPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ScreenShakeState>()
            .init_resource::<ScreenFxConfig>()
            .add_systems(Startup, setup_flash_overlay)
            .add_systems(
                Update,
                (
                    handle_screen_shake,
                    handle_screen_flash,
                    update_low_health_vignette,
                ),
            );

        use crate::contracts::{AppContractExt, ContractEntry, PluginContract};
        app.register_contract(PluginContract {
            name: "ScreenFxPlugin".into(),
            description: "Screen shake, flash, and low-health vignette effects".into(),
            resources: vec![
                ContractEntry::of::<ScreenFxConfig>("Global screen effects on/off toggles"),
                ContractEntry::of::<ScreenShakeState>("Active shake animation state"),
            ],
            components: vec![
                ContractEntry::of::<ScreenFlashOverlay>("Fullscreen flash overlay marker"),
                ContractEntry::of::<LowHealthVignette>("HP-driven vignette overlay"),
            ],
            events: vec![
                ContractEntry::of::<ScreenShakeEvent>("Trigger a screen shake"),
                ContractEntry::of::<ScreenFlashEvent>("Trigger a fullscreen color flash"),
            ],
            system_sets: vec![],
        });
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shake_presets() {
        let light = ScreenShakeEvent::light();
        let medium = ScreenShakeEvent::medium();
        let heavy = ScreenShakeEvent::heavy();
        assert!(light.intensity < medium.intensity);
        assert!(medium.intensity < heavy.intensity);
        assert!(light.duration < heavy.duration);
    }

    #[test]
    fn test_flash_presets() {
        let gold = ScreenFlashEvent::gold();
        let damage = ScreenFlashEvent::damage();
        let white = ScreenFlashEvent::white();
        let heal = ScreenFlashEvent::heal();
        assert!(gold.duration > 0.0);
        assert!(damage.duration > 0.0);
        assert!(white.duration > 0.0);
        assert!(heal.duration > 0.0);
    }

    #[test]
    fn test_shake_state_default() {
        let state = ScreenShakeState::default();
        assert!(state.timer.is_none());
        assert_eq!(state.intensity, 0.0);
    }

    #[test]
    fn test_screen_fx_config_defaults() {
        let config = ScreenFxConfig::default();
        assert!(config.shake_enabled);
        assert!(config.flash_enabled);
        assert!(config.vignette_enabled);
    }

    #[test]
    fn test_low_health_vignette_defaults() {
        let v = LowHealthVignette::default();
        assert_eq!(v.hp_fraction, 1.0);
        assert_eq!(v.threshold, 0.3);
    }

    #[test]
    fn test_vignette_alpha_calculation() {
        // Above threshold: no vignette
        let hp_fraction = 0.5_f32;
        let threshold = 0.3_f32;
        assert!(hp_fraction >= threshold);

        // Below threshold: vignette intensifies
        let hp_fraction = 0.15;
        let severity = 1.0 - (hp_fraction / threshold);
        let alpha = severity * 0.3;
        assert!(alpha > 0.0);
        assert!(alpha < 0.3);

        // At zero HP: max vignette
        let hp_fraction = 0.0_f32;
        let severity = 1.0 - (hp_fraction / threshold);
        let alpha = severity * 0.3;
        assert!((alpha - 0.3).abs() < f32::EPSILON);
    }

    #[test]
    fn test_shake_decay_math() {
        let intensity = 4.0_f32;
        // At start (progress = 1.0), decay is at max
        let progress = 1.0_f32;
        let decay = progress * intensity;
        assert_eq!(decay, 4.0);

        // Near end (progress = 0.1), decay is low
        let progress = 0.1;
        let decay = progress * intensity;
        assert!((decay - 0.4).abs() < f32::EPSILON);
    }
}
