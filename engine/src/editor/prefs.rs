use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct EditorPrefs {
    pub completed_tutorials: HashSet<String>,
}

pub fn editor_prefs_path() -> PathBuf {
    if let Ok(override_dir) = std::env::var("DJ_ENGINE_PREFS_DIR") {
        return PathBuf::from(override_dir).join("editor_prefs.json");
    }

    if let Ok(xdg_data_home) = std::env::var("XDG_DATA_HOME") {
        return PathBuf::from(xdg_data_home).join("dj_engine/editor_prefs.json");
    }

    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    PathBuf::from(home).join(".local/share/dj_engine/editor_prefs.json")
}

pub fn load_editor_prefs() -> EditorPrefs {
    let path = editor_prefs_path();
    if let Ok(json) = std::fs::read_to_string(&path) {
        if let Ok(prefs) = serde_json::from_str(&json) {
            return prefs;
        }
    }
    EditorPrefs::default()
}

pub fn save_editor_prefs(prefs: &EditorPrefs) {
    let path = editor_prefs_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(json) = serde_json::to_string_pretty(prefs) {
        let _ = std::fs::write(&path, json);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_editor_prefs_save_load() {
        let temp_dir = tempfile::tempdir().unwrap();
        std::env::set_var("DJ_ENGINE_PREFS_DIR", temp_dir.path());

        let mut prefs = EditorPrefs::default();
        prefs.completed_tutorials.insert("TestTutorial".to_string());

        save_editor_prefs(&prefs);

        let loaded = load_editor_prefs();
        assert!(loaded.completed_tutorials.contains("TestTutorial"));

        std::env::remove_var("DJ_ENGINE_PREFS_DIR");
    }
}
