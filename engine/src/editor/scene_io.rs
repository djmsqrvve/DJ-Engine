use super::types::ActiveStoryGraph;
use crate::data::components::common::{ColorData, Vec3Data};
use crate::data::components::entity::EntityComponents;
use crate::data::components::rendering::{SpriteComponent, TransformComponent};
use crate::data::scene::{Entity as SceneEntity, Scene};
use crate::data::{loader, DataError};
use crate::project_mount::{
    ensure_default_project_refs, load_mounted_project_manifest, resolve_startup_scene_ref,
    resolve_startup_story_graph_ref, MountedProject,
};
use bevy::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};

pub(crate) fn load_initial_project_system(world: &mut World) {
    if let Err(error) = load_mounted_project(world) {
        warn!("Editor: Failed to load mounted project: {}", error);
    }
}

pub(crate) fn load_mounted_project(world: &mut World) -> Result<(), DataError> {
    let (root_path, manifest_path) = {
        let mounted_project = world.resource::<MountedProject>();
        let Some(root_path) = mounted_project.root_path.clone() else {
            return Ok(());
        };
        let Some(manifest_path) = mounted_project.manifest_path.clone() else {
            return Ok(());
        };
        (root_path, manifest_path)
    };

    world.resource_mut::<MountedProject>().project = None;

    let project = match load_mounted_project_manifest(&mut world.resource_mut::<MountedProject>()) {
        Ok(Some(project)) => project,
        Ok(None) => return Ok(()),
        Err(error) => {
            load_scene_into_editor(world, Scene::new("editor_empty", "Empty Scene"));
            world.insert_resource(ActiveStoryGraph::default());
            return Err(error);
        }
    };

    let scene_ref = resolve_startup_scene_ref(&project).cloned();
    let story_graph_ref = resolve_startup_story_graph_ref(&project).cloned();

    {
        let mut mounted_project = world.resource_mut::<MountedProject>();
        mounted_project.root_path = Some(root_path.clone());
        mounted_project.manifest_path = Some(manifest_path);
        mounted_project.project = Some(project.clone());
    }

    if let Some(scene_ref) = scene_ref {
        let scene_path = root_path.join(&scene_ref.path);
        match loader::load_scene(&scene_path) {
            Ok(scene) => load_scene_into_editor(world, scene),
            Err(error) => {
                warn!(
                    "Editor: Failed to load startup scene {:?}: {}. Loading empty scene instead.",
                    scene_path, error
                );
                load_scene_into_editor(world, Scene::new("editor_empty", "Empty Scene"));
            }
        }
    } else {
        load_scene_into_editor(world, Scene::new("editor_empty", "Empty Scene"));
    }

    if let Some(story_graph_ref) = story_graph_ref {
        let graph_path = root_path.join(&story_graph_ref.path);
        match loader::load_story_graph(&graph_path) {
            Ok(graph) => {
                world.insert_resource(ActiveStoryGraph(graph));
            }
            Err(error) => {
                warn!(
                    "Editor: Failed to load startup story graph {:?}: {}. Using empty graph instead.",
                    graph_path, error
                );
                world.insert_resource(ActiveStoryGraph::default());
            }
        }
    } else {
        world.insert_resource(ActiveStoryGraph::default());
    }

    if let Some(mut ui_state) = world.get_resource_mut::<super::types::EditorUiState>() {
        ui_state.selected_node_id = None;
        ui_state.connection_start_id = None;
        ui_state.dragged_node_id = None;
    }

    info!("Editor: Loaded project '{}'", project.name);
    Ok(())
}

pub(crate) fn resolve_asset_root(mounted_project: &MountedProject) -> Option<PathBuf> {
    let root_path = mounted_project.root_path.as_ref()?;
    let project = mounted_project.project.as_ref()?;
    Some(root_path.join(&project.settings.paths.assets))
}

fn ensure_parent_dir(path: &Path) -> Result<(), DataError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    Ok(())
}

pub(crate) fn save_project_impl(world: &mut World) {
    let (root_path, manifest_path, mut project) = {
        let mounted_project = world.resource::<MountedProject>();
        let Some(root_path) = mounted_project.root_path.clone() else {
            warn!("Cannot save: No project root set.");
            return;
        };
        let Some(manifest_path) = mounted_project.manifest_path.clone() else {
            warn!("Cannot save: No project manifest set.");
            return;
        };
        let Some(project) = mounted_project.project.clone() else {
            warn!("Cannot save: No project loaded.");
            return;
        };
        (root_path, manifest_path, project)
    };

    info!("Saving project to {:?}", manifest_path);

    ensure_default_project_refs(&mut project);

    let scene_ref = resolve_startup_scene_ref(&project)
        .cloned()
        .expect("default scene ref should exist after ensure_default_project_refs");
    let story_graph_ref = resolve_startup_story_graph_ref(&project)
        .cloned()
        .expect("default story graph ref should exist after ensure_default_project_refs");

    if let Err(error) = loader::save_project_structure(&project, &root_path) {
        error!("Failed to save project structure: {}", error);
        return;
    }

    let scene_path = root_path.join(&scene_ref.path);
    if let Err(error) = ensure_parent_dir(&scene_path) {
        error!(
            "Failed to create scene directory {:?}: {}",
            scene_path, error
        );
        return;
    }

    let scene = world_to_scene(world);
    if let Err(error) = loader::save_scene(&scene, &scene_path) {
        error!("Failed to save scene {:?}: {}", scene_path, error);
        return;
    }

    let graph_path = root_path.join(&story_graph_ref.path);
    if let Err(error) = ensure_parent_dir(&graph_path) {
        error!(
            "Failed to create story graph directory {:?}: {}",
            graph_path, error
        );
        return;
    }

    let graph = world.resource::<ActiveStoryGraph>().0.clone();
    if let Err(error) = loader::save_story_graph(&graph, &graph_path) {
        error!("Failed to save story graph {:?}: {}", graph_path, error);
        return;
    }

    if let Err(error) = loader::save_project(&project, &manifest_path) {
        error!(
            "Failed to save project manifest {:?}: {}",
            manifest_path, error
        );
        return;
    }

    world.resource_mut::<MountedProject>().project = Some(project);
    info!("Successfully saved project to {:?}", manifest_path);
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
    use crate::data::project::Project;
    use crate::data::project::ProjectSettings;
    use crate::data::story::graph::StoryGraphData;

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

    #[test]
    fn test_resolve_startup_scene_ref_prefers_configured_default() {
        let mut project = Project::new("Test");
        project.add_scene("first", "scenes/first.json");
        project.add_scene("preferred", "scenes/preferred.json");
        project.settings.startup.default_scene_id = Some("preferred".into());

        let scene_ref = resolve_startup_scene_ref(&project).unwrap();
        assert_eq!(scene_ref.id, "preferred");
    }

    #[test]
    fn test_resolve_startup_story_graph_ref_falls_back_to_first() {
        let mut project = Project::new("Test");
        project.add_story_graph("alpha", "story_graphs/alpha.json");
        project.add_story_graph("beta", "story_graphs/beta.json");

        let graph_ref = resolve_startup_story_graph_ref(&project).unwrap();
        assert_eq!(graph_ref.id, "alpha");
    }

    #[test]
    fn test_ensure_default_project_refs_creates_deterministic_defaults() {
        let mut project = Project::new("Test");
        project.settings = ProjectSettings::default();

        ensure_default_project_refs(&mut project);

        assert_eq!(
            project.settings.startup.default_scene_id.as_deref(),
            Some("current_scene")
        );
        assert_eq!(
            project.settings.startup.default_story_graph_id.as_deref(),
            Some("main")
        );
        assert_eq!(
            project.find_scene("current_scene").unwrap().path,
            "scenes/current_scene.json"
        );
        assert_eq!(
            project.find_story_graph("main").unwrap().path,
            "story_graphs/main.json"
        );
    }

    #[test]
    fn test_resolve_asset_root_uses_project_assets_path() {
        let mut project = Project::new("Test");
        project.settings.paths.assets = "content/assets".into();

        let mounted_project = MountedProject {
            root_path: Some(PathBuf::from("/tmp/project")),
            manifest_path: Some(PathBuf::from("/tmp/project/project.json")),
            project: Some(project),
        };

        assert_eq!(
            resolve_asset_root(&mounted_project),
            Some(PathBuf::from("/tmp/project/content/assets"))
        );
    }

    #[test]
    fn test_load_mounted_project_loads_manifest_scene_and_graph() {
        let temp_dir = tempfile::tempdir().unwrap();
        let root_path = temp_dir.path().to_path_buf();
        let manifest_path = root_path.join("project.json");

        fs::create_dir_all(root_path.join("scenes")).unwrap();
        fs::create_dir_all(root_path.join("story_graphs")).unwrap();

        let mut project = Project::new("Engine Project");
        project.add_scene("intro", "scenes/intro.json");
        project.add_story_graph("opening", "story_graphs/opening.json");
        project.settings.startup.default_scene_id = Some("intro".into());
        project.settings.startup.default_story_graph_id = Some("opening".into());

        loader::save_project(&project, &manifest_path).unwrap();

        let mut scene = Scene::new("intro", "Intro Scene");
        scene.entities.push(SceneEntity::new("hero", "Hero"));
        loader::save_scene(&scene, &root_path.join("scenes/intro.json")).unwrap();

        let mut graph = StoryGraphData::new("opening", "Opening");
        graph.add_node(crate::data::story::StoryNodeData::end("end"));
        loader::save_story_graph(&graph, &root_path.join("story_graphs/opening.json")).unwrap();

        let mut world = World::new();
        world.insert_resource(MountedProject {
            root_path: Some(root_path.clone()),
            manifest_path: Some(manifest_path),
            project: None,
        });
        world.insert_resource(ActiveStoryGraph::default());

        load_mounted_project(&mut world).unwrap();

        let loaded_project = world.resource::<MountedProject>();
        assert_eq!(
            loaded_project.project.as_ref().unwrap().name,
            "Engine Project"
        );

        let mut name_query = world.query::<&Name>();
        let names: Vec<String> = name_query
            .iter(&world)
            .map(|name| name.to_string())
            .collect();
        assert!(names.contains(&"Hero".to_string()));

        let active_graph = &world.resource::<ActiveStoryGraph>().0;
        assert_eq!(active_graph.id, "opening");
        assert_eq!(active_graph.name, "Opening");
    }
}
