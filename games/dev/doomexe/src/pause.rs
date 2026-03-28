//! Pause menu for DoomExe.
//!
//! ESC toggles pause overlay. Works in Overworld and Battle states.
//! Resume continues gameplay, Quit returns to title screen.

use bevy::prelude::*;
use dj_engine::input::{ActionState, InputAction};

use crate::state::GameState;

#[derive(Resource, Default)]
pub struct PauseState {
    pub paused: bool,
    pub selection: usize,
}

const MENU_ITEMS: &[&str] = &["Resume", "Quit to Title"];

#[derive(Component)]
struct PauseOverlay;

#[derive(Component)]
struct PauseMenuText;

pub struct PausePlugin;

impl Plugin for PausePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PauseState>()
            .add_systems(Startup, setup_pause_overlay)
            .add_systems(
                Update,
                (toggle_pause, handle_pause_input, update_pause_text).chain(),
            );
    }
}

fn setup_pause_overlay(mut commands: Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(0.0),
                left: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                display: Display::None,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.75)),
            PauseOverlay,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("PAUSED"),
                TextFont {
                    font_size: 48.0,
                    ..default()
                },
                TextColor(Color::srgb(0.8, 0.8, 0.8)),
            ));
            parent.spawn((
                Text::new(""),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                PauseMenuText,
            ));
        });
}

fn toggle_pause(
    keys: Res<ButtonInput<KeyCode>>,
    mut pause: ResMut<PauseState>,
    game_state: Res<State<GameState>>,
    mut overlay_query: Query<&mut Node, With<PauseOverlay>>,
) {
    if !keys.just_pressed(KeyCode::Escape) {
        return;
    }

    // Only allow pause in gameplay states
    match game_state.get() {
        GameState::Overworld | GameState::Battle => {}
        _ => return,
    }

    pause.paused = !pause.paused;
    pause.selection = 0;

    if let Ok(mut node) = overlay_query.single_mut() {
        node.display = if pause.paused {
            Display::Flex
        } else {
            Display::None
        };
    }
}

fn handle_pause_input(
    actions: Res<ActionState>,
    keys: Res<ButtonInput<KeyCode>>,
    mut pause: ResMut<PauseState>,
    mut next_state: ResMut<NextState<GameState>>,
    mut overlay_query: Query<&mut Node, With<PauseOverlay>>,
) {
    if !pause.paused {
        return;
    }

    // Navigate
    if keys.just_pressed(KeyCode::ArrowUp) || keys.just_pressed(KeyCode::KeyW) {
        if pause.selection > 0 {
            pause.selection -= 1;
        }
    }
    if keys.just_pressed(KeyCode::ArrowDown) || keys.just_pressed(KeyCode::KeyS) {
        if pause.selection < MENU_ITEMS.len() - 1 {
            pause.selection += 1;
        }
    }

    // Confirm
    if actions.just_pressed(InputAction::Confirm) {
        match pause.selection {
            0 => {
                // Resume
                pause.paused = false;
                if let Ok(mut node) = overlay_query.single_mut() {
                    node.display = Display::None;
                }
            }
            1 => {
                // Quit to title
                pause.paused = false;
                if let Ok(mut node) = overlay_query.single_mut() {
                    node.display = Display::None;
                }
                info!("STATE: Paused -> TitleScreen (quit)");
                next_state.set(GameState::TitleScreen);
            }
            _ => {}
        }
    }
}

fn update_pause_text(
    pause: Res<PauseState>,
    mut text_query: Query<&mut Text, With<PauseMenuText>>,
) {
    if !pause.paused {
        return;
    }

    let Ok(mut text) = text_query.single_mut() else {
        return;
    };

    let mut menu = String::new();
    for (i, item) in MENU_ITEMS.iter().enumerate() {
        let marker = if i == pause.selection { "> " } else { "  " };
        menu.push_str(&format!("\n{marker}{item}"));
    }

    *text = Text::new(menu);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pause_state_defaults() {
        let state = PauseState::default();
        assert!(!state.paused);
        assert_eq!(state.selection, 0);
    }

    #[test]
    fn test_menu_items_exist() {
        assert_eq!(MENU_ITEMS.len(), 2);
        assert_eq!(MENU_ITEMS[0], "Resume");
        assert_eq!(MENU_ITEMS[1], "Quit to Title");
    }
}
