use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Audio source component data.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Component, Reflect)]
#[reflect(Component)]
pub struct AudioSourceComponent {
    /// Audio clip asset ID
    pub clip_id: String,
    /// Volume (0.0 - 1.0)
    #[serde(default = "default_volume")]
    pub volume: f32,
    /// Whether to loop
    #[serde(default)]
    pub loop_audio: bool,
    /// Whether to use spatial (3D) audio
    #[serde(default)]
    pub spatial: bool,
}

fn default_volume() -> f32 {
    1.0
}

impl Default for AudioSourceComponent {
    fn default() -> Self {
        Self {
            clip_id: String::new(),
            volume: 1.0,
            loop_audio: false,
            spatial: false,
        }
    }
}

pub(super) fn register_types(app: &mut App) {
    app.register_type::<AudioSourceComponent>();
}
