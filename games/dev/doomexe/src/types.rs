//! Game-specific types for doomexe.
//!
//! These types were moved from engine/src/types.rs to decouple
//! game-specific concepts from the core engine.

use bevy::prelude::*;
use bevy::window::WindowResolution;

/// DoomExe-specific app window configuration.
#[derive(Debug, Clone)]
pub struct DoomExeAppConfig {
    pub window_title: &'static str,
    pub window_width: u32,
    pub window_height: u32,
    pub scale_factor_override: f32,
}

impl Default for DoomExeAppConfig {
    fn default() -> Self {
        Self {
            window_title: "DoomExe",
            window_width: 800,
            window_height: 600,
            scale_factor_override: 1.0,
        }
    }
}

impl DoomExeAppConfig {
    pub fn primary_window(&self) -> Window {
        Window {
            title: self.window_title.into(),
            resolution: WindowResolution::new(self.window_width, self.window_height)
                .with_scale_factor_override(self.scale_factor_override),
            position: WindowPosition::Centered(MonitorSelection::Primary),
            present_mode: bevy::window::PresentMode::AutoVsync,
            ..default()
        }
    }
}

/// The main hamster character component with state tracking.
#[derive(Component, Resource, Default, Clone)]
pub struct HamsterNarrator {
    /// Corruption level (0.0–100.0)
    pub corruption: f32,
    /// Current facial expression
    pub expression: Expression,
    /// Animation time accumulator
    pub _animation_time: f32,
    /// Current mood state
    pub _mood: Mood,
}

impl HamsterNarrator {
    /// Creates a new hamster narrator with default state.
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            _animation_time: 0.0,
            _mood: Mood::Neutral,
            ..Default::default()
        }
    }

    /// Sets corruption, clamping to valid range.
    pub fn set_corruption(&mut self, value: f32) {
        self.corruption = value.clamp(0.0, 100.0);
    }

    /// Gets corruption as normalized value (0.0–1.0).
    pub fn corruption_normalized(&self) -> f32 {
        self.corruption / 100.0
    }
}

/// Facial expression variants for the hamster.
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum Expression {
    #[default]
    Neutral,
    Happy,
    Angry,
    Sad,
    Corrupted,
    Confused,
    Amused,
}

impl Expression {
    /// Converts a string to an Expression, returning None if invalid.
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "neutral" => Some(Self::Neutral),
            "happy" => Some(Self::Happy),
            "angry" => Some(Self::Angry),
            "sad" => Some(Self::Sad),
            "corrupted" => Some(Self::Corrupted),
            "confused" => Some(Self::Confused),
            "amused" => Some(Self::Amused),
            _ => None,
        }
    }

    /// Returns the expression name as a string.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Neutral => "neutral",
            Self::Happy => "happy",
            Self::Angry => "angry",
            Self::Sad => "sad",
            Self::Corrupted => "corrupted",
            Self::Confused => "confused",
            Self::Amused => "amused",
        }
    }
}

/// Mood state for the hamster, affects animation intensity.
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
#[allow(dead_code)]
pub enum Mood {
    #[default]
    Normal,
    Excited,
    Melancholy,
    Neutral,
}

/// Represents a hamster sprite part (child entity).
#[derive(Component, Clone)]
#[allow(dead_code)]
pub struct HamsterPart {
    /// Part type identifier (e.g., "body", "head", "eye_left")
    pub part_type: String,
    /// Offset from parent
    pub offset: Vec2,
    /// Z-order layer (0 = back, higher = front)
    pub layer: u32,
}

/// Shader uniform data for corruption effects.
#[derive(Clone, Copy, Default)]
#[allow(dead_code)]
pub struct CorruptionUniforms {
    /// Corruption level (0.0–1.0, normalized)
    pub corruption: f32,
    /// Time for animated effects
    pub time: f32,
    /// Which palette variant to use (0, 1, 2, 3)
    pub palette_shift: i32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_doomexe_app_config_defaults() {
        let config = DoomExeAppConfig::default();

        assert_eq!(config.window_title, "DoomExe");
        assert_eq!(config.window_width, 800);
        assert_eq!(config.window_height, 600);
        assert!((config.scale_factor_override - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_save_load_roundtrip() {
        use dj_engine::save::{SaveData, SaveScope};
        use std::collections::HashMap;

        let scope = SaveScope::Project("doomexe_test".into());
        let mut flags = HashMap::new();
        flags.insert("MetHamster".to_string(), true);
        flags.insert("DefeatedGlitch".to_string(), true);

        let data = SaveData {
            flags,
            variables: HashMap::new(),
            current_node: Some(5),
            game_state: "Overworld".into(),
            scene_background: None,
            project_id: Some("doomexe".into()),
            scene_id: None,
            story_graph_id: None,
        };

        // Save
        let path = dj_engine::save::save_game_scoped(&scope, 99, &data).unwrap();
        assert!(path.exists());

        // Load
        let loaded = dj_engine::save::load_game_scoped(&scope, 99).unwrap();
        assert_eq!(loaded.flags.len(), 2);
        assert_eq!(loaded.flags.get("MetHamster"), Some(&true));
        assert_eq!(loaded.game_state, "Overworld");
        assert_eq!(loaded.current_node, Some(5));

        // Cleanup
        dj_engine::save::delete_save_scoped(&scope, 99).unwrap();
        assert!(!dj_engine::save::has_save_scoped(&scope, 99));
    }
}
