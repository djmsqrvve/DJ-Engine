use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::common::Vec3Data;

/// Physics body type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, Reflect)]
#[serde(rename_all = "snake_case")]
pub enum BodyType {
    /// Does not move, affected by nothing
    #[default]
    Static,
    /// Fully simulated physics body
    Dynamic,
    /// Controlled programmatically, affects other bodies
    Kinematic,
}

/// Collision shape type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, Reflect)]
#[serde(rename_all = "snake_case")]
pub enum CollisionShape {
    #[default]
    Box,
    Circle,
    Polygon,
}

/// Collision/physics component data.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Component, Reflect)]
#[reflect(Component)]
pub struct CollisionComponent {
    /// Whether collision is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Physics body type
    #[serde(default)]
    pub body_type: BodyType,
    /// Collision shape
    #[serde(default)]
    pub shape: CollisionShape,
    /// Box dimensions (if shape is Box)
    #[serde(default)]
    pub box_size: Option<Vec3Data>,
    /// Circle radius (if shape is Circle)
    #[serde(default)]
    pub circle_radius: Option<f32>,
    /// Polygon points (if shape is Polygon)
    #[serde(default)]
    pub polygon_points: Vec<Vec3Data>,
    /// Shape offset from entity center
    #[serde(default)]
    pub offset: Vec3Data,
    /// Collision layer name
    #[serde(default)]
    pub layer: String,
    /// Collision mask (layers this collides with)
    #[serde(default)]
    pub mask: Vec<String>,
    /// Whether this is a trigger (non-solid)
    #[serde(default)]
    pub is_trigger: bool,
}

fn default_true() -> bool {
    true
}

impl Default for CollisionComponent {
    fn default() -> Self {
        Self {
            enabled: true,
            body_type: BodyType::Static,
            shape: CollisionShape::Box,
            box_size: Some(Vec3Data::new(32.0, 32.0, 0.0)),
            circle_radius: None,
            polygon_points: Vec::new(),
            offset: Vec3Data::default(),
            layer: "default".to_string(),
            mask: vec!["default".to_string()],
            is_trigger: false,
        }
    }
}

pub(super) fn register_types(app: &mut App) {
    app.register_type::<BodyType>()
        .register_type::<CollisionShape>()
        .register_type::<CollisionComponent>();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_body_type_default_is_static() {
        assert_eq!(BodyType::default(), BodyType::Static);
    }

    #[test]
    fn test_collision_shape_default_is_box() {
        assert_eq!(CollisionShape::default(), CollisionShape::Box);
    }

    #[test]
    fn test_collision_component_default() {
        let c = CollisionComponent::default();
        assert!(c.enabled);
        assert_eq!(c.body_type, BodyType::Static);
        assert_eq!(c.shape, CollisionShape::Box);
        assert!(c.box_size.is_some());
        assert!(c.circle_radius.is_none());
        assert!(c.polygon_points.is_empty());
        assert!(!c.is_trigger);
        assert_eq!(c.layer, "default");
        assert_eq!(c.mask, vec!["default"]);
    }

    #[test]
    fn test_collision_component_serde_roundtrip() {
        let c = CollisionComponent {
            body_type: BodyType::Kinematic,
            shape: CollisionShape::Circle,
            circle_radius: Some(16.0),
            is_trigger: true,
            ..Default::default()
        };
        let json = serde_json::to_string(&c).unwrap();
        let c2: CollisionComponent = serde_json::from_str(&json).unwrap();
        assert_eq!(c, c2);
    }

    #[test]
    fn test_body_type_serde() {
        let json = serde_json::to_string(&BodyType::Dynamic).unwrap();
        assert_eq!(json, "\"dynamic\"");
        let bt: BodyType = serde_json::from_str(&json).unwrap();
        assert_eq!(bt, BodyType::Dynamic);
    }
}
