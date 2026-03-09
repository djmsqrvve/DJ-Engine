use super::player::Player;
use bevy::prelude::*;

#[derive(Component)]
pub struct CameraFollow;

pub fn camera_follow_system(
    player_query: Query<&Transform, (With<Player>, Without<CameraFollow>)>,
    mut camera_query: Query<&mut Transform, With<CameraFollow>>,
) {
    if let Ok(player_transform) = player_query.single() {
        for mut camera_transform in &mut camera_query {
            // Smooth follow could go here, but locked is requested.
            camera_transform.translation.x = player_transform.translation.x;
            camera_transform.translation.y = player_transform.translation.y;
        }
    }
}
