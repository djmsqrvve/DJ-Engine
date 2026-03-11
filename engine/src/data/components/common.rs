use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// 3D vector (used for positions, rotations, scales).
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize, Reflect)]
pub struct Vec3Data {
    pub x: f32,
    pub y: f32,
    #[serde(default)]
    pub z: f32,
}

impl Vec3Data {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    pub fn xy(x: f32, y: f32) -> Self {
        Self { x, y, z: 0.0 }
    }
}

/// RGBA color with float components.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Reflect)]
pub struct ColorData {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    #[serde(default = "default_alpha")]
    pub a: f32,
}

fn default_alpha() -> f32 {
    1.0
}

impl Default for ColorData {
    fn default() -> Self {
        Self {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        }
    }
}

impl ColorData {
    pub fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 1.0 }
    }

    pub fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub fn white() -> Self {
        Self::rgb(1.0, 1.0, 1.0)
    }

    pub fn black() -> Self {
        Self::rgb(0.0, 0.0, 0.0)
    }
}

pub(super) fn register_types(app: &mut App) {
    app.register_type::<Vec3Data>().register_type::<ColorData>();
}
