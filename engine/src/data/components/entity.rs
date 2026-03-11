use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::audio::AudioSourceComponent;
use super::collision::CollisionComponent;
use super::gameplay::{
    CombatStatsComponent, EnemyComponent, NpcComponent, SpawnerComponent, TowerComponent,
};
use super::interaction::InteractivityComponent;
use super::rendering::{CameraAnchorComponent, SpriteComponent, TransformComponent};

/// Container for all possible entity components.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, Reflect)]
pub struct EntityComponents {
    /// Transform (always present)
    pub transform: TransformComponent,
    /// Sprite/visual appearance
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sprite: Option<SpriteComponent>,
    /// Collision/physics
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub collision: Option<CollisionComponent>,
    /// Interactivity (triggers, events)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub interactivity: Option<InteractivityComponent>,
    /// NPC data
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub npc: Option<NpcComponent>,
    /// Enemy data
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enemy: Option<EnemyComponent>,
    /// Combat stats
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub combat_stats: Option<CombatStatsComponent>,
    /// Tower data (TD)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tower: Option<TowerComponent>,
    /// Spawner data
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spawner: Option<SpawnerComponent>,
    /// Audio source
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audio_source: Option<AudioSourceComponent>,
    /// Camera anchor
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub camera_anchor: Option<CameraAnchorComponent>,
    /// Custom/extension properties
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    #[reflect(ignore)]
    pub custom: HashMap<String, serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::components::common::Vec3Data;
    use serde_json::json;

    #[test]
    fn test_transform_serialization() {
        let transform = TransformComponent {
            position: Vec3Data::xy(100.0, 200.0),
            rotation: Vec3Data::default(),
            scale: Vec3Data::new(1.0, 1.0, 1.0),
            lock_uniform_scale: false,
        };
        let json = serde_json::to_string(&transform).unwrap();
        let parsed: TransformComponent = serde_json::from_str(&json).unwrap();
        assert_eq!(transform, parsed);
    }

    #[test]
    fn test_entity_components_optional_fields() {
        let components = EntityComponents {
            transform: TransformComponent::default(),
            sprite: Some(SpriteComponent {
                sprite_id: "hero.png".to_string(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let json = serde_json::to_string_pretty(&components).unwrap();
        assert!(json.contains("hero.png"));
        assert!(!json.contains("collision"));
    }

    #[test]
    fn test_entity_components_json_shape_stable() {
        let components = EntityComponents {
            transform: TransformComponent::default(),
            sprite: Some(SpriteComponent {
                sprite_id: "hero.png".to_string(),
                ..Default::default()
            }),
            custom: HashMap::from([("tag".to_string(), json!("hero"))]),
            ..Default::default()
        };

        let json = serde_json::to_value(&components).unwrap();
        assert_eq!(json["transform"]["position"]["x"], json!(0.0));
        assert_eq!(json["sprite"]["sprite_id"], json!("hero.png"));
        assert_eq!(json["custom"]["tag"], json!("hero"));
        assert!(json.get("collision").is_none());
    }
}
