//! Input handling: hover tracking, entity placement/removal, terrain cycling.

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::grid::{IsoGrid, PlacedEntity};
use crate::palette::SelectedPalette;
use crate::rendering::{world_to_iso, StatusText};

/// Tracks which tile the cursor is hovering over.
#[derive(Resource, Default, Debug)]
pub struct HoverTile {
    pub tile: Option<(usize, usize)>,
}

/// Temporary feedback message.
#[derive(Resource, Default, Debug)]
pub struct FeedbackMessage {
    pub text: String,
    pub timer: f32,
}

impl FeedbackMessage {
    pub fn set(&mut self, msg: impl Into<String>) {
        self.text = msg.into();
        self.timer = 2.0;
    }

    pub fn is_active(&self) -> bool {
        self.timer > 0.0
    }
}

/// Get cursor position in world coordinates.
fn cursor_world_pos(
    windows: &Query<&Window, With<PrimaryWindow>>,
    camera_q: &Query<(&Camera, &GlobalTransform)>,
) -> Option<Vec2> {
    let window = windows.single().ok()?;
    let (camera, camera_transform) = camera_q.single().ok()?;
    let cursor_pos = window.cursor_position()?;
    camera
        .viewport_to_world_2d(camera_transform, cursor_pos)
        .ok()
}

/// Update hover tile based on cursor position.
pub fn hover_system(
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    mut hover: ResMut<HoverTile>,
) {
    let new_tile = cursor_world_pos(&windows, &camera_q).and_then(world_to_iso);

    if hover.tile != new_tile {
        hover.tile = new_tile;
    }
}

/// A short-lived sparkle particle spawned on placement/removal.
#[derive(Component)]
pub struct PlacementFx {
    pub timer: Timer,
    pub velocity: Vec2,
}

/// Left click: place entity at hovered tile.
pub fn click_place_system(
    mut commands: Commands,
    mouse: Res<ButtonInput<MouseButton>>,
    hover: Res<HoverTile>,
    palette: Res<SelectedPalette>,
    mut grid: ResMut<IsoGrid>,
    mut feedback: ResMut<FeedbackMessage>,
) {
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    let Some((x, y)) = hover.tile else {
        return;
    };

    let Some(cell) = grid.get_mut(x, y) else {
        return;
    };

    if cell.entity.is_some() {
        feedback.set(format!(
            "({}, {}) already has an entity — right-click to remove.",
            x, y
        ));
        return;
    }

    cell.entity = Some(PlacedEntity { kind: palette.kind });
    feedback.set(format!("Placed {} at ({}, {})", palette.kind.name(), x, y));

    // Spawn sparkle particles at placement location
    let world_pos = crate::rendering::iso_to_world(x, y);
    for i in 0..6 {
        let angle = i as f32 * std::f32::consts::TAU / 6.0;
        let offset = Vec2::new(angle.cos(), angle.sin()) * 8.0;
        commands.spawn((
            PlacementFx {
                timer: Timer::from_seconds(0.5, TimerMode::Once),
                velocity: offset * 40.0,
            },
            Sprite {
                color: Color::srgba(1.0, 0.85, 0.2, 0.8),
                custom_size: Some(Vec2::new(3.0, 3.0)),
                ..default()
            },
            Transform::from_xyz(world_pos.x + offset.x, world_pos.y + offset.y, 10.0),
        ));
    }
}

/// Right click: remove entity from hovered tile.
pub fn click_remove_system(
    mut commands: Commands,
    mouse: Res<ButtonInput<MouseButton>>,
    hover: Res<HoverTile>,
    mut grid: ResMut<IsoGrid>,
    mut feedback: ResMut<FeedbackMessage>,
) {
    if !mouse.just_pressed(MouseButton::Right) {
        return;
    }

    let Some((x, y)) = hover.tile else {
        return;
    };

    let Some(cell) = grid.get_mut(x, y) else {
        return;
    };

    if cell.entity.take().is_some() {
        feedback.set(format!("Removed entity at ({}, {})", x, y));

        // Red burst particles on removal
        let world_pos = crate::rendering::iso_to_world(x, y);
        for i in 0..8 {
            let angle = i as f32 * std::f32::consts::TAU / 8.0;
            let offset = Vec2::new(angle.cos(), angle.sin()) * 6.0;
            commands.spawn((
                PlacementFx {
                    timer: Timer::from_seconds(0.4, TimerMode::Once),
                    velocity: offset * 60.0,
                },
                Sprite {
                    color: Color::srgba(0.9, 0.2, 0.1, 0.9),
                    custom_size: Some(Vec2::new(3.0, 3.0)),
                    ..default()
                },
                Transform::from_xyz(world_pos.x + offset.x, world_pos.y + offset.y, 10.0),
            ));
        }
    }
}

/// Animate and despawn placement FX particles.
pub fn animate_fx_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut PlacementFx, &mut Transform, &mut Sprite)>,
) {
    let dt = time.delta_secs();
    for (entity, mut fx, mut transform, mut sprite) in &mut query {
        fx.timer.tick(time.delta());
        transform.translation.x += fx.velocity.x * dt;
        transform.translation.y += fx.velocity.y * dt;

        let alpha = 1.0 - fx.timer.fraction();
        sprite.color = sprite.color.with_alpha(alpha * 0.8);

        if fx.timer.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

/// T key: cycle terrain at hovered tile.
pub fn terrain_cycle_system(
    keys: Res<ButtonInput<KeyCode>>,
    hover: Res<HoverTile>,
    mut grid: ResMut<IsoGrid>,
    mut feedback: ResMut<FeedbackMessage>,
) {
    if !keys.just_pressed(KeyCode::KeyT) {
        return;
    }

    let Some((x, y)) = hover.tile else {
        return;
    };

    let Some(cell) = grid.get_mut(x, y) else {
        return;
    };

    cell.terrain = cell.terrain.next();
    feedback.set(format!("({}, {}) → {}", x, y, cell.terrain.name()));
}

/// Tick feedback timer.
pub fn tick_feedback_system(time: Res<Time>, mut feedback: ResMut<FeedbackMessage>) {
    if feedback.timer > 0.0 {
        feedback.timer -= time.delta_secs();
    }
}

/// Update status text.
pub fn status_system(
    feedback: Res<FeedbackMessage>,
    palette: Res<SelectedPalette>,
    hover: Res<HoverTile>,
    mut text_q: Query<&mut Text2d, With<StatusText>>,
) {
    let Ok(mut text) = text_q.single_mut() else {
        return;
    };

    if feedback.is_active() {
        **text = feedback.text.clone();
        return;
    }

    let hover_str = if let Some((x, y)) = hover.tile {
        format!(" | Hover: ({}, {})", x, y)
    } else {
        String::new()
    };

    **text = format!(
        "Selected: {} [1-4]{} | LMB:Place RMB:Remove T:Terrain",
        palette.kind.name(),
        hover_str,
    );
}
