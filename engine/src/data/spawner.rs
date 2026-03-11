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
        Name::new(entity.name.clone()),
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

    if let Some(collision) = &components.collision {
        entity_commands.insert(collision.clone());
    }

    if let Some(audio_source) = &components.audio_source {
        entity_commands.insert(audio_source.clone());
    }

    if let Some(interactivity) = &components.interactivity {
        entity_commands.insert(interactivity.clone());
    }

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
    use crate::collision::RuntimeCollider;
    use crate::data::components::{
        AudioSourceComponent, CollisionComponent, InteractivityComponent, TriggerType, Vec3Data,
    };
    use crate::data::scene::Entity as SceneEntityData;
    use bevy::asset::AssetPlugin;

    #[test]
    fn test_vec3_conversion() {
        let data = Vec3Data::new(1.0, 2.0, 3.0);
        let vec3: Vec3 = data.into();
        assert_eq!(vec3, Vec3::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn test_spawn_entity_inserts_authored_runtime_components() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(AssetPlugin::default());
        app.add_plugins(crate::collision::CollisionPlugin);
        app.add_plugins(SceneDataPlugin);

        let mut scene = Scene::new("scene", "Scene");
        let mut entity = SceneEntityData::new("door", "Door");
        entity.entity_type = EntityType::Trigger;
        entity.components.collision = Some(CollisionComponent {
            is_trigger: true,
            ..Default::default()
        });
        entity.components.audio_source = Some(AudioSourceComponent {
            clip_id: "sfx/door.ogg".into(),
            ..Default::default()
        });
        entity.components.interactivity = Some(InteractivityComponent {
            trigger_type: TriggerType::Door,
            trigger_id: "door_1".into(),
            ..Default::default()
        });
        scene.entities.push(entity);

        app.world_mut().insert_resource(LoadedScene::new(scene));
        app.update();
        app.update();

        let world = app.world_mut();
        let mut spawned = world.query::<(
            &CollisionComponent,
            &RuntimeCollider,
            &AudioSourceComponent,
            &InteractivityComponent,
            &Name,
        )>();
        let (collision, _runtime, audio, interaction, name) = spawned.iter(world).next().unwrap();
        assert!(collision.is_trigger);
        assert_eq!(audio.clip_id, "sfx/door.ogg");
        assert_eq!(interaction.trigger_id, "door_1");
        assert_eq!(name.as_str(), "Door");
    }
}
