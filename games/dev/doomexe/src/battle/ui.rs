use super::BattleResultEvent;
use bevy::prelude::*;

#[derive(Component)]
pub struct BattleButton(pub BattleResultEvent);

#[derive(Component)]
pub struct BattleUIRoot;

pub fn setup_battle_ui(mut commands: Commands) {
    // Root UI Node
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::FlexEnd,
                justify_content: JustifyContent::Center,
                ..default()
            },
            // Transparent background
            BackgroundColor(Color::NONE),
            BattleUIRoot,
        ))
        .with_children(|parent| {
            // Control Panel
            parent
                .spawn((
                    Node {
                        padding: UiRect::all(Val::Px(10.0)),
                        column_gap: Val::Px(20.0),
                        margin: UiRect::bottom(Val::Px(20.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)),
                ))
                .with_children(|panel| {
                    // Win Button
                    spawn_button(
                        panel,
                        "Simulate WIN",
                        Color::srgb(0.2, 0.8, 0.2),
                        BattleResultEvent::Win,
                    );

                    // Lose Button
                    spawn_button(
                        panel,
                        "Simulate LOSE",
                        Color::srgb(0.8, 0.2, 0.2),
                        BattleResultEvent::Lose,
                    );
                });
        });
}

fn spawn_button(
    parent: &mut bevy::ecs::hierarchy::ChildSpawnerCommands<'_>,
    text: &str,
    color: Color,
    event: BattleResultEvent,
) {
    parent
        .spawn((
            Button,
            Node {
                width: Val::Px(150.0),
                height: Val::Px(50.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(color),
            BattleButton(event),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(text),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

pub fn battle_ui_interaction(
    mut interaction_query: Query<
        (&Interaction, &BattleButton, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
    mut ev_writer: MessageWriter<BattleResultEvent>,
) {
    for (interaction, button_type, mut bg_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                // Dim on press
                bg_color.0.set_alpha(0.5);

                // Fire event based on button type
                match button_type.0 {
                    BattleResultEvent::Win => {
                        ev_writer.write(BattleResultEvent::Win);
                    }
                    BattleResultEvent::Lose => {
                        ev_writer.write(BattleResultEvent::Lose);
                    }
                }
            }
            Interaction::Hovered => {
                // Slight highlight (handled by changing alpha or brightness if we had base color stored)
                bg_color.0.set_alpha(0.9);
            }
            Interaction::None => {
                bg_color.0.set_alpha(1.0);
            }
        }
    }
}

pub fn cleanup_battle_ui(mut commands: Commands, query: Query<Entity, With<BattleUIRoot>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}
