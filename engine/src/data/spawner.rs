//! Bevy systems for spawning entities from scene data.
//!
//! These systems convert the serializable data types into actual Bevy ECS
//! entities with components.

use bevy::prelude::*;

use super::components::{SpawnerComponent, Vec3Data};
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

/// Runtime state for authored spawner entities.
#[derive(Component, Debug, Clone, PartialEq, Reflect)]
#[reflect(Component)]
pub struct SpawnerRuntimeState {
    pub current_wave_index: Option<usize>,
    pub remaining_in_wave: u32,
    pub time_until_start: f32,
    pub time_until_next_spawn: f32,
    pub completed: bool,
    pub loop_waves: bool,
    pub path_id: Option<String>,
}

impl SpawnerRuntimeState {
    pub fn from_component(spawner: &SpawnerComponent) -> Self {
        if let Some(first_wave) = spawner.waves.first() {
            Self {
                current_wave_index: Some(0),
                remaining_in_wave: first_wave.count,
                time_until_start: spawner.start_delay.max(0.0),
                time_until_next_spawn: spawner.spawn_interval.max(0.0),
                completed: false,
                loop_waves: spawner.loop_waves,
                path_id: spawner.path_id.clone(),
            }
        } else {
            Self {
                current_wave_index: None,
                remaining_in_wave: 0,
                time_until_start: 0.0,
                time_until_next_spawn: 0.0,
                completed: !spawner.loop_waves,
                loop_waves: spawner.loop_waves,
                path_id: spawner.path_id.clone(),
            }
        }
    }
}

/// Event fired when a spawner should create an enemy entity.
#[derive(Message, Debug, Clone, PartialEq)]
pub struct SpawnWaveEvent {
    /// The spawner entity that triggered this.
    pub spawner: Entity,
    /// The enemy template ID to spawn.
    pub enemy_template_id: String,
    /// Position to spawn at (spawner's position).
    pub position: Vec3,
    /// Path ID for the spawned enemy to follow.
    pub path_id: Option<String>,
}

/// System that ticks spawner timers and emits [`SpawnWaveEvent`] when ready.
pub fn tick_spawners(
    time: Res<Time>,
    mut query: Query<(
        Entity,
        &Transform,
        &SpawnerComponent,
        &mut SpawnerRuntimeState,
    )>,
    mut events: MessageWriter<SpawnWaveEvent>,
) {
    let dt = time.delta_secs();

    for (entity, transform, component, mut state) in query.iter_mut() {
        if state.completed {
            continue;
        }

        // Wait for start delay
        if state.time_until_start > 0.0 {
            state.time_until_start -= dt;
            continue;
        }

        let Some(wave_index) = state.current_wave_index else {
            continue;
        };

        let Some(wave) = component.waves.get(wave_index) else {
            state.completed = true;
            continue;
        };

        // Tick spawn timer
        state.time_until_next_spawn -= dt;
        if state.time_until_next_spawn > 0.0 {
            continue;
        }

        // Spawn an enemy
        events.write(SpawnWaveEvent {
            spawner: entity,
            enemy_template_id: wave.enemy_template_id.clone(),
            position: transform.translation,
            path_id: state.path_id.clone(),
        });

        state.remaining_in_wave = state.remaining_in_wave.saturating_sub(1);
        state.time_until_next_spawn = wave.interval;

        // Check if wave is done
        if state.remaining_in_wave == 0 {
            let next_index = wave_index + 1;
            if next_index < component.waves.len() {
                // Advance to next wave
                state.current_wave_index = Some(next_index);
                state.remaining_in_wave = component.waves[next_index].count;
                state.time_until_next_spawn = component.spawn_interval;
            } else if state.loop_waves {
                // Loop back to first wave
                state.current_wave_index = Some(0);
                state.remaining_in_wave = component.waves[0].count;
                state.time_until_next_spawn = component.spawn_interval;
            } else {
                state.completed = true;
            }
        }
    }
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
                entity_commands.insert((
                    SpawnerMarker {
                        spawner_id: entity.id.clone(),
                    },
                    SpawnerRuntimeState::from_component(spawner),
                ));
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
            .register_type::<SpawnerRuntimeState>()
            .add_message::<SpawnWaveEvent>()
            .add_systems(Update, (spawn_scene_entities, tick_spawners));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collision::RuntimeCollider;
    use crate::data::components::{
        AudioSourceComponent, CollisionComponent, InteractivityComponent, SpawnerWave, TriggerType,
        Vec3Data,
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

    #[test]
    fn test_spawner_runtime_state_seeds_single_wave_with_start_delay() {
        let component = SpawnerComponent {
            wave_count: 1,
            spawn_interval: 1.5,
            start_delay: 2.5,
            loop_waves: false,
            waves: vec![SpawnerWave {
                enemy_template_id: "slime".into(),
                count: 3,
                interval: 0.75,
            }],
            path_id: Some("path_alpha".into()),
        };

        let runtime = SpawnerRuntimeState::from_component(&component);

        assert_eq!(runtime.current_wave_index, Some(0));
        assert_eq!(runtime.remaining_in_wave, 3);
        assert!((runtime.time_until_start - 2.5).abs() < f32::EPSILON);
        assert!((runtime.time_until_next_spawn - 1.5).abs() < f32::EPSILON);
        assert!(!runtime.completed);
        assert!(!runtime.loop_waves);
        assert_eq!(runtime.path_id.as_deref(), Some("path_alpha"));
    }

    #[test]
    fn test_spawner_runtime_state_uses_first_wave_only() {
        let component = SpawnerComponent {
            wave_count: 2,
            spawn_interval: 3.0,
            start_delay: 0.0,
            loop_waves: true,
            waves: vec![
                SpawnerWave {
                    enemy_template_id: "slime".into(),
                    count: 2,
                    interval: 0.25,
                },
                SpawnerWave {
                    enemy_template_id: "ogre".into(),
                    count: 9,
                    interval: 4.0,
                },
            ],
            path_id: None,
        };

        let runtime = SpawnerRuntimeState::from_component(&component);

        assert_eq!(runtime.current_wave_index, Some(0));
        assert_eq!(runtime.remaining_in_wave, 2);
        assert!((runtime.time_until_next_spawn - 3.0).abs() < f32::EPSILON);
        assert!(runtime.loop_waves);
        assert!(!runtime.completed);
    }

    #[test]
    fn test_spawner_runtime_state_marks_empty_non_looping_spawner_complete() {
        let component = SpawnerComponent {
            wave_count: 0,
            spawn_interval: 5.0,
            start_delay: 1.0,
            loop_waves: false,
            waves: Vec::new(),
            path_id: Some("empty_path".into()),
        };

        let runtime = SpawnerRuntimeState::from_component(&component);

        assert_eq!(runtime.current_wave_index, None);
        assert_eq!(runtime.remaining_in_wave, 0);
        assert_eq!(runtime.time_until_start, 0.0);
        assert_eq!(runtime.time_until_next_spawn, 0.0);
        assert!(runtime.completed);
        assert_eq!(runtime.path_id.as_deref(), Some("empty_path"));
    }

    #[test]
    fn test_spawner_runtime_state_keeps_empty_looping_spawner_idle() {
        let component = SpawnerComponent {
            wave_count: 0,
            spawn_interval: 5.0,
            start_delay: 1.0,
            loop_waves: true,
            waves: Vec::new(),
            path_id: None,
        };

        let runtime = SpawnerRuntimeState::from_component(&component);

        assert_eq!(runtime.current_wave_index, None);
        assert_eq!(runtime.remaining_in_wave, 0);
        assert_eq!(runtime.time_until_start, 0.0);
        assert_eq!(runtime.time_until_next_spawn, 0.0);
        assert!(!runtime.completed);
        assert!(runtime.loop_waves);
    }

    #[test]
    fn test_spawn_entity_initializes_spawner_runtime_state() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(AssetPlugin::default());
        app.add_plugins(SceneDataPlugin);

        let mut scene = Scene::new("scene", "Scene");
        let mut entity = SceneEntityData::new("wave_machine", "Wave Machine");
        entity.entity_type = EntityType::Spawner;
        entity.components.spawner = Some(SpawnerComponent {
            wave_count: 2,
            spawn_interval: 1.25,
            start_delay: 0.5,
            loop_waves: true,
            waves: vec![
                SpawnerWave {
                    enemy_template_id: "slime".into(),
                    count: 4,
                    interval: 0.5,
                },
                SpawnerWave {
                    enemy_template_id: "bat".into(),
                    count: 7,
                    interval: 1.0,
                },
            ],
            path_id: Some("route_01".into()),
        });
        scene.entities.push(entity);

        app.world_mut().insert_resource(LoadedScene::new(scene));
        app.update();

        let world = app.world_mut();
        let mut query = world.query::<(&SpawnerMarker, &SpawnerRuntimeState, &Name)>();
        let (marker, runtime, name) = query.iter(world).next().unwrap();
        assert_eq!(marker.spawner_id, "wave_machine");
        assert_eq!(name.as_str(), "Wave Machine");
        assert_eq!(runtime.current_wave_index, Some(0));
        assert_eq!(runtime.remaining_in_wave, 4);
        assert!((runtime.time_until_start - 0.5).abs() < f32::EPSILON);
        assert!((runtime.time_until_next_spawn - 1.25).abs() < f32::EPSILON);
        assert!(runtime.loop_waves);
        assert_eq!(runtime.path_id.as_deref(), Some("route_01"));
    }

    #[test]
    fn test_tick_spawner_emits_events() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<SpawnWaveEvent>();
        app.add_systems(Update, tick_spawners);

        let component = SpawnerComponent {
            wave_count: 1,
            spawn_interval: 0.0,
            start_delay: 0.0,
            loop_waves: false,
            waves: vec![SpawnerWave {
                enemy_template_id: "goblin".into(),
                count: 2,
                interval: 0.0,
            }],
            path_id: None,
        };
        let state = SpawnerRuntimeState::from_component(&component);

        app.world_mut()
            .spawn((Transform::from_xyz(100.0, 200.0, 0.0), component, state));

        // First update spawns first enemy
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(std::time::Duration::from_millis(16));
        app.update();

        // Second update spawns second and completes
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(std::time::Duration::from_millis(16));
        app.update();

        // Verify spawner completed
        let mut query = app.world_mut().query::<&SpawnerRuntimeState>();
        let state = query.single(app.world()).unwrap();
        assert!(state.completed);
    }

    #[test]
    fn test_tick_spawner_respects_start_delay() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<SpawnWaveEvent>();
        app.add_systems(Update, tick_spawners);

        let component = SpawnerComponent {
            wave_count: 1,
            spawn_interval: 0.0,
            start_delay: 1.0, // 1 second delay
            loop_waves: false,
            waves: vec![SpawnerWave {
                enemy_template_id: "wolf".into(),
                count: 1,
                interval: 0.0,
            }],
            path_id: None,
        };
        let state = SpawnerRuntimeState::from_component(&component);

        app.world_mut()
            .spawn((Transform::from_xyz(0.0, 0.0, 0.0), component, state));

        // Tick 0.5s — should still be waiting
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(std::time::Duration::from_millis(500));
        app.update();

        let mut query = app.world_mut().query::<&SpawnerRuntimeState>();
        let state = query.single(app.world()).unwrap();
        assert!(!state.completed);
        assert!(state.time_until_start > 0.0);
    }

    #[test]
    fn test_tick_spawner_loops_waves() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<SpawnWaveEvent>();
        app.add_systems(Update, tick_spawners);

        let component = SpawnerComponent {
            wave_count: 1,
            spawn_interval: 0.0,
            start_delay: 0.0,
            loop_waves: true,
            waves: vec![SpawnerWave {
                enemy_template_id: "bat".into(),
                count: 1,
                interval: 0.0,
            }],
            path_id: None,
        };
        let state = SpawnerRuntimeState::from_component(&component);

        app.world_mut()
            .spawn((Transform::from_xyz(0.0, 0.0, 0.0), component, state));

        // Run several ticks
        for _ in 0..5 {
            app.world_mut()
                .resource_mut::<Time>()
                .advance_by(std::time::Duration::from_millis(16));
            app.update();
        }

        // Should NOT be completed (loops forever)
        let mut query = app.world_mut().query::<&SpawnerRuntimeState>();
        let state = query.single(app.world()).unwrap();
        assert!(!state.completed);
    }
}
