use crate::state::GameState;
use bevy::prelude::*;
use dj_engine::data::{
    BodyType, CollisionComponent, InteractivityComponent, TriggerType, Vec3Data,
};
use dj_engine::interaction::InteractionSource;
use dj_engine::prelude::{CollisionSet, MovementIntent, SaveData, StoryFlags, StoryVariables};
use dj_engine::rendering::MainCamera;

mod camera;
pub mod interaction;
pub mod player;

pub struct OverworldPlugin;

impl Plugin for OverworldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Overworld), (setup_overworld, auto_save))
            .add_systems(
                Update,
                (
                    player::player_movement.before(CollisionSet::MoveBodies),
                    camera::camera_follow_system.after(CollisionSet::MoveBodies),
                    interaction::interaction_check.after(CollisionSet::DetectTriggers),
                    npc_proximity_highlight,
                )
                    .run_if(in_state(GameState::Overworld)),
            )
            .add_systems(OnExit(GameState::Overworld), teardown_overworld);
    }
}

#[derive(Component)]
pub struct OverworldEntity; // Marker for cleanup

#[derive(Component)]
#[allow(clippy::upper_case_acronyms)]
pub struct NPC {
    pub id: String,
}

fn setup_overworld(
    mut commands: Commands,
    mut camera_query: Query<(Entity, &mut Projection), With<MainCamera>>,
) {
    // Configure existing Main Camera
    if let Ok((entity, mut projection)) = camera_query.single_mut() {
        if let Projection::Orthographic(ortho) = &mut *projection {
            ortho.scale = 2.0;
        }
        commands.entity(entity).insert(camera::CameraFollow);
    }

    // Player (Blue Square)
    commands.spawn((
        Name::new("Player"),
        Sprite {
            color: Color::srgb(0.2, 0.2, 0.8),
            custom_size: Some(Vec2::new(32.0, 32.0)),
            ..default()
        },
        Transform::from_xyz(-180.0, 0.0, 10.0),
        player::Player { speed: 150.0 },
        MovementIntent::default(),
        InteractionSource,
        CollisionComponent {
            body_type: BodyType::Kinematic,
            box_size: Some(Vec3Data::new(28.0, 28.0, 0.0)),
            ..Default::default()
        },
        OverworldEntity,
    ));

    spawn_npc(
        &mut commands,
        "Hamster Narrator",
        "hamster_narrator",
        Color::srgb(0.5, 0.3, 0.1),
        Vec3::new(-120.0, 60.0, 10.0),
    );
    spawn_npc(
        &mut commands,
        "Glitch Puddle",
        "glitch_puddle",
        Color::srgb(0.8, 0.2, 0.8),
        Vec3::new(150.0, -80.0, 10.0),
    );

    // Central blocker used to prove collision resolution in the overworld.
    commands.spawn((
        Name::new("Corrupted Wall"),
        Sprite {
            color: Color::srgb(0.3, 0.32, 0.35),
            custom_size: Some(Vec2::new(32.0, 220.0)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 9.0),
        CollisionComponent {
            body_type: BodyType::Static,
            box_size: Some(Vec3Data::new(32.0, 220.0, 0.0)),
            ..Default::default()
        },
        OverworldEntity,
    ));

    // Simple Floor (Dark Gray)
    commands.spawn((
        Name::new("Floor"),
        Sprite {
            color: Color::srgb(0.1, 0.1, 0.1),
            custom_size: Some(Vec2::new(800.0, 600.0)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 0.0),
        OverworldEntity,
    ));
}

fn spawn_npc(commands: &mut Commands, name: &str, id: &str, color: Color, position: Vec3) {
    commands.spawn((
        Name::new(name.to_string()),
        Sprite {
            color,
            custom_size: Some(Vec2::new(32.0, 32.0)),
            ..default()
        },
        Transform::from_translation(position),
        NPC { id: id.to_string() },
        CollisionComponent {
            body_type: BodyType::Static,
            box_size: Some(Vec3Data::new(44.0, 44.0, 0.0)),
            is_trigger: true,
            ..Default::default()
        },
        InteractivityComponent {
            trigger_type: TriggerType::Npc,
            trigger_id: id.to_string(),
            events: dj_engine::data::InteractivityEvents {
                on_interact: Some("start_dialogue".to_string()),
                ..Default::default()
            },
            ..Default::default()
        },
        OverworldEntity,
    ));
}

/// Highlight NPCs when player is within interaction range.
/// NPCs pulse brighter when close, return to normal when far.
fn npc_proximity_highlight(
    player_query: Query<&Transform, With<player::Player>>,
    mut npc_query: Query<(&Transform, &mut Sprite, &NPC), Without<player::Player>>,
) {
    let Ok(player_pos) = player_query.single() else {
        return;
    };

    let highlight_range = 60.0;

    for (npc_pos, mut sprite, npc) in npc_query.iter_mut() {
        let dist = player_pos
            .translation
            .truncate()
            .distance(npc_pos.translation.truncate());

        if dist < highlight_range {
            // Brighten — pulse toward white
            let t = 1.0 - (dist / highlight_range);
            let base = match npc.id.as_str() {
                "hamster_narrator" => Color::srgb(0.5, 0.3, 0.1),
                "glitch_puddle" => Color::srgb(0.8, 0.2, 0.8),
                _ => Color::WHITE,
            };
            let r = base.to_srgba();
            sprite.color = Color::srgb(
                (r.red + t * 0.4).min(1.0),
                (r.green + t * 0.4).min(1.0),
                (r.blue + t * 0.4).min(1.0),
            );
        } else {
            // Reset to base color
            sprite.color = match npc.id.as_str() {
                "hamster_narrator" => Color::srgb(0.5, 0.3, 0.1),
                "glitch_puddle" => Color::srgb(0.8, 0.2, 0.8),
                _ => Color::WHITE,
            };
        }
    }
}

fn teardown_overworld(mut commands: Commands, query: Query<Entity, With<OverworldEntity>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

fn auto_save(flags: Res<StoryFlags>, variables: Res<StoryVariables>) {
    let data = SaveData {
        flags: flags.0.clone(),
        variables: variables.0.clone(),
        current_node: None,
        game_state: "Overworld".into(),
        scene_background: None,
        project_id: None,
        scene_id: None,
        story_graph_id: None,
    };
    if let Err(e) = dj_engine::save::save_game(0, &data) {
        error!("Auto-save failed: {e}");
    }
}
