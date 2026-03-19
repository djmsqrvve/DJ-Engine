//! Runtime tile spawner — converts GridLevel data into Bevy sprite entities
//! with collision bodies when RUN PROJECT starts.

use super::grid::{self, GridLevel, LayerType, TileType};
use crate::collision::RuntimeCollider;
use crate::collision::RuntimeColliderShape;
use crate::data::components::BodyType;
use bevy::prelude::*;

/// Marker component for runtime-spawned tile sprites.
#[derive(Component)]
pub struct TileSprite {
    pub tx: i32,
    pub ty: i32,
    pub layer: LayerType,
}

/// Marker for runtime-spawned grid entities.
#[derive(Component)]
pub struct GridEntitySprite;

/// Spawn all GridLevel tiles as Bevy sprite entities.
pub fn spawn_grid_tiles(commands: &mut Commands, grid: &GridLevel) {
    for tile_layer in &grid.layers {
        if !tile_layer.visible {
            continue;
        }
        let z = match tile_layer.name {
            LayerType::Ground => 0.0,
            LayerType::Collision => 1.0,
            LayerType::Objects => 2.0,
            LayerType::Triggers => 3.0,
            LayerType::Entities => 4.0,
        };
        for (&(tx, ty), &tt) in &tile_layer.tiles {
            if tt == TileType::Empty {
                continue;
            }
            let world_x = tx as f32 * grid.tile_size;
            let world_y = ty as f32 * grid.tile_size;
            let color = grid::tile_color(tt);
            let bevy_color = Color::srgba_u8(color.r(), color.g(), color.b(), 255);
            let mut cmd = commands.spawn((
                TileSprite {
                    tx,
                    ty,
                    layer: tile_layer.name,
                },
                Sprite {
                    color: bevy_color,
                    custom_size: Some(Vec2::splat(grid.tile_size - 2.0)),
                    ..default()
                },
                Transform::from_xyz(world_x, world_y, z),
            ));

            // Collision layer tiles get static collision bodies
            if tile_layer.name == LayerType::Collision {
                let half = grid.tile_size / 2.0;
                cmd.insert(RuntimeCollider {
                    enabled: true,
                    body_type: BodyType::Static,
                    shape: RuntimeColliderShape::Box {
                        half_extents: Vec2::splat(half),
                    },
                    offset: Vec2::ZERO,
                    layer_bits: 0xFFFF,
                    mask_bits: 0xFFFF,
                    is_trigger: false,
                });
            }

            // Trigger layer tiles get trigger collision
            if tile_layer.name == LayerType::Triggers {
                let half = grid.tile_size / 2.0;
                cmd.insert(RuntimeCollider {
                    enabled: true,
                    body_type: BodyType::Static,
                    shape: RuntimeColliderShape::Box {
                        half_extents: Vec2::splat(half),
                    },
                    offset: Vec2::ZERO,
                    layer_bits: 0xFFFF,
                    mask_bits: 0xFFFF,
                    is_trigger: true,
                });
            }
        }
    }
}

/// Spawn all GridLevel entities as Bevy entities with colored sprites.
pub fn spawn_grid_entities(commands: &mut Commands, grid: &GridLevel) {
    for entity in &grid.entities {
        let world_x = entity.x as f32 * grid.tile_size;
        let world_y = entity.y as f32 * grid.tile_size;
        let color = match entity.entity_type.as_str() {
            "mob" | "training_dummy" => Color::srgb(0.96, 0.26, 0.21),
            "npc" | "npc_spawn" => Color::srgb(0.30, 0.69, 0.31),
            "spawn_point" => Color::srgb(0.20, 0.80, 0.20),
            "teleporter" => Color::srgb(0.58, 0.44, 0.86),
            "chest" => Color::srgb(0.85, 0.65, 0.13),
            _ => Color::srgb(0.53, 0.81, 0.92),
        };
        commands.spawn((
            GridEntitySprite,
            Sprite {
                color,
                custom_size: Some(Vec2::splat(grid.tile_size)),
                ..default()
            },
            Transform::from_xyz(world_x, world_y, 5.0),
            Name::new(format!("GridEntity:{}", entity.entity_type)),
        ));
    }
}
