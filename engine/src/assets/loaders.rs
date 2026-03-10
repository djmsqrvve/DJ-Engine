//! Asset loaders for DJ Engine JSON-based assets.

use bevy::asset::{io::Reader, AssetLoader, LoadContext};
use bevy::reflect::TypePath;

use super::definitions::{HamsterPartDefinition, PaletteDefinition};

/// Loads [`HamsterPartDefinition`] from JSON files.
#[derive(Default, TypePath)]
pub struct HamsterPartLoader;

impl AssetLoader for HamsterPartLoader {
    type Asset = HamsterPartDefinition;
    type Settings = ();
    type Error = std::io::Error;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &(),
        _load_context: &mut LoadContext<'_>,
    ) -> Result<HamsterPartDefinition, Self::Error> {
        let mut bytes = Vec::new();
        bevy::asset::AsyncReadExt::read_to_end(reader, &mut bytes).await?;
        serde_json::from_slice(&bytes)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }

    fn extensions(&self) -> &[&str] {
        &["hamsterpart.json"]
    }
}

/// Loads [`PaletteDefinition`] from JSON files.
#[derive(Default, TypePath)]
pub struct PaletteLoader;

impl AssetLoader for PaletteLoader {
    type Asset = PaletteDefinition;
    type Settings = ();
    type Error = std::io::Error;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &(),
        _load_context: &mut LoadContext<'_>,
    ) -> Result<PaletteDefinition, Self::Error> {
        let mut bytes = Vec::new();
        bevy::asset::AsyncReadExt::read_to_end(reader, &mut bytes).await?;
        serde_json::from_slice(&bytes)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }

    fn extensions(&self) -> &[&str] {
        &["palette.json"]
    }
}

#[cfg(test)]
mod tests {
    use super::super::definitions::*;

    #[test]
    fn test_hamster_part_definition_loads_from_json_bytes() {
        let json = r#"{
            "part_name": "body",
            "sprite_file": "body.png",
            "sprite_size": [64, 64],
            "pivot": [0.5, 0.5]
        }"#;
        let def: HamsterPartDefinition = serde_json::from_str(json).unwrap();
        assert_eq!(def.part_name, "body");
        assert_eq!(def.sprite_file, "body.png");
        assert_eq!(def.sprite_size.x, 64);
        assert_eq!(def.pivot.x, 0.5);
        assert_eq!(def.layer_index, 0); // default
        assert!(def.trim_rect.is_none()); // default
    }

    #[test]
    fn test_palette_definition_loads_from_json_bytes() {
        let json = r#"{
            "palette_name": "cursed",
            "colors": [
                {"index": 0, "r": 255, "g": 0, "b": 0},
                {"index": 1, "r": 0, "g": 255, "b": 0}
            ]
        }"#;
        let def: PaletteDefinition = serde_json::from_str(json).unwrap();
        assert_eq!(def.palette_name, "cursed");
        assert_eq!(def.colors.len(), 2);
        assert_eq!(def.colors[0].index, 0);
        assert_eq!(def.colors[1].g, 255);
    }

    #[test]
    fn test_hamster_part_definition_missing_optional_fields_defaults() {
        // original_offset, layer_index, trim_rect are #[serde(default)]
        let json = r#"{
            "part_name": "head",
            "sprite_file": "head.png",
            "sprite_size": [32, 32],
            "pivot": [0.5, 1.0]
        }"#;
        let def: HamsterPartDefinition = serde_json::from_str(json).unwrap();
        assert_eq!(def.original_offset.x, 0);
        assert_eq!(def.layer_index, 0);
        assert!(def.trim_rect.is_none());
    }

    #[test]
    fn test_palette_definition_empty_colors() {
        let json = r#"{"palette_name": "empty", "colors": []}"#;
        let def: PaletteDefinition = serde_json::from_str(json).unwrap();
        assert!(def.colors.is_empty());
    }
}
