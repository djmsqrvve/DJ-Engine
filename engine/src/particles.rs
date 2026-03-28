//! Lightweight particle system for DJ Engine.
//!
//! Provides configurable particle emitters for hit sparks, death bursts,
//! heal effects, and other visual feedback. Trigger particles via
//! [`ParticleEvent`] or attach a [`ParticleEmitter`] component for continuous emission.

use bevy::prelude::*;

use crate::combat::DamageEvent;

// ---------------------------------------------------------------------------
// Components
// ---------------------------------------------------------------------------

/// A single particle entity — spawned by the system, animated, then despawned.
#[derive(Component)]
pub struct Particle {
    pub lifetime: Timer,
    pub velocity: Vec2,
    pub gravity: f32,
    pub fade_start: f32,
    pub initial_scale: f32,
    pub shrink: bool,
}

/// Attach to an entity for continuous particle emission.
#[derive(Component)]
pub struct ParticleEmitter {
    pub config: ParticleConfig,
    pub timer: Timer,
    pub active: bool,
}

impl ParticleEmitter {
    pub fn new(config: ParticleConfig, rate: f32) -> Self {
        Self {
            config,
            timer: Timer::from_seconds(1.0 / rate, TimerMode::Repeating),
            active: true,
        }
    }
}

// ---------------------------------------------------------------------------
// Events
// ---------------------------------------------------------------------------

/// Fire this event to spawn a burst of particles at a position.
#[derive(Message, Debug, Clone)]
pub struct ParticleEvent {
    pub position: Vec3,
    pub config: ParticleConfig,
}

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

/// Describes a particle burst or stream.
#[derive(Debug, Clone)]
pub struct ParticleConfig {
    pub count: u32,
    pub color: Color,
    pub color_end: Option<Color>,
    pub lifetime: f32,
    pub speed: f32,
    pub spread: f32,
    pub gravity: f32,
    pub size: f32,
    pub shrink: bool,
}

impl Default for ParticleConfig {
    fn default() -> Self {
        Self {
            count: 8,
            color: Color::WHITE,
            color_end: None,
            lifetime: 0.6,
            speed: 80.0,
            spread: std::f32::consts::TAU,
            gravity: -120.0,
            size: 3.0,
            shrink: true,
        }
    }
}

impl ParticleConfig {
    /// White sparks that fly outward and fall — for hit impacts.
    pub fn hit_sparks() -> Self {
        Self {
            count: 6,
            color: Color::srgb(1.0, 0.9, 0.6),
            color_end: Some(Color::srgb(1.0, 0.4, 0.1)),
            lifetime: 0.4,
            speed: 100.0,
            spread: std::f32::consts::TAU,
            gravity: -200.0,
            size: 2.5,
            shrink: true,
        }
    }

    /// Red burst that expands outward — for death/defeat.
    pub fn death_burst() -> Self {
        Self {
            count: 12,
            color: Color::srgb(0.9, 0.1, 0.1),
            color_end: Some(Color::srgb(0.3, 0.0, 0.0)),
            lifetime: 0.8,
            speed: 60.0,
            spread: std::f32::consts::TAU,
            gravity: -40.0,
            size: 4.0,
            shrink: true,
        }
    }

    /// Green upward particles — for healing.
    pub fn heal_swirl() -> Self {
        Self {
            count: 8,
            color: Color::srgb(0.2, 1.0, 0.4),
            color_end: Some(Color::srgb(0.8, 1.0, 0.8)),
            lifetime: 0.7,
            speed: 50.0,
            spread: std::f32::consts::FRAC_PI_4,
            gravity: 40.0, // float upward (positive = up in our coordinate system)
            size: 3.0,
            shrink: false,
        }
    }

    /// Gold sparkle — for level up, loot, rewards.
    pub fn gold_sparkle() -> Self {
        Self {
            count: 10,
            color: Color::srgb(1.0, 0.85, 0.2),
            color_end: Some(Color::srgb(1.0, 1.0, 0.8)),
            lifetime: 1.0,
            speed: 30.0,
            spread: std::f32::consts::TAU,
            gravity: 20.0,
            size: 2.0,
            shrink: false,
        }
    }
}

/// Global particle config resource.
#[derive(Resource)]
pub struct ParticleSystemConfig {
    pub enabled: bool,
    pub combat_particles: bool,
}

impl Default for ParticleSystemConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            combat_particles: true,
        }
    }
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

fn spawn_particles_from_events(
    mut commands: Commands,
    events: Option<MessageReader<ParticleEvent>>,
    sys_config: Res<ParticleSystemConfig>,
) {
    if !sys_config.enabled {
        return;
    }
    let Some(mut events) = events else { return };

    for event in events.read() {
        spawn_burst(&mut commands, event.position, &event.config);
    }
}

fn spawn_combat_particles(
    mut commands: Commands,
    damage_events: Option<MessageReader<DamageEvent>>,
    sys_config: Res<ParticleSystemConfig>,
    transform_query: Query<&Transform>,
) {
    if !sys_config.enabled || !sys_config.combat_particles {
        return;
    }
    let Some(mut damage_events) = damage_events else {
        return;
    };

    for event in damage_events.read() {
        let position = transform_query
            .get(event.target)
            .map(|t| t.translation)
            .unwrap_or(Vec3::ZERO);

        let config = if event.is_critical {
            ParticleConfig {
                count: 10,
                ..ParticleConfig::hit_sparks()
            }
        } else {
            ParticleConfig::hit_sparks()
        };

        spawn_burst(&mut commands, position, &config);
    }
}

fn tick_emitters(
    mut commands: Commands,
    time: Res<Time>,
    sys_config: Res<ParticleSystemConfig>,
    mut emitters: Query<(&mut ParticleEmitter, &Transform)>,
) {
    if !sys_config.enabled {
        return;
    }

    for (mut emitter, transform) in emitters.iter_mut() {
        if !emitter.active {
            continue;
        }
        emitter.timer.tick(time.delta());
        let fires = emitter.timer.times_finished_this_tick();
        for _ in 0..fires {
            spawn_burst(&mut commands, transform.translation, &emitter.config);
        }
    }
}

fn animate_particles(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut Sprite, &mut Particle)>,
) {
    let dt = time.delta_secs();

    for (entity, mut transform, mut sprite, mut particle) in query.iter_mut() {
        particle.lifetime.tick(time.delta());

        // Move
        transform.translation.x += particle.velocity.x * dt;
        transform.translation.y += particle.velocity.y * dt;
        particle.velocity.y += particle.gravity * dt;

        // Fade + shrink
        let progress = particle.lifetime.fraction();
        if progress > particle.fade_start {
            let fade = 1.0 - (progress - particle.fade_start) / (1.0 - particle.fade_start);
            sprite.color = sprite.color.with_alpha(fade.max(0.0));
        }
        if particle.shrink {
            let scale = particle.initial_scale * (1.0 - progress * 0.7);
            transform.scale = Vec3::splat(scale);
        }

        if particle.lifetime.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn spawn_burst(commands: &mut Commands, position: Vec3, config: &ParticleConfig) {
    let half_spread = config.spread * 0.5;

    for i in 0..config.count {
        let angle = if config.count == 1 {
            std::f32::consts::FRAC_PI_2
        } else {
            let base = (i as f32 / config.count as f32) * config.spread - half_spread;
            base + std::f32::consts::FRAC_PI_2
        };

        let speed_variance = 0.7 + (i as f32 % 3.0) * 0.15;
        let speed = config.speed * speed_variance;

        let velocity = Vec2::new(angle.cos() * speed, angle.sin() * speed);

        let color = config.color;

        commands.spawn((
            Sprite {
                color,
                custom_size: Some(Vec2::splat(config.size)),
                ..default()
            },
            Transform::from_translation(position + Vec3::new(0.0, 0.0, 90.0)),
            Particle {
                lifetime: Timer::from_seconds(config.lifetime, TimerMode::Once),
                velocity,
                gravity: config.gravity,
                fade_start: 0.4,
                initial_scale: 1.0,
                shrink: config.shrink,
            },
        ));
    }
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct ParticlesPlugin;

impl Plugin for ParticlesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ParticleSystemConfig>().add_systems(
            Update,
            (
                spawn_particles_from_events,
                spawn_combat_particles,
                tick_emitters,
                animate_particles,
            )
                .chain(),
        );

        use crate::contracts::{AppContractExt, ContractEntry, PluginContract};
        app.register_contract(PluginContract {
            name: "ParticlesPlugin".into(),
            description: "Lightweight particle emitters for combat, death, heal, and ability FX"
                .into(),
            resources: vec![ContractEntry::of::<ParticleSystemConfig>(
                "Global particle system on/off + combat auto-spawn toggle",
            )],
            components: vec![
                ContractEntry::of::<Particle>("Individual particle animation state"),
                ContractEntry::of::<ParticleEmitter>("Continuous particle emitter on an entity"),
            ],
            events: vec![ContractEntry::of::<ParticleEvent>(
                "Trigger a particle burst at a position",
            )],
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
    fn test_particle_config_defaults() {
        let config = ParticleConfig::default();
        assert_eq!(config.count, 8);
        assert_eq!(config.lifetime, 0.6);
        assert!(config.shrink);
    }

    #[test]
    fn test_hit_sparks_preset() {
        let config = ParticleConfig::hit_sparks();
        assert_eq!(config.count, 6);
        assert!(config.color_end.is_some());
        assert!(config.gravity < 0.0); // falls down
    }

    #[test]
    fn test_death_burst_preset() {
        let config = ParticleConfig::death_burst();
        assert_eq!(config.count, 12);
        assert!(config.lifetime > 0.5);
    }

    #[test]
    fn test_heal_swirl_preset() {
        let config = ParticleConfig::heal_swirl();
        assert!(config.gravity > 0.0); // floats up
        assert!(!config.shrink);
    }

    #[test]
    fn test_gold_sparkle_preset() {
        let config = ParticleConfig::gold_sparkle();
        assert_eq!(config.count, 10);
    }

    #[test]
    fn test_particle_lifetime_ticks() {
        let mut particle = Particle {
            lifetime: Timer::from_seconds(1.0, TimerMode::Once),
            velocity: Vec2::new(10.0, 20.0),
            gravity: -100.0,
            fade_start: 0.4,
            initial_scale: 1.0,
            shrink: true,
        };
        assert!(!particle.lifetime.is_finished());

        particle
            .lifetime
            .tick(std::time::Duration::from_secs_f32(0.5));
        assert!(!particle.lifetime.is_finished());
        assert!(particle.lifetime.fraction() > particle.fade_start);

        particle
            .lifetime
            .tick(std::time::Duration::from_secs_f32(0.6));
        assert!(particle.lifetime.is_finished());
    }

    #[test]
    fn test_emitter_construction() {
        let emitter = ParticleEmitter::new(ParticleConfig::hit_sparks(), 10.0);
        assert!(emitter.active);
        assert!((emitter.timer.duration().as_secs_f32() - 0.1).abs() < 0.01);
    }

    #[test]
    fn test_system_config_defaults() {
        let config = ParticleSystemConfig::default();
        assert!(config.enabled);
        assert!(config.combat_particles);
    }
}
