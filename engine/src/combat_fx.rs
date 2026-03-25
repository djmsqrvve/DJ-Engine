//! Floating combat text for DJ Engine.
//!
//! Spawns animated damage numbers that float upward and fade out
//! when [`DamageEvent`] fires. Crits show in a different color and size.

use bevy::prelude::*;

use crate::combat::DamageEvent;

/// Component on floating text entities — drives the float + fade animation.
#[derive(Component)]
pub struct FloatingCombatText {
    pub lifetime: Timer,
    pub velocity: Vec2,
    pub fade_start: f32,
}

impl FloatingCombatText {
    pub fn new(duration: f32) -> Self {
        Self {
            lifetime: Timer::from_seconds(duration, TimerMode::Once),
            velocity: Vec2::new(0.0, 60.0), // float upward
            fade_start: duration * 0.5,     // start fading at 50%
        }
    }
}

/// Configuration for combat text appearance.
#[derive(Resource)]
pub struct CombatTextConfig {
    pub normal_color: Color,
    pub crit_color: Color,
    pub normal_size: f32,
    pub crit_size: f32,
    pub duration: f32,
}

impl Default for CombatTextConfig {
    fn default() -> Self {
        Self {
            normal_color: Color::WHITE,
            crit_color: Color::srgb(1.0, 0.8, 0.0),
            normal_size: 20.0,
            crit_size: 28.0,
            duration: 1.2,
        }
    }
}

/// System that spawns floating text on damage events.
pub fn spawn_combat_text(
    mut commands: Commands,
    mut damage_events: MessageReader<DamageEvent>,
    config: Res<CombatTextConfig>,
    transform_query: Query<&Transform>,
) {
    for event in damage_events.read() {
        // Get target position for text spawn
        let position = transform_query
            .get(event.target)
            .map(|t| t.translation)
            .unwrap_or(Vec3::ZERO);

        let (color, size) = if event.is_critical {
            (config.crit_color, config.crit_size)
        } else {
            (config.normal_color, config.normal_size)
        };

        let display = if event.is_critical {
            format!("{}!", event.final_damage)
        } else {
            format!("{}", event.final_damage)
        };

        // Spawn as world-space Text2d (not UI) so it floats at the entity position
        commands.spawn((
            Text2d::new(display),
            TextFont {
                font_size: size,
                ..default()
            },
            TextColor(color),
            Transform::from_translation(position + Vec3::new(0.0, 20.0, 100.0)),
            FloatingCombatText::new(config.duration),
        ));
    }
}

/// System that animates floating text — move up, fade out, despawn when done.
pub fn animate_combat_text(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(
        Entity,
        &mut Transform,
        &mut TextColor,
        &mut FloatingCombatText,
    )>,
) {
    let dt = time.delta_secs();

    for (entity, mut transform, mut color, mut fct) in query.iter_mut() {
        fct.lifetime.tick(time.delta());

        // Float upward
        transform.translation.x += fct.velocity.x * dt;
        transform.translation.y += fct.velocity.y * dt;

        // Fade out in second half of lifetime
        let elapsed = fct.lifetime.elapsed_secs();
        if elapsed > fct.fade_start {
            let fade_progress = (elapsed - fct.fade_start)
                / (fct.lifetime.duration().as_secs_f32() - fct.fade_start);
            let alpha = (1.0 - fade_progress).max(0.0);
            color.0 = color.0.with_alpha(alpha);
        }

        // Despawn when done
        if fct.lifetime.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

/// Plugin providing floating combat text.
pub struct CombatFxPlugin;

impl Plugin for CombatFxPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CombatTextConfig>()
            .add_systems(Update, (spawn_combat_text, animate_combat_text).chain());

        use crate::contracts::{AppContractExt, ContractEntry, PluginContract};
        app.register_contract(PluginContract {
            name: "CombatFxPlugin".into(),
            description: "Floating damage numbers on combat hits".into(),
            resources: vec![ContractEntry::of::<CombatTextConfig>(
                "Combat text color/size/duration config",
            )],
            components: vec![ContractEntry::of::<FloatingCombatText>(
                "Floating text animation state",
            )],
            events: vec![],
            system_sets: vec![],
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_floating_text_fades() {
        let mut fct = FloatingCombatText::new(1.0);
        assert!(!fct.lifetime.is_finished());

        // Simulate time passing
        fct.lifetime.tick(std::time::Duration::from_secs_f32(0.6));
        assert!(fct.lifetime.elapsed_secs() > fct.fade_start);
        assert!(!fct.lifetime.is_finished());

        fct.lifetime.tick(std::time::Duration::from_secs_f32(0.5));
        assert!(fct.lifetime.is_finished());
    }

    #[test]
    fn test_config_defaults() {
        let config = CombatTextConfig::default();
        assert_eq!(config.normal_size, 20.0);
        assert_eq!(config.crit_size, 28.0);
        assert_eq!(config.duration, 1.2);
    }
}
