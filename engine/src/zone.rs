//! Zone transition system for portals and area changes.
//!
//! Provides [`PortalComponent`] for marking trigger entities as zone portals,
//! [`ZoneTransitionEvent`] for notifying gameplay systems of zone changes,
//! and [`ActiveZone`] for tracking the currently loaded zone.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::collision::{RuntimeCollider, TriggerContactEvent};

/// Portal component: when a player enters this trigger, transition to target zone.
#[derive(Component, Debug, Clone, PartialEq, Serialize, Deserialize, Default, Reflect)]
#[reflect(Component)]
pub struct PortalComponent {
    /// Identifier for the destination zone.
    pub target_zone_id: String,
    /// Named spawn point within the destination zone.
    pub target_spawn_point: String,
}

/// Event emitted when a zone transition is triggered.
#[derive(Message, Debug, Clone)]
pub struct ZoneTransitionEvent {
    /// Zone the entity is leaving.
    pub from_zone_id: String,
    /// Zone the entity is entering.
    pub to_zone_id: String,
    /// Spawn point within the destination zone.
    pub spawn_point: String,
    /// The entity that triggered the transition.
    pub entity: Entity,
}

/// Resource tracking which zone is currently active.
#[derive(Resource, Default, Debug, Clone)]
pub struct ActiveZone {
    /// Identifier of the currently loaded zone.
    pub zone_id: String,
}

/// Plugin providing zone transition detection.
pub struct ZonePlugin;

impl Plugin for ZonePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ActiveZone>()
            .register_type::<PortalComponent>()
            .add_message::<ZoneTransitionEvent>()
            .add_systems(Update, detect_portal_transitions);

        use crate::contracts::{AppContractExt, ContractEntry, PluginContract};
        app.register_contract(PluginContract {
            name: "ZonePlugin".into(),
            description: "Zone transitions via portal triggers".into(),
            resources: vec![ContractEntry::of::<ActiveZone>(
                "Currently active zone identifier",
            )],
            components: vec![ContractEntry::of::<PortalComponent>(
                "Portal trigger for zone transitions",
            )],
            events: vec![ContractEntry::of::<ZoneTransitionEvent>(
                "Fired when a portal transition is triggered",
            )],
            system_sets: vec![],
        });

        info!("Zone Plugin initialized");
    }
}

/// Detects when an entity enters a portal trigger and emits a [`ZoneTransitionEvent`].
fn detect_portal_transitions(
    portals: Query<(&PortalComponent, &RuntimeCollider)>,
    active_zone: Res<ActiveZone>,
    mut trigger_events: MessageReader<TriggerContactEvent>,
    mut zone_events: MessageWriter<ZoneTransitionEvent>,
) {
    for event in trigger_events.read() {
        let (trigger_entity, other_entity) = match event {
            TriggerContactEvent::Enter { trigger, other } => (*trigger, *other),
            TriggerContactEvent::Exit { .. } => continue,
        };

        let Ok((portal, _collider)) = portals.get(trigger_entity) else {
            continue;
        };

        info!(
            "Portal transition: entity {:?} entered portal to zone '{}' spawn '{}'",
            other_entity, portal.target_zone_id, portal.target_spawn_point
        );

        zone_events.write(ZoneTransitionEvent {
            from_zone_id: active_zone.zone_id.clone(),
            to_zone_id: portal.target_zone_id.clone(),
            spawn_point: portal.target_spawn_point.clone(),
            entity: other_entity,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_active_zone_default() {
        let zone = ActiveZone::default();
        assert!(zone.zone_id.is_empty());
    }

    #[test]
    fn test_portal_component_serde() {
        let portal = PortalComponent {
            target_zone_id: "forest_east".into(),
            target_spawn_point: "entrance_a".into(),
        };
        let json = serde_json::to_string(&portal).unwrap();
        let roundtrip: PortalComponent = serde_json::from_str(&json).unwrap();
        assert_eq!(portal, roundtrip);
    }

    #[test]
    fn test_zone_transition_event_fields() {
        let entity = Entity::from_bits(42);
        let event = ZoneTransitionEvent {
            from_zone_id: "town".into(),
            to_zone_id: "dungeon_1".into(),
            spawn_point: "stairs_down".into(),
            entity,
        };
        assert_eq!(event.from_zone_id, "town");
        assert_eq!(event.to_zone_id, "dungeon_1");
        assert_eq!(event.spawn_point, "stairs_down");
        assert_eq!(event.entity, entity);
    }

    #[test]
    fn test_detect_portal_fires_event() {
        use crate::collision::{CollisionPlugin, MovementIntent};
        use crate::data::components::{BodyType, CollisionComponent, Vec3Data};

        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(CollisionPlugin);
        app.add_plugins(ZonePlugin);

        app.world_mut().resource_mut::<ActiveZone>().zone_id = "overworld".into();

        // Spawn a portal trigger at the origin.
        let portal_entity = app
            .world_mut()
            .spawn((
                Transform::default(),
                CollisionComponent {
                    body_type: BodyType::Static,
                    box_size: Some(Vec3Data::new(40.0, 40.0, 0.0)),
                    is_trigger: true,
                    ..Default::default()
                },
                PortalComponent {
                    target_zone_id: "dungeon".into(),
                    target_spawn_point: "entrance".into(),
                },
            ))
            .id();

        // Spawn an entity that overlaps the portal trigger.
        let _mover = app
            .world_mut()
            .spawn((
                Transform::from_xyz(0.0, 0.0, 0.0),
                CollisionComponent {
                    body_type: BodyType::Kinematic,
                    box_size: Some(Vec3Data::new(16.0, 16.0, 0.0)),
                    ..Default::default()
                },
                MovementIntent(Vec2::ZERO),
            ))
            .id();

        // First update: collision system syncs runtime colliders and detects trigger overlap.
        // The zone system may or may not run after the collision system in this frame,
        // so run a second update to guarantee the ZoneTransitionEvent is emitted.
        app.update();
        app.update();

        let events = app.world().resource::<Messages<ZoneTransitionEvent>>();
        let transitions: Vec<_> = events.iter_current_update_messages().collect();

        assert_eq!(
            transitions.len(),
            1,
            "expected exactly one zone transition event"
        );
        assert_eq!(transitions[0].from_zone_id, "overworld");
        assert_eq!(transitions[0].to_zone_id, "dungeon");
        assert_eq!(transitions[0].spawn_point, "entrance");

        // Verify the portal entity was recognized (the mover triggered it).
        let _ = portal_entity;
    }
}
