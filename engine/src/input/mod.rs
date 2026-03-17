//! Input abstraction for DJ Engine.
//!
//! Provides action-based input mapping so games don't need to handle raw keycodes.
//! Supports keyboard with gamepad support planned for the future.

use bevy::prelude::*;
use std::collections::HashSet;

/// Logical game actions, abstracted from physical input.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InputAction {
    /// Confirm selection (Space, Enter, gamepad A)
    Confirm,
    /// Cancel / back (Escape, Backspace, gamepad B)
    Cancel,
    /// Menu / pause (Escape, Tab, gamepad Start)
    Menu,
    /// Directional inputs
    Up,
    Down,
    Left,
    Right,
}

/// Resource tracking the current state of all input actions.
#[derive(Resource, Default)]
pub struct ActionState {
    /// Actions that are currently held down
    pressed: HashSet<InputAction>,
    /// Actions that were just pressed this frame
    just_pressed: HashSet<InputAction>,
    /// Actions that were just released this frame
    just_released: HashSet<InputAction>,
}

impl ActionState {
    /// Returns true if the action is currently held down.
    pub fn pressed(&self, action: InputAction) -> bool {
        self.pressed.contains(&action)
    }

    /// Returns true if the action was just pressed this frame.
    pub fn just_pressed(&self, action: InputAction) -> bool {
        self.just_pressed.contains(&action)
    }

    /// Returns true if the action was just released this frame.
    pub fn just_released(&self, action: InputAction) -> bool {
        self.just_released.contains(&action)
    }
}

/// Configuration for input mappings.
#[derive(Resource, Clone)]
pub struct InputConfig {
    /// Keyboard mappings: KeyCode -> InputAction
    pub keyboard_map: Vec<(KeyCode, InputAction)>,
}

impl Default for InputConfig {
    fn default() -> Self {
        Self {
            keyboard_map: vec![
                // Confirm
                (KeyCode::Space, InputAction::Confirm),
                (KeyCode::Enter, InputAction::Confirm),
                (KeyCode::KeyZ, InputAction::Confirm),
                // Cancel
                (KeyCode::Escape, InputAction::Cancel),
                (KeyCode::Backspace, InputAction::Cancel),
                (KeyCode::KeyX, InputAction::Cancel),
                // Menu
                (KeyCode::Tab, InputAction::Menu),
                // Directions - Arrow keys
                (KeyCode::ArrowUp, InputAction::Up),
                (KeyCode::ArrowDown, InputAction::Down),
                (KeyCode::ArrowLeft, InputAction::Left),
                (KeyCode::ArrowRight, InputAction::Right),
                // Directions - WASD
                (KeyCode::KeyW, InputAction::Up),
                (KeyCode::KeyS, InputAction::Down),
                (KeyCode::KeyA, InputAction::Left),
                (KeyCode::KeyD, InputAction::Right),
            ],
        }
    }
}

/// Plugin providing input abstraction.
pub struct DJInputPlugin;

impl Plugin for DJInputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ActionState>()
            .init_resource::<InputConfig>()
            .add_systems(PreUpdate, update_action_state);

        use crate::contracts::{AppContractExt, ContractEntry, PluginContract};
        app.register_contract(PluginContract {
            name: "DJInputPlugin".into(),
            description: "Action-based input mapping (keyboard, gamepad planned)".into(),
            resources: vec![
                ContractEntry::of::<ActionState>("Current input action state"),
                ContractEntry::of::<InputConfig>("Keyboard/gamepad mappings"),
            ],
            components: vec![],
            events: vec![],
            system_sets: vec![],
        });

        info!("DJ Input Plugin initialized");
    }
}

/// System that updates the ActionState based on keyboard input.
fn update_action_state(
    keys: Res<ButtonInput<KeyCode>>,
    config: Res<InputConfig>,
    mut action_state: ResMut<ActionState>,
) {
    // Clear previous frame's just_pressed/just_released
    action_state.just_pressed.clear();
    action_state.just_released.clear();
    action_state.pressed.clear();

    // Check each mapped key
    for (keycode, action) in &config.keyboard_map {
        if keys.pressed(*keycode) {
            action_state.pressed.insert(*action);
        }
        if keys.just_pressed(*keycode) {
            action_state.just_pressed.insert(*action);
        }
        if keys.just_released(*keycode) {
            action_state.just_released.insert(*action);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_has_mappings() {
        let config = InputConfig::default();
        assert!(!config.keyboard_map.is_empty());

        let has_confirm = config
            .keyboard_map
            .iter()
            .any(|(_, action)| *action == InputAction::Confirm);
        assert!(has_confirm);
    }

    #[test]
    fn test_default_config_all_directions_mapped() {
        let config = InputConfig::default();
        for action in [
            InputAction::Up,
            InputAction::Down,
            InputAction::Left,
            InputAction::Right,
        ] {
            assert!(
                config.keyboard_map.iter().any(|(_, a)| *a == action),
                "{:?} not mapped",
                action
            );
        }
    }

    #[test]
    fn test_action_state_pressed() {
        let mut state = ActionState::default();
        assert!(!state.pressed(InputAction::Confirm));
        state.pressed.insert(InputAction::Confirm);
        assert!(state.pressed(InputAction::Confirm));
        assert!(!state.pressed(InputAction::Cancel));
    }

    #[test]
    fn test_action_state_just_pressed() {
        let mut state = ActionState::default();
        state.just_pressed.insert(InputAction::Up);
        assert!(state.just_pressed(InputAction::Up));
        assert!(!state.just_pressed(InputAction::Down));
    }

    #[test]
    fn test_action_state_just_released() {
        let mut state = ActionState::default();
        state.just_released.insert(InputAction::Left);
        assert!(state.just_released(InputAction::Left));
        assert!(!state.just_released(InputAction::Right));
    }

    #[test]
    fn test_action_state_default_is_empty() {
        let state = ActionState::default();
        for action in [
            InputAction::Confirm,
            InputAction::Cancel,
            InputAction::Menu,
            InputAction::Up,
            InputAction::Down,
            InputAction::Left,
            InputAction::Right,
        ] {
            assert!(!state.pressed(action));
            assert!(!state.just_pressed(action));
            assert!(!state.just_released(action));
        }
    }
}
