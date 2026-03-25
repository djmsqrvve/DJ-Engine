//! Animation systems for DJ Engine.
//!
//! Provides systems for procedural breathing, blinking, and idle motion.

use bevy::prelude::*;
use std::f32::consts::PI;

use super::components::{BlinkingAnimation, BreathingAnimation, IdleMotion, SpriteAnimationPlayer};

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

/// System that advances sprite frame animations.
///
/// Updates the `Sprite.texture_atlas` index based on elapsed time and frame duration.
pub fn tick_sprite_animations(
    time: Res<Time>,
    mut query: Query<(&mut SpriteAnimationPlayer, &mut Sprite)>,
) {
    let dt = time.delta_secs();
    for (mut player, mut sprite) in query.iter_mut() {
        if !player.playing || player.frame_count == 0 {
            continue;
        }

        player.elapsed += dt * player.speed;

        while player.elapsed >= player.frame_duration {
            player.elapsed -= player.frame_duration;

            if player.current_frame + 1 < player.frame_count {
                player.current_frame += 1;
            } else if player.looping {
                player.current_frame = 0;
            } else {
                player.playing = false;
                break;
            }
        }

        if let Some(atlas) = sprite.texture_atlas.as_mut() {
            atlas.index = player.current_frame;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Simulate the tick logic from tick_sprite_animations for a given delta.
    fn simulate_tick(player: &mut SpriteAnimationPlayer, dt: f32) {
        if !player.playing || player.frame_count == 0 {
            return;
        }
        player.elapsed += dt * player.speed;
        while player.elapsed >= player.frame_duration {
            player.elapsed -= player.frame_duration;
            if player.current_frame + 1 < player.frame_count {
                player.current_frame += 1;
            } else if player.looping {
                player.current_frame = 0;
            } else {
                player.playing = false;
                break;
            }
        }
    }

    #[test]
    fn test_sprite_animation_advances_frames() {
        let mut player = SpriteAnimationPlayer::new(4, 1.0, true);
        assert_eq!(player.current_frame, 0);
        assert_eq!(player.frame_duration, 0.25);

        simulate_tick(&mut player, 0.3);
        assert_eq!(player.current_frame, 1);
    }

    #[test]
    fn test_sprite_animation_loops() {
        let mut player = SpriteAnimationPlayer::new(3, 0.3, true);
        // Each frame = 0.1s. Advance 0.35s = 3 frames + partial → loops to frame 0
        simulate_tick(&mut player, 0.35);
        assert_eq!(player.current_frame, 0);
    }

    #[test]
    fn test_sprite_animation_one_shot_stops() {
        let mut player = SpriteAnimationPlayer::new(3, 0.3, false);
        // Advance well past the end
        simulate_tick(&mut player, 1.0);
        assert_eq!(player.current_frame, 2);
        assert!(!player.playing);
        assert!(player.is_finished());
    }

    #[test]
    fn test_sprite_animation_restart() {
        let mut player = SpriteAnimationPlayer::new(4, 1.0, false);
        player.current_frame = 3;
        player.playing = false;
        player.elapsed = 0.5;

        player.restart();
        assert_eq!(player.current_frame, 0);
        assert!(player.playing);
        assert_eq!(player.elapsed, 0.0);
    }

    #[test]
    fn test_sprite_animation_speed_multiplier() {
        let mut player = SpriteAnimationPlayer::new(4, 1.0, true);
        player.speed = 2.0;
        // 0.15s real time at 2x = 0.3s animation → advances 1 frame (0.25s per frame)
        simulate_tick(&mut player, 0.15);
        assert_eq!(player.current_frame, 1);
    }
}
