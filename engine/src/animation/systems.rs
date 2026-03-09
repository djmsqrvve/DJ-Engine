//! Animation systems for DJ Engine.
//!
//! Provides systems for procedural breathing, blinking, and idle motion.

use bevy::prelude::*;
use std::f32::consts::PI;

use super::components::{BlinkingAnimation, BreathingAnimation, IdleMotion};

/// System that applies breathing animation to entities.
///
/// Uses a sine wave to smoothly scale the entity up and down.
pub fn breathing_system(time: Res<Time>, mut query: Query<(&BreathingAnimation, &mut Transform)>) {
    for (breathing, mut transform) in query.iter_mut() {
        // Calculate current scale based on sine wave
        let t = time.elapsed_secs() * breathing.frequency * 2.0 * PI + breathing.phase;
        let scale_factor = 1.0 + breathing.amplitude * t.sin();

        // Apply scale with area preservation (squash and stretch)
        // When Y expands, X contracts slightly to maintain volume feel
        let inverse_scale = 1.0 + breathing.amplitude * 0.3 * (-t).sin();

        transform.scale.x = inverse_scale;
        transform.scale.y = scale_factor;
    }
}

/// System that manages blinking animation timing.
///
/// Updates blink timer and toggles blink state.
pub fn blinking_system(time: Res<Time>, mut query: Query<&mut BlinkingAnimation>) {
    for mut blinking in query.iter_mut() {
        blinking.timer -= time.delta_secs();

        if blinking.timer <= 0.0 {
            if blinking.is_blinking {
                // End blink, set timer for next blink
                blinking.is_blinking = false;
                // Random interval between min and max (simplified for now)
                blinking.timer =
                    blinking.interval_min + (blinking.interval_max - blinking.interval_min) * 0.5;
            } else {
                // Start blink
                blinking.is_blinking = true;
                blinking.timer = blinking.blink_duration;
            }
        }
    }
}

/// System that applies idle motion jitter to entities.
///
/// Uses simplified noise-like motion based on sine waves.
pub fn idle_motion_system(time: Res<Time>, mut query: Query<(&mut IdleMotion, &mut Transform)>) {
    for (mut idle, mut transform) in query.iter_mut() {
        idle.time += time.delta_secs() * idle.speed;

        // Simplified "noise" using combination of sine waves
        let x_offset = (idle.time * 1.3).sin() * 0.5 + (idle.time * 2.7).sin() * 0.3;
        let y_offset = (idle.time * 1.7).sin() * 0.4 + (idle.time * 3.1).sin() * 0.2;

        // Apply small jitter to position
        transform.translation.x += x_offset * idle.noise_scale * time.delta_secs();
        transform.translation.y += y_offset * idle.noise_scale * time.delta_secs();
    }
}
