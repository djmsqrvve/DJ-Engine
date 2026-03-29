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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vec3data_new() {
        let v = Vec3Data::new(1.0, 2.0, 3.0);
        assert_eq!(v.x, 1.0);
        assert_eq!(v.y, 2.0);
        assert_eq!(v.z, 3.0);
    }

    #[test]
    fn test_vec3data_xy() {
        let v = Vec3Data::xy(5.0, 10.0);
        assert_eq!(v.x, 5.0);
        assert_eq!(v.y, 10.0);
        assert_eq!(v.z, 0.0);
    }

    #[test]
    fn test_vec3data_default_is_zero() {
        let v = Vec3Data::default();
        assert_eq!(v.x, 0.0);
        assert_eq!(v.y, 0.0);
        assert_eq!(v.z, 0.0);
    }

    #[test]
    fn test_vec3data_serde_roundtrip() {
        let v = Vec3Data::new(1.5, -2.5, 3.5);
        let json = serde_json::to_string(&v).unwrap();
        let v2: Vec3Data = serde_json::from_str(&json).unwrap();
        assert_eq!(v, v2);
    }

    #[test]
    fn test_colordata_default_is_white() {
        let c = ColorData::default();
        assert_eq!(c.r, 1.0);
        assert_eq!(c.g, 1.0);
        assert_eq!(c.b, 1.0);
        assert_eq!(c.a, 1.0);
    }

    #[test]
    fn test_colordata_rgb_sets_alpha_1() {
        let c = ColorData::rgb(0.5, 0.3, 0.1);
        assert_eq!(c.a, 1.0);
    }

    #[test]
    fn test_colordata_rgba() {
        let c = ColorData::rgba(0.1, 0.2, 0.3, 0.4);
        assert_eq!(c.r, 0.1);
        assert_eq!(c.a, 0.4);
    }

    #[test]
    fn test_colordata_black() {
        let c = ColorData::black();
        assert_eq!(c.r, 0.0);
        assert_eq!(c.g, 0.0);
        assert_eq!(c.b, 0.0);
        assert_eq!(c.a, 1.0);
    }

    #[test]
    fn test_colordata_white() {
        let c = ColorData::white();
        assert_eq!(c, ColorData::default());
    }

    #[test]
    fn test_colordata_serde_roundtrip() {
        let c = ColorData::rgba(0.1, 0.2, 0.3, 0.5);
        let json = serde_json::to_string(&c).unwrap();
        let c2: ColorData = serde_json::from_str(&json).unwrap();
        assert_eq!(c, c2);
    }
}
