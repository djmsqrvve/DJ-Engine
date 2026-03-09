//! Animation systems for character sprites.

use bevy::prelude::*;
use std::f32::consts::PI;

use super::components::*;

use crate::state::GameState;

/// Toggle visibility of the hamster narrator based on game state.
pub fn toggle_visibility_system(
    state: Res<State<GameState>>,
    mut query: Query<&mut Visibility, With<CharacterRoot>>,
) {
    if state.is_changed() {
        let should_show = matches!(
            *state.get(),
            GameState::NarratorDialogue | GameState::Battle
        );
        for mut vis in &mut query {
            *vis = if should_show {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            };
            info!("Hamster visibility set to: {:?}", *vis);
        }
    }
}

/// Breathing animation - smooth scale oscillation on body.
pub fn breathing_system(time: Res<Time>, mut query: Query<(&mut Transform, &BreathingAnimation)>) {
    let t = time.elapsed_secs();
    for (mut transform, anim) in &mut query {
        let phase = t * anim.frequency * 2.0 * PI;
        let scale_y = 1.0 + anim.amplitude * phase.sin();
        let scale_x = 1.0 / scale_y; // Conserve area
        transform.scale = Vec3::new(scale_x, scale_y, 1.0);
    }
}

/// Blinking animation - toggle eye visibility.
pub fn blinking_system(time: Res<Time>, mut query: Query<(&mut BlinkingAnimation, &mut Sprite)>) {
    for (mut blink, mut sprite) in &mut query {
        blink.timer.tick(time.delta());

        if blink.timer.just_finished() {
            blink.is_closed = !blink.is_closed;

            let duration = if blink.is_closed {
                0.1
            } else {
                blink.min_interval
                    + rand::random::<f32>() * (blink.max_interval - blink.min_interval)
            };

            blink.timer = Timer::from_seconds(duration, TimerMode::Once);
        }

        sprite
            .color
            .set_alpha(if blink.is_closed { 0.0 } else { 1.0 });
    }
}

/// Idle motion - subtle head movement.
pub fn idle_motion_system(time: Res<Time>, mut query: Query<(&mut Transform, &IdleMotion)>) {
    let t = time.elapsed_secs();
    for (mut transform, motion) in &mut query {
        let phase = t * motion.frequency * 2.0 * PI;
        let offset_x = motion.amplitude.x * phase.sin();
        let offset_y = motion.amplitude.y * (phase * 1.3).cos();
        transform.translation.x = motion.base_offset.x + offset_x;
        transform.translation.y = motion.base_offset.y + offset_y;
    }
}

/// Expression switching - update head sprite when expression changes.
pub fn expression_system(
    query: Query<&CharacterRoot, Changed<CharacterRoot>>,
    mut parts: Query<(&mut Sprite, &ExpressionSprite)>,
) {
    for root in query.iter() {
        info!("Expression changed to {:?}", root.expression);
        for (mut sprite, expr_sprite) in parts.iter_mut() {
            // Only update if this part belongs to a changed root
            // (simplified - in production would check parent chain)
            let handle = expr_sprite.handle_for(root.expression);
            sprite.image = handle.clone();
        }
    }
}

/// Corruption visual effects - update all parts based on corruption level.
pub fn corruption_system(
    query: Query<&CharacterRoot, Changed<CharacterRoot>>,
    mut effects: Query<(&mut CorruptionEffect, &mut Sprite)>,
) {
    for root in query.iter() {
        for (mut effect, mut sprite) in effects.iter_mut() {
            effect.update_from_level(root.corruption);

            // Apply color tint based on corruption palette
            let tint = match effect.palette_index {
                0 => Color::WHITE,
                1 => Color::srgba(1.0, 0.9, 0.9, 1.0), // Slight red
                2 => Color::srgba(0.9, 0.7, 0.9, 1.0), // Purple tint
                3 => Color::srgba(0.7, 0.9, 1.0, 1.0), // Blue tint
                _ => Color::srgba(1.0, 0.6, 0.6, 1.0), // Strong red
            };
            sprite.color = tint;
        }
    }
}

/// Debug input for testing expressions and corruption.
pub fn debug_input_system(keys: Res<ButtonInput<KeyCode>>, mut query: Query<&mut CharacterRoot>) {
    for mut root in &mut query {
        // Number keys for expressions
        if keys.just_pressed(KeyCode::Digit1) {
            root.expression = Expression::Neutral;
            info!("Expression: Neutral");
        }
        if keys.just_pressed(KeyCode::Digit2) {
            root.expression = Expression::Happy;
            info!("Expression: Happy");
        }
        if keys.just_pressed(KeyCode::Digit3) {
            root.expression = Expression::Angry;
            info!("Expression: Angry");
        }

        // A to cycle expressions
        if keys.just_pressed(KeyCode::KeyA) {
            root.expression = root.expression.next();
            info!("Expression cycled to: {:?}", root.expression);
        }

        // U/D for corruption
        if keys.pressed(KeyCode::KeyU) {
            root.corruption = (root.corruption + 0.5).min(100.0);
            if (root.corruption as u32).is_multiple_of(10) {
                info!("Corruption: {:.0}%", root.corruption);
            }
        }
        if keys.pressed(KeyCode::KeyD) {
            root.corruption = (root.corruption - 0.5).max(0.0);
            if (root.corruption as u32).is_multiple_of(10) {
                info!("Corruption: {:.0}%", root.corruption);
            }
        }
    }
}
