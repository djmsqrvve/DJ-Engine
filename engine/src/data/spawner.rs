//! Bevy systems for spawning entities from scene data.
//!
//! These systems convert the serializable data types into actual Bevy ECS
//! entities with components.

use bevy::prelude::*;

use super::components::Vec3Data;
use super::scene::{Entity as SceneEntity, EntityType, Scene};

/// Resource holding the currently loaded scene data.
#[derive(Resource, Default)]
pub struct LoadedScene {
    /// The scene data
    pub scene: Option<Scene>,
    /// Whether the scene needs to be spawned
    pub needs_spawn: bool,
}

impl LoadedScene {
    /// Create with a scene ready to spawn.
    pub fn new(scene: Scene) -> Self {
        Self {
            scene: Some(scene),
            needs_spawn: true,
        }
    }
}

/// Marker component for entities spawned from scene data.
#[derive(Component)]
pub struct SceneEntityMarker {
    /// Original entity ID from the scene
    pub scene_entity_id: String,
    /// Entity type from the scene
    pub entity_type: EntityType,
}

/// Marker component for NPC entities.
#[derive(Component)]
pub struct NpcMarker {
    pub npc_id: String,
}

/// Marker component for enemy entities.
#[derive(Component)]
pub struct EnemyMarker {
    pub enemy_id: String,
}

/// Marker component for tower entities (TD).
#[derive(Component)]
pub struct TowerMarker {
    pub tower_id: String,
}

/// Marker component for spawner entities.
#[derive(Component)]
pub struct SpawnerMarker {
    pub spawner_id: String,
}

/// Convert Vec3Data to Bevy Vec3.
impl From<Vec3Data> for Vec3 {
    fn from(v: Vec3Data) -> Self {
        Vec3::new(v.x, v.y, v.z)
    }
}

/// System to spawn entities from the loaded scene.
///
/// This system checks if there's a scene that needs spawning and creates
/// Bevy entities for each entity in the scene.
pub fn spawn_scene_entities(
    mut commands: Commands,
    mut loaded_scene: ResMut<LoadedScene>,
    asset_server: Res<AssetServer>,
) {
    if !loaded_scene.needs_spawn {
        return;
    }

    let Some(scene) = &loaded_scene.scene else {
        return;
    };

    info!(
        "Spawning {} entities from scene '{}'",
        scene.entities.len(),
        scene.name
    );

    for entity in &scene.entities {
        spawn_entity(&mut commands, entity, &asset_server);
    }

    loaded_scene.needs_spawn = false;
}

/// Spawn a single entity from scene data.
fn spawn_entity(commands: &mut Commands, entity: &SceneEntity, asset_server: &AssetServer) {
    let components = &entity.components;
    let transform = Transform {
        translation: components.transform.position.into(),
        rotation: Quat::from_euler(
            EulerRot::XYZ,
            components.transform.rotation.x.to_radians(),
            components.transform.rotation.y.to_radians(),
            components.transform.rotation.z.to_radians(),
        ),
        scale: components.transform.scale.into(),
    };

    let mut entity_commands = commands.spawn((
        transform,
        GlobalTransform::default(),
        Visibility::default(),
        InheritedVisibility::default(),
        ViewVisibility::default(),
        SceneEntityMarker {
            scene_entity_id: entity.id.clone(),
            entity_type: entity.entity_type,
        },
    ));

    // Add sprite if present
    if let Some(sprite_data) = &components.sprite {
        if !sprite_data.sprite_id.is_empty() {
            let texture: Handle<Image> = asset_server.load(&sprite_data.sprite_id);
            entity_commands.insert(Sprite {
                image: texture,
                flip_x: sprite_data.flip_x,
                flip_y: sprite_data.flip_y,
                color: Color::srgba(
                    sprite_data.tint.r,
                    sprite_data.tint.g,
                    sprite_data.tint.b,
                    sprite_data.tint.a,
                ),
                ..default()
            });
        }
    }

    // Add type-specific markers
    match entity.entity_type {
        EntityType::Npc => {
            if let Some(npc) = &components.npc {
                entity_commands.insert(NpcMarker {
                    npc_id: npc.npc_id.clone(),
                });
            }
        }
        EntityType::Enemy => {
            if let Some(enemy) = &components.enemy {
                entity_commands.insert(EnemyMarker {
                    enemy_id: enemy.enemy_id.clone(),
                });
            }
        }
        EntityType::Tower => {
            if let Some(tower) = &components.tower {
                entity_commands.insert(TowerMarker {
                    tower_id: tower.tower_id.clone(),
                });
            }
        }
        EntityType::Spawner => {
            if let Some(spawner) = &components.spawner {
                entity_commands.insert(SpawnerMarker {
                    spawner_id: entity.id.clone(),
                });
                // TODO: Initialize spawner state
                let _ = spawner;
            }
        }
        _ => {}
    }

    // TODO: Add collision components (requires physics plugin)
    // TODO: Add audio source components
    // TODO: Add interactivity components

    debug!(
        "Spawned entity '{}' ({:?})",
        entity.name, entity.entity_type
    );
}

/// System to despawn all scene entities.
pub fn despawn_scene_entities(
    mut commands: Commands,
    query: Query<Entity, With<SceneEntityMarker>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

/// Plugin for scene data spawning.
pub struct SceneDataPlugin;

impl Plugin for SceneDataPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LoadedScene>()
            .add_systems(Update, spawn_scene_entities);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::components::Vec3Data;

    #[test]
    fn test_vec3_conversion() {
        let data = Vec3Data::new(1.0, 2.0, 3.0);
        let vec3: Vec3 = data.into();
        assert_eq!(vec3, Vec3::new(1.0, 2.0, 3.0));
    }
}
