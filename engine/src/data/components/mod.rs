//! Component data structures for scene entities.
//!
//! These are serializable data structures that describe entity components.
//! They map to Bevy ECS components at runtime via the spawner system.

pub mod audio;
pub mod collision;
pub mod common;
pub mod entity;
pub mod gameplay;
pub mod interaction;
pub mod rendering;

pub use audio::AudioSourceComponent;
pub use collision::{BodyType, CollisionComponent, CollisionShape};
pub use common::{ColorData, Vec3Data};
pub use entity::EntityComponents;
pub use gameplay::{
    CombatStatsComponent, EnemyComponent, LocalizedString, NpcComponent, SpawnerComponent,
    SpawnerWave, TargetingMode, TowerComponent,
};
pub use interaction::{InteractivityComponent, InteractivityEvents, TriggerType};
pub use rendering::{
    AnimationData, CameraAnchorComponent, CameraBounds, SpriteComponent, TransformComponent,
};

use bevy::prelude::*;

pub(crate) fn register_types(app: &mut App) {
    common::register_types(app);
    rendering::register_types(app);
    collision::register_types(app);
    interaction::register_types(app);
    gameplay::register_types(app);
    audio::register_types(app);
}
