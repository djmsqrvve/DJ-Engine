use crate::state::GameState;
use bevy::prelude::*;
use dj_engine::input::{ActionState, InputAction};

#[derive(Component)]
struct TitleMenu;

#[derive(Component)]
struct MenuOption {
    index: usize,
    action: MenuAction,
}

#[derive(Clone, Copy)]
enum MenuAction {
    NewGame,
    Continue,
    Quit,
}

#[derive(Resource, Default)]
struct TitleState {
    selected_index: usize,
    options_count: usize,
}

pub struct TitlePlugin;

impl Plugin for TitlePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TitleState>()
            .add_systems(OnEnter(GameState::TitleScreen), setup_title_ui)
            .add_systems(Update, title_input.run_if(in_state(GameState::TitleScreen)))
            .add_systems(OnExit(GameState::TitleScreen), teardown_title_ui);
    }
}

fn setup_title_ui(mut commands: Commands, mut state: ResMut<TitleState>) {
    state.selected_index = 0;
    state.options_count = 3;

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(Color::BLACK),
            TitleMenu,
        ))
        .with_children(|parent| {
            // Title Text
            parent.spawn((
                Text::new("DOOM EXE"),
                TextFont {
                    font_size: 60.0,
                    ..default()
                },
                TextColor(Color::srgb(0.8, 0.0, 0.0)),
                Node {
                    margin: UiRect::bottom(Val::Px(50.0)),
                    ..default()
                },
            ));

            // Menu Options
            spawn_menu_option(parent, "NEW GAME", 0, MenuAction::NewGame);
            spawn_menu_option(parent, "CONTINUE", 1, MenuAction::Continue);
            spawn_menu_option(parent, "QUIT", 2, MenuAction::Quit);
        });
}

fn spawn_menu_option(
    parent: &mut bevy::ecs::hierarchy::ChildSpawnerCommands<'_>,
    text: &str,
    index: usize,
    action: MenuAction,
) {
    parent.spawn((
        Text::new(text),
        TextFont {
            font_size: 30.0,
            ..default()
        },
        TextColor(Color::WHITE), // System will update color
        Node {
            margin: UiRect::all(Val::Px(10.0)),
            ..default()
        },
        MenuOption { index, action },
    ));
}

fn title_input(
    mut _commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
    mut state: ResMut<TitleState>,
    actions: Res<ActionState>,
    mut app_exit: MessageWriter<AppExit>,
    mut query: Query<(&MenuOption, &mut TextColor)>,
) {
    // Handle Navigation
    if actions.just_pressed(InputAction::Up) {
        if state.selected_index > 0 {
            state.selected_index -= 1;
        } else {
            state.selected_index = state.options_count - 1;
        }
    }
    if actions.just_pressed(InputAction::Down) {
        state.selected_index = (state.selected_index + 1) % state.options_count;
    }

    // Update Visuals
    for (option, mut color) in query.iter_mut() {
        if option.index == state.selected_index {
            color.0 = Color::srgb(1.0, 1.0, 0.0); // Yellow selected
        } else {
            color.0 = Color::WHITE;
        }
    }

    // Handle Selection
    if actions.just_pressed(InputAction::Confirm) {
        // Find selected action
        let action = query
            .iter()
            .find(|(opt, _)| opt.index == state.selected_index)
            .map(|(opt, _)| opt.action);

        if let Some(act) = action {
            match act {
                MenuAction::NewGame => {
                    info!("Starting New Game");
                    next_state.set(GameState::NarratorDialogue);
                    // TODO: Reset StoryState here?
                }
                MenuAction::Continue => {
                    info!("Continue Game (TODO: Load save)");
                    next_state.set(GameState::Overworld);
                }
                MenuAction::Quit => {
                    app_exit.write(AppExit::Success);
                }
            }
        }
    }
}

fn teardown_title_ui(mut commands: Commands, query: Query<Entity, With<TitleMenu>>) {
    if let Ok(entity) = query.single() {
        commands.entity(entity).despawn();
    }
}
