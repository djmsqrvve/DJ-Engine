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
    camera.viewport_to_world_2d(camera_transform, cursor_pos).ok()
}

/// Update hover tile based on cursor position.
pub fn hover_system(
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    mut hover: ResMut<HoverTile>,
) {
    let new_tile = cursor_world_pos(&windows, &camera_q)
        .and_then(world_to_iso);

    if hover.tile != new_tile {
        hover.tile = new_tile;
    }
}

/// Left click: place entity at hovered tile.
pub fn click_place_system(
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
        feedback.set(format!("({}, {}) already has an entity — right-click to remove.", x, y));
        return;
    }

    cell.entity = Some(PlacedEntity {
        kind: palette.kind,
    });
    feedback.set(format!("Placed {} at ({}, {})", palette.kind.name(), x, y));
}

/// Right click: remove entity from hovered tile.
pub fn click_remove_system(
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
