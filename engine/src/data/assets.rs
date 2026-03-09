//! Asset index and prefab definitions.
//!
//! The asset index catalogs all game assets (sprites, audio, scripts, etc.)
//! for quick lookup and validation.

use serde::{Deserialize, Serialize};

use super::scene::Entity;

/// Audio asset type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AudioType {
    /// Background music
    #[default]
    Music,
    /// Sound effect
    Sfx,
    /// Voice line
    Voice,
}

/// A sprite asset reference.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct SpriteAsset {
    /// Unique asset identifier
    pub id: String,
    /// File path relative to assets folder
    pub path: String,
    /// Tags for categorization/filtering
    #[serde(default)]
    pub tags: Vec<String>,
}

impl SpriteAsset {
    /// Create a new sprite asset.
    pub fn new(id: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            path: path.into(),
            tags: Vec::new(),
        }
    }

    /// Add tags to the sprite.
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }
}

/// An audio asset reference.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct AudioAsset {
    /// Unique asset identifier
    pub id: String,
    /// File path relative to assets folder
    pub path: String,
    /// Audio type (music, sfx, voice)
    #[serde(default)]
    pub audio_type: AudioType,
}

impl AudioAsset {
    /// Create a new audio asset.
    pub fn new(id: impl Into<String>, path: impl Into<String>, audio_type: AudioType) -> Self {
        Self {
            id: id.into(),
            path: path.into(),
            audio_type,
        }
    }
}

/// A script asset reference.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ScriptAsset {
    /// Unique asset identifier
    pub id: String,
    /// File path relative to scripts folder
    pub path: String,
}

impl ScriptAsset {
    /// Create a new script asset.
    pub fn new(id: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            path: path.into(),
        }
    }
}

/// A prefab (reusable entity template).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Prefab {
    /// Unique prefab identifier
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// The entity template
    pub entity: Entity,
}

impl Prefab {
    /// Create a new prefab from an entity.
    pub fn new(id: impl Into<String>, name: impl Into<String>, entity: Entity) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            entity,
        }
    }
}

/// A story graph asset reference.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct StoryGraphAsset {
    /// Unique asset identifier
    pub id: String,
    /// File path relative to story_graphs folder
    pub path: String,
}

impl StoryGraphAsset {
    /// Create a new story graph asset.
    pub fn new(id: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            path: path.into(),
        }
    }
}

/// A scene asset reference.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct SceneAsset {
    /// Unique asset identifier
    pub id: String,
    /// File path relative to scenes folder
    pub path: String,
}

impl SceneAsset {
    /// Create a new scene asset.
    pub fn new(id: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            path: path.into(),
        }
    }
}

/// Index of all game assets.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct AssetIndex {
    /// Sprite assets
    #[serde(default)]
    pub sprites: Vec<SpriteAsset>,
    /// Audio assets
    #[serde(default)]
    pub audio: Vec<AudioAsset>,
    /// Script assets
    #[serde(default)]
    pub scripts: Vec<ScriptAsset>,
    /// Prefab definitions
    #[serde(default)]
    pub prefabs: Vec<Prefab>,
    /// Story graph assets
    #[serde(default)]
    pub story_graphs: Vec<StoryGraphAsset>,
    /// Scene assets
    #[serde(default)]
    pub scenes: Vec<SceneAsset>,
}

impl AssetIndex {
    /// Create a new empty asset index.
    pub fn new() -> Self {
        Self::default()
    }

    /// Find a sprite by ID.
    pub fn find_sprite(&self, id: &str) -> Option<&SpriteAsset> {
        self.sprites.iter().find(|s| s.id == id)
    }

    /// Find an audio asset by ID.
    pub fn find_audio(&self, id: &str) -> Option<&AudioAsset> {
        self.audio.iter().find(|a| a.id == id)
    }

    /// Find a script by ID.
    pub fn find_script(&self, id: &str) -> Option<&ScriptAsset> {
        self.scripts.iter().find(|s| s.id == id)
    }

    /// Find a prefab by ID.
    pub fn find_prefab(&self, id: &str) -> Option<&Prefab> {
        self.prefabs.iter().find(|p| p.id == id)
    }

    /// Find a story graph asset by ID.
    pub fn find_story_graph(&self, id: &str) -> Option<&StoryGraphAsset> {
        self.story_graphs.iter().find(|s| s.id == id)
    }

    /// Find a scene asset by ID.
    pub fn find_scene(&self, id: &str) -> Option<&SceneAsset> {
        self.scenes.iter().find(|s| s.id == id)
    }

    /// Get all sprites with a specific tag.
    pub fn sprites_with_tag(&self, tag: &str) -> Vec<&SpriteAsset> {
        self.sprites
            .iter()
            .filter(|s| s.tags.contains(&tag.to_string()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asset_index_serialization() {
        let mut index = AssetIndex::new();
        index.sprites.push(
            SpriteAsset::new("hero", "sprites/hero.png").with_tags(vec!["character".to_string()]),
        );
        index.audio.push(AudioAsset::new(
            "bgm_intro",
            "audio/intro.ogg",
            AudioType::Music,
        ));

        let json = serde_json::to_string_pretty(&index).unwrap();
        let parsed: AssetIndex = serde_json::from_str(&json).unwrap();

        assert_eq!(index.sprites.len(), parsed.sprites.len());
        assert!(parsed.find_sprite("hero").is_some());
    }
}
