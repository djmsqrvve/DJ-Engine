//! Interaction system for DJ Engine.
//!
//! Detects when the player is near interactable entities and fires
//! [`InteractionEvent`] when they press Confirm. Games react to these
//! events to trigger dialogue, open doors, loot chests, etc.

use bevy::prelude::*;

use crate::collision::SpatialHash;
use crate::data::components::{InteractivityComponent, NpcComponent, TriggerType};
use crate::input::{ActionState, InputAction};

/// The maximum distance (in world units) at which an entity can be interacted with.
pub const DEFAULT_INTERACTION_RANGE: f32 = 48.0;

/// Resource tracking the closest interactable entity to the player.
#[derive(Resource, Default, Debug, Clone)]
pub struct InteractionTarget {
    /// The entity that can be interacted with, if any.
    pub entity: Option<Entity>,
    /// Distance to the target.
    pub distance: f32,
    /// The trigger type of the target.
    pub trigger_type: Option<TriggerType>,
    /// NPC display name (if applicable).
    pub display_name: Option<String>,
}

/// Marker component for the entity that receives interaction checks (the player).
#[derive(Component, Debug)]
pub struct InteractionSource;

/// Fired when the player interacts with an entity.
#[derive(Message, Debug, Clone, PartialEq)]
pub struct InteractionEvent {
    /// The entity being interacted with.
    pub target: Entity,
    /// What kind of interaction trigger this is.
    pub trigger_type: TriggerType,
    /// The trigger ID from InteractivityComponent.
    pub trigger_id: String,
    /// The NPC's dialogue_set_id (if this is an NPC interaction).
    pub dialogue_set_id: Option<String>,
    /// The on_interact script path (if configured).
    pub on_interact_script: Option<String>,
}

/// Plugin that provides the interaction system.
pub struct InteractionPlugin;

impl Plugin for InteractionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InteractionTarget>()
            .add_message::<InteractionEvent>()
            .add_systems(
                Update,
                (find_nearest_interactable, handle_interaction_input).chain(),
            );

        use crate::contracts::{AppContractExt, ContractEntry, PluginContract};
        app.register_contract(PluginContract {
            name: "InteractionPlugin".into(),
            description: "Proximity-based NPC/object interaction".into(),
            resources: vec![ContractEntry::of::<InteractionTarget>(
                "Current interaction target",
            )],
            components: vec![ContractEntry::of::<InteractionSource>(
                "Marks the player as interaction source",
            )],
            events: vec![ContractEntry::of::<InteractionEvent>(
                "Fired on Confirm near interactable",
            )],
            system_sets: vec![],
        });
    }
}

/// Each frame, find the closest interactable entity within range of the player.
fn find_nearest_interactable(
    mut target: ResMut<InteractionTarget>,
    spatial_hash: Res<SpatialHash>,
    source_query: Query<&Transform, With<InteractionSource>>,
    interactable_query: Query<(
        Entity,
        &Transform,
        &InteractivityComponent,
        Option<&NpcComponent>,
    )>,
) {
    let Ok(player_transform) = source_query.single() else {
        *target = InteractionTarget::default();
        return;
    };

    let player_pos = player_transform.translation.truncate();
    let neighbors = spatial_hash.query_neighbors(player_pos);

    let mut best: Option<(Entity, f32, &InteractivityComponent, Option<&NpcComponent>)> = None;

    for neighbor_entity in &neighbors {
        if let Ok((entity, transform, interactivity, npc)) =
            interactable_query.get(*neighbor_entity)
        {
            if interactivity.trigger_type == TriggerType::None {
                continue;
            }

            let dist = player_pos.distance(transform.translation.truncate());
            if dist > DEFAULT_INTERACTION_RANGE {
                continue;
            }

            if best.is_none() || dist < best.unwrap().1 {
                best = Some((entity, dist, interactivity, npc));
            }
        }
    }

    if let Some((entity, distance, interactivity, npc)) = best {
        *target = InteractionTarget {
            entity: Some(entity),
            distance,
            trigger_type: Some(interactivity.trigger_type),
            display_name: npc.and_then(|n| n.display_name.get("en")).cloned(),
        };
    } else {
        *target = InteractionTarget::default();
    }
}

/// When the player presses Confirm and there's an interaction target, fire the event.
fn handle_interaction_input(
    actions: Res<ActionState>,
    target: Res<InteractionTarget>,
    mut events: MessageWriter<InteractionEvent>,
    interactable_query: Query<(&InteractivityComponent, Option<&NpcComponent>)>,
) {
    if !actions.just_pressed(InputAction::Confirm) {
        return;
    }

    let Some(target_entity) = target.entity else {
        return;
    };

    let Ok((interactivity, npc)) = interactable_query.get(target_entity) else {
        return;
    };

    events.write(InteractionEvent {
        target: target_entity,
        trigger_type: interactivity.trigger_type,
        trigger_id: interactivity.trigger_id.clone(),
        dialogue_set_id: npc.map(|n| n.dialogue_set_id.clone()),
        on_interact_script: interactivity.events.on_interact.clone(),
    });

    info!(
        "Interaction: {:?} trigger_id='{}' dialogue='{}'",
        interactivity.trigger_type,
        interactivity.trigger_id,
        npc.map(|n| n.dialogue_set_id.as_str()).unwrap_or("none")
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interaction_target_default_is_none() {
        let target = InteractionTarget::default();
        assert!(target.entity.is_none());
        assert!(target.trigger_type.is_none());
    }

    #[test]
    fn test_interaction_event_carries_npc_data() {
        let event = InteractionEvent {
            target: Entity::from_bits(1),
            trigger_type: TriggerType::Npc,
            trigger_id: "npc_guard".into(),
            dialogue_set_id: Some("guard_greeting".into()),
            on_interact_script: None,
        };
        assert_eq!(event.trigger_type, TriggerType::Npc);
        assert_eq!(event.dialogue_set_id.as_deref(), Some("guard_greeting"));
    }

    #[test]
    fn test_interaction_event_carries_door_data() {
        let event = InteractionEvent {
            target: Entity::from_bits(2),
            trigger_type: TriggerType::Door,
            trigger_id: "tavern_door".into(),
            dialogue_set_id: None,
            on_interact_script: Some("scripts/open_door.lua".into()),
        };
        assert_eq!(event.trigger_type, TriggerType::Door);
        assert!(event.dialogue_set_id.is_none());
        assert_eq!(
            event.on_interact_script.as_deref(),
            Some("scripts/open_door.lua")
        );
    }

    #[test]
    fn test_find_nearest_ignores_none_trigger_type() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<InteractionTarget>();
        app.insert_resource(SpatialHash::new(64.0));
        app.add_systems(Update, find_nearest_interactable);

        // Spawn player
        app.world_mut()
            .spawn((Transform::from_xyz(0.0, 0.0, 0.0), InteractionSource));

        // Spawn entity with TriggerType::None — should be ignored
        app.world_mut().spawn((
            Transform::from_xyz(10.0, 0.0, 0.0),
            InteractivityComponent {
                trigger_type: TriggerType::None,
                ..default()
            },
        ));

        // Rebuild spatial hash manually (normally done by collision plugin)
        {
            let mut hash = app.world_mut().resource_mut::<SpatialHash>();
            hash.rebuild(std::iter::once((Entity::from_bits(1), Vec2::new(10.0, 0.0))));
        }

        app.update();

        let target = app.world().resource::<InteractionTarget>();
        assert!(target.entity.is_none());
    }
}
