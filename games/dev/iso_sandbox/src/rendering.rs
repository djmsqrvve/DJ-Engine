//! Isometric rendering — diamond tiles, entity sprites, hover highlights.

use bevy::prelude::*;

use crate::grid::{EntityKind, IsoGrid, IsoTerrain, GRID_HEIGHT, GRID_WIDTH};
use crate::input::HoverTile;

/// Tile dimensions for 2:1 isometric projection.
pub const TILE_WIDTH: f32 = 64.0;
pub const TILE_HEIGHT: f32 = 32.0;

// Terrain colors.
const COLOR_GRASS: Color = Color::srgb(0.36, 0.68, 0.35);
const COLOR_STONE: Color = Color::srgb(0.60, 0.58, 0.55);
const COLOR_WATER: Color = Color::srgb(0.25, 0.48, 0.78);
const COLOR_SAND: Color = Color::srgb(0.88, 0.82, 0.60);

// Entity colors.
const COLOR_ACTOR: Color = Color::srgb(0.9, 0.3, 0.3);
const COLOR_PROP: Color = Color::srgb(0.3, 0.7, 0.9);
const COLOR_BLOCKER: Color = Color::srgb(0.5, 0.5, 0.5);
const COLOR_SPAWN: Color = Color::srgb(0.9, 0.8, 0.2);

// Highlight.
const COLOR_HOVER: Color = Color::srgba(1.0, 1.0, 1.0, 0.25);

/// Marker for tile sprites.
#[derive(Component)]
pub struct TileSprite {
    pub x: usize,
    pub y: usize,
}

/// Marker for entity sprites on tiles.
#[derive(Component)]
pub struct EntitySprite;

/// Marker for hover highlight overlay.
#[derive(Component)]
pub struct HoverHighlight;

/// Marker for status text.
#[derive(Component)]
pub struct StatusText;

/// Convert grid coordinates to world position (isometric 2:1 projection).
/// Y is negated because Bevy's Y goes up, but iso "down-right" should decrease screen Y.
/// Z provides depth sorting: tiles with higher (x+y) are rendered behind.
pub fn iso_to_world(x: usize, y: usize) -> Vec3 {
    let sx = (x as f32 - y as f32) * TILE_WIDTH / 2.0;
    let sy = (x as f32 + y as f32) * TILE_HEIGHT / 2.0;
    let depth = -((x + y) as f32) * 0.01;
    Vec3::new(sx, -sy, depth)
}

/// Convert world position to grid coordinates (reverse isometric projection).
pub fn world_to_iso(pos: Vec2) -> Option<(usize, usize)> {
    // Reverse the iso projection:
    //   sx = (gx - gy) * TW/2   =>  sx / (TW/2) = gx - gy
    //   sy = (gx + gy) * TH/2   =>  -pos.y / (TH/2) = gx + gy  (negate Y)
    let a = pos.x / (TILE_WIDTH / 2.0);
    let b = -pos.y / (TILE_HEIGHT / 2.0);
    let gx = (a + b) / 2.0;
    let gy = (b - a) / 2.0;

    // Round to nearest cell.
    let ix = gx.round() as i32;
    let iy = gy.round() as i32;

    if ix >= 0 && iy >= 0 && (ix as usize) < GRID_WIDTH && (iy as usize) < GRID_HEIGHT {
        Some((ix as usize, iy as usize))
    } else {
        None
    }
}

fn terrain_color(terrain: IsoTerrain) -> Color {
    match terrain {
        IsoTerrain::Grass => COLOR_GRASS,
        IsoTerrain::Stone => COLOR_STONE,
        IsoTerrain::Water => COLOR_WATER,
        IsoTerrain::Sand => COLOR_SAND,
    }
}

fn entity_color(kind: EntityKind) -> Color {
    match kind {
        EntityKind::Actor => COLOR_ACTOR,
        EntityKind::Prop => COLOR_PROP,
        EntityKind::Blocker => COLOR_BLOCKER,
        EntityKind::Spawn => COLOR_SPAWN,
    }
}

/// Spawn isometric tile sprites for the entire grid.
pub fn spawn_grid_system(mut commands: Commands, grid: Res<IsoGrid>) {
    for (x, y, cell) in grid.grid.iter() {
        let pos = iso_to_world(x, y);
        let color = terrain_color(cell.terrain);

        // Diamond tile: a square sprite rotated 45 degrees and scaled to 2:1 ratio.
        commands.spawn((
            TileSprite { x, y },
            Sprite {
                color,
                custom_size: Some(Vec2::new(TILE_WIDTH * 0.7, TILE_WIDTH * 0.7)),
                ..default()
            },
            Transform {
                translation: pos,
                rotation: Quat::from_rotation_z(std::f32::consts::FRAC_PI_4),
                scale: Vec3::new(1.0, 0.5, 1.0),
                ..default()
            },
        ));
    }

    // Status text at top.
    commands.spawn((
        StatusText,
        Text2d::new(
            "Iso Sandbox — 1:Actor 2:Prop 3:Blocker 4:Spawn | LMB:Place RMB:Remove T:Terrain",
        ),
        TextFont {
            font_size: 18.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Transform::from_xyz(0.0, 280.0, 10.0),
    ));
}

/// Update tile colors when terrain changes.
pub fn sync_tiles_system(grid: Res<IsoGrid>, mut tile_q: Query<(&TileSprite, &mut Sprite)>) {
    if !grid.is_changed() {
        return;
    }

    for (tile, mut sprite) in &mut tile_q {
        if let Some(cell) = grid.get(tile.x, tile.y) {
            sprite.color = terrain_color(cell.terrain);
        }
    }
}

/// Rebuild entity sprites when the grid changes.
pub fn sync_entities_system(
    mut commands: Commands,
    grid: Res<IsoGrid>,
    existing: Query<Entity, With<EntitySprite>>,
) {
    if !grid.is_changed() {
        return;
    }

    for entity in &existing {
        commands.entity(entity).despawn();
    }

    for (x, y, cell) in grid.grid.iter() {
        let Some(placed) = &cell.entity else {
            continue;
        };

        let pos = iso_to_world(x, y);
        let color = entity_color(placed.kind);

        // Entity marker: a smaller diamond on top of the tile.
        commands
            .spawn((
                EntitySprite,
                Sprite {
                    color,
                    custom_size: Some(Vec2::new(TILE_WIDTH * 0.4, TILE_WIDTH * 0.4)),
                    ..default()
                },
                Transform {
                    translation: pos + Vec3::new(0.0, TILE_HEIGHT * 0.3, 0.5),
                    rotation: Quat::from_rotation_z(std::f32::consts::FRAC_PI_4),
                    scale: Vec3::new(1.0, 0.5, 1.0),
                    ..default()
                },
            ))
            .with_children(|parent| {
                parent.spawn((
                    Text2d::new(placed.kind.label()),
                    TextFont {
                        font_size: 16.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                    Transform {
                        translation: Vec3::new(0.0, 0.0, 1.0),
                        // Counter-rotate the text so it reads normally.
                        rotation: Quat::from_rotation_z(-std::f32::consts::FRAC_PI_4),
                        scale: Vec3::new(1.0, 2.0, 1.0), // counter the parent's Y squish
                        ..default()
                    },
                ));
            });
    }
}

/// Show hover highlight on the tile under the cursor.
pub fn sync_hover_system(
    mut commands: Commands,
    hover: Res<HoverTile>,
    existing: Query<Entity, With<HoverHighlight>>,
) {
    for entity in &existing {
        commands.entity(entity).despawn();
    }

    if let Some((x, y)) = hover.tile {
        let pos = iso_to_world(x, y);
        commands.spawn((
            HoverHighlight,
            Sprite {
                color: COLOR_HOVER,
                custom_size: Some(Vec2::new(TILE_WIDTH * 0.7, TILE_WIDTH * 0.7)),
                ..default()
            },
            Transform {
                translation: pos + Vec3::Z * 0.3,
                rotation: Quat::from_rotation_z(std::f32::consts::FRAC_PI_4),
                scale: Vec3::new(1.0, 0.5, 1.0),
                ..default()
            },
        ));
    }
}
