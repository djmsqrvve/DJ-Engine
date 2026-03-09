use super::minimap::MapTarget;
use crate::overworld::NPC;
use crate::story::StoryState;
use bevy::prelude::*;

#[derive(Component)]
pub struct ObjectiveText;

#[derive(Component)]
pub struct TrackerRoot;

pub fn setup_tracker(mut commands: Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(10.0),
                left: Val::Px(10.0),
                padding: UiRect::all(Val::Px(10.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.5)),
            TrackerRoot,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Objective: Loading..."),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                ObjectiveText,
            ));
        });
}

pub fn update_tracker(story: Res<StoryState>, mut query: Query<&mut Text, With<ObjectiveText>>) {
    if story.is_changed() {
        for mut text in &mut query {
            if !story.has_flag("MetHamster") {
                text.0 = "Objective: Find the Narrator (East)".to_string();
            } else if !story.has_flag("DefeatedGlitch") {
                text.0 = "Objective: Investigate Glitch (South-West)".to_string();
            } else {
                text.0 = "Objective: Return to Narrator".to_string();
            }
        }
    }
}

pub fn update_objective_markers(
    mut commands: Commands,
    story: Res<StoryState>,
    npc_query: Query<(Entity, &NPC), Without<MapTarget>>,
    target_query: Query<(Entity, &NPC), With<MapTarget>>,
) {
    if story.is_changed() {
        let target_id = if !story.has_flag("MetHamster") {
            "hamster_narrator"
        } else if !story.has_flag("DefeatedGlitch") {
            "glitch_puddle"
        } else {
            "hamster_narrator"
        };

        // Remove old targets
        for (entity, npc) in &target_query {
            if npc.id != target_id {
                commands.entity(entity).remove::<MapTarget>();
            }
        }

        // Add new target
        for (entity, npc) in &npc_query {
            if npc.id == target_id {
                commands.entity(entity).insert(MapTarget);
            }
        }
    }
}
