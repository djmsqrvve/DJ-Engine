//! Save/load system for DJ Engine.
//!
//! Persists game state (story flags, variables, scene, graph position) as JSON
//! to `~/.local/share/dj_engine/saves/`.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
#[cfg(test)]
use std::sync::{Mutex, OnceLock};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum SaveScope {
    #[default]
    Global,
    Project(String),
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct SaveData {
    pub flags: HashMap<String, bool>,
    pub variables: HashMap<String, serde_json::Value>,
    pub current_node: Option<usize>,
    pub game_state: String,
    pub scene_background: Option<String>,
    pub project_id: Option<String>,
    pub scene_id: Option<String>,
    pub story_graph_id: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum SaveError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

fn save_dir() -> PathBuf {
    if let Ok(override_dir) = std::env::var("DJ_ENGINE_SAVE_DIR") {
        return PathBuf::from(override_dir);
    }

    default_save_dir()
}

fn default_save_dir() -> PathBuf {
    if let Ok(xdg_data_home) = std::env::var("XDG_DATA_HOME") {
        return PathBuf::from(xdg_data_home).join("dj_engine/saves");
    }

    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    PathBuf::from(home).join(".local/share/dj_engine/saves")
}

fn sanitize_scope_component(value: &str) -> String {
    value
        .chars()
        .map(|ch| match ch {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' => ch,
            _ => '_',
        })
        .collect()
}

fn scoped_save_dir(scope: &SaveScope) -> PathBuf {
    match scope {
        SaveScope::Global => save_dir(),
        SaveScope::Project(project_id) => save_dir()
            .join("projects")
            .join(sanitize_scope_component(project_id)),
    }
}

fn save_path_for_scope(scope: &SaveScope, slot: usize) -> PathBuf {
    scoped_save_dir(scope).join(format!("save_{slot}.json"))
}

#[cfg(test)]
pub(crate) fn save_test_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

pub fn save_game_scoped(
    scope: &SaveScope,
    slot: usize,
    data: &SaveData,
) -> Result<PathBuf, SaveError> {
    let dir = scoped_save_dir(scope);
    std::fs::create_dir_all(&dir)?;
    let path = save_path_for_scope(scope, slot);
    let json = serde_json::to_string_pretty(data)?;
    std::fs::write(&path, json)?;
    info!("Game saved to scope {:?}, slot {slot}", scope);
    Ok(path)
}

pub fn load_game_scoped(scope: &SaveScope, slot: usize) -> Result<SaveData, SaveError> {
    let path = save_path_for_scope(scope, slot);
    let json = std::fs::read_to_string(path)?;
    let data: SaveData = serde_json::from_str(&json)?;
    info!("Game loaded from scope {:?}, slot {slot}", scope);
    Ok(data)
}

pub fn has_save_scoped(scope: &SaveScope, slot: usize) -> bool {
    save_path_for_scope(scope, slot).exists()
}

pub fn delete_save_scoped(scope: &SaveScope, slot: usize) -> Result<(), SaveError> {
    let path = save_path_for_scope(scope, slot);
    if path.exists() {
        std::fs::remove_file(path)?;
    }
    Ok(())
}

pub fn save_game(slot: usize, data: &SaveData) -> Result<PathBuf, SaveError> {
    save_game_scoped(&SaveScope::Global, slot, data)
}

pub fn load_game(slot: usize) -> Result<SaveData, SaveError> {
    load_game_scoped(&SaveScope::Global, slot)
}

pub fn has_save(slot: usize) -> bool {
    has_save_scoped(&SaveScope::Global, slot)
}

pub fn delete_save(slot: usize) -> Result<(), SaveError> {
    delete_save_scoped(&SaveScope::Global, slot)
}

/// Message to request a game save.
#[derive(Message, Debug, Clone)]
pub struct SaveCommand {
    pub slot: usize,
    pub data: SaveData,
}

/// Message to request a game load.
#[derive(Message, Debug, Clone)]
pub struct LoadCommand {
    pub slot: usize,
}

/// Holds the most recently loaded save data for the game to consume.
#[derive(Resource, Default)]
pub struct LoadedSave(pub Option<SaveData>);

pub struct SavePlugin;

impl Plugin for SavePlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<SaveCommand>()
            .add_message::<LoadCommand>()
            .init_resource::<LoadedSave>()
            .add_systems(Update, (handle_save_commands, handle_load_commands));

        use crate::contracts::{AppContractExt, ContractEntry, PluginContract};
        app.register_contract(PluginContract {
            name: "SavePlugin".into(),
            description: "JSON save/load to ~/.local/share/dj_engine/saves/".into(),
            resources: vec![ContractEntry::of::<LoadedSave>(
                "Most recently loaded save data",
            )],
            components: vec![],
            events: vec![
                ContractEntry::of::<SaveCommand>("Request game save"),
                ContractEntry::of::<LoadCommand>("Request game load"),
            ],
            system_sets: vec![],
        });

        info!("Save Plugin initialized");
    }
}

fn handle_save_commands(mut commands: MessageReader<SaveCommand>) {
    for cmd in commands.read() {
        if let Err(e) = save_game(cmd.slot, &cmd.data) {
            error!("Failed to save game: {e}");
        }
    }
}

fn handle_load_commands(mut commands: MessageReader<LoadCommand>, mut loaded: ResMut<LoadedSave>) {
    for cmd in commands.read() {
        match load_game(cmd.slot) {
            Ok(data) => {
                loaded.0 = Some(data);
            }
            Err(e) => {
                error!("Failed to load game: {e}");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn with_temp_save_dir<T>(f: impl FnOnce(&Path) -> T) -> T {
        let _guard = crate::save::save_test_lock().lock().unwrap_or_else(|e| e.into_inner());
        let temp_dir = tempfile::tempdir().unwrap();
        let previous = std::env::var_os("DJ_ENGINE_SAVE_DIR");

        std::env::set_var("DJ_ENGINE_SAVE_DIR", temp_dir.path());
        let result = f(temp_dir.path());

        match previous {
            Some(value) => std::env::set_var("DJ_ENGINE_SAVE_DIR", value),
            None => std::env::remove_var("DJ_ENGINE_SAVE_DIR"),
        }

        result
    }

    #[test]
    fn test_save_data_default() {
        let data = SaveData::default();
        assert!(data.flags.is_empty());
        assert!(data.variables.is_empty());
        assert!(data.current_node.is_none());
        assert!(data.game_state.is_empty());
        assert!(data.scene_background.is_none());
        assert_eq!(data.project_id, None);
        assert_eq!(data.scene_id, None);
        assert_eq!(data.story_graph_id, None);
    }

    #[test]
    fn test_save_data_roundtrip() {
        let mut data = SaveData::default();
        data.flags.insert("intro_complete".into(), true);
        data.flags.insert("boss_defeated".into(), false);
        data.variables
            .insert("health".into(), serde_json::json!(75));
        data.variables
            .insert("name".into(), serde_json::json!("DJ"));
        data.current_node = Some(42);
        data.game_state = "Overworld".into();
        data.scene_background = Some("bg/forest.png".into());
        data.project_id = Some("project-alpha".into());
        data.scene_id = Some("forest".into());
        data.story_graph_id = Some("opening".into());

        let json = serde_json::to_string(&data).unwrap();
        let loaded: SaveData = serde_json::from_str(&json).unwrap();

        assert_eq!(loaded.flags.len(), 2);
        assert_eq!(loaded.flags["intro_complete"], true);
        assert_eq!(loaded.variables["health"], serde_json::json!(75));
        assert_eq!(loaded.current_node, Some(42));
        assert_eq!(loaded.game_state, "Overworld");
        assert_eq!(loaded.scene_background.as_deref(), Some("bg/forest.png"));
        assert_eq!(loaded.project_id.as_deref(), Some("project-alpha"));
        assert_eq!(loaded.scene_id.as_deref(), Some("forest"));
        assert_eq!(loaded.story_graph_id.as_deref(), Some("opening"));
    }

    #[test]
    fn test_save_load_file() {
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().join("test_save.json");

        let mut data = SaveData::default();
        data.flags.insert("flag_a".into(), true);
        data.game_state = "Battle".into();

        let json = serde_json::to_string_pretty(&data).unwrap();
        std::fs::write(&path, &json).unwrap();

        let loaded_json = std::fs::read_to_string(&path).unwrap();
        let loaded: SaveData = serde_json::from_str(&loaded_json).unwrap();
        assert_eq!(loaded.flags["flag_a"], true);
        assert_eq!(loaded.game_state, "Battle");
    }

    #[test]
    fn test_save_dir_is_xdg_compliant() {
        let dir = default_save_dir();
        let dir_str = dir.to_string_lossy();
        assert!(dir_str.contains(".local/share/dj_engine/saves"));
    }

    #[test]
    fn test_has_save_false_for_missing() {
        with_temp_save_dir(|_| {
            assert!(!has_save(9999));
        });
    }

    #[test]
    fn test_save_data_backward_compatible_with_old_json() {
        let json = r#"{
          "flags": {"intro_complete": true},
          "variables": {"score": 12},
          "current_node": 4,
          "game_state": "Overworld",
          "scene_background": "bg/forest.png"
        }"#;

        let loaded: SaveData = serde_json::from_str(json).unwrap();
        assert_eq!(loaded.project_id, None);
        assert_eq!(loaded.scene_id, None);
        assert_eq!(loaded.story_graph_id, None);
        assert_eq!(loaded.game_state, "Overworld");
    }

    #[test]
    fn test_save_dir_prefers_override() {
        with_temp_save_dir(|dir| {
            assert_eq!(save_dir(), dir);
        });
    }

    #[test]
    fn test_save_dir_uses_xdg_data_home() {
        let _guard = crate::save::save_test_lock().lock().unwrap_or_else(|e| e.into_inner());
        let previous_override = std::env::var_os("DJ_ENGINE_SAVE_DIR");
        let previous_xdg = std::env::var_os("XDG_DATA_HOME");
        let temp_dir = tempfile::tempdir().unwrap();

        std::env::remove_var("DJ_ENGINE_SAVE_DIR");
        std::env::set_var("XDG_DATA_HOME", temp_dir.path());

        let expected = temp_dir.path().join("dj_engine/saves");
        assert_eq!(save_dir(), expected);

        match previous_override {
            Some(value) => std::env::set_var("DJ_ENGINE_SAVE_DIR", value),
            None => std::env::remove_var("DJ_ENGINE_SAVE_DIR"),
        }
        match previous_xdg {
            Some(value) => std::env::set_var("XDG_DATA_HOME", value),
            None => std::env::remove_var("XDG_DATA_HOME"),
        }
    }

    #[test]
    fn test_save_and_load_integration() {
        with_temp_save_dir(|_| {
            let slot = 7777;
            let mut data = SaveData::default();
            data.flags.insert("test_flag".into(), true);
            data.game_state = "Overworld".into();
            data.current_node = Some(5);

            let result = save_game(slot, &data);
            assert!(result.is_ok());
            assert!(has_save(slot));

            let loaded = load_game(slot).unwrap();
            assert_eq!(loaded.flags["test_flag"], true);
            assert_eq!(loaded.game_state, "Overworld");
            assert_eq!(loaded.current_node, Some(5));

            delete_save(slot).unwrap();
            assert!(!has_save(slot));
        });
    }

    #[test]
    fn test_project_scoped_save_path_uses_project_id() {
        let scope = SaveScope::Project("project-alpha".into());
        let path = save_path_for_scope(&scope, 0);
        let path_string = path.to_string_lossy();

        assert!(path_string.contains("projects/project-alpha/save_0.json"));
    }

    #[test]
    fn test_project_scoped_save_isolation() {
        with_temp_save_dir(|_| {
            let project_a = SaveScope::Project("project-a".into());
            let project_b = SaveScope::Project("project-b".into());

            let data = SaveData {
                game_state: "Overworld".into(),
                project_id: Some("project-a".into()),
                ..Default::default()
            };

            save_game_scoped(&project_a, 0, &data).unwrap();

            assert!(has_save_scoped(&project_a, 0));
            assert!(!has_save_scoped(&project_b, 0));
            assert!(!has_save(0));

            let loaded = load_game_scoped(&project_a, 0).unwrap();
            assert_eq!(loaded.project_id.as_deref(), Some("project-a"));

            delete_save_scoped(&project_a, 0).unwrap();
            assert!(!has_save_scoped(&project_a, 0));
        });
    }
}
