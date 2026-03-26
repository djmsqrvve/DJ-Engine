use crate::state::GameState;
use bevy::prelude::*;
use dj_engine::input::{ActionState, InputAction};

#[derive(Component)]
struct GameOverUI;

pub struct GameOverPlugin;

impl Plugin for GameOverPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::GameOver), setup_gameover)
            .add_systems(Update, gameover_input.run_if(in_state(GameState::GameOver)))
            .add_systems(OnExit(GameState::GameOver), teardown_gameover);
    }
}

fn setup_gameover(mut commands: Commands) {
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
            BackgroundColor(Color::srgba(0.1, 0.0, 0.0, 0.95)),
            GameOverUI,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("GAME OVER"),
                TextFont {
                    font_size: 64.0,
                    ..default()
                },
                TextColor(Color::srgb(0.8, 0.0, 0.0)),
                Node {
                    margin: UiRect::bottom(Val::Px(30.0)),
                    ..default()
                },
            ));

            parent.spawn((
                Text::new("The corruption consumed you..."),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::srgb(0.6, 0.3, 0.3)),
                Node {
                    margin: UiRect::bottom(Val::Px(40.0)),
                    ..default()
                },
            ));

            parent.spawn((
                Text::new("[Press Space to return to title]"),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.5, 0.5, 0.5)),
            ));
        });

    info!("STATE: GameOver screen displayed");
}

fn gameover_input(actions: Res<ActionState>, mut next_state: ResMut<NextState<GameState>>) {
    if actions.just_pressed(InputAction::Confirm) {
        info!("STATE: GameOver -> TitleScreen");
        next_state.set(GameState::TitleScreen);
    }
}

fn teardown_gameover(mut commands: Commands, query: Query<Entity, With<GameOverUI>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}
