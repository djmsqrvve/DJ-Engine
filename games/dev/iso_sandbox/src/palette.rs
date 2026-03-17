//! Entity palette — keyboard-driven selection of what to place.

use bevy::prelude::*;

use crate::grid::EntityKind;

/// Currently selected entity type to place.
#[derive(Resource, Debug)]
pub struct SelectedPalette {
    pub kind: EntityKind,
}

impl Default for SelectedPalette {
    fn default() -> Self {
        Self {
            kind: EntityKind::Actor,
        }
    }
}

/// Switch palette with number keys.
pub fn palette_system(keys: Res<ButtonInput<KeyCode>>, mut palette: ResMut<SelectedPalette>) {
    if keys.just_pressed(KeyCode::Digit1) {
        palette.kind = EntityKind::Actor;
    }
    if keys.just_pressed(KeyCode::Digit2) {
        palette.kind = EntityKind::Prop;
    }
    if keys.just_pressed(KeyCode::Digit3) {
        palette.kind = EntityKind::Blocker;
    }
    if keys.just_pressed(KeyCode::Digit4) {
        palette.kind = EntityKind::Spawn;
    }
}
