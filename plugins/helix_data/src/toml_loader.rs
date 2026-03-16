//! TOML-based loading for helix3d data files.
//!
//! Reads the 22 curated TOML files from `dist/helix3d/` and deserializes
//! them into typed `Registry<T>` instances via the `helix-data` crate.

use helix_data::registry::Registry;
use serde::de::DeserializeOwned;
use std::path::Path;
use thiserror::Error;

/// Errors that can occur when loading helix3d TOML data.
#[derive(Debug, Error)]
pub enum HelixLoadError {
    #[error("failed to read '{file}': {source}")]
    Io {
        file: String,
        source: std::io::Error,
    },

    #[error("failed to parse '{file}': {source}")]
    Toml {
        file: String,
        source: toml::de::Error,
    },

    #[error("helix3d directory does not exist: {0}")]
    DirNotFound(String),
}

/// The 22 expected TOML filenames in `dist/helix3d/`.
pub const EXPECTED_TOML_FILES: &[&str] = &[
    "abilities.toml",
    "achievements.toml",
    "auras.toml",
    "class_data.toml",
    "consumables.toml",
    "currencies.toml",
    "equipment.toml",
    "guilds.toml",
    "inventory.toml",
    "items.toml",
    "mobs.toml",
    "mounts.toml",
    "npcs.toml",
    "professions.toml",
    "pvp.toml",
    "quests.toml",
    "raids.toml",
    "talents.toml",
    "titles.toml",
    "trade_goods.toml",
    "weapon_skills.toml",
    "zones.toml",
];

/// Load a single `Registry<T>` from a TOML file in the given directory.
///
/// The file is expected to contain named TOML tables where each table key
/// is an entity ID:
/// ```toml
/// [fireball]
/// name = { en = "Fireball" }
/// ability_type = "offensive"
/// ```
pub fn load_registry<T: DeserializeOwned>(
    dir: &Path,
    filename: &str,
) -> Result<Registry<T>, HelixLoadError> {
    let path = dir.join(filename);
    let content = std::fs::read_to_string(&path).map_err(|e| HelixLoadError::Io {
        file: filename.to_string(),
        source: e,
    })?;
    Registry::from_toml_str(&content).map_err(|e| HelixLoadError::Toml {
        file: filename.to_string(),
        source: e,
    })
}

/// Verify that the given path is a directory and exists.
pub fn validate_helix3d_dir(dir: &Path) -> Result<(), HelixLoadError> {
    if !dir.is_dir() {
        return Err(HelixLoadError::DirNotFound(dir.display().to_string()));
    }
    Ok(())
}

/// List which of the 22 expected TOML files are present in the directory.
pub fn discover_toml_files(dir: &Path) -> Vec<String> {
    EXPECTED_TOML_FILES
        .iter()
        .filter(|f| dir.join(f).is_file())
        .map(|f| f.to_string())
        .collect()
}

/// List which of the 22 expected TOML files are missing from the directory.
pub fn missing_toml_files(dir: &Path) -> Vec<String> {
    EXPECTED_TOML_FILES
        .iter()
        .filter(|f| !dir.join(f).is_file())
        .map(|f| f.to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use helix_data::types::BaseEntity;

    #[test]
    fn load_registry_from_inline_toml() {
        let dir = tempfile::tempdir().unwrap();
        let content = r#"
[alpha]
name = { en = "Alpha" }
category = "test"

[beta]
name = { en = "Beta" }
tags = ["x"]
"#;
        std::fs::write(dir.path().join("test.toml"), content).unwrap();

        let reg: Registry<BaseEntity> = load_registry(dir.path(), "test.toml").unwrap();
        assert_eq!(reg.len(), 2);
        assert!(reg.contains("alpha"));
        assert!(reg.contains("beta"));
        assert_eq!(reg.get("alpha").unwrap().name.en(), "Alpha");
    }

    #[test]
    fn load_registry_missing_file_returns_io_error() {
        let dir = tempfile::tempdir().unwrap();
        let result: Result<Registry<BaseEntity>, _> = load_registry(dir.path(), "nonexistent.toml");
        assert!(matches!(result, Err(HelixLoadError::Io { .. })));
    }

    #[test]
    fn load_registry_malformed_toml_returns_parse_error() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("bad.toml"), "not valid { toml").unwrap();
        let result: Result<Registry<BaseEntity>, _> = load_registry(dir.path(), "bad.toml");
        assert!(matches!(result, Err(HelixLoadError::Toml { .. })));
    }

    #[test]
    fn validate_helix3d_dir_rejects_nonexistent() {
        let result = validate_helix3d_dir(Path::new("/nonexistent/path"));
        assert!(matches!(result, Err(HelixLoadError::DirNotFound(_))));
    }

    #[test]
    fn discover_and_missing_toml_files() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("abilities.toml"), "").unwrap();
        std::fs::write(dir.path().join("items.toml"), "").unwrap();

        let found = discover_toml_files(dir.path());
        assert!(found.contains(&"abilities.toml".to_string()));
        assert!(found.contains(&"items.toml".to_string()));
        assert_eq!(found.len(), 2);

        let missing = missing_toml_files(dir.path());
        assert_eq!(missing.len(), 20);
        assert!(missing.contains(&"mobs.toml".to_string()));
    }
}
