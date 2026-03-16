//! Balance overlay system for DJ Engine-specific tuning of Helix base data.
//!
//! Each game can define per-entity numeric overrides in TOML files
//! (e.g. `data/balance/mobs.toml`) that are applied when converting
//! Helix entities to engine database rows.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Per-entity balance override. Only the fields present are overridden;
/// everything else passes through from the Helix base data.
///
/// ```toml
/// [wolf]
/// health = 30.0
/// damage_max = 8.0
///
/// [inferno_drake]
/// health = 8000.0
/// ```
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BalanceOverlay {
    #[serde(flatten)]
    pub overrides: HashMap<String, f64>,
}

impl BalanceOverlay {
    /// Get an override value by field name.
    pub fn get_f64(&self, field: &str) -> Option<f64> {
        self.overrides.get(field).copied()
    }

    /// Set an override value.
    pub fn set(&mut self, field: &str, value: f64) {
        self.overrides.insert(field.to_string(), value);
    }
}

/// All balance overlays organized by entity kind and entity ID.
///
/// ```text
/// layers["mobs"]["wolf"] = BalanceOverlay { health: 30.0 }
/// layers["items"]["health_potion"] = BalanceOverlay { sell_price: 5.0 }
/// ```
#[derive(Resource, Debug, Clone, Default, Serialize, Deserialize)]
pub struct BalanceOverlays {
    pub layers: HashMap<String, HashMap<String, BalanceOverlay>>,
}

impl BalanceOverlays {
    /// Get the balance overlay for a specific entity in a kind.
    pub fn get(&self, kind: &str, entity_id: &str) -> Option<&BalanceOverlay> {
        self.layers.get(kind)?.get(entity_id)
    }

    /// Get a mutable reference to the balance overlay, creating it if needed.
    pub fn get_or_insert(&mut self, kind: &str, entity_id: &str) -> &mut BalanceOverlay {
        self.layers
            .entry(kind.to_string())
            .or_default()
            .entry(entity_id.to_string())
            .or_default()
    }
}

/// Load balance overlays from a directory containing per-kind TOML files.
///
/// Each file is named after the entity kind (e.g. `mobs.toml`, `items.toml`).
/// The file contains named TOML tables with numeric overrides:
///
/// ```toml
/// # mobs.toml
/// [wolf]
/// health = 30.0
///
/// [inferno_drake]
/// health = 8000.0
/// ```
pub fn load_balance_overlays(balance_dir: &Path) -> Result<BalanceOverlays, BalanceLoadError> {
    let mut overlays = BalanceOverlays::default();

    if !balance_dir.is_dir() {
        return Ok(overlays); // No balance dir = no overrides
    }

    for entry in std::fs::read_dir(balance_dir).map_err(|e| BalanceLoadError::Io {
        path: balance_dir.display().to_string(),
        source: e,
    })? {
        let entry = entry.map_err(|e| BalanceLoadError::Io {
            path: balance_dir.display().to_string(),
            source: e,
        })?;
        let path = entry.path();

        if path.extension().and_then(|e| e.to_str()) != Some("toml") {
            continue;
        }

        let kind = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        let content = std::fs::read_to_string(&path).map_err(|e| BalanceLoadError::Io {
            path: path.display().to_string(),
            source: e,
        })?;

        let table: HashMap<String, BalanceOverlay> =
            toml::from_str(&content).map_err(|e| BalanceLoadError::Toml {
                file: kind.clone(),
                source: e,
            })?;

        overlays.layers.insert(kind, table);
    }

    Ok(overlays)
}

/// Errors from loading balance TOML files.
#[derive(Debug, thiserror::Error)]
pub enum BalanceLoadError {
    #[error("failed to read '{path}': {source}")]
    Io {
        path: String,
        source: std::io::Error,
    },
    #[error("failed to parse balance file '{file}': {source}")]
    Toml {
        file: String,
        source: toml::de::Error,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn balance_overlay_get_set() {
        let mut overlay = BalanceOverlay::default();
        assert!(overlay.get_f64("health").is_none());

        overlay.set("health", 30.0);
        assert_eq!(overlay.get_f64("health"), Some(30.0));
    }

    #[test]
    fn balance_overlays_get_nested() {
        let mut overlays = BalanceOverlays::default();
        overlays.get_or_insert("mobs", "wolf").set("health", 30.0);

        let overlay = overlays.get("mobs", "wolf").unwrap();
        assert_eq!(overlay.get_f64("health"), Some(30.0));

        assert!(overlays.get("mobs", "dragon").is_none());
        assert!(overlays.get("items", "wolf").is_none());
    }

    #[test]
    fn load_balance_overlays_from_toml() {
        let dir = tempfile::tempdir().unwrap();
        let content = r#"
[wolf]
health = 30.0
damage_max = 8.0

[inferno_drake]
health = 8000.0
"#;
        std::fs::write(dir.path().join("mobs.toml"), content).unwrap();

        let overlays = load_balance_overlays(dir.path()).unwrap();
        let wolf = overlays.get("mobs", "wolf").unwrap();
        assert_eq!(wolf.get_f64("health"), Some(30.0));
        assert_eq!(wolf.get_f64("damage_max"), Some(8.0));

        let drake = overlays.get("mobs", "inferno_drake").unwrap();
        assert_eq!(drake.get_f64("health"), Some(8000.0));
    }

    #[test]
    fn load_balance_overlays_nonexistent_dir_returns_empty() {
        let overlays = load_balance_overlays(Path::new("/nonexistent/dir")).unwrap();
        assert!(overlays.layers.is_empty());
    }
}
