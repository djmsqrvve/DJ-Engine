use crate::data::loader;
use crate::data::project::{Project, SceneRef, StoryGraphRef};
use crate::data::DataError;
use bevy::prelude::*;
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
}
