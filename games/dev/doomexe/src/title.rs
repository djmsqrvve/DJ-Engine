use crate::state::GameState;
use crate::story::StoryState;
use bevy::prelude::*;
use dj_engine::input::{ActionState, InputAction};
use dj_engine::prelude::{has_save, LoadedSave, StoryFlags, StoryVariables};

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
                    margin: UiRect::bottom(Val::Px(8.0)),
                    ..default()
                },
            ));

            // Subtitle
            parent.spawn((
                Text::new("A Corrupted Hamster Narrator JRPG"),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.5, 0.3, 0.3)),
                Node {
                    margin: UiRect::bottom(Val::Px(6.0)),
                    ..default()
                },
            ));

            // Version
            parent.spawn((
                Text::new("DJ Engine v0.1.0 | CRT Mode"),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(Color::srgb(0.3, 0.3, 0.3)),
                Node {
                    margin: UiRect::bottom(Val::Px(40.0)),
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
    mut next_state: ResMut<NextState<GameState>>,
    mut state: ResMut<TitleState>,
    actions: Res<ActionState>,
    mut app_exit: MessageWriter<AppExit>,
    mut query: Query<(&MenuOption, &mut TextColor)>,
    mut story_state: ResMut<StoryState>,
    mut flags: ResMut<StoryFlags>,
    mut variables: ResMut<StoryVariables>,
    mut loaded_save: ResMut<LoadedSave>,
) {
    let save_exists = has_save(0);

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
            color.0 = Color::srgb(1.0, 1.0, 0.0);
        } else if matches!(option.action, MenuAction::Continue) && !save_exists {
            color.0 = Color::srgb(0.4, 0.4, 0.4);
        } else {
            color.0 = Color::WHITE;
        }
    }

    // Handle Selection
    if actions.just_pressed(InputAction::Confirm) {
        let action = query
            .iter()
            .find(|(opt, _)| opt.index == state.selected_index)
            .map(|(opt, _)| opt.action);

        if let Some(act) = action {
            match act {
                MenuAction::NewGame => {
                    info!("Starting New Game — entering overworld");
                    *story_state = StoryState::default();
                    *flags = StoryFlags::default();
                    *variables = StoryVariables::default();
                    loaded_save.0 = None;
                    next_state.set(GameState::Overworld);
                }
                MenuAction::Continue => {
                    if !save_exists {
                        warn!("No save file found");
                        return;
                    }
                    match dj_engine::save::load_game(0) {
                        Ok(data) => {
                            info!("Loading saved game");
                            flags.0 = data.flags.clone();
                            variables.0 = data.variables.clone();
                            loaded_save.0 = Some(data.clone());
                            let target = match data.game_state.as_str() {
                                "Overworld" => GameState::Overworld,
                                "NarratorDialogue" => GameState::NarratorDialogue,
                                "Battle" => GameState::Battle,
                                _ => GameState::Overworld,
                            };
                            next_state.set(target);
                        }
                        Err(e) => {
                            error!("Failed to load save: {e}");
                        }
                    }
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
