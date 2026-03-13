use crate::data::loader;
use crate::data::project::{Project, SceneRef, StoryGraphRef};
use crate::data::DataError;
use bevy::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};

/// Resource holding the currently mounted project for editor/runtime flows.
#[derive(Resource, Default, Debug, Clone)]
pub struct MountedProject {
    pub root_path: Option<PathBuf>,
    pub manifest_path: Option<PathBuf>,
    pub project: Option<Project>,
}

impl MountedProject {
    pub fn from_path(path: &Path) -> Result<Self, DataError> {
        let (root_path, manifest_path) = normalize_project_path(path)?;
        Ok(Self {
            root_path: Some(root_path),
            manifest_path: Some(manifest_path),
            project: None,
        })
    }
}

pub fn normalize_project_path(path: &Path) -> Result<(PathBuf, PathBuf), DataError> {
    if path.file_name().and_then(|name| name.to_str()) == Some("project.json") {
        let Some(root_path) = path.parent() else {
            return Err(DataError::InvalidProject(
                "project.json must live inside a project directory".into(),
            ));
        };
        return Ok((root_path.to_path_buf(), path.to_path_buf()));
    }

    if path.extension().is_some() {
        return Err(DataError::InvalidProject(
            "Project path must be a directory or a project.json manifest".into(),
        ));
    }

    Ok((path.to_path_buf(), path.join("project.json")))
}

pub fn load_mounted_project_manifest(
    mounted_project: &mut MountedProject,
) -> Result<Option<Project>, DataError> {
    let Some(manifest_path) = mounted_project.manifest_path.clone() else {
        mounted_project.project = None;
        return Ok(None);
    };

    let project = loader::load_project(&manifest_path)?;
    mounted_project.project = Some(project.clone());
    Ok(Some(project))
}

pub fn resolve_custom_data_root(mounted_project: &MountedProject) -> Option<PathBuf> {
    let root_path = mounted_project.root_path.as_ref()?;
    let project = mounted_project.project.as_ref()?;
    Some(root_path.join(&project.settings.paths.data))
}

pub fn resolve_custom_data_manifest_path(mounted_project: &MountedProject) -> Option<PathBuf> {
    Some(resolve_custom_data_root(mounted_project)?.join("registry.json"))
}

pub fn resolve_startup_scene_ref(project: &Project) -> Option<&SceneRef> {
    project
        .settings
        .startup
        .default_scene_id
        .as_deref()
        .and_then(|scene_id| project.find_scene(scene_id))
        .or_else(|| project.scenes.first())
}

pub fn resolve_startup_story_graph_ref(project: &Project) -> Option<&StoryGraphRef> {
    project
        .settings
        .startup
        .default_story_graph_id
        .as_deref()
        .and_then(|graph_id| project.find_story_graph(graph_id))
        .or_else(|| project.story_graphs.first())
}

pub fn ensure_default_project_refs(project: &mut Project) {
    if project
        .settings
        .startup
        .default_scene_id
        .as_deref()
        .and_then(|scene_id| project.find_scene(scene_id))
        .is_none()
    {
        let default_id = "current_scene".to_string();
        let default_path = PathBuf::from(&project.settings.paths.scenes)
            .join("current_scene.json")
            .to_string_lossy()
            .into_owned();

        if let Some(scene_ref) = project
            .scenes
            .iter_mut()
            .find(|scene| scene.id == default_id)
        {
            scene_ref.path = default_path.clone();
        } else {
            project.add_scene(default_id.clone(), default_path);
        }

        project.settings.startup.default_scene_id = Some(default_id);
    }

    if project
        .settings
        .startup
        .default_story_graph_id
        .as_deref()
        .and_then(|graph_id| project.find_story_graph(graph_id))
        .is_none()
    {
        let default_id = "main".to_string();
        let default_path = PathBuf::from(&project.settings.paths.story_graphs)
            .join("main.json")
            .to_string_lossy()
            .into_owned();

        if let Some(graph_ref) = project
            .story_graphs
            .iter_mut()
            .find(|graph| graph.id == default_id)
        {
            graph_ref.path = default_path.clone();
        } else {
            project.add_story_graph(default_id.clone(), default_path);
        }

        project.settings.startup.default_story_graph_id = Some(default_id);
    }
}

/// Discover `project.json` files under `root`, recursively up to `max_depth`.
pub fn discover_projects_in_directory(root: &Path, max_depth: usize) -> Vec<PathBuf> {
    let mut results = Vec::new();
    discover_projects_recursive(root, max_depth, 0, &mut results);
    results.sort();
    results
}

fn discover_projects_recursive(
    dir: &Path,
    max_depth: usize,
    current_depth: usize,
    results: &mut Vec<PathBuf>,
) {
    let manifest = dir.join("project.json");
    if manifest.is_file() {
        results.push(manifest);
        // Don't recurse into a project directory — it won't contain nested projects.
        return;
    }

    if current_depth >= max_depth {
        return;
    }

    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };

    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        if path.is_dir() {
            // Skip hidden directories and common non-project dirs.
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if name.starts_with('.') || name == "target" || name == "node_modules" {
                continue;
            }
            discover_projects_recursive(&path, max_depth, current_depth + 1, results);
        }
    }
}

/// Returns the workspace root (parent of the engine crate).
pub fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("engine crate should live inside the workspace root")
        .to_path_buf()
}

/// Default project directory inside the workspace.
pub fn default_project_dir() -> PathBuf {
    workspace_root().join("projects").join("default")
}

/// Create a new default project on disk and return its mount info.
/// Creates the directory structure and saves a minimal `project.json`.
pub fn create_default_project() -> Result<MountedProject, DataError> {
    let project_dir = default_project_dir();
    fs::create_dir_all(&project_dir).map_err(|e| {
        DataError::InvalidProject(format!(
            "Failed to create default project directory {:?}: {}",
            project_dir, e
        ))
    })?;

    let mut project = Project::new("Default Project");
    ensure_default_project_refs(&mut project);

    let manifest_path = project_dir.join("project.json");
    loader::save_project(&project, &manifest_path)?;

    info!("Created default project at {:?}", manifest_path.display());

    Ok(MountedProject {
        root_path: Some(project_dir),
        manifest_path: Some(manifest_path),
        project: None, // Will be loaded by the startup system.
    })
}

/// Auto-discover or create a project. Returns mount info for the best candidate.
///
/// Priority:
/// 1. Discover existing projects under the workspace root (depth 3).
/// 2. If none found, create a default project at `projects/default/`.
pub fn auto_discover_or_create_project() -> Result<MountedProject, DataError> {
    let root = workspace_root();
    let discovered = discover_projects_in_directory(&root, 3);

    if let Some(manifest_path) = discovered.first() {
        let (root_path, manifest_path) = normalize_project_path(manifest_path)?;
        info!("Auto-discovered project at {:?}", manifest_path.display());
        return Ok(MountedProject {
            root_path: Some(root_path),
            manifest_path: Some(manifest_path),
            project: None,
        });
    }

    info!("No projects found in workspace, creating default project");
    create_default_project()
}

const TEMPLATE_SCENE_JSON: &str = include_str!("../template/scenes/starter.json");
const TEMPLATE_STORY_GRAPH_JSON: &str = include_str!("../template/story_graphs/intro.json");

/// Create a new game project with a working starter scene and story graph.
///
/// The project is immediately playable in the editor and runtime preview.
pub fn create_new_project(name: &str, project_dir: &Path) -> Result<MountedProject, DataError> {
    if project_dir.join("project.json").exists() {
        return Err(DataError::InvalidProject(format!(
            "A project already exists at {:?}",
            project_dir
        )));
    }

    fs::create_dir_all(project_dir).map_err(|e| {
        DataError::InvalidProject(format!(
            "Failed to create project directory {:?}: {}",
            project_dir, e
        ))
    })?;

    let mut project = Project::new(name);
    project.add_scene("starter", "scenes/starter.json");
    project.add_story_graph("intro", "story_graphs/intro.json");
    project.settings.startup.default_scene_id = Some("starter".into());
    project.settings.startup.default_story_graph_id = Some("intro".into());

    // Create directory structure.
    loader::save_project_structure(&project, project_dir)?;

    // Write the manifest.
    let manifest_path = project_dir.join("project.json");
    loader::save_project(&project, &manifest_path)?;

    // Write starter content files.
    let scene_path = project_dir.join("scenes/starter.json");
    fs::write(&scene_path, TEMPLATE_SCENE_JSON)
        .map_err(|e| DataError::InvalidProject(format!("Failed to write starter scene: {}", e)))?;

    let story_graph_path = project_dir.join("story_graphs/intro.json");
    fs::write(&story_graph_path, TEMPLATE_STORY_GRAPH_JSON).map_err(|e| {
        DataError::InvalidProject(format!("Failed to write starter story graph: {}", e))
    })?;

    info!(
        "Created new project '{}' at {:?}",
        name,
        manifest_path.display()
    );

    Ok(MountedProject {
        root_path: Some(project_dir.to_path_buf()),
        manifest_path: Some(manifest_path),
        project: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_project_path_accepts_directory() {
        let (root, manifest) = normalize_project_path(Path::new("projects/sample")).unwrap();
        assert_eq!(root, PathBuf::from("projects/sample"));
        assert_eq!(manifest, PathBuf::from("projects/sample/project.json"));
    }

    #[test]
    fn test_normalize_project_path_accepts_project_manifest() {
        let (root, manifest) =
            normalize_project_path(Path::new("projects/sample/project.json")).unwrap();
        assert_eq!(root, PathBuf::from("projects/sample"));
        assert_eq!(manifest, PathBuf::from("projects/sample/project.json"));
    }

    #[test]
    fn test_normalize_project_path_rejects_non_manifest_file() {
        let result = normalize_project_path(Path::new("projects/sample/notes.json"));
        assert!(result.is_err());
    }

    #[test]
    fn test_load_mounted_project_manifest_roundtrip() {
        let temp_dir = tempfile::tempdir().unwrap();
        let manifest_path = temp_dir.path().join("project.json");

        let project = Project::new("Mounted Project");
        loader::save_project(&project, &manifest_path).unwrap();

        let mut mounted_project = MountedProject {
            root_path: Some(temp_dir.path().to_path_buf()),
            manifest_path: Some(manifest_path),
            project: None,
        };

        let loaded = load_mounted_project_manifest(&mut mounted_project)
            .unwrap()
            .expect("project should load");

        assert_eq!(loaded.name, "Mounted Project");
        assert_eq!(
            mounted_project
                .project
                .as_ref()
                .map(|project| project.name.as_str()),
            Some("Mounted Project")
        );
    }

    #[test]
    fn test_discover_projects_finds_nested_manifests() {
        let temp_dir = tempfile::tempdir().unwrap();
        let project_a = temp_dir.path().join("projects").join("alpha");
        let project_b = temp_dir.path().join("projects").join("beta");
        fs::create_dir_all(&project_a).unwrap();
        fs::create_dir_all(&project_b).unwrap();
        fs::write(project_a.join("project.json"), "{}").unwrap();
        fs::write(project_b.join("project.json"), "{}").unwrap();

        let discovered = discover_projects_in_directory(temp_dir.path(), 3);
        assert_eq!(discovered.len(), 2);
        assert!(discovered.iter().any(|p| p.ends_with("alpha/project.json")));
        assert!(discovered.iter().any(|p| p.ends_with("beta/project.json")));
    }

    #[test]
    fn test_discover_projects_respects_max_depth() {
        let temp_dir = tempfile::tempdir().unwrap();
        let deep = temp_dir.path().join("a").join("b").join("c").join("d");
        fs::create_dir_all(&deep).unwrap();
        fs::write(deep.join("project.json"), "{}").unwrap();

        // Depth 2 should not reach a/b/c/d (that's depth 4).
        let discovered = discover_projects_in_directory(temp_dir.path(), 2);
        assert!(discovered.is_empty());

        // Depth 4 should find it.
        let discovered = discover_projects_in_directory(temp_dir.path(), 4);
        assert_eq!(discovered.len(), 1);
    }

    #[test]
    fn test_discover_projects_skips_hidden_and_target_dirs() {
        let temp_dir = tempfile::tempdir().unwrap();
        let hidden = temp_dir.path().join(".hidden");
        let target = temp_dir.path().join("target");
        let visible = temp_dir.path().join("visible");
        fs::create_dir_all(&hidden).unwrap();
        fs::create_dir_all(&target).unwrap();
        fs::create_dir_all(&visible).unwrap();
        fs::write(hidden.join("project.json"), "{}").unwrap();
        fs::write(target.join("project.json"), "{}").unwrap();
        fs::write(visible.join("project.json"), "{}").unwrap();

        let discovered = discover_projects_in_directory(temp_dir.path(), 2);
        assert_eq!(discovered.len(), 1);
        assert!(discovered[0].to_string_lossy().contains("visible"));
    }

    #[test]
    fn test_create_default_project_creates_manifest_on_disk() {
        // We can't test create_default_project directly since it writes to the
        // workspace. Instead test the underlying logic with a temp dir.
        let temp_dir = tempfile::tempdir().unwrap();
        let project_dir = temp_dir.path().join("projects").join("default");
        fs::create_dir_all(&project_dir).unwrap();

        let mut project = Project::new("Default Project");
        ensure_default_project_refs(&mut project);

        let manifest_path = project_dir.join("project.json");
        loader::save_project(&project, &manifest_path).unwrap();

        assert!(manifest_path.is_file());
        let loaded = loader::load_project(&manifest_path).unwrap();
        assert_eq!(loaded.name, "Default Project");
    }

    #[test]
    fn test_resolve_custom_data_manifest_path_uses_project_data_root() {
        let mut project = Project::new("Data Project");
        project.settings.paths.data = "custom_data".into();

        let mounted_project = MountedProject {
            root_path: Some(PathBuf::from("/tmp/data_project")),
            manifest_path: Some(PathBuf::from("/tmp/data_project/project.json")),
            project: Some(project),
        };

        assert_eq!(
            resolve_custom_data_manifest_path(&mounted_project),
            Some(PathBuf::from("/tmp/data_project/custom_data/registry.json"))
        );
    }

    #[test]
    fn test_create_new_project_scaffolds_playable_project() {
        let temp_dir = tempfile::tempdir().unwrap();
        let project_dir = temp_dir.path().join("my_game");

        let mount = create_new_project("My Game", &project_dir).unwrap();
        assert!(mount.manifest_path.is_some());

        // Manifest exists and loads.
        let manifest_path = project_dir.join("project.json");
        assert!(manifest_path.is_file());
        let project = loader::load_project(&manifest_path).unwrap();
        assert_eq!(project.name, "My Game");
        assert_eq!(
            project.settings.startup.default_scene_id.as_deref(),
            Some("starter")
        );
        assert_eq!(
            project.settings.startup.default_story_graph_id.as_deref(),
            Some("intro")
        );

        // Scene and story graph files exist.
        assert!(project_dir.join("scenes/starter.json").is_file());
        assert!(project_dir.join("story_graphs/intro.json").is_file());

        // Directory structure was created.
        assert!(project_dir.join("assets").is_dir());
        assert!(project_dir.join("data").is_dir());
        assert!(project_dir.join("database").is_dir());
    }

    #[test]
    fn test_create_new_project_rejects_existing_project() {
        let temp_dir = tempfile::tempdir().unwrap();
        let project_dir = temp_dir.path().join("existing");
        fs::create_dir_all(&project_dir).unwrap();
        fs::write(project_dir.join("project.json"), "{}").unwrap();

        let result = create_new_project("Existing", &project_dir);
        assert!(result.is_err());
    }
}
