//! Animation components for DJ Engine.
//!
//! Provides components for breathing, blinking, and idle motion animations.

use bevy::prelude::*;

/// Component for breathing animation (body scale).
#[derive(Component, Default)]
pub struct BreathingAnimation {
    /// Maximum scale deviation (0.0 = no breathing)
    pub amplitude: f32,
    /// Cycles per second (Hz)
    pub frequency: f32,
    /// Current phase offset (radians)
    pub phase: f32,
}

impl BreathingAnimation {
    /// Creates a new breathing animation with the given amplitude and frequency.
    pub fn new(amplitude: f32, frequency: f32) -> Self {
        Self {
            amplitude,
            frequency,
            phase: 0.0,
        }
    }

    /// Default breathing animation for hamster body.
    pub fn hamster_default() -> Self {
        Self::new(0.05, 1.5) // 5% scale change at 1.5 Hz
    }
}

/// Component for blinking animation (eye sprite toggle).
#[derive(Component, Default)]
pub struct BlinkingAnimation {
    /// Duration of a blink in seconds
    pub blink_duration: f32,
    /// Minimum time between blinks
    pub interval_min: f32,
    /// Maximum time between blinks
    pub interval_max: f32,
    /// Time until next blink
    pub timer: f32,
    /// Currently blinking
    pub is_blinking: bool,
}

impl BlinkingAnimation {
    /// Creates a new blinking animation with the given parameters.
    pub fn new(blink_duration: f32, interval_min: f32, interval_max: f32) -> Self {
        Self {
            blink_duration,
            interval_min,
            interval_max,
            timer: interval_min,
            is_blinking: false,
        }
    }

    /// Default blinking for hamster.
    pub fn hamster_default() -> Self {
        Self::new(0.15, 2.0, 5.0)
    }
}

/// Component for idle motion (noise-based jitter).
#[derive(Component, Default)]
pub struct IdleMotion {
    /// Scale of noise effect
    pub noise_scale: f32,
    /// Speed of motion
    pub speed: f32,
    /// Internal time accumulator
    pub time: f32,
}

impl IdleMotion {
    /// Creates a new idle motion component.
    pub fn new(noise_scale: f32, speed: f32) -> Self {
        Self {
            noise_scale,
            speed,
            time: 0.0,
        }
    }

    /// Default idle motion for hamster head.
    pub fn hamster_default() -> Self {
        Self::new(2.0, 0.5)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_breathing_new() {
        let b = BreathingAnimation::new(0.05, 1.5);
        assert_eq!(b.amplitude, 0.05);
        assert_eq!(b.frequency, 1.5);
        assert_eq!(b.phase, 0.0);
    }

    #[test]
    fn test_breathing_hamster_default() {
        let b = BreathingAnimation::hamster_default();
        assert_eq!(b.amplitude, 0.05);
        assert_eq!(b.frequency, 1.5);
    }

    #[test]
    fn test_blinking_new() {
        let b = BlinkingAnimation::new(0.15, 2.0, 5.0);
        assert_eq!(b.blink_duration, 0.15);
        assert_eq!(b.interval_min, 2.0);
        assert_eq!(b.interval_max, 5.0);
        assert_eq!(b.timer, 2.0);
        assert!(!b.is_blinking);
    }

    #[test]
    fn test_blinking_hamster_default() {
        let b = BlinkingAnimation::hamster_default();
        assert_eq!(b.blink_duration, 0.15);
        assert_eq!(b.interval_min, 2.0);
        assert_eq!(b.interval_max, 5.0);
    }

    #[test]
    fn test_idle_motion_new() {
        let m = IdleMotion::new(2.0, 0.5);
        assert_eq!(m.noise_scale, 2.0);
        assert_eq!(m.speed, 0.5);
        assert_eq!(m.time, 0.0);
    }

    #[test]
    fn test_idle_motion_hamster_default() {
        let m = IdleMotion::hamster_default();
        assert_eq!(m.noise_scale, 2.0);
        assert_eq!(m.speed, 0.5);
    }
}
