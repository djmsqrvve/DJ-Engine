use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::common::{ColorData, Vec3Data};

/// Animation configuration for sprites.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, Reflect)]
pub struct AnimationData {
    /// Animation clip asset ID
    pub clip_id: Option<String>,
    /// Playback speed multiplier
    #[serde(default = "default_speed")]
    pub speed: f32,
    /// Whether to loop the animation
    #[serde(default = "default_true")]
    pub loop_anim: bool,
}

fn default_speed() -> f32 {
    1.0
}

fn default_true() -> bool {
    true
}

/// Transform component data.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Component, Reflect)]
#[reflect(Component)]
pub struct TransformComponent {
    /// World position
    pub position: Vec3Data,
    /// Rotation (degrees)
    #[serde(default)]
    pub rotation: Vec3Data,
    /// Scale factor
    #[serde(default = "default_scale")]
    pub scale: Vec3Data,
    /// Lock uniform scaling
    #[serde(default)]
    pub lock_uniform_scale: bool,
}

fn default_scale() -> Vec3Data {
    Vec3Data::new(1.0, 1.0, 1.0)
}

impl Default for TransformComponent {
    fn default() -> Self {
        Self {
            position: Vec3Data::default(),
            rotation: Vec3Data::default(),
            scale: default_scale(),
            lock_uniform_scale: false,
        }
    }
}

/// Sprite/visual appearance component data.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, Component, Reflect)]
#[reflect(Component)]
pub struct SpriteComponent {
    /// Sprite asset ID
    pub sprite_id: String,
    /// Sorting layer name
    #[serde(default)]
    pub sorting_layer: String,
    /// Order within sorting layer
    #[serde(default)]
    pub sorting_order: i32,
    /// Color tint
    #[serde(default)]
    pub tint: ColorData,
    /// Flip horizontally
    #[serde(default)]
    pub flip_x: bool,
    /// Flip vertically
    #[serde(default)]
    pub flip_y: bool,
    /// Animation settings
    #[serde(default)]
    pub animation: AnimationData,
}

/// Camera bounds for anchoring.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, Reflect)]
pub struct CameraBounds {
    pub min_x: f32,
    pub max_x: f32,
    pub min_y: f32,
    pub max_y: f32,
}

/// Camera anchor component data.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, Component, Reflect)]
#[reflect(Component)]
pub struct CameraAnchorComponent {
    /// Camera movement bounds
    #[serde(default)]
    pub bounds: CameraBounds,
    /// Entity ID to follow
    #[serde(default)]
    pub follow_entity_id: Option<String>,
}

pub(super) fn register_types(app: &mut App) {
    app.register_type::<AnimationData>()
        .register_type::<TransformComponent>()
        .register_type::<SpriteComponent>()
        .register_type::<CameraBounds>()
        .register_type::<CameraAnchorComponent>();
}
