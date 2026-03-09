use crate::state::GameState;
use bevy::prelude::*;

mod minimap;
mod tracker;

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameState::Overworld),
            (tracker::setup_tracker, minimap::setup_minimap),
        )
        .add_systems(
            Update,
            (
                tracker::update_tracker,
                tracker::update_objective_markers,
                minimap::update_minimap_and_waypoint,
            )
                .run_if(in_state(GameState::Overworld)),
        )
        .add_systems(OnExit(GameState::Overworld), teardown_hud);
    }
}

// pub struct HudEntity; // Tag for cleanup if needed, though we spawned nodes directly.
// Wait, I didn't tag the UI nodes with HudEntity in the sub-modules.
// I should rely on DespawnRecursive for the root nodes.
// I'll update the setup functions to return Entity or tag them inside.
// Or just query the Components I defined (ObjectiveText parent, MinimapRoot, WaypointArrow)

fn teardown_hud(
    mut commands: Commands,
    q_tracker: Query<Entity, With<tracker::TrackerRoot>>,
    q_minimap: Query<Entity, With<minimap::MinimapRoot>>,
    q_arrow: Query<Entity, With<minimap::WaypointArrow>>,
) {
    for e in &q_tracker {
        commands.entity(e).despawn();
    }
    for e in &q_minimap {
        commands.entity(e).despawn();
    }
    for e in &q_arrow {
        commands.entity(e).despawn();
    }
}
