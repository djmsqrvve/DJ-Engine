//! Lightweight collision and trigger support for authored scene data.
//!
//! This module keeps the engine dependency-light by providing a small custom
//! collision layer that is sufficient for overworld blockers and trigger zones.

use crate::data::components::{BodyType, CollisionComponent, CollisionShape, Vec3Data};
use bevy::prelude::*;
use std::collections::HashMap;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum CollisionSet {
    SyncData,
    MoveBodies,
    DetectTriggers,
}

/// Movement intent consumed by the collision system each frame.
#[derive(Component, Debug, Clone, Copy, Default, PartialEq, Reflect)]
#[reflect(Component)]
pub struct MovementIntent(pub Vec2);

impl MovementIntent {
    pub fn clear(&mut self) {
        self.0 = Vec2::ZERO;
    }
}

/// Simplified runtime collider shape.
#[derive(Debug, Clone, Copy, PartialEq, Reflect)]
pub enum RuntimeColliderShape {
    Box { half_extents: Vec2 },
    Circle { radius: f32 },
    PolygonBounds { half_extents: Vec2 },
}

impl RuntimeColliderShape {
    fn bounds_half_extents(self) -> Vec2 {
        match self {
            Self::Box { half_extents } | Self::PolygonBounds { half_extents } => half_extents,
            Self::Circle { radius } => Vec2::splat(radius),
        }
    }
}

/// Runtime collider derived from authored `CollisionComponent` data.
#[derive(Component, Debug, Clone, Copy, PartialEq, Reflect)]
#[reflect(Component)]
pub struct RuntimeCollider {
    pub enabled: bool,
    pub body_type: BodyType,
    pub shape: RuntimeColliderShape,
    pub offset: Vec2,
    pub layer_bits: u32,
    pub mask_bits: u32,
    pub is_trigger: bool,
}

impl RuntimeCollider {
    fn center_at(self, translation: Vec3) -> Vec2 {
        translation.truncate() + self.offset
    }

    fn bounds_aabb_at(self, translation: Vec3) -> Aabb {
        Aabb::from_center_half_extents(
            self.center_at(translation),
            self.shape.bounds_half_extents(),
        )
    }
}

/// Current trigger overlaps keyed by entity.
#[derive(Resource, Default, Debug, Clone)]
pub struct TriggerContacts {
    by_entity: HashMap<Entity, Vec<Entity>>,
}

impl TriggerContacts {
    pub fn contacts_for(&self, entity: Entity) -> &[Entity] {
        self.by_entity
            .get(&entity)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }
}

/// Trigger enter/exit event emitted by the collision system.
#[derive(Message, Debug, Clone, Copy, PartialEq, Eq)]
pub enum TriggerContactEvent {
    Enter { trigger: Entity, other: Entity },
    Exit { trigger: Entity, other: Entity },
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Aabb {
    min: Vec2,
    max: Vec2,
}

impl Aabb {
    fn from_center_half_extents(center: Vec2, half_extents: Vec2) -> Self {
        Self {
            min: center - half_extents,
            max: center + half_extents,
        }
    }

    fn overlaps(self, other: Self) -> bool {
        self.min.x < other.max.x
            && self.max.x > other.min.x
            && self.min.y < other.max.y
            && self.max.y > other.min.y
    }

    fn clamp_point(self, point: Vec2) -> Vec2 {
        point.clamp(self.min, self.max)
    }
}

#[derive(Resource, Debug)]
struct CollisionLayers {
    layer_bits: HashMap<String, u32>,
    warned_overflow: bool,
}

impl Default for CollisionLayers {
    fn default() -> Self {
        let mut layer_bits = HashMap::new();
        layer_bits.insert("default".to_string(), 1);
        Self {
            layer_bits,
            warned_overflow: false,
        }
    }
}

impl CollisionLayers {
    fn bit_for(&mut self, layer: &str) -> u32 {
        let key = if layer.trim().is_empty() {
            "default"
        } else {
            layer
        };
        if let Some(bit) = self.layer_bits.get(key) {
            return *bit;
        }

        if self.layer_bits.len() >= 32 {
            if !self.warned_overflow {
                warn!("Collision layer limit reached; falling back to 'default' for extra layers");
                self.warned_overflow = true;
            }
            return 1;
        }

        let bit = 1u32 << self.layer_bits.len();
        self.layer_bits.insert(key.to_string(), bit);
        bit
    }

    fn mask_bits(&mut self, mask: &[String]) -> u32 {
        if mask.is_empty() {
            return self.bit_for("default");
        }

        mask.iter()
            .fold(0u32, |bits, layer| bits | self.bit_for(layer))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct TriggerPairInfo {
    first_is_trigger: bool,
    second_is_trigger: bool,
}

#[derive(Resource, Default)]
struct TriggerPairState {
    pairs: HashMap<(Entity, Entity), TriggerPairInfo>,
}

/// Public collision plugin.
pub struct CollisionPlugin;

impl Plugin for CollisionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CollisionLayers>()
            .init_resource::<TriggerContacts>()
            .init_resource::<TriggerPairState>()
            .register_type::<MovementIntent>()
            .register_type::<RuntimeCollider>()
            .register_type::<RuntimeColliderShape>()
            .add_message::<TriggerContactEvent>()
            .configure_sets(
                Update,
                (
                    CollisionSet::SyncData,
                    CollisionSet::MoveBodies,
                    CollisionSet::DetectTriggers,
                )
                    .chain(),
            )
            .add_systems(
                Update,
                (
                    sync_runtime_colliders.in_set(CollisionSet::SyncData),
                    apply_movement_intents.in_set(CollisionSet::MoveBodies),
                    detect_trigger_contacts.in_set(CollisionSet::DetectTriggers),
                ),
            );

        use crate::contracts::{AppContractExt, ContractEntry, ContractSystemSet, PluginContract};
        app.register_contract(PluginContract {
            name: "CollisionPlugin".into(),
            description: "AABB collision detection and trigger zones".into(),
            resources: vec![ContractEntry::of::<TriggerContacts>(
                "Active trigger overlaps by entity",
            )],
            components: vec![
                ContractEntry::of::<MovementIntent>("Per-frame movement delta"),
                ContractEntry::of::<RuntimeCollider>("Runtime collider shape and config"),
            ],
            events: vec![ContractEntry::of::<TriggerContactEvent>(
                "Trigger enter/exit events",
            )],
            system_sets: vec![
                ContractSystemSet {
                    name: "CollisionSet::SyncData".into(),
                    schedule: "Update".into(),
                },
                ContractSystemSet {
                    name: "CollisionSet::MoveBodies".into(),
                    schedule: "Update".into(),
                },
                ContractSystemSet {
                    name: "CollisionSet::DetectTriggers".into(),
                    schedule: "Update".into(),
                },
            ],
        });
    }
}

#[allow(clippy::type_complexity)]
fn sync_runtime_colliders(
    mut commands: Commands,
    mut layers: ResMut<CollisionLayers>,
    query: Query<
        (Entity, &CollisionComponent),
        Or<(Added<CollisionComponent>, Changed<CollisionComponent>)>,
    >,
) {
    for (entity, component) in &query {
        let runtime_body_type = match component.body_type {
            BodyType::Dynamic => {
                warn!(
                    "Entity {entity:?} uses dynamic collision; treating it as kinematic in the custom collision layer"
                );
                BodyType::Kinematic
            }
            other => other,
        };

        let shape = build_runtime_shape(entity, component);
        let runtime = RuntimeCollider {
            enabled: component.enabled,
            body_type: runtime_body_type,
            shape,
            offset: component.offset.into_vec2(),
            layer_bits: layers.bit_for(&component.layer),
            mask_bits: layers.mask_bits(&component.mask),
            is_trigger: component.is_trigger,
        };

        commands.entity(entity).insert(runtime);
    }
}

fn build_runtime_shape(entity: Entity, component: &CollisionComponent) -> RuntimeColliderShape {
    match component.shape {
        CollisionShape::Box => RuntimeColliderShape::Box {
            half_extents: half_extents_from_size(
                component.box_size.unwrap_or(Vec3Data::new(32.0, 32.0, 0.0)),
            ),
        },
        CollisionShape::Circle => RuntimeColliderShape::Circle {
            radius: component.circle_radius.unwrap_or(16.0).max(0.0),
        },
        CollisionShape::Polygon => {
            warn!(
                "Entity {entity:?} uses polygon collision; falling back to a bounding AABB in the custom collision layer"
            );
            RuntimeColliderShape::PolygonBounds {
                half_extents: polygon_half_extents(component),
            }
        }
    }
}

fn polygon_half_extents(component: &CollisionComponent) -> Vec2 {
    if component.polygon_points.is_empty() {
        return half_extents_from_size(
            component.box_size.unwrap_or(Vec3Data::new(32.0, 32.0, 0.0)),
        );
    }

    let mut min = Vec2::splat(f32::INFINITY);
    let mut max = Vec2::splat(f32::NEG_INFINITY);
    for point in &component.polygon_points {
        let point = point.into_vec2();
        min = min.min(point);
        max = max.max(point);
    }
    ((max - min) * 0.5).max(Vec2::splat(0.5))
}

fn half_extents_from_size(size: Vec3Data) -> Vec2 {
    Vec2::new(size.x.abs().max(1.0) * 0.5, size.y.abs().max(1.0) * 0.5)
}

#[allow(clippy::type_complexity)]
fn apply_movement_intents(
    mut query_set: ParamSet<(
        Query<(
            Entity,
            &mut Transform,
            &RuntimeCollider,
            &mut MovementIntent,
        )>,
        Query<(Entity, &Transform, &RuntimeCollider)>,
    )>,
) {
    let blockers: Vec<(Entity, Vec3, RuntimeCollider)> = query_set
        .p1()
        .iter()
        .map(|(entity, transform, collider)| (entity, transform.translation, *collider))
        .collect();

    for (entity, mut transform, collider, mut intent) in &mut query_set.p0() {
        if !collider.enabled || collider.body_type == BodyType::Static {
            intent.clear();
            continue;
        }

        let delta = intent.0;
        if delta == Vec2::ZERO {
            continue;
        }

        let mut next_translation = transform.translation;
        next_translation.x = resolve_axis(
            entity,
            next_translation,
            delta.x,
            Axis::X,
            *collider,
            &blockers,
        );
        next_translation.y = resolve_axis(
            entity,
            next_translation,
            delta.y,
            Axis::Y,
            *collider,
            &blockers,
        );
        transform.translation = next_translation;
        intent.clear();
    }
}

#[derive(Debug, Clone, Copy)]
enum Axis {
    X,
    Y,
}

fn resolve_axis(
    entity: Entity,
    mut translation: Vec3,
    axis_delta: f32,
    axis: Axis,
    collider: RuntimeCollider,
    blockers: &[(Entity, Vec3, RuntimeCollider)],
) -> f32 {
    match axis {
        Axis::X => translation.x += axis_delta,
        Axis::Y => translation.y += axis_delta,
    }

    if axis_delta == 0.0 || collider.is_trigger {
        return match axis {
            Axis::X => translation.x,
            Axis::Y => translation.y,
        };
    }

    let mut adjusted = translation;
    for (other_entity, other_translation, other_collider) in blockers {
        if *other_entity == entity
            || !other_collider.enabled
            || other_collider.is_trigger
            || !should_collide(collider, *other_collider)
        {
            continue;
        }

        if !collider
            .bounds_aabb_at(adjusted)
            .overlaps(other_collider.bounds_aabb_at(*other_translation))
        {
            continue;
        }

        let mover_half = collider.shape.bounds_half_extents();
        let other_aabb = other_collider.bounds_aabb_at(*other_translation);
        match axis {
            Axis::X if axis_delta > 0.0 => {
                adjusted.x = other_aabb.min.x - mover_half.x - collider.offset.x;
            }
            Axis::X => {
                adjusted.x = other_aabb.max.x + mover_half.x - collider.offset.x;
            }
            Axis::Y if axis_delta > 0.0 => {
                adjusted.y = other_aabb.min.y - mover_half.y - collider.offset.y;
            }
            Axis::Y => {
                adjusted.y = other_aabb.max.y + mover_half.y - collider.offset.y;
            }
        }
    }

    match axis {
        Axis::X => adjusted.x,
        Axis::Y => adjusted.y,
    }
}

fn detect_trigger_contacts(
    query: Query<(Entity, &Transform, &RuntimeCollider)>,
    mut contacts: ResMut<TriggerContacts>,
    mut state: ResMut<TriggerPairState>,
    mut events: MessageWriter<TriggerContactEvent>,
) {
    let colliders: Vec<(Entity, Vec3, RuntimeCollider)> = query
        .iter()
        .map(|(entity, transform, collider)| (entity, transform.translation, *collider))
        .collect();

    let mut current_pairs = HashMap::new();
    let mut by_entity: HashMap<Entity, Vec<Entity>> = HashMap::new();

    for i in 0..colliders.len() {
        for j in (i + 1)..colliders.len() {
            let (entity_a, translation_a, collider_a) = colliders[i];
            let (entity_b, translation_b, collider_b) = colliders[j];

            if !collider_a.enabled
                || !collider_b.enabled
                || !(collider_a.is_trigger || collider_b.is_trigger)
                || !should_collide(collider_a, collider_b)
                || !colliders_overlap(collider_a, translation_a, collider_b, translation_b)
            {
                continue;
            }

            let (pair, info) = canonical_pair(
                entity_a,
                entity_b,
                TriggerPairInfo {
                    first_is_trigger: collider_a.is_trigger,
                    second_is_trigger: collider_b.is_trigger,
                },
            );
            current_pairs.insert(pair, info);
            by_entity.entry(entity_a).or_default().push(entity_b);
            by_entity.entry(entity_b).or_default().push(entity_a);
        }
    }

    for overlaps in by_entity.values_mut() {
        overlaps.sort_by_key(|entity| entity.to_bits());
    }

    for (pair, info) in &current_pairs {
        if !state.pairs.contains_key(pair) {
            emit_pair_events(*pair, *info, true, &mut events);
        }
    }

    for (pair, info) in &state.pairs {
        if !current_pairs.contains_key(pair) {
            emit_pair_events(*pair, *info, false, &mut events);
        }
    }

    state.pairs = current_pairs;
    contacts.by_entity = by_entity;
}

fn emit_pair_events(
    pair: (Entity, Entity),
    info: TriggerPairInfo,
    entering: bool,
    events: &mut MessageWriter<TriggerContactEvent>,
) {
    if info.first_is_trigger {
        let event = if entering {
            TriggerContactEvent::Enter {
                trigger: pair.0,
                other: pair.1,
            }
        } else {
            TriggerContactEvent::Exit {
                trigger: pair.0,
                other: pair.1,
            }
        };
        events.write(event);
    }

    if info.second_is_trigger {
        let event = if entering {
            TriggerContactEvent::Enter {
                trigger: pair.1,
                other: pair.0,
            }
        } else {
            TriggerContactEvent::Exit {
                trigger: pair.1,
                other: pair.0,
            }
        };
        events.write(event);
    }
}

fn canonical_pair(
    first: Entity,
    second: Entity,
    info: TriggerPairInfo,
) -> ((Entity, Entity), TriggerPairInfo) {
    if first.to_bits() <= second.to_bits() {
        ((first, second), info)
    } else {
        (
            (second, first),
            TriggerPairInfo {
                first_is_trigger: info.second_is_trigger,
                second_is_trigger: info.first_is_trigger,
            },
        )
    }
}

fn should_collide(first: RuntimeCollider, second: RuntimeCollider) -> bool {
    first.layer_bits & second.mask_bits != 0 && second.layer_bits & first.mask_bits != 0
}

fn colliders_overlap(
    first: RuntimeCollider,
    first_translation: Vec3,
    second: RuntimeCollider,
    second_translation: Vec3,
) -> bool {
    let first_center = first.center_at(first_translation);
    let second_center = second.center_at(second_translation);
    match (first.shape, second.shape) {
        (
            RuntimeColliderShape::Circle { radius: radius_a },
            RuntimeColliderShape::Circle { radius: radius_b },
        ) => first_center.distance_squared(second_center) < (radius_a + radius_b).powi(2),
        (RuntimeColliderShape::Circle { radius }, shape)
        | (shape, RuntimeColliderShape::Circle { radius }) => {
            let box_aabb = match shape {
                RuntimeColliderShape::Box { half_extents }
                | RuntimeColliderShape::PolygonBounds { half_extents } => {
                    let center = if matches!(first.shape, RuntimeColliderShape::Circle { .. }) {
                        second_center
                    } else {
                        first_center
                    };
                    Aabb::from_center_half_extents(center, half_extents)
                }
                RuntimeColliderShape::Circle { .. } => unreachable!(),
            };
            let circle_center = if matches!(first.shape, RuntimeColliderShape::Circle { .. }) {
                first_center
            } else {
                second_center
            };
            let closest = box_aabb.clamp_point(circle_center);
            closest.distance_squared(circle_center) < radius.powi(2)
        }
        _ => first
            .bounds_aabb_at(first_translation)
            .overlaps(second.bounds_aabb_at(second_translation)),
    }
}

trait Vec3DataExt {
    fn into_vec2(self) -> Vec2;
}

impl Vec3DataExt for Vec3Data {
    fn into_vec2(self) -> Vec2 {
        Vec2::new(self.x, self.y)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layer_registry_allocates_default_and_custom_bits() {
        let mut layers = CollisionLayers::default();
        assert_eq!(layers.bit_for("default"), 1);
        assert_eq!(layers.bit_for("walls"), 2);
        assert_eq!(layers.bit_for("npcs"), 4);
        assert_eq!(layers.mask_bits(&["default".into(), "walls".into()]), 3);
    }

    #[test]
    fn test_layer_registry_falls_back_after_32_layers() {
        let mut layers = CollisionLayers::default();
        for index in 0..40 {
            let _ = layers.bit_for(&format!("layer_{index}"));
        }
        assert_eq!(layers.bit_for("overflow"), 1);
    }

    #[test]
    fn test_box_aabb_overlap() {
        let collider = RuntimeCollider {
            enabled: true,
            body_type: BodyType::Static,
            shape: RuntimeColliderShape::Box {
                half_extents: Vec2::splat(16.0),
            },
            offset: Vec2::ZERO,
            layer_bits: 1,
            mask_bits: 1,
            is_trigger: false,
        };
        assert!(colliders_overlap(
            collider,
            Vec3::ZERO,
            collider,
            Vec3::new(20.0, 0.0, 0.0)
        ));
        assert!(!colliders_overlap(
            collider,
            Vec3::ZERO,
            collider,
            Vec3::new(40.0, 0.0, 0.0)
        ));
    }

    #[test]
    fn test_circle_box_overlap() {
        let circle = RuntimeCollider {
            enabled: true,
            body_type: BodyType::Static,
            shape: RuntimeColliderShape::Circle { radius: 12.0 },
            offset: Vec2::ZERO,
            layer_bits: 1,
            mask_bits: 1,
            is_trigger: true,
        };
        let box_collider = RuntimeCollider {
            shape: RuntimeColliderShape::Box {
                half_extents: Vec2::splat(10.0),
            },
            is_trigger: true,
            ..circle
        };
        assert!(colliders_overlap(
            circle,
            Vec3::ZERO,
            box_collider,
            Vec3::new(16.0, 0.0, 0.0)
        ));
        assert!(!colliders_overlap(
            circle,
            Vec3::ZERO,
            box_collider,
            Vec3::new(30.0, 0.0, 0.0)
        ));
    }

    #[test]
    fn test_kinematic_resolution_stops_at_static_blocker() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(CollisionPlugin);

        let mover = app
            .world_mut()
            .spawn((
                Transform::from_xyz(0.0, 0.0, 0.0),
                CollisionComponent {
                    body_type: BodyType::Kinematic,
                    box_size: Some(Vec3Data::new(20.0, 20.0, 0.0)),
                    ..Default::default()
                },
                MovementIntent(Vec2::new(20.0, 0.0)),
            ))
            .id();
        app.world_mut().spawn((
            Transform::from_xyz(30.0, 0.0, 0.0),
            CollisionComponent {
                body_type: BodyType::Static,
                box_size: Some(Vec3Data::new(20.0, 20.0, 0.0)),
                ..Default::default()
            },
        ));

        app.update();
        app.update();

        let transform = app.world().entity(mover).get::<Transform>().unwrap();
        assert!((transform.translation.x - 10.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_trigger_contact_events_enter_and_exit() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(CollisionPlugin);

        let trigger = app
            .world_mut()
            .spawn((
                Transform::default(),
                CollisionComponent {
                    body_type: BodyType::Static,
                    box_size: Some(Vec3Data::new(20.0, 20.0, 0.0)),
                    is_trigger: true,
                    ..Default::default()
                },
            ))
            .id();
        let mover = app
            .world_mut()
            .spawn((
                Transform::from_xyz(0.0, 0.0, 0.0),
                CollisionComponent {
                    body_type: BodyType::Kinematic,
                    box_size: Some(Vec3Data::new(20.0, 20.0, 0.0)),
                    ..Default::default()
                },
                MovementIntent(Vec2::ZERO),
            ))
            .id();

        app.update();

        {
            let events = app.world().resource::<Messages<TriggerContactEvent>>();
            let enter_count = events
                .iter_current_update_messages()
                .filter(|event| matches!(event, TriggerContactEvent::Enter { trigger: entity, other } if *entity == trigger && *other == mover))
                .count();
            assert_eq!(enter_count, 1);
        }

        app.world_mut()
            .entity_mut(mover)
            .insert(MovementIntent(Vec2::new(40.0, 0.0)));
        app.update();

        let events = app.world().resource::<Messages<TriggerContactEvent>>();
        let saw_exit = events
            .iter_current_update_messages()
            .any(|event| matches!(event, TriggerContactEvent::Exit { trigger: entity, other } if *entity == trigger && *other == mover));
        assert!(saw_exit);
    }
}
