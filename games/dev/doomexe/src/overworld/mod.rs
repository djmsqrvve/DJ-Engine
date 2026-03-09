use crate::state::GameState;
use bevy::prelude::*;

mod camera;
pub mod interaction;
pub mod player;

pub struct OverworldPlugin;

impl Plugin for OverworldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Overworld), setup_overworld)
            .add_systems(
                Update,
                (
                    player::player_movement,
                    interaction::interaction_check,
                    camera::camera_follow_system,
                )
                    .run_if(in_state(GameState::Overworld)),
            )
            .add_systems(OnExit(GameState::Overworld), teardown_overworld);
    }
}

#[derive(Component)]
pub struct OverworldEntity; // Marker for cleanup

#[derive(Component)]
pub struct NPC {
    pub id: String,
}

use dj_engine::rendering::MainCamera;

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
        Sprite {
            color: Color::srgb(0.2, 0.2, 0.8),
            custom_size: Some(Vec2::new(32.0, 32.0)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 10.0),
        player::Player { speed: 150.0 },
        OverworldEntity,
    ));

    // Hamster NPC (Brown Square)
    commands.spawn((
        Sprite {
            color: Color::srgb(0.5, 0.3, 0.1),
            custom_size: Some(Vec2::new(32.0, 32.0)),
            ..default()
        },
        Transform::from_xyz(100.0, 50.0, 10.0),
        NPC {
            id: "hamster_narrator".to_string(),
        },
        OverworldEntity,
    ));

    // Glitch NPC (Purple Square)
    commands.spawn((
        Sprite {
            color: Color::srgb(0.8, 0.2, 0.8),
            custom_size: Some(Vec2::new(32.0, 32.0)),
            ..default()
        },
        Transform::from_xyz(-100.0, -50.0, 10.0),
        NPC {
            id: "glitch_puddle".to_string(),
        },
        OverworldEntity,
    ));

    // Simple Floor (Dark Gray)
    commands.spawn((
        Sprite {
            color: Color::srgb(0.1, 0.1, 0.1),
            custom_size: Some(Vec2::new(800.0, 600.0)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 0.0),
        OverworldEntity,
    ));
}

fn teardown_overworld(mut commands: Commands, query: Query<Entity, With<OverworldEntity>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}
