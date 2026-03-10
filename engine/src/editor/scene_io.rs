use super::types::{ActiveStoryGraph, ProjectMetadata};
use crate::data::components::{
    ColorData, EntityComponents, SpriteComponent, TransformComponent, Vec3Data,
};
use crate::data::scene::{Entity as SceneEntity, Scene};
use crate::data::{loader, project::Project};
use bevy::prelude::*;

pub(crate) fn save_project_impl(world: &mut World) {
    // Clone necessary data to avoid holding borrow on world
    let (project_name, project_path) = {
        let project_meta = world.resource::<ProjectMetadata>();
        (project_meta.name.clone(), project_meta.path.clone())
    };

    if let Some(path) = project_path {
        info!("Saving project to {:?}", path);

        // 1. Save Project Structure
        let project_data = Project::new(&project_name);
        match loader::save_project_structure(&project_data, &path) {
            Ok(_) => info!("Successfully saved project structure"),
            Err(e) => error!("Failed to save project structure: {}", e),
        }

        // 2. Save Current Scene
        let scene = world_to_scene(world);
        let scene_path = path.join("scenes/current_scene.json");
        match loader::save_scene(&scene, &scene_path) {
            Ok(_) => info!("Successfully saved scene to {:?}", scene_path),
            Err(e) => error!("Failed to save scene: {}", e),
        }

        // 3. Save Story Graph
        let graph = &world.resource::<ActiveStoryGraph>().0;
        let graph_path = path.join("story_graphs/main.json");
        match loader::save_story_graph(graph, &graph_path) {
            Ok(_) => info!("Successfully saved story graph to {:?}", graph_path),
            Err(e) => error!("Failed to save story graph: {}", e),
        }
    } else {
        warn!("Cannot save: No project path set!");
    }
}

pub(crate) fn world_to_scene(world: &mut World) -> Scene {
    let mut scene = Scene::new("current_scene", "Current Scene");

    // In a real implementation, we'd query for all entities with specific marker components.
    // For this prototype, we'll query all entities with a Name and Transform.

    let mut entities = Vec::new();
    let mut query = world.query::<(Entity, &Name, &Transform, Option<&Sprite>)>();

    // We need to collect first to avoid borrowing world inside loop if we needed mutable access,
    // though query iteration is fine. But constructing SceneEntity might need data types.
    let mut world_entities = Vec::new();
    for (_e, name, transform, sprite) in query.iter(world) {
        // Clone data out of world
        let pos = transform.translation;
        let scale = transform.scale;

        let sprite_color = sprite.map(|s| s.color.to_linear().to_f32_array());

        world_entities.push((name.to_string(), pos, scale, sprite_color));
    }

    for (name, pos, scale, sprite_color) in world_entities {
        // Skip editor-only entities (like cameras or UI, unless tagged)
        // For now, simple filter: if it has a name starting with "Editor", skip?
        // Or better, only save things we know we spawned.

        let mut components = EntityComponents {
            transform: TransformComponent {
                position: Vec3Data::new(pos.x, pos.y, pos.z),
                rotation: Vec3Data::default(),
                scale: Vec3Data::new(scale.x, scale.y, scale.z),
                lock_uniform_scale: false,
            },
            ..EntityComponents::default()
        };

        if let Some([r, g, b, a]) = sprite_color {
            components.sprite = Some(SpriteComponent {
                sprite_id: "pixel".to_string(), // Placeholder
                tint: ColorData::rgba(r, g, b, a),
                ..Default::default()
            });
        }

        let entity = SceneEntity::new(name.clone(), name) // using name as ID for prototype
            .with_components(components);

        entities.push(entity);
    }

    scene.entities = entities;
    scene
}

pub(crate) fn load_scene_into_editor(world: &mut World, scene: Scene) {
    // 1. Clear existing entities (naive approach: despawn everything with a Name)
    // Real engine would use a SceneRoot component
    let entities_to_despawn: Vec<Entity> = world
        .query_filtered::<Entity, With<Name>>()
        .iter(world)
        .collect();
    for e in entities_to_despawn {
        world.despawn(e);
    }

    // 2. Spawn new entities
    let entity_count = scene.entities.len();
    for entity_data in scene.entities {
        let transform = entity_data.components.transform;
        let pos = transform.position;
        let scale = transform.scale;

        let mut entity_cmd = world.spawn((
            Name::new(entity_data.name),
            Transform::from_xyz(pos.x, pos.y, pos.z)
                .with_scale(Vec3::new(scale.x, scale.y, scale.z)),
        ));

        if let Some(sprite) = entity_data.components.sprite {
            let c = sprite.tint;
            entity_cmd.insert(Sprite {
                color: Color::srgba(c.r, c.g, c.b, c.a),
                custom_size: Some(Vec2::new(30.0, 30.0)), // Default size for now
                ..default()
            });
        }
    }
    info!("Loaded scene with {} entities", entity_count);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_world_to_scene_captures_named_entities() {
        let mut world = World::new();
        world.spawn((Name::new("alpha"), Transform::from_xyz(1.0, 2.0, 3.0)));
        world.spawn((Name::new("beta"), Transform::from_xyz(4.0, 5.0, 6.0)));

        let scene = world_to_scene(&mut world);

        assert_eq!(scene.entities.len(), 2);
        let names: Vec<&str> = scene.entities.iter().map(|e| e.name.as_str()).collect();
        assert!(names.contains(&"alpha"));
        assert!(names.contains(&"beta"));

        let alpha = scene.entities.iter().find(|e| e.name == "alpha").unwrap();
        let pos = &alpha.components.transform.position;
        assert!((pos.x - 1.0).abs() < f32::EPSILON);
        assert!((pos.y - 2.0).abs() < f32::EPSILON);
        assert!((pos.z - 3.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_world_to_scene_empty_world() {
        let mut world = World::new();
        let scene = world_to_scene(&mut world);
        assert!(scene.entities.is_empty());
    }

    #[test]
    fn test_load_scene_into_editor_spawns_entities() {
        use crate::data::components::{EntityComponents, TransformComponent, Vec3Data};
        use crate::data::scene::Entity as SceneEntity;

        let mut world = World::new();

        let mut scene = Scene::new("test", "Test Scene");
        let components = EntityComponents {
            transform: TransformComponent {
                position: Vec3Data::new(10.0, 20.0, 0.0),
                rotation: Vec3Data::default(),
                scale: Vec3Data::new(1.0, 1.0, 1.0),
                lock_uniform_scale: false,
            },
            ..EntityComponents::default()
        };
        scene.entities.push(
            SceneEntity::new("ent_a".to_string(), "ent_a".to_string()).with_components(components),
        );

        load_scene_into_editor(&mut world, scene);

        let mut query = world.query::<(&Name, &Transform)>();
        let results: Vec<_> = query
            .iter(&world)
            .map(|(n, t)| (n.to_string(), t.translation))
            .collect();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, "ent_a");
        assert!((results[0].1.x - 10.0).abs() < f32::EPSILON);
        assert!((results[0].1.y - 20.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_load_scene_into_editor_clears_existing_entities() {
        use crate::data::components::{EntityComponents, TransformComponent, Vec3Data};
        use crate::data::scene::Entity as SceneEntity;

        let mut world = World::new();
        // Pre-existing named entity
        world.spawn((Name::new("old"), Transform::default()));

        let mut scene = Scene::new("test", "Test");
        let components = EntityComponents {
            transform: TransformComponent {
                position: Vec3Data::new(0.0, 0.0, 0.0),
                rotation: Vec3Data::default(),
                scale: Vec3Data::new(1.0, 1.0, 1.0),
                lock_uniform_scale: false,
            },
            ..EntityComponents::default()
        };
        scene.entities.push(
            SceneEntity::new("new".to_string(), "new".to_string()).with_components(components),
        );

        load_scene_into_editor(&mut world, scene);

        let mut query = world.query::<&Name>();
        let names: Vec<String> = query.iter(&world).map(|n| n.to_string()).collect();
        assert_eq!(names.len(), 1);
        assert_eq!(names[0], "new");
    }
}
