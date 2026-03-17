//! Asset system for DJ Engine.
//!
//! Provides loaders for sprite metadata, palettes, and game assets.

use bevy::prelude::*;

pub mod definitions;
pub mod loaders;

pub use definitions::{
    ColorEntry, PaletteDefinition, PartEntry, SpritePartDefinition, SpritePartLibrary,
    SpritePartsManifest,
};
pub use loaders::{PaletteLoader, SpritePartLoader};

/// Asset plugin that registers custom asset loaders and resources.
pub struct DJAssetPlugin;

impl Plugin for DJAssetPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SpritePartLibrary>()
            .init_asset::<SpritePartDefinition>()
            .init_asset::<PaletteDefinition>()
            .register_asset_loader(SpritePartLoader)
            .register_asset_loader(PaletteLoader);

        use crate::contracts::{AppContractExt, ContractEntry, PluginContract};
        app.register_contract(PluginContract {
            name: "DJAssetPlugin".into(),
            description: "Sprite and palette asset loaders".into(),
            resources: vec![ContractEntry::of::<SpritePartLibrary>(
                "Loaded sprite part assets",
            )],
            components: vec![],
            events: vec![],
            system_sets: vec![],
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asset_plugin_registers_sprite_part_types() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(AssetPlugin::default());
        app.add_plugins(DJAssetPlugin);

        assert!(app.world().contains_resource::<SpritePartLibrary>());
        assert!(app
            .world()
            .contains_resource::<Assets<SpritePartDefinition>>());
        assert!(app.world().contains_resource::<Assets<PaletteDefinition>>());
    }
}
