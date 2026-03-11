//! Save/load system for DJ Engine.
//!
//! Persists game state (story flags, variables, scene, graph position) as JSON
//! to `~/.local/share/dj_engine/saves/`.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct SaveData {
    pub flags: HashMap<String, bool>,
    pub variables: HashMap<String, serde_json::Value>,
    pub current_node: Option<usize>,
    pub game_state: String,
    pub scene_background: Option<String>,
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

fn save_path(slot: usize) -> PathBuf {
    save_dir().join(format!("save_{slot}.json"))
}

pub fn save_game(slot: usize, data: &SaveData) -> Result<PathBuf, SaveError> {
    let dir = save_dir();
    std::fs::create_dir_all(&dir)?;
    let path = save_path(slot);
    let json = serde_json::to_string_pretty(data)?;
    std::fs::write(&path, json)?;
    info!("Game saved to slot {slot}");
    Ok(path)
}

pub fn load_game(slot: usize) -> Result<SaveData, SaveError> {
    let path = save_path(slot);
    let json = std::fs::read_to_string(path)?;
    let data: SaveData = serde_json::from_str(&json)?;
    info!("Game loaded from slot {slot}");
    Ok(data)
}

pub fn has_save(slot: usize) -> bool {
    save_path(slot).exists()
}

pub fn delete_save(slot: usize) -> Result<(), SaveError> {
    let path = save_path(slot);
    if path.exists() {
        std::fs::remove_file(path)?;
    }
    Ok(())
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
    use std::sync::{Mutex, OnceLock};

    fn save_test_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    fn with_temp_save_dir<T>(f: impl FnOnce(&Path) -> T) -> T {
        let _guard = save_test_lock().lock().unwrap();
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

        let json = serde_json::to_string(&data).unwrap();
        let loaded: SaveData = serde_json::from_str(&json).unwrap();

        assert_eq!(loaded.flags.len(), 2);
        assert_eq!(loaded.flags["intro_complete"], true);
        assert_eq!(loaded.variables["health"], serde_json::json!(75));
        assert_eq!(loaded.current_node, Some(42));
        assert_eq!(loaded.game_state, "Overworld");
        assert_eq!(loaded.scene_background.as_deref(), Some("bg/forest.png"));
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
    fn test_save_dir_prefers_override() {
        with_temp_save_dir(|dir| {
            assert_eq!(save_dir(), dir);
        });
    }

    #[test]
    fn test_save_dir_uses_xdg_data_home() {
        let _guard = save_test_lock().lock().unwrap();
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
}
