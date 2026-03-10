//! Asset system for DJ Engine.
//!
//! Provides loaders for sprite metadata, palettes, and game assets.

use bevy::prelude::*;

pub mod definitions;
pub mod loaders;

pub use definitions::{
    ColorEntry, HamsterPartDefinition, HamsterPartLibrary, HamsterPartsManifest, PaletteDefinition,
    PartEntry,
};
pub use loaders::{HamsterPartLoader, PaletteLoader};

/// Asset plugin that registers custom asset loaders and resources.
pub struct DJAssetPlugin;

impl Plugin for DJAssetPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HamsterPartLibrary>()
            .init_asset::<HamsterPartDefinition>()
            .init_asset::<PaletteDefinition>()
            .register_asset_loader(HamsterPartLoader)
            .register_asset_loader(PaletteLoader);
    }
}
