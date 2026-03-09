use crate::overworld::player::Player;
use bevy::prelude::*;

#[derive(Component)]
pub struct MinimapRoot;

#[derive(Component)]
pub struct MinimapPlayerIcon;

#[derive(Component)]
pub struct MinimapObjectiveIcon;

// Waypoint Arrow on the main screen
#[derive(Component)]
pub struct WaypointArrow;

// Generic Marker for the current objective
#[derive(Component)]
pub struct MapTarget;

pub fn setup_minimap(mut commands: Commands) {
    // Minimap Container (Top Right)
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(10.0),
                right: Val::Px(10.0),
                width: Val::Px(150.0),
                height: Val::Px(150.0),
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BorderColor::all(Color::WHITE),
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)),
            MinimapRoot,
        ))
        .with_children(|parent| {
            // Player Icon (Center)
            parent.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Percent(50.0),
                    top: Val::Percent(50.0),
                    width: Val::Px(6.0),
                    height: Val::Px(6.0),
                    margin: UiRect {
                        left: Val::Px(-3.0),
                        top: Val::Px(-3.0),
                        ..default()
                    },
                    ..default()
                },
                BackgroundColor(Color::srgb(0.2, 0.2, 0.8)), // Blue
                MinimapPlayerIcon,
            ));

            // Objective Icon (Relative)
            parent.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    width: Val::Px(6.0),
                    height: Val::Px(6.0),
                    // Default hidden until target exists
                    display: Display::None,
                    ..default()
                },
                BackgroundColor(Color::srgb(1.0, 1.0, 0.0)), // Yellow
                MinimapObjectiveIcon,
            ));
        });

    // Waypoint Arrow (Center Screen overlay)
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: Val::Percent(50.0),
            top: Val::Percent(50.0),
            width: Val::Px(20.0),
            height: Val::Px(20.0),
            display: Display::None,
            ..default()
        },
        BackgroundColor(Color::srgb(1.0, 0.0, 0.0)), // Red Arrow
        WaypointArrow,
    ));
}

pub fn update_minimap_and_waypoint(
    player_query: Query<&Transform, With<Player>>,
    target_query: Query<&Transform, With<MapTarget>>,
    mut minimap_obj_query: Query<&mut Node, (With<MinimapObjectiveIcon>, Without<WaypointArrow>)>,
    mut arrow_query: Query<
        (&mut Node, &mut Transform),
        (
            With<WaypointArrow>,
            Without<Player>,
            Without<MapTarget>,
            Without<MinimapObjectiveIcon>,
        ),
    >,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };

    // Find generic target
    let target_transform = target_query.single().ok();

    // Toggle Visibility based on if target exists
    let display_mode = if target_transform.is_some() {
        Display::Flex
    } else {
        Display::None
    };

    for mut node in &mut minimap_obj_query {
        node.display = display_mode;
    }
    for (mut node, _) in &mut arrow_query {
        node.display = display_mode;
    }

    // If no target, we are done
    let Some(target_t) = target_transform else {
        return;
    };
    let delta = target_t.translation - player_transform.translation;

    // --- Update Minimap Dot ---
    // Scale down world coordinates to map pixels (e.g., 1 world unit = 0.5 map pixels)
    // Map is 150x150, Center is 75,75.
    let map_scale: f32 = 0.5;
    let map_x: f32 = 75.0 + (delta.x * map_scale);
    let map_y: f32 = 75.0 - (delta.y * map_scale); // UI +Top is Down.

    for mut node in &mut minimap_obj_query {
        // Clamp to map bounds (0-150)
        let clamped_x = map_x.clamp(0.0, 144.0);
        let clamped_y = map_y.clamp(0.0, 144.0);

        node.left = Val::Px(clamped_x);
        node.top = Val::Px(clamped_y);
    }

    // --- Update Waypoint Arrow ---
    let dir = delta.truncate().normalize_or_zero();
    let angle = dir.y.atan2(dir.x);

    for (_, mut transform) in &mut arrow_query {
        transform.rotation = Quat::from_rotation_z(angle);

        // Offset from center
        transform.translation = Vec3::new(dir.x * 60.0, dir.y * 60.0, 0.0);
    }
}
