use bevy::prelude::*;
use dj_engine::prelude::MovementIntent;

#[derive(Component)]
pub struct Player {
    pub speed: f32,
}

pub fn player_movement(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut MovementIntent, &Player)>,
) {
    let mut direction = Vec2::ZERO;
    if keys.pressed(KeyCode::KeyW) || keys.pressed(KeyCode::ArrowUp) {
        direction.y += 1.0;
    }
    if keys.pressed(KeyCode::KeyS) || keys.pressed(KeyCode::ArrowDown) {
        direction.y -= 1.0;
    }
    if keys.pressed(KeyCode::KeyA) || keys.pressed(KeyCode::ArrowLeft) {
        direction.x -= 1.0;
    }
    if keys.pressed(KeyCode::KeyD) || keys.pressed(KeyCode::ArrowRight) {
        direction.x += 1.0;
    }

    let direction = direction.normalize_or_zero();
    for (mut intent, player) in &mut query {
        intent.0 = direction * player.speed * time.delta_secs();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::SystemState;

    type PlayerMovementState = SystemState<(
        Res<'static, Time>,
        Res<'static, ButtonInput<KeyCode>>,
        Query<'static, 'static, (&'static mut MovementIntent, &'static Player)>,
    )>;

    #[test]
    fn test_player_movement_sets_intent_from_input() {
        let mut world = World::new();
        world.insert_resource(Time::<()>::default());
        world.insert_resource(ButtonInput::<KeyCode>::default());

        let entity = world
            .spawn((Player { speed: 120.0 }, MovementIntent::default()))
            .id();

        world
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::KeyD);
        world
            .resource_mut::<Time<()>>()
            .advance_by(std::time::Duration::from_secs_f32(0.5));

        let mut system_state: PlayerMovementState = SystemState::new(&mut world);
        let (time, keys, query) = system_state.get_mut(&mut world);
        player_movement(time, keys, query);

        let intent = world.entity(entity).get::<MovementIntent>().unwrap();
        assert!((intent.0.x - 60.0).abs() < f32::EPSILON);
        assert_eq!(intent.0.y, 0.0);
    }
}
