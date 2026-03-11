//! Asset definitions for DJ Engine.
//!
//! Provides data structures for sprite metadata, palettes, and part libraries.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Definition for a single composited sprite part loaded from JSON.
#[derive(Serialize, Deserialize, Clone, Debug, Asset, TypePath)]
pub struct SpritePartDefinition {
    /// Identifier used in code (e.g., "body", "head")
    pub part_name: String,
    /// Relative path to PNG sprite file
    pub sprite_file: String,
    /// Full PNG dimensions in pixels
    pub sprite_size: IVec2,
    /// Position offset in the assembled sprite
    #[serde(default)]
    pub original_offset: IVec2,
    /// Z-order (0 = back, higher = front)
    #[serde(default)]
    pub layer_index: u32,
    /// Rotation/scale center point
    pub pivot: Vec2,
    /// Optional bounding box of actual drawn content
    #[serde(default)]
    pub trim_rect: Option<URect>,
}

/// Definition for a color palette loaded from JSON.
#[derive(Serialize, Deserialize, Clone, Debug, Asset, TypePath)]
pub struct PaletteDefinition {
    /// Palette name identifier
    pub palette_name: String,
    /// Color entries (index → RGB)
    pub colors: Vec<ColorEntry>,
}

/// A single color entry in a palette.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ColorEntry {
    /// Palette index (0–255)
    pub index: u32,
    /// Red component (0–255)
    pub r: u8,
    /// Green component (0–255)
    pub g: u8,
    /// Blue component (0–255)
    pub b: u8,
}

impl ColorEntry {
    /// Converts to RGBA bytes (alpha always 255).
    pub fn to_rgba(&self) -> [u8; 4] {
        [self.r, self.g, self.b, 255]
    }

    /// Converts to Bevy Color.
    pub fn to_color(&self) -> Color {
        Color::srgba_u8(self.r, self.g, self.b, 255)
    }
}

/// Manifest listing all sprite parts to load.
#[derive(Serialize, Deserialize, Clone, Debug, Asset, TypePath)]
pub struct SpritePartsManifest {
    /// List of parts to load
    pub parts: Vec<PartEntry>,
}

/// Entry in the parts manifest.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PartEntry {
    /// Part name identifier
    pub part_name: String,
    /// Subdirectory containing the part
    pub directory: String,
    /// Filename of the metadata JSON
    pub metadata_file: String,
}

/// Resource storing all loaded sprite parts.
#[derive(Resource, Default)]
pub struct SpritePartLibrary {
    /// Map of part name to (definition, image handle)
    pub parts: HashMap<String, (SpritePartDefinition, Handle<Image>)>,
}

impl SpritePartLibrary {
    /// Creates a new empty library.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a part to the library.
    pub fn insert(&mut self, name: String, definition: SpritePartDefinition, image: Handle<Image>) {
        self.parts.insert(name, (definition, image));
    }

    /// Gets a part by name.
    pub fn get(&self, name: &str) -> Option<&(SpritePartDefinition, Handle<Image>)> {
        self.parts.get(name)
    }

    /// Returns the number of loaded parts.
    pub fn len(&self) -> usize {
        self.parts.len()
    }

    /// Returns true if no parts are loaded.
    pub fn is_empty(&self) -> bool {
        self.parts.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_entry_to_rgba() {
        let c = ColorEntry {
            index: 0,
            r: 255,
            g: 128,
            b: 0,
        };
        assert_eq!(c.to_rgba(), [255, 128, 0, 255]);
    }

    #[test]
    fn test_color_entry_to_rgba_black() {
        let c = ColorEntry {
            index: 0,
            r: 0,
            g: 0,
            b: 0,
        };
        assert_eq!(c.to_rgba(), [0, 0, 0, 255]);
    }

    #[test]
    fn test_color_entry_alpha_always_255() {
        let c = ColorEntry {
            index: 5,
            r: 10,
            g: 20,
            b: 30,
        };
        assert_eq!(c.to_rgba()[3], 255);
    }

    #[test]
    fn test_palette_definition_serde() {
        let palette = PaletteDefinition {
            palette_name: "test_palette".into(),
            colors: vec![
                ColorEntry {
                    index: 0,
                    r: 255,
                    g: 0,
                    b: 0,
                },
                ColorEntry {
                    index: 1,
                    r: 0,
                    g: 255,
                    b: 0,
                },
            ],
        };
        let json = serde_json::to_string(&palette).unwrap();
        let decoded: PaletteDefinition = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.palette_name, "test_palette");
        assert_eq!(decoded.colors.len(), 2);
        assert_eq!(decoded.colors[0].r, 255);
    }

    #[test]
    fn test_part_entry_serde() {
        let entry = PartEntry {
            part_name: "body".into(),
            directory: "sprites/body".into(),
            metadata_file: "body.json".into(),
        };
        let json = serde_json::to_string(&entry).unwrap();
        let decoded: PartEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.part_name, "body");
        assert_eq!(decoded.directory, "sprites/body");
    }

    #[test]
    fn test_part_library_empty() {
        let lib = SpritePartLibrary::new();
        assert!(lib.is_empty());
        assert_eq!(lib.len(), 0);
    }

    #[test]
    fn test_part_library_insert_get() {
        let mut lib = SpritePartLibrary::new();
        let def = SpritePartDefinition {
            part_name: "body".into(),
            sprite_file: "body.png".into(),
            sprite_size: IVec2::new(64, 64),
            original_offset: IVec2::ZERO,
            layer_index: 0,
            pivot: Vec2::new(0.5, 0.5),
            trim_rect: None,
        };
        lib.insert("body".into(), def, Handle::default());
        assert_eq!(lib.len(), 1);
        assert!(!lib.is_empty());
        assert!(lib.get("body").is_some());
        assert!(lib.get("head").is_none());
    }
}
