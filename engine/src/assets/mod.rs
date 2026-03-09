//! Asset system for DJ Engine.
//!
//! Provides loaders for sprite metadata, palettes, and game assets.

use bevy::prelude::*;

pub mod definitions;

pub use definitions::{
    ColorEntry, HamsterPartDefinition, HamsterPartLibrary, HamsterPartsManifest, PaletteDefinition,
    PartEntry,
};

/// Asset plugin that registers custom asset loaders and resources.
pub struct DJAssetPlugin;

impl Plugin for DJAssetPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HamsterPartLibrary>();
        // TODO: Register HamsterPartLoader
        // TODO: Register PaletteLoader
    }
}
