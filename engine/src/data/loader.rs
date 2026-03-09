//! Loading functions for project data.
//!
//! Provides functions to load projects, scenes, databases, and story graphs
//! from JSON files.

use std::fs;
use std::path::Path;
use thiserror::Error;

use super::assets::AssetIndex;
use super::database::Database;
use super::project::Project;
use super::scene::Scene;
use super::story::StoryGraphData;

/// Error type for data loading operations.
#[derive(Debug, Error)]
pub enum DataError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("File not found: {0}")]
    NotFound(String),

    #[error("Invalid project structure: {0}")]
    InvalidProject(String),
}

/// Load a project from a JSON file.
///
/// # Arguments
/// * `path` - Path to the project.json file
///
/// # Returns
/// The loaded project or an error
pub fn load_project(path: &Path) -> Result<Project, DataError> {
    if !path.exists() {
        return Err(DataError::NotFound(path.display().to_string()));
    }

    let content = fs::read_to_string(path)?;
    let project: Project = serde_json::from_str(&content)?;
    Ok(project)
}

/// Load a scene from a JSON file.
///
/// # Arguments
/// * `path` - Path to the scene JSON file
///
/// # Returns
/// The loaded scene or an error
pub fn load_scene(path: &Path) -> Result<Scene, DataError> {
    if !path.exists() {
        return Err(DataError::NotFound(path.display().to_string()));
    }

    let content = fs::read_to_string(path)?;
    let scene: Scene = serde_json::from_str(&content)?;
    Ok(scene)
}

/// Load a database from a JSON file.
///
/// # Arguments
/// * `path` - Path to the database JSON file
///
/// # Returns
/// The loaded database or an error
pub fn load_database(path: &Path) -> Result<Database, DataError> {
    if !path.exists() {
        return Err(DataError::NotFound(path.display().to_string()));
    }

    let content = fs::read_to_string(path)?;
    let database: Database = serde_json::from_str(&content)?;
    Ok(database)
}

/// Load a story graph from a JSON file.
///
/// # Arguments
/// * `path` - Path to the story graph JSON file
///
/// # Returns
/// The loaded story graph or an error
pub fn load_story_graph(path: &Path) -> Result<StoryGraphData, DataError> {
    if !path.exists() {
        return Err(DataError::NotFound(path.display().to_string()));
    }

    let content = fs::read_to_string(path)?;
    let graph: StoryGraphData = serde_json::from_str(&content)?;
    Ok(graph)
}

/// Load an asset index from a JSON file.
///
/// # Arguments
/// * `path` - Path to the asset index JSON file
///
/// # Returns
/// The loaded asset index or an error
pub fn load_asset_index(path: &Path) -> Result<AssetIndex, DataError> {
    if !path.exists() {
        return Err(DataError::NotFound(path.display().to_string()));
    }

    let content = fs::read_to_string(path)?;
    let index: AssetIndex = serde_json::from_str(&content)?;
    Ok(index)
}

/// Save a project to a JSON file.
pub fn save_project(project: &Project, path: &Path) -> Result<(), DataError> {
    let content = serde_json::to_string_pretty(project)?;
    fs::write(path, content)?;
    Ok(())
}

/// Save a scene to a JSON file.
pub fn save_scene(scene: &Scene, path: &Path) -> Result<(), DataError> {
    let content = serde_json::to_string_pretty(scene)?;
    fs::write(path, content)?;
    Ok(())
}

/// Save a database to a JSON file.
pub fn save_database(database: &Database, path: &Path) -> Result<(), DataError> {
    let content = serde_json::to_string_pretty(database)?;
    fs::write(path, content)?;
    Ok(())
}

/// Save a story graph to a JSON file.
pub fn save_story_graph(graph: &StoryGraphData, path: &Path) -> Result<(), DataError> {
    let content = serde_json::to_string_pretty(graph)?;
    fs::write(path, content)?;
    Ok(())
}

/// Save the entire project structure to a directory.
///
/// This creates the necessary subdirectories (scenes, assets, etc.) and saves the `project.json` file.
///
/// # Arguments
/// * `project` - The project data to save
/// * `root_path` - The root directory for the project
pub fn save_project_structure(project: &Project, root_path: &Path) -> Result<(), DataError> {
    if !root_path.exists() {
        fs::create_dir_all(root_path)?;
    }

    let paths = &project.settings.paths;

    // Create subdirectories
    fs::create_dir_all(root_path.join(&paths.scenes))?;
    fs::create_dir_all(root_path.join(&paths.story_graphs))?;
    fs::create_dir_all(root_path.join(&paths.database))?;
    fs::create_dir_all(root_path.join(&paths.assets))?;

    // Save project.json
    let project_file = root_path.join("project.json");
    save_project(project, &project_file)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_load_save_project() {
        let project = Project::new("Test Project");

        let mut file = NamedTempFile::new().unwrap();
        let json = serde_json::to_string_pretty(&project).unwrap();
        file.write_all(json.as_bytes()).unwrap();

        let loaded = load_project(file.path()).unwrap();
        assert_eq!(project.name, loaded.name);
    }

    #[test]
    fn test_load_not_found() {
        let result = load_project(Path::new("/nonexistent/path.json"));
        assert!(matches!(result, Err(DataError::NotFound(_))));
    }

    #[test]
    fn test_save_project_structure() {
        let project = Project::new("Structure Test");
        let temp_dir = tempfile::tempdir().unwrap();
        let root_path = temp_dir.path();

        let result = save_project_structure(&project, root_path);
        assert!(result.is_ok());

        assert!(root_path.join("project.json").exists());
        assert!(root_path.join("scenes").exists());
        assert!(root_path.join("story_graphs").exists());
        assert!(root_path.join("database").exists());
        assert!(root_path.join("assets").exists());
    }
}
