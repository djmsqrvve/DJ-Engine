use crate::dialogue::DialogueUiState;
use crate::state::GameState;
use bevy::prelude::*;
use dj_engine::input::{ActionState, InputAction};
use dj_engine::story_graph::{StoryFlowEvent, StoryInputEvent};

#[derive(Component)]
pub struct DialogueUI;

#[derive(Component)]
pub struct DialogueText;

#[derive(Component)]
pub struct SpeakerText;

#[derive(Component)]
pub struct PortraitImage;

#[derive(Component)]
pub struct ChoiceButton {
    pub _index: usize,
}

#[derive(Component)]
pub struct ChoiceSelector;

#[derive(Component)]
pub struct Typewriter {
    pub full_text: String,
    pub current_len: usize,
    pub timer: Timer,
    pub _speed: f32, // Seconds per character
}

pub fn setup_dialogue_ui(mut commands: Commands, _asset_server: Res<AssetServer>) {
    // Main Container
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::FlexEnd,
                display: Display::None,
                ..default()
            },
            DialogueUI,
        ))
        .with_children(|parent| {
            // Layout: Portrait (Left/Right) + Choices (Middle) + Text Box (Bottom)
            // Actually, usually Portrait sits on top of Text Box or side-by-side.
            // Let's make a container for the bottom area.

            // Choice Container (Above Text Box)
            parent.spawn((
                Node {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    margin: UiRect::bottom(Val::Px(10.0)),
                    ..default()
                },
                ChoiceSelector,
            ));

            // Dialog Area Container (Portrait + Text)
            parent
                .spawn((Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(30.0),
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::FlexEnd,
                    padding: UiRect::all(Val::Px(20.0)),
                    ..default()
                },))
                .with_children(|area| {
                    // Portrait Box
                    area.spawn((
                        Node {
                            width: Val::Px(150.0),
                            height: Val::Px(150.0),
                            margin: UiRect::right(Val::Px(20.0)),
                            ..default()
                        },
                        ImageNode::default(), // Bevy 0.15 Use ImageNode or ImageBundle?
                        // In 0.15 ImageBundle -> ImageNode component mostly.
                        // Wait, let's stick to standard bundles if possible or just ImageNode.
                        // Checking `bevy::ui::node_bundles` might be deprecated?
                        // Let's use `ImageNode` component directly if new API.
                        // Actually, let's use standard pattern: Node + ImageNode
                        PortraitImage,
                        // Typically invisible if no portrait
                        BackgroundColor(Color::NONE),
                    ));

                    // Text Box (The rest of width)
                    area.spawn((
                        Node {
                            flex_grow: 1.0,
                            height: Val::Percent(100.0),
                            flex_direction: FlexDirection::Column,
                            padding: UiRect::all(Val::Px(20.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.9)),
                    ))
                    .with_children(|box_parent| {
                        // Speaker Name
                        box_parent.spawn((
                            Text::new(""),
                            TextFont {
                                font_size: 20.0,
                                ..default()
                            },
                            TextColor(Color::srgb(1.0, 0.8, 0.2)),
                            SpeakerText,
                            Node {
                                margin: UiRect::bottom(Val::Px(10.0)),
                                ..default()
                            },
                        ));

                        // Dialogue Content
                        box_parent.spawn((
                            Text::new(""),
                            TextFont {
                                font_size: 24.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                            DialogueText,
                            Typewriter {
                                full_text: "".to_string(),
                                current_len: 0,
                                timer: Timer::from_seconds(0.02, TimerMode::Repeating), // Fast text
                                _speed: 0.02,
                            },
                        ));
                    });
                });
        });
}

pub fn update_dialogue_ui(
    mut commands: Commands,
    mut events: MessageReader<StoryFlowEvent>,
    mut ui_state: ResMut<DialogueUiState>,
    mut ui_query: Query<&mut Node, With<DialogueUI>>,
    mut text_query: Query<(&mut Text, &mut Typewriter), With<DialogueText>>,
    mut speaker_query: Query<&mut Text, (With<SpeakerText>, Without<DialogueText>)>,
    mut portrait_query: Query<&mut BackgroundColor, With<PortraitImage>>,
    choice_query: Query<Entity, With<ChoiceSelector>>,
    mut next_state: ResMut<NextState<GameState>>,
    battle_pending: Res<crate::story::BattlePending>,
) {
    for event in events.read() {
        match event {
            StoryFlowEvent::ShowDialogue {
                speaker,
                text,
                portrait,
            } => {
                ui_state.visible = true;
                ui_state.is_choice_mode = false;
                ui_state.speaker = speaker.clone();
                ui_state.text = text.clone();
                ui_state.choices.clear();

                // Show UI
                if let Ok(mut node) = ui_query.single_mut() {
                    node.display = Display::Flex;
                }

                // Reset Typewriter
                for (mut t, mut writer) in text_query.iter_mut() {
                    t.0 = "".to_string(); // Start empty
                    writer.full_text = text.clone();
                    writer.current_len = 0;
                    writer.timer.reset();
                }

                // Update Speaker
                for mut s in speaker_query.iter_mut() {
                    s.0 = speaker.clone();
                }

                // Update Portrait — procedural color portraits for named characters
                for mut bg in portrait_query.iter_mut() {
                    if let Some(id) = portrait {
                        bg.0 = match id.as_str() {
                            "hamster" => Color::srgb(0.6, 0.4, 0.2), // brown
                            "elder" => Color::srgb(0.2, 0.6, 0.3),   // green
                            "vendor" => Color::srgb(0.5, 0.35, 0.2), // dark brown
                            "system" => Color::srgb(0.3, 0.0, 0.5),  // purple
                            _ => Color::srgb(0.3, 0.3, 0.3),         // gray fallback
                        };
                    } else {
                        bg.0 = Color::NONE;
                    }
                }

                // Clear choices
                if let Ok(container) = choice_query.single() {
                    commands.entity(container).despawn_children();
                }
            }
            StoryFlowEvent::ShowChoices { prompt, options } => {
                ui_state.visible = true;
                ui_state.is_choice_mode = true;
                ui_state.prompt = prompt.clone();
                ui_state.choices = options.clone();
                ui_state.selected_index = 0;

                if let Ok(mut node) = ui_query.single_mut() {
                    node.display = Display::Flex;
                }

                // Text (Prompt) - Instant show, no typewriter for prompt usually?
                // Or yes? Let's Instant show for prompt to avoid delay.
                for (mut t, mut writer) in text_query.iter_mut() {
                    t.0 = prompt.clone();
                    writer.full_text = prompt.clone();
                    writer.current_len = prompt.len(); // Skip effect
                }

                // Render choices
                if let Ok(container) = choice_query.single() {
                    spawn_choices(&mut commands, container, options, 0);
                }
            }
            StoryFlowEvent::GraphComplete => {
                ui_state.visible = false;
                if let Ok(mut node) = ui_query.single_mut() {
                    node.display = Display::None;
                }
                // Don't return to Overworld if a battle was triggered by a
                // story event in this or a previous frame.
                if battle_pending.0 {
                    info!("STATE: NarratorDialogue -> Battle (via BattlePending)");
                } else {
                    info!("STATE: NarratorDialogue -> Overworld (graph complete)");
                    next_state.set(GameState::Overworld);
                }
            }
        }
    }
}

pub fn typewriter_system(time: Res<Time>, mut query: Query<(&mut Text, &mut Typewriter)>) {
    for (mut text, mut writer) in query.iter_mut() {
        if writer.current_len < writer.full_text.len() {
            writer.timer.tick(time.delta());
            if writer.timer.is_finished() {
                writer.current_len += 1;
                // Determine byte index for safe slicing (UTF-8)
                // writer.full_text.chars().take(writer.current_len).collect() is slow.
                // Better: maintain byte index. For now, simple char iteration.
                let display_text: String =
                    writer.full_text.chars().take(writer.current_len).collect();
                text.0 = display_text;
            }
        }
    }
}

// Helper to spawn choices
fn spawn_choices(commands: &mut Commands, parent: Entity, choices: &[String], selected_idx: usize) {
    commands.entity(parent).despawn_children();

    commands.entity(parent).with_children(|p| {
        for (i, text) in choices.iter().enumerate() {
            let is_selected = i == selected_idx;
            let color = if is_selected {
                Color::srgb(1.0, 1.0, 0.0)
            } else {
                Color::WHITE
            };
            let bg_color = if is_selected {
                Color::srgba(0.2, 0.2, 0.2, 0.9)
            } else {
                Color::srgba(0.0, 0.0, 0.0, 0.8)
            };

            p.spawn((
                Node {
                    width: Val::Percent(80.0),
                    padding: UiRect::all(Val::Px(10.0)),
                    margin: UiRect::all(Val::Px(5.0)),
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                BackgroundColor(bg_color),
                ChoiceButton { _index: i },
            ))
            .with_children(|btn| {
                btn.spawn((
                    Text::new(text),
                    TextFont {
                        font_size: 20.0,
                        ..default()
                    },
                    TextColor(color),
                ));
            });
        }
    });
}

pub fn dialogue_input(
    mut commands: Commands,
    actions: Res<ActionState>,
    mut ui_state: ResMut<DialogueUiState>,
    mut input_events: MessageWriter<StoryInputEvent>,
    mut text_query: Query<(&mut Text, &mut Typewriter), With<DialogueText>>,
    choice_query: Query<Entity, With<ChoiceSelector>>,
) {
    if !ui_state.visible {
        if actions.just_pressed(InputAction::Confirm) {
            info!("Dialogue: Space pressed but UI not visible yet");
        }
        return;
    }

    if ui_state.is_choice_mode {
        // Handle Choice Navigation
        let count = ui_state.choices.len();
        if count > 0 {
            if actions.just_pressed(InputAction::Up) {
                if ui_state.selected_index > 0 {
                    ui_state.selected_index -= 1;
                } else {
                    ui_state.selected_index = count - 1;
                }
                if let Ok(container) = choice_query.single() {
                    spawn_choices(
                        &mut commands,
                        container,
                        &ui_state.choices,
                        ui_state.selected_index,
                    );
                }
            }
            if actions.just_pressed(InputAction::Down) {
                ui_state.selected_index = (ui_state.selected_index + 1) % count;
                if let Ok(container) = choice_query.single() {
                    spawn_choices(
                        &mut commands,
                        container,
                        &ui_state.choices,
                        ui_state.selected_index,
                    );
                }
            }
            if actions.just_pressed(InputAction::Confirm) {
                input_events.write(StoryInputEvent::SelectChoice(ui_state.selected_index));
            }
        }
    } else {
        // Handle Dialogue Advance
        if actions.just_pressed(InputAction::Confirm) {
            // Check if typewriter finished?
            let mut all_finished = true;
            for (mut text, mut writer) in text_query.iter_mut() {
                if writer.current_len < writer.full_text.len() {
                    // Fast forward
                    writer.current_len = writer.full_text.len();
                    text.0 = writer.full_text.clone();
                    all_finished = false;
                }
            }

            if all_finished {
                info!("Dialogue: advancing to next node");
                input_events.write(StoryInputEvent::Advance);
            } else {
                info!("Dialogue: fast-forwarding typewriter");
            }
        }
    }
}

pub fn teardown_dialogue_ui(mut commands: Commands, query: Query<Entity, With<DialogueUI>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}
