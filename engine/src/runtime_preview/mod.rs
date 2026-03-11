//! Engine-owned runtime preview for mounted projects.
//!
//! This module provides a generic playable loop for manifest-driven projects:
//! title screen, startup dialogue, and startup scene preview.

use crate::audio::AudioCommand;
use crate::collision::{CollisionSet, MovementIntent};
use crate::data::loader;
use crate::data::spawner::{LoadedScene, SceneEntityMarker};
use crate::data::{BodyType, CollisionComponent, DataError, Scene, StoryGraphData, Vec3Data};
use crate::input::{ActionState, InputAction};
use crate::project_mount::{
    load_mounted_project_manifest, resolve_startup_scene_ref, resolve_startup_story_graph_ref,
    MountedProject,
};
use crate::rendering::MainCamera;
use crate::scene::ChangeSceneEvent;
use crate::scripting::ScriptCommand;
use crate::story_graph::{
    GraphExecutor, StoryFlags, StoryFlowEvent, StoryInputEvent, StoryVariables,
};
use bevy::app::AppExit;
use bevy::prelude::*;
use std::path::{Path, PathBuf};

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum PreviewState {
    #[default]
    Title,
    Dialogue,
    Overworld,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RuntimePreviewCliOptions {
    pub project_path: Option<PathBuf>,
    pub test_mode: bool,
}

pub fn parse_runtime_preview_cli_args(
    args: impl IntoIterator<Item = String>,
) -> RuntimePreviewCliOptions {
    let args: Vec<String> = args.into_iter().collect();
    let mut options = RuntimePreviewCliOptions::default();
    let mut positional_project = None;

    let mut index = 1;
    while index < args.len() {
        match args[index].as_str() {
            "--project" => {
                if index + 1 < args.len() {
                    options.project_path = Some(PathBuf::from(&args[index + 1]));
                    index += 1;
                }
            }
            "--test-mode" => {
                options.test_mode = true;
            }
            arg if !arg.starts_with("--") && positional_project.is_none() => {
                positional_project = Some(PathBuf::from(arg));
            }
            _ => {}
        }
        index += 1;
    }

    if options.project_path.is_none() {
        options.project_path = positional_project;
    }

    options
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TitleAction {
    NewGame,
    Quit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NewGameTransition {
    Dialogue,
    Overworld,
    StayOnTitle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DialogueFlowResolution {
    StayInDialogue,
    ToOverworld,
    ToTitle,
}

#[derive(Resource, Default, Debug, Clone, PartialEq, Eq)]
struct TitleMenuState {
    selected_index: usize,
}

#[derive(Resource, Default, Debug, Clone, PartialEq, Eq)]
struct PreviewStatus {
    message: Option<String>,
}

#[derive(Resource, Default, Debug, Clone, PartialEq)]
struct PreviewStartupContent {
    scene: Option<Scene>,
    story_graph: Option<StoryGraphData>,
    entry_script_path: Option<PathBuf>,
}

#[derive(Resource, Default, Debug, Clone, PartialEq, Eq)]
struct DialoguePresentation {
    visible: bool,
    speaker: String,
    text: String,
    portrait: Option<String>,
    prompt: String,
    choices: Vec<String>,
    selected_index: usize,
}

impl DialoguePresentation {
    fn is_choice_mode(&self) -> bool {
        !self.choices.is_empty()
    }
}

#[derive(Resource)]
struct AutomatedPreviewTest {
    timer: Timer,
}

#[derive(Component)]
struct TitleRoot;

#[derive(Component)]
struct TitleOption {
    index: usize,
    action: TitleAction,
}

#[derive(Component)]
struct TitleStatusText;

#[derive(Component)]
struct DialogueRoot;

#[derive(Component)]
struct DialogueSpeakerText;

#[derive(Component)]
struct DialogueBodyText;

#[derive(Component)]
struct DialoguePortraitNode;

#[derive(Component)]
struct DialogueChoicesContainer;

#[derive(Component)]
pub struct PreviewPlayer;

#[derive(Component, Debug, Clone, Copy, PartialEq, Reflect)]
#[reflect(Component)]
pub struct PreviewPlayerController {
    pub speed: f32,
}

#[derive(Component)]
struct PreviewCameraFollow;

pub struct RuntimePreviewPlugin {
    pub test_mode: bool,
}

impl RuntimePreviewPlugin {
    pub fn new(test_mode: bool) -> Self {
        Self { test_mode }
    }
}

impl Default for RuntimePreviewPlugin {
    fn default() -> Self {
        Self::new(false)
    }
}

impl Plugin for RuntimePreviewPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<PreviewState>()
            .init_resource::<MountedProject>()
            .init_resource::<LoadedScene>()
            .init_resource::<ActionState>()
            .init_resource::<TitleMenuState>()
            .init_resource::<PreviewStatus>()
            .init_resource::<PreviewStartupContent>()
            .init_resource::<DialoguePresentation>()
            .init_resource::<GraphExecutor>()
            .init_resource::<StoryFlags>()
            .init_resource::<StoryVariables>()
            .register_type::<PreviewPlayerController>()
            .add_message::<AudioCommand>()
            .add_message::<ChangeSceneEvent>()
            .add_message::<ScriptCommand>()
            .add_systems(Startup, prepare_runtime_preview_system)
            .add_systems(OnEnter(PreviewState::Title), setup_title_ui)
            .add_systems(
                Update,
                (update_title_ui, title_input_system).run_if(in_state(PreviewState::Title)),
            )
            .add_systems(OnExit(PreviewState::Title), cleanup_title_ui)
            .add_systems(
                OnEnter(PreviewState::Dialogue),
                (setup_dialogue_ui, start_dialogue_preview),
            )
            .add_systems(
                Update,
                (
                    handle_story_flow_events,
                    dialogue_input_system,
                    update_dialogue_ui,
                )
                    .run_if(in_state(PreviewState::Dialogue)),
            )
            .add_systems(OnExit(PreviewState::Dialogue), cleanup_dialogue_ui)
            .add_systems(OnEnter(PreviewState::Overworld), setup_overworld_preview)
            .add_systems(
                Update,
                (
                    preview_player_movement.before(CollisionSet::MoveBodies),
                    preview_camera_follow_system.after(CollisionSet::MoveBodies),
                )
                    .run_if(in_state(PreviewState::Overworld)),
            )
            .add_systems(OnExit(PreviewState::Overworld), cleanup_overworld_preview);

        if self.test_mode {
            app.insert_resource(AutomatedPreviewTest {
                timer: Timer::from_seconds(0.15, TimerMode::Repeating),
            })
            .add_systems(Update, automated_preview_test_system);
        }
    }
}

fn determine_new_game_transition(has_story_graph: bool, has_scene: bool) -> NewGameTransition {
    if has_story_graph {
        NewGameTransition::Dialogue
    } else if has_scene {
        NewGameTransition::Overworld
    } else {
        NewGameTransition::StayOnTitle
    }
}

fn apply_story_flow_event(
    presentation: &mut DialoguePresentation,
    event: &StoryFlowEvent,
    has_startup_scene: bool,
) -> DialogueFlowResolution {
    match event {
        StoryFlowEvent::ShowDialogue {
            speaker,
            text,
            portrait,
        } => {
            presentation.visible = true;
            presentation.speaker = speaker.clone();
            presentation.text = text.clone();
            presentation.portrait = portrait.clone();
            presentation.prompt.clear();
            presentation.choices.clear();
            presentation.selected_index = 0;
            DialogueFlowResolution::StayInDialogue
        }
        StoryFlowEvent::ShowChoices { prompt, options } => {
            presentation.visible = true;
            presentation.prompt = prompt.clone();
            presentation.text = prompt.clone();
            presentation.choices = options.clone();
            presentation.selected_index = 0;
            DialogueFlowResolution::StayInDialogue
        }
        StoryFlowEvent::GraphComplete => {
            presentation.visible = false;
            if has_startup_scene {
                DialogueFlowResolution::ToOverworld
            } else {
                DialogueFlowResolution::ToTitle
            }
        }
    }
}

fn compute_preview_movement_vector(
    up: bool,
    down: bool,
    left: bool,
    right: bool,
    speed: f32,
    delta_seconds: f32,
) -> Vec2 {
    let mut direction = Vec2::ZERO;
    if up {
        direction.y += 1.0;
    }
    if down {
        direction.y -= 1.0;
    }
    if left {
        direction.x -= 1.0;
    }
    if right {
        direction.x += 1.0;
    }

    direction.normalize_or_zero() * speed * delta_seconds
}

fn follow_camera_translation(current_camera: Vec3, target: Vec3) -> Vec3 {
    Vec3::new(target.x, target.y, current_camera.z)
}

fn load_preview_startup_content(
    mounted_project: &MountedProject,
) -> Result<PreviewStartupContent, DataError> {
    let Some(root_path) = mounted_project.root_path.as_ref() else {
        return Ok(PreviewStartupContent::default());
    };
    let Some(project) = mounted_project.project.as_ref() else {
        return Ok(PreviewStartupContent::default());
    };

    let scene = resolve_startup_scene_ref(project).and_then(|scene_ref| {
        let scene_path = root_path.join(&scene_ref.path);
        match loader::load_scene(&scene_path) {
            Ok(scene) => Some(scene),
            Err(error) => {
                warn!(
                    "Runtime Preview: Failed to load startup scene {:?}: {}",
                    scene_path, error
                );
                None
            }
        }
    });

    let story_graph = resolve_startup_story_graph_ref(project).and_then(|graph_ref| {
        let graph_path = root_path.join(&graph_ref.path);
        match loader::load_story_graph(&graph_path) {
            Ok(graph) => Some(graph),
            Err(error) => {
                warn!(
                    "Runtime Preview: Failed to load startup story graph {:?}: {}",
                    graph_path, error
                );
                None
            }
        }
    });

    let entry_script_path = project
        .settings
        .startup
        .entry_script
        .as_deref()
        .map(|path| root_path.join(path));

    Ok(PreviewStartupContent {
        scene,
        story_graph,
        entry_script_path,
    })
}

fn prepare_runtime_preview_system(
    mut mounted_project: ResMut<MountedProject>,
    mut startup_content: ResMut<PreviewStartupContent>,
    mut status: ResMut<PreviewStatus>,
) {
    if mounted_project.manifest_path.is_none() {
        status.message =
            Some("No project mounted. Launch with --project <dir|project.json>.".into());
        return;
    }

    if mounted_project.project.is_none() {
        if let Err(error) = load_mounted_project_manifest(&mut mounted_project) {
            warn!("Runtime Preview: Failed to load mounted project: {}", error);
            status.message = Some(format!("Failed to load mounted project: {error}"));
            return;
        }
    }

    match load_preview_startup_content(&mounted_project) {
        Ok(content) => {
            *startup_content = content;
            status.message = None;
            if let Some(project) = mounted_project.project.as_ref() {
                info!("Runtime Preview: Mounted project '{}'", project.name);
            }
        }
        Err(error) => {
            warn!(
                "Runtime Preview: Failed to resolve startup content: {}",
                error
            );
            status.message = Some(format!("Failed to resolve startup content: {error}"));
        }
    }
}

fn setup_title_ui(
    mut commands: Commands,
    mounted_project: Res<MountedProject>,
    mut menu_state: ResMut<TitleMenuState>,
) {
    menu_state.selected_index = 0;

    let project_name = mounted_project
        .project
        .as_ref()
        .map(|project| project.name.clone())
        .unwrap_or_else(|| "No Project Mounted".to_string());
    let resolution_label = mounted_project
        .project
        .as_ref()
        .map(|project| {
            format!(
                "{}x{}",
                project.settings.default_resolution.width,
                project.settings.default_resolution.height
            )
        })
        .unwrap_or_else(|| "Resolution unavailable".to_string());

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
            TitleRoot,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(project_name),
                TextFont {
                    font_size: 48.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.95, 1.0)),
                Node {
                    margin: UiRect::bottom(Val::Px(8.0)),
                    ..default()
                },
            ));

            parent.spawn((
                Text::new(format!("Runtime Preview  {resolution_label}")),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgb(0.5, 0.8, 0.85)),
                Node {
                    margin: UiRect::bottom(Val::Px(28.0)),
                    ..default()
                },
            ));

            spawn_title_option(parent, "NEW GAME", 0, TitleAction::NewGame);
            spawn_title_option(parent, "QUIT", 1, TitleAction::Quit);

            parent.spawn((
                Text::new(""),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.95, 0.45, 0.45)),
                Node {
                    margin: UiRect::top(Val::Px(24.0)),
                    ..default()
                },
                TitleStatusText,
            ));
        });
}

fn spawn_title_option(
    parent: &mut bevy::ecs::hierarchy::ChildSpawnerCommands<'_>,
    label: &str,
    index: usize,
    action: TitleAction,
) {
    parent.spawn((
        Text::new(label),
        TextFont {
            font_size: 28.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            margin: UiRect::all(Val::Px(8.0)),
            ..default()
        },
        TitleOption { index, action },
    ));
}

fn update_title_ui(
    menu_state: Res<TitleMenuState>,
    status: Res<PreviewStatus>,
    mut option_query: Query<(&TitleOption, &mut TextColor)>,
    mut status_query: Query<&mut Text, With<TitleStatusText>>,
) {
    for (option, mut color) in &mut option_query {
        color.0 = if option.index == menu_state.selected_index {
            Color::srgb(1.0, 0.92, 0.35)
        } else {
            Color::WHITE
        };
    }

    let message = status.message.clone().unwrap_or_default();
    for mut text in &mut status_query {
        text.0 = message.clone();
    }
}

fn dispatch_entry_script(
    startup_content: &PreviewStartupContent,
    script_events: &mut MessageWriter<ScriptCommand>,
) {
    let Some(script_path) = startup_content.entry_script_path.as_ref() else {
        return;
    };

    if !script_path.exists() {
        warn!(
            "Runtime Preview: Configured entry script {:?} was not found; continuing without scripting.",
            script_path
        );
        return;
    }

    info!("Runtime Preview: Loading entry script {:?}", script_path);
    script_events.write(ScriptCommand::Load {
        path: script_path.to_string_lossy().into_owned(),
    });
}

fn start_new_game_preview(
    startup_content: &PreviewStartupContent,
    status: &mut PreviewStatus,
    next_state: &mut NextState<PreviewState>,
    executor: &mut GraphExecutor,
    flags: &mut StoryFlags,
    variables: &mut StoryVariables,
    loaded_scene: &mut LoadedScene,
    dialogue_presentation: &mut DialoguePresentation,
    script_events: &mut MessageWriter<ScriptCommand>,
) {
    *executor = GraphExecutor::default();
    *flags = StoryFlags::default();
    *variables = StoryVariables::default();
    *loaded_scene = LoadedScene::default();
    *dialogue_presentation = DialoguePresentation::default();

    dispatch_entry_script(startup_content, script_events);

    match determine_new_game_transition(
        startup_content.story_graph.is_some(),
        startup_content.scene.is_some(),
    ) {
        NewGameTransition::Dialogue => {
            status.message = None;
            next_state.set(PreviewState::Dialogue);
        }
        NewGameTransition::Overworld => {
            status.message = None;
            next_state.set(PreviewState::Overworld);
        }
        NewGameTransition::StayOnTitle => {
            status.message = Some(
                "Mounted project is missing both a startup story graph and startup scene.".into(),
            );
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn title_input_system(
    actions: Res<ActionState>,
    mut menu_state: ResMut<TitleMenuState>,
    startup_content: Res<PreviewStartupContent>,
    mut status: ResMut<PreviewStatus>,
    mut next_state: ResMut<NextState<PreviewState>>,
    mut executor: ResMut<GraphExecutor>,
    mut flags: ResMut<StoryFlags>,
    mut variables: ResMut<StoryVariables>,
    mut loaded_scene: ResMut<LoadedScene>,
    mut dialogue_presentation: ResMut<DialoguePresentation>,
    mut script_events: MessageWriter<ScriptCommand>,
    option_query: Query<&TitleOption>,
    mut app_exit: MessageWriter<AppExit>,
) {
    let options_count = option_query.iter().count().max(1);

    if actions.just_pressed(InputAction::Up) {
        if menu_state.selected_index == 0 {
            menu_state.selected_index = options_count - 1;
        } else {
            menu_state.selected_index -= 1;
        }
    }

    if actions.just_pressed(InputAction::Down) {
        menu_state.selected_index = (menu_state.selected_index + 1) % options_count;
    }

    if !actions.just_pressed(InputAction::Confirm) {
        return;
    }

    let selected_action = option_query
        .iter()
        .find(|option| option.index == menu_state.selected_index)
        .map(|option| option.action)
        .unwrap_or(TitleAction::NewGame);

    match selected_action {
        TitleAction::NewGame => start_new_game_preview(
            &startup_content,
            &mut status,
            &mut next_state,
            &mut executor,
            &mut flags,
            &mut variables,
            &mut loaded_scene,
            &mut dialogue_presentation,
            &mut script_events,
        ),
        TitleAction::Quit => {
            app_exit.write(AppExit::Success);
        }
    }
}

fn cleanup_title_ui(mut commands: Commands, query: Query<Entity, With<TitleRoot>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

fn setup_dialogue_ui(mut commands: Commands) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::FlexEnd,
                ..default()
            },
            DialogueRoot,
        ))
        .with_children(|parent| {
            parent
                .spawn((Node {
                    width: Val::Percent(100.0),
                    padding: UiRect::all(Val::Px(20.0)),
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::FlexStart,
                    ..default()
                },))
                .with_children(|content| {
                    content.spawn((
                        Node {
                            width: Val::Px(128.0),
                            height: Val::Px(128.0),
                            margin: UiRect::right(Val::Px(16.0)),
                            ..default()
                        },
                        ImageNode::default(),
                        BackgroundColor(Color::NONE),
                        DialoguePortraitNode,
                    ));

                    content
                        .spawn((
                            Node {
                                flex_grow: 1.0,
                                padding: UiRect::all(Val::Px(20.0)),
                                flex_direction: FlexDirection::Column,
                                ..default()
                            },
                            BackgroundColor(Color::srgba(0.02, 0.02, 0.04, 0.92)),
                        ))
                        .with_children(|text_box| {
                            text_box.spawn((
                                Text::new(""),
                                TextFont {
                                    font_size: 20.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.95, 0.85, 0.45)),
                                Node {
                                    margin: UiRect::bottom(Val::Px(12.0)),
                                    ..default()
                                },
                                DialogueSpeakerText,
                            ));

                            text_box.spawn((
                                Text::new(""),
                                TextFont {
                                    font_size: 24.0,
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                                Node {
                                    margin: UiRect::bottom(Val::Px(16.0)),
                                    ..default()
                                },
                                DialogueBodyText,
                            ));

                            text_box.spawn((
                                Node {
                                    width: Val::Percent(100.0),
                                    flex_direction: FlexDirection::Column,
                                    ..default()
                                },
                                DialogueChoicesContainer,
                            ));
                        });
                });
        });
}

fn start_dialogue_preview(
    startup_content: Res<PreviewStartupContent>,
    mut dialogue_presentation: ResMut<DialoguePresentation>,
    mut executor: ResMut<GraphExecutor>,
    mut next_state: ResMut<NextState<PreviewState>>,
    mut status: ResMut<PreviewStatus>,
) {
    *dialogue_presentation = DialoguePresentation::default();

    let Some(graph) = startup_content.story_graph.as_ref() else {
        status.message = Some("No startup story graph is available for dialogue preview.".into());
        next_state.set(PreviewState::Title);
        return;
    };

    executor.load_from_data(graph);
}

fn handle_story_flow_events(
    mut events: MessageReader<StoryFlowEvent>,
    startup_content: Res<PreviewStartupContent>,
    mut dialogue_presentation: ResMut<DialoguePresentation>,
    mut next_state: ResMut<NextState<PreviewState>>,
    mut status: ResMut<PreviewStatus>,
) {
    for event in events.read() {
        match apply_story_flow_event(
            &mut dialogue_presentation,
            event,
            startup_content.scene.is_some(),
        ) {
            DialogueFlowResolution::StayInDialogue => {}
            DialogueFlowResolution::ToOverworld => {
                status.message = None;
                next_state.set(PreviewState::Overworld);
            }
            DialogueFlowResolution::ToTitle => {
                status.message = Some(
                    "Startup story graph completed but no startup scene is configured.".into(),
                );
                next_state.set(PreviewState::Title);
            }
        }
    }
}

fn dialogue_input_system(
    actions: Res<ActionState>,
    mut dialogue_presentation: ResMut<DialoguePresentation>,
    mut input_events: MessageWriter<StoryInputEvent>,
) {
    if !dialogue_presentation.visible {
        return;
    }

    if dialogue_presentation.is_choice_mode() {
        if actions.just_pressed(InputAction::Up) && !dialogue_presentation.choices.is_empty() {
            if dialogue_presentation.selected_index == 0 {
                dialogue_presentation.selected_index = dialogue_presentation.choices.len() - 1;
            } else {
                dialogue_presentation.selected_index -= 1;
            }
        }

        if actions.just_pressed(InputAction::Down) && !dialogue_presentation.choices.is_empty() {
            dialogue_presentation.selected_index =
                (dialogue_presentation.selected_index + 1) % dialogue_presentation.choices.len();
        }

        if actions.just_pressed(InputAction::Confirm) {
            input_events.write(StoryInputEvent::SelectChoice(
                dialogue_presentation.selected_index,
            ));
        }
    } else if actions.just_pressed(InputAction::Confirm) {
        input_events.write(StoryInputEvent::Advance);
    }
}

fn update_dialogue_ui(
    dialogue_presentation: Res<DialoguePresentation>,
    mut commands: Commands,
    asset_server: Option<Res<AssetServer>>,
    mut speaker_query: Query<&mut Text, With<DialogueSpeakerText>>,
    mut body_query: Query<&mut Text, (With<DialogueBodyText>, Without<DialogueSpeakerText>)>,
    mut portrait_query: Query<(&mut ImageNode, &mut BackgroundColor), With<DialoguePortraitNode>>,
    container_query: Query<Entity, With<DialogueChoicesContainer>>,
) {
    if !dialogue_presentation.is_changed() {
        return;
    }

    for mut speaker in &mut speaker_query {
        speaker.0 = dialogue_presentation.speaker.clone();
    }

    for mut body in &mut body_query {
        body.0 = if dialogue_presentation.is_choice_mode() {
            dialogue_presentation.prompt.clone()
        } else {
            dialogue_presentation.text.clone()
        };
    }

    for (mut portrait, mut background) in &mut portrait_query {
        if let Some(path) = dialogue_presentation.portrait.as_ref() {
            if let Some(asset_server) = asset_server.as_ref() {
                portrait.image = asset_server.load(path.to_string());
                background.0 = Color::WHITE;
            } else {
                background.0 = Color::NONE;
            }
        } else {
            background.0 = Color::NONE;
        }
    }

    if let Ok(container) = container_query.single() {
        commands.entity(container).despawn_children();

        if dialogue_presentation.is_choice_mode() {
            commands.entity(container).with_children(|parent| {
                for (index, choice) in dialogue_presentation.choices.iter().enumerate() {
                    let is_selected = index == dialogue_presentation.selected_index;
                    parent.spawn((
                        Text::new(choice.clone()),
                        TextFont {
                            font_size: 20.0,
                            ..default()
                        },
                        TextColor(if is_selected {
                            Color::srgb(1.0, 0.92, 0.35)
                        } else {
                            Color::WHITE
                        }),
                        Node {
                            margin: UiRect::all(Val::Px(4.0)),
                            ..default()
                        },
                    ));
                }
            });
        }
    }
}

fn cleanup_dialogue_ui(
    mut commands: Commands,
    query: Query<Entity, With<DialogueRoot>>,
    mut dialogue_presentation: ResMut<DialoguePresentation>,
) {
    *dialogue_presentation = DialoguePresentation::default();
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

fn setup_overworld_preview(
    mut commands: Commands,
    startup_content: Res<PreviewStartupContent>,
    mut loaded_scene: ResMut<LoadedScene>,
    mut camera_query: Query<(Entity, &mut Transform), With<MainCamera>>,
) {
    let Some(scene) = startup_content.scene.clone() else {
        warn!("Runtime Preview: No startup scene available for overworld preview");
        return;
    };

    let spawn = scene.default_spawn.clone();
    *loaded_scene = LoadedScene::new(scene);

    let player_translation = Vec3::new(spawn.player.x, spawn.player.y, spawn.player.z.max(10.0));

    commands.spawn((
        Name::new("Preview Player"),
        Sprite {
            color: Color::srgb(0.2, 0.8, 1.0),
            custom_size: Some(Vec2::new(28.0, 28.0)),
            ..default()
        },
        Transform::from_translation(player_translation),
        PreviewPlayer,
        PreviewPlayerController { speed: 180.0 },
        MovementIntent::default(),
        CollisionComponent {
            body_type: BodyType::Kinematic,
            box_size: Some(Vec3Data::new(28.0, 28.0, 0.0)),
            ..Default::default()
        },
    ));

    if let Ok((camera_entity, mut camera_transform)) = camera_query.single_mut() {
        camera_transform.translation = follow_camera_translation(
            camera_transform.translation,
            Vec3::new(spawn.camera.x, spawn.camera.y, spawn.camera.z),
        );
        commands.entity(camera_entity).insert(PreviewCameraFollow);
    }
}

fn preview_player_movement(
    time: Res<Time>,
    actions: Res<ActionState>,
    mut query: Query<(&PreviewPlayerController, &mut MovementIntent), With<PreviewPlayer>>,
) {
    for (controller, mut intent) in &mut query {
        intent.0 = compute_preview_movement_vector(
            actions.pressed(InputAction::Up),
            actions.pressed(InputAction::Down),
            actions.pressed(InputAction::Left),
            actions.pressed(InputAction::Right),
            controller.speed,
            time.delta_secs(),
        );
    }
}

fn preview_camera_follow_system(
    player_query: Query<&Transform, (With<PreviewPlayer>, Without<PreviewCameraFollow>)>,
    mut camera_query: Query<&mut Transform, With<PreviewCameraFollow>>,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };

    for mut camera_transform in &mut camera_query {
        camera_transform.translation =
            follow_camera_translation(camera_transform.translation, player_transform.translation);
    }
}

fn cleanup_overworld_preview(
    mut commands: Commands,
    preview_player_query: Query<Entity, With<PreviewPlayer>>,
    scene_entity_query: Query<Entity, With<SceneEntityMarker>>,
    camera_query: Query<Entity, With<PreviewCameraFollow>>,
    mut loaded_scene: ResMut<LoadedScene>,
) {
    *loaded_scene = LoadedScene::default();

    for entity in &preview_player_query {
        commands.entity(entity).despawn();
    }

    for entity in &scene_entity_query {
        commands.entity(entity).despawn();
    }

    for entity in &camera_query {
        commands.entity(entity).remove::<PreviewCameraFollow>();
    }
}

#[allow(clippy::too_many_arguments)]
fn automated_preview_test_system(
    time: Res<Time>,
    state: Res<State<PreviewState>>,
    mut automation: ResMut<AutomatedPreviewTest>,
    startup_content: Res<PreviewStartupContent>,
    mut status: ResMut<PreviewStatus>,
    mut next_state: ResMut<NextState<PreviewState>>,
    mut executor: ResMut<GraphExecutor>,
    mut flags: ResMut<StoryFlags>,
    mut variables: ResMut<StoryVariables>,
    mut loaded_scene: ResMut<LoadedScene>,
    mut dialogue_presentation: ResMut<DialoguePresentation>,
    mut story_input: MessageWriter<StoryInputEvent>,
    mut script_events: MessageWriter<ScriptCommand>,
    mut app_exit: MessageWriter<AppExit>,
    preview_player_query: Query<Entity, With<PreviewPlayer>>,
) {
    automation.timer.tick(time.delta());
    if !automation.timer.is_finished() {
        return;
    }

    match state.get() {
        PreviewState::Title => start_new_game_preview(
            &startup_content,
            &mut status,
            &mut next_state,
            &mut executor,
            &mut flags,
            &mut variables,
            &mut loaded_scene,
            &mut dialogue_presentation,
            &mut script_events,
        ),
        PreviewState::Dialogue => {
            if dialogue_presentation.is_choice_mode() {
                story_input.write(StoryInputEvent::SelectChoice(0));
            } else if dialogue_presentation.visible {
                story_input.write(StoryInputEvent::Advance);
            }
        }
        PreviewState::Overworld => {
            if !preview_player_query.is_empty() && loaded_scene.scene.is_some() {
                app_exit.write(AppExit::Success);
            }
        }
    }
}

pub fn bootstrap_mounted_project(path: &Path) -> Result<MountedProject, DataError> {
    let mut mounted_project = MountedProject::from_path(path)?;
    let _ = load_mounted_project_manifest(&mut mounted_project)?;
    Ok(mounted_project)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collision::CollisionPlugin;
    use crate::data::components::{EntityComponents, TransformComponent};
    use crate::data::scene::Entity as SceneEntity;
    use crate::data::story::StoryNodeData;
    use crate::data::{loader, Project};
    use crate::story_graph::StoryGraphPlugin;
    use bevy::ecs::message::Messages;
    use bevy::ecs::system::SystemState;
    use std::time::Duration;

    #[test]
    fn test_parse_runtime_preview_cli_args_supports_positional_project_path() {
        let cli = parse_runtime_preview_cli_args([
            "runtime_preview".into(),
            "projects/sample".into(),
            "--test-mode".into(),
        ]);

        assert_eq!(cli.project_path, Some(PathBuf::from("projects/sample")));
        assert!(cli.test_mode);
    }

    #[test]
    fn test_parse_runtime_preview_cli_args_prefers_project_flag() {
        let cli = parse_runtime_preview_cli_args([
            "runtime_preview".into(),
            "projects/ignored".into(),
            "--project".into(),
            "projects/explicit".into(),
        ]);

        assert_eq!(cli.project_path, Some(PathBuf::from("projects/explicit")));
    }

    #[test]
    fn test_determine_new_game_transition_prefers_dialogue() {
        assert_eq!(
            determine_new_game_transition(true, true),
            NewGameTransition::Dialogue
        );
    }

    #[test]
    fn test_determine_new_game_transition_falls_back_to_overworld() {
        assert_eq!(
            determine_new_game_transition(false, true),
            NewGameTransition::Overworld
        );
    }

    #[test]
    fn test_determine_new_game_transition_stays_on_title_without_startup_content() {
        assert_eq!(
            determine_new_game_transition(false, false),
            NewGameTransition::StayOnTitle
        );
    }

    #[test]
    fn test_apply_story_flow_event_updates_dialogue_state() {
        let mut presentation = DialoguePresentation::default();
        let outcome = apply_story_flow_event(
            &mut presentation,
            &StoryFlowEvent::ShowDialogue {
                speaker: "Guide".into(),
                text: "Welcome".into(),
                portrait: Some("portraits/guide.png".into()),
            },
            true,
        );

        assert_eq!(outcome, DialogueFlowResolution::StayInDialogue);
        assert!(presentation.visible);
        assert_eq!(presentation.speaker, "Guide");
        assert_eq!(presentation.text, "Welcome");
        assert_eq!(
            presentation.portrait.as_deref(),
            Some("portraits/guide.png")
        );
    }

    #[test]
    fn test_apply_story_flow_event_updates_choices() {
        let mut presentation = DialoguePresentation::default();
        let outcome = apply_story_flow_event(
            &mut presentation,
            &StoryFlowEvent::ShowChoices {
                prompt: "Choose".into(),
                options: vec!["A".into(), "B".into()],
            },
            true,
        );

        assert_eq!(outcome, DialogueFlowResolution::StayInDialogue);
        assert!(presentation.visible);
        assert_eq!(presentation.prompt, "Choose");
        assert_eq!(presentation.choices, vec!["A".to_string(), "B".to_string()]);
    }

    #[test]
    fn test_apply_story_flow_event_graph_complete_moves_to_overworld() {
        let mut presentation = DialoguePresentation::default();
        let outcome =
            apply_story_flow_event(&mut presentation, &StoryFlowEvent::GraphComplete, true);

        assert_eq!(outcome, DialogueFlowResolution::ToOverworld);
    }

    #[test]
    fn test_apply_story_flow_event_graph_complete_returns_to_title_without_scene() {
        let mut presentation = DialoguePresentation::default();
        let outcome =
            apply_story_flow_event(&mut presentation, &StoryFlowEvent::GraphComplete, false);

        assert_eq!(outcome, DialogueFlowResolution::ToTitle);
    }

    #[test]
    fn test_compute_preview_movement_vector_diagonal() {
        let movement = compute_preview_movement_vector(true, false, false, true, 120.0, 0.5);
        let expected = Vec2::new(1.0, 1.0).normalize() * 60.0;
        assert!((movement.x - expected.x).abs() < f32::EPSILON);
        assert!((movement.y - expected.y).abs() < f32::EPSILON);
    }

    #[test]
    fn test_follow_camera_translation_preserves_depth() {
        let camera = Vec3::new(0.0, 0.0, 999.0);
        let target = Vec3::new(32.0, -12.0, 4.0);
        assert_eq!(
            follow_camera_translation(camera, target),
            Vec3::new(32.0, -12.0, 999.0)
        );
    }

    #[test]
    fn test_setup_overworld_preview_spawns_player_at_default_spawn() {
        let mut world = World::new();
        world.init_resource::<LoadedScene>();
        world.insert_resource(PreviewStartupContent {
            scene: Some(test_scene_with_spawn(Vec3::new(48.0, -24.0, 0.0))),
            ..Default::default()
        });

        let mut system_state: SystemState<(
            Commands,
            Res<PreviewStartupContent>,
            ResMut<LoadedScene>,
            Query<(Entity, &mut Transform), With<MainCamera>>,
        )> = SystemState::new(&mut world);

        let (commands, startup_content, loaded_scene, camera_query) =
            system_state.get_mut(&mut world);
        setup_overworld_preview(commands, startup_content, loaded_scene, camera_query);
        system_state.apply(&mut world);

        let mut query = world.query::<(&Transform, &PreviewPlayer)>();
        let (transform, _) = query.single(&world).unwrap();
        assert_eq!(transform.translation.x, 48.0);
        assert_eq!(transform.translation.y, -24.0);

        let loaded_scene = world.resource::<LoadedScene>();
        assert!(loaded_scene.scene.is_some());
    }

    #[test]
    fn test_preview_player_collision_uses_engine_collision_runtime() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(CollisionPlugin);

        app.world_mut().spawn((
            Transform::default(),
            PreviewPlayer,
            MovementIntent(Vec2::new(40.0, 0.0)),
            CollisionComponent {
                body_type: BodyType::Kinematic,
                box_size: Some(Vec3Data::new(20.0, 20.0, 0.0)),
                ..Default::default()
            },
        ));
        app.world_mut().spawn((
            Transform::from_xyz(24.0, 0.0, 0.0),
            CollisionComponent {
                body_type: BodyType::Static,
                box_size: Some(Vec3Data::new(20.0, 20.0, 0.0)),
                ..Default::default()
            },
        ));

        app.update();

        let mut query = app
            .world_mut()
            .query_filtered::<&Transform, With<PreviewPlayer>>();
        let transform = query.single(app.world()).unwrap();
        assert!(transform.translation.x < 40.0);
    }

    #[test]
    fn test_runtime_preview_mounts_project_and_reaches_overworld() {
        let temp_dir = tempfile::tempdir().unwrap();
        let root_path = temp_dir.path().to_path_buf();
        let manifest_path = root_path.join("project.json");

        std::fs::create_dir_all(root_path.join("scenes")).unwrap();
        std::fs::create_dir_all(root_path.join("story_graphs")).unwrap();

        let mut project = Project::new("Preview Project");
        project.add_scene("intro", "scenes/intro.json");
        project.add_story_graph("opening", "story_graphs/opening.json");
        project.settings.startup.default_scene_id = Some("intro".into());
        project.settings.startup.default_story_graph_id = Some("opening".into());
        loader::save_project(&project, &manifest_path).unwrap();

        let scene = test_scene_with_spawn(Vec3::new(8.0, 16.0, 0.0));
        loader::save_scene(&scene, &root_path.join("scenes/intro.json")).unwrap();

        let graph = test_story_graph();
        loader::save_story_graph(&graph, &root_path.join("story_graphs/opening.json")).unwrap();

        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(bevy::state::app::StatesPlugin);
        app.add_plugins(StoryGraphPlugin);
        app.add_plugins(RuntimePreviewPlugin::new(false));
        app.insert_resource(MountedProject {
            root_path: Some(root_path),
            manifest_path: Some(manifest_path),
            project: None,
        });

        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(Duration::from_millis(16));
        app.update();

        let startup_content = app.world().resource::<PreviewStartupContent>();
        assert!(startup_content.scene.is_some());
        assert!(startup_content.story_graph.is_some());

        app.world_mut()
            .resource_mut::<NextState<PreviewState>>()
            .set(PreviewState::Dialogue);
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(Duration::from_millis(16));
        app.update();

        app.world_mut()
            .resource_mut::<Messages<StoryInputEvent>>()
            .write(StoryInputEvent::Advance);
        for _ in 0..4 {
            app.world_mut()
                .resource_mut::<Time>()
                .advance_by(Duration::from_millis(16));
            app.update();
        }

        assert_eq!(
            *app.world().resource::<State<PreviewState>>().get(),
            PreviewState::Overworld
        );
        assert!(app.world().resource::<LoadedScene>().scene.is_some());

        let mut player_query = app
            .world_mut()
            .query_filtered::<Entity, With<PreviewPlayer>>();
        assert!(player_query.iter(app.world()).next().is_some());
    }

    fn test_scene_with_spawn(player_spawn: Vec3) -> Scene {
        let mut scene = Scene::new("intro", "Intro");
        scene.default_spawn.player = Vec3Data::new(player_spawn.x, player_spawn.y, player_spawn.z);
        scene.entities.push(
            SceneEntity::new("prop".to_string(), "Prop".to_string()).with_components(
                EntityComponents {
                    transform: TransformComponent {
                        position: Vec3Data::new(0.0, 0.0, 0.0),
                        rotation: Vec3Data::default(),
                        scale: Vec3Data::new(1.0, 1.0, 1.0),
                        lock_uniform_scale: false,
                    },
                    ..Default::default()
                },
            ),
        );
        scene
    }

    fn test_story_graph() -> StoryGraphData {
        let mut graph = StoryGraphData::new("opening", "Opening");
        graph.root_node_id = "start".into();
        graph.add_node(StoryNodeData::start("start", Some("dialogue")));
        let mut dialogue = StoryNodeData::dialogue("dialogue", "Guide", "Welcome");
        dialogue.data.set_next_node_id("end".into());
        graph.add_node(dialogue);
        graph.add_node(StoryNodeData::end("end"));
        graph
    }
}
