use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
// Use default plugins for World Inspector for MVP, but user wants "Engine around it"
// which implies we might want EguiPlugin and custom layout later.
// For now, let's start with WorldInspectorPlugin to get immediate value,
// and then layer a custom UI if needed.
// Actually, `WorldInspectorPlugin` is great.
// We should also check if we want to add `ResourceInspectorPlugin`.

pub struct InspectorPlugin;

impl Plugin for InspectorPlugin {
    fn build(&self, app: &mut App) {
        // Toggle via Key? Default specific key?
        // WorldInspectorPlugin defaults to enabled.
        // Let's add it.
        app.add_plugins(WorldInspectorPlugin::new());

        info!("Inspector Plugin initialized");
    }
}
