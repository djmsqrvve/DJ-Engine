//! DJ Engine Core - Master plugin bundle
//!
//! Provides `DJEnginePlugin` which bundles all engine systems for easy integration.

use bevy::prelude::*;

use crate::animation::DJAnimationPlugin;
use crate::assets::DJAssetPlugin;
use crate::audio::DJAudioPlugin;
use crate::collision::CollisionPlugin;
use crate::diagnostics::DiagnosticsPlugin;
use crate::input::DJInputPlugin;
use crate::rendering::RenderingPlugin;
use crate::scene::DJScenePlugin;
use crate::scripting::DJScriptingPlugin;
use crate::story_graph::StoryGraphPlugin;
use crate::types::EngineConfig;

/// Master plugin that bundles all DJ Engine systems.
///
/// # Example
/// ```ignore
/// use dj_engine::prelude::*;
///
/// App::new()
///     .add_plugins(DefaultPlugins)
///     .add_plugins(DJEnginePlugin::default())
///     .run();
/// ```
pub struct DJEnginePlugin {
    /// Engine configuration
    pub config: EngineConfig,
    /// Whether to include diagnostics overlay
    pub with_diagnostics: bool,
}

impl Default for DJEnginePlugin {
    fn default() -> Self {
        Self {
            config: EngineConfig::default(),
            with_diagnostics: true,
        }
    }
}

impl Plugin for DJEnginePlugin {
    fn build(&self, app: &mut App) {
        // Insert engine configuration
        app.insert_resource(self.config.clone());

        // Add core engine plugins
        app.add_plugins(RenderingPlugin);
        app.add_plugins(DJAnimationPlugin);
        app.add_plugins(DJAssetPlugin);
        app.add_plugins(DJAudioPlugin);
        app.add_plugins(CollisionPlugin);
        app.add_plugins(DJInputPlugin);
        app.add_plugins(DJScenePlugin);
        app.add_plugins(StoryGraphPlugin);
        app.add_plugins(DJScriptingPlugin);
        app.add_plugins(crate::midi::MidiPlugin);
        app.add_plugins(crate::save::SavePlugin);
        app.add_plugins(crate::data::DataPlugin);
        app.add_plugins(crate::zone::ZonePlugin);
        app.add_plugins(crate::interaction::InteractionPlugin);
        app.add_plugins(crate::quest::QuestPlugin);
        app.add_plugins(crate::combat::CombatPlugin);
        app.add_plugins(crate::inventory::InventoryPlugin);
        app.add_plugins(crate::status::StatusPlugin);
        app.add_plugins(crate::loot::LootPlugin);
        app.add_plugins(crate::ability::AbilityPlugin);
        app.add_plugins(crate::economy::EconomyPlugin);

        // Conditionally add diagnostics
        if self.with_diagnostics {
            app.add_plugins(DiagnosticsPlugin);
        }

        info!(
            "DJ Engine v{} initialized ({}x{}, debug={})",
            crate::engine_version(),
            self.config.internal_width,
            self.config.internal_height,
            self.config.debug_mode
        );
    }
}

impl DJEnginePlugin {
    /// Create a new engine plugin with custom configuration.
    pub fn new(config: EngineConfig) -> Self {
        Self {
            config,
            with_diagnostics: true,
        }
    }

    /// Disable the diagnostics overlay.
    pub fn without_diagnostics(mut self) -> Self {
        self.with_diagnostics = false;
        self
    }
}
