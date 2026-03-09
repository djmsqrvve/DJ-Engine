//! Project-level configuration and settings.
//!
//! The [`Project`] struct is the top-level container for an entire game project,
//! referencing all scenes, story graphs, databases, and assets.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Input profile for the game (determines default control schemes).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InputProfile {
    /// JRPG-style keyboard/gamepad input
    #[default]
    Jrpg,
    /// RTS-style mouse/keyboard input
    Rts,
    /// Hybrid input supporting both styles
    Hybrid,
}

/// Editor theme preference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EditorTheme {
    Light,
    #[default]
    Dark,
}

/// Editor gizmo mode for transform manipulation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GizmoMode {
    #[default]
    Move,
    Rotate,
    Scale,
}

/// Editor layout preset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LayoutPreset {
    /// Optimized for JRPG map editing
    #[default]
    JrpgMapping,
    /// Optimized for TD balance/wave tuning
    TdBalancing,
    /// Custom user-defined layout
    Custom,
}

/// 2D size with integer dimensions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Size2i {
    pub width: u32,
    pub height: u32,
}

/// 2D size with float dimensions.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct Size2f {
    pub x: f32,
    pub y: f32,
}

/// Localization settings.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct LocalizationSettings {
    /// Available languages (e.g., ["en", "fr", "jp"])
    pub languages: Vec<String>,
    /// Default language code
    pub default_language: String,
}

/// File path configuration for project assets.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectPaths {
    /// Path to scenes directory
    pub scenes: String,
    /// Path to story graphs directory
    pub story_graphs: String,
    /// Path to database files
    pub database: String,
    /// Path to assets directory
    pub assets: String,
}

impl Default for ProjectPaths {
    fn default() -> Self {
        Self {
            scenes: "scenes".to_string(),
            story_graphs: "story_graphs".to_string(),
            database: "database".to_string(),
            assets: "assets".to_string(),
        }
    }
}

/// Autosave configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutosaveSettings {
    /// Whether autosave is enabled
    pub enabled: bool,
    /// Autosave interval in seconds
    pub interval_seconds: u32,
    /// Maximum number of backup files to keep
    pub max_backups: u32,
}

impl Default for AutosaveSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            interval_seconds: 300, // 5 minutes
            max_backups: 10,
        }
    }
}

/// Editor snap settings for transform gizmos.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SnapSettings {
    /// Position snap increment
    pub position: f32,
    /// Rotation snap increment (degrees)
    pub rotation: f32,
    /// Scale snap increment
    pub scale: f32,
    /// Whether snapping is enabled
    pub enabled: bool,
}

impl Default for SnapSettings {
    fn default() -> Self {
        Self {
            position: 16.0, // Pixel grid
            rotation: 15.0, // 15 degree increments
            scale: 0.25,
            enabled: true,
        }
    }
}

/// Project-wide settings that affect both editor and runtime.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProjectSettings {
    /// Target platforms
    pub platforms: Vec<String>,
    /// Default resolution
    pub default_resolution: Size2i,
    /// Target frames per second
    pub target_fps: u32,
    /// Enable VSync
    pub vsync: bool,
    /// Enable pixel-perfect rendering
    pub pixel_perfect: bool,
    /// Input profile (JRPG, RTS, or Hybrid)
    pub input_profile: InputProfile,
    /// Localization settings
    pub localization: LocalizationSettings,
    /// Project file paths
    pub paths: ProjectPaths,
    /// Autosave settings
    pub autosave: AutosaveSettings,
}

impl Default for ProjectSettings {
    fn default() -> Self {
        Self {
            platforms: vec!["pc".to_string()],
            default_resolution: Size2i {
                width: 1280,
                height: 720,
            },
            target_fps: 60,
            vsync: true,
            pixel_perfect: true,
            input_profile: InputProfile::default(),
            localization: LocalizationSettings {
                languages: vec!["en".to_string()],
                default_language: "en".to_string(),
            },
            paths: ProjectPaths::default(),
            autosave: AutosaveSettings::default(),
        }
    }
}

/// Per-user editor preferences (not stored in project).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EditorPreferences {
    /// Editor color theme
    pub theme: EditorTheme,
    /// UI scale factor
    pub ui_scale: f32,
    /// Font size in pixels
    pub font_size: u32,
    /// Grid cell size
    pub grid_size: Size2i,
    /// Snap settings
    pub snap: SnapSettings,
    /// Default gizmo mode
    pub default_gizmo_mode: GizmoMode,
    /// Custom keybindings (action -> key)
    pub keybindings: HashMap<String, String>,
    /// Layout preset
    pub layout_preset: LayoutPreset,
}

impl Default for EditorPreferences {
    fn default() -> Self {
        Self {
            theme: EditorTheme::Dark,
            ui_scale: 1.0,
            font_size: 14,
            grid_size: Size2i {
                width: 32,
                height: 32,
            },
            snap: SnapSettings::default(),
            default_gizmo_mode: GizmoMode::Move,
            keybindings: HashMap::new(),
            layout_preset: LayoutPreset::JrpgMapping,
        }
    }
}

/// Reference to a scene file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SceneRef {
    /// Unique scene identifier
    pub id: String,
    /// Path to scene file (relative to project)
    pub path: String,
}

/// Reference to a story graph file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoryGraphRef {
    /// Unique story graph identifier
    pub id: String,
    /// Path to story graph file (relative to project)
    pub path: String,
}

/// Top-level project container.
///
/// A project encompasses all game content: scenes, story graphs,
/// databases, and asset references.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Project {
    /// Unique project identifier
    pub id: String,
    /// Human-readable project name
    pub name: String,
    /// Project version string
    pub version: String,
    /// Project-wide settings
    pub settings: ProjectSettings,
    /// Per-user editor preferences
    #[serde(default)]
    pub editor_preferences: EditorPreferences,
    /// List of scene references
    #[serde(default)]
    pub scenes: Vec<SceneRef>,
    /// List of story graph references
    #[serde(default)]
    pub story_graphs: Vec<StoryGraphRef>,
}

impl Default for Project {
    fn default() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: "New Project".to_string(),
            version: "0.1.0".to_string(),
            settings: ProjectSettings::default(),
            editor_preferences: EditorPreferences::default(),
            scenes: Vec::new(),
            story_graphs: Vec::new(),
        }
    }
}

impl Project {
    /// Create a new project with the given name.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ..Default::default()
        }
    }

    /// Add a scene reference to the project.
    pub fn add_scene(&mut self, id: impl Into<String>, path: impl Into<String>) {
        self.scenes.push(SceneRef {
            id: id.into(),
            path: path.into(),
        });
    }

    /// Add a story graph reference to the project.
    pub fn add_story_graph(&mut self, id: impl Into<String>, path: impl Into<String>) {
        self.story_graphs.push(StoryGraphRef {
            id: id.into(),
            path: path.into(),
        });
    }

    /// Find a scene reference by ID.
    pub fn find_scene(&self, id: &str) -> Option<&SceneRef> {
        self.scenes.iter().find(|s| s.id == id)
    }

    /// Find a story graph reference by ID.
    pub fn find_story_graph(&self, id: &str) -> Option<&StoryGraphRef> {
        self.story_graphs.iter().find(|s| s.id == id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_serialization() {
        let project = Project::new("Test Game");
        let json = serde_json::to_string_pretty(&project).unwrap();
        let deserialized: Project = serde_json::from_str(&json).unwrap();
        assert_eq!(project.name, deserialized.name);
    }

    #[test]
    fn test_add_scene() {
        let mut project = Project::new("Test");
        project.add_scene("level_01", "scenes/level_01.json");
        assert_eq!(project.scenes.len(), 1);
        assert!(project.find_scene("level_01").is_some());
    }
}
