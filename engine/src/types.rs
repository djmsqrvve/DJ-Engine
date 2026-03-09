//! Shared data types used across all DJ Engine systems.
//!
//! This module contains only engine-generic types.
//! Game-specific types should live in the game crate.

use bevy::prelude::*;

/// Error types for DJ Engine operations.
#[derive(Debug, thiserror::Error)]
pub enum DJEngineError {
    #[error("Asset loading failed: {0}")]
    AssetLoadError(String),

    #[error("Lua error: {0}")]
    LuaError(String),

    #[error("Shader compilation failed: {0}")]
    ShaderError(String),

    #[error("Animation error: {0}")]
    AnimationError(String),
}

/// Result type alias for DJ Engine operations.
pub type DJResult<T> = std::result::Result<T, DJEngineError>;

/// Configuration for engine diagnostics.
#[derive(Resource, Debug, Clone, Reflect)]
#[reflect(Resource)]
pub struct DiagnosticConfig {
    /// Whether to show the diagnostic overlay
    pub enabled: bool,
    /// Color of the diagnostic text
    pub text_color: Color,
    /// Font size of the diagnostic text
    pub font_size: f32,
    /// Update timer (to avoid updating every frame)
    pub update_timer: Timer,
}

impl Default for DiagnosticConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            text_color: Color::srgb(0.0, 1.0, 0.0), // Neon Green
            font_size: 16.0,
            update_timer: Timer::from_seconds(0.1, TimerMode::Repeating),
        }
    }
}

/// Engine configuration resource.
#[derive(Resource, Debug, Clone, Reflect)]
#[reflect(Resource)]
pub struct EngineConfig {
    /// Target internal resolution width
    pub internal_width: u32,
    /// Target internal resolution height
    pub internal_height: u32,
    /// Enable debug features
    pub debug_mode: bool,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            internal_width: 320,
            internal_height: 240,
            debug_mode: cfg!(debug_assertions),
        }
    }
}
