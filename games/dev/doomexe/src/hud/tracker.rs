use super::minimap::MapTarget;
use crate::overworld::NPC;
use crate::story::StoryState;
use bevy::prelude::*;
use dj_engine::story_graph::StoryFlags;

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

pub fn update_tracker(
    story: Res<StoryState>,
    flags: Res<StoryFlags>,
    mut query: Query<&mut Text, With<ObjectiveText>>,
) {
    if !story.is_changed() && !flags.is_changed() {
        return;
    }

    // Check both DoomExe's StoryState AND engine's StoryFlags
    let met_hamster =
        story.has_flag("MetHamster") || flags.0.get("MetHamster").copied().unwrap_or(false);
    let defeated_glitch =
        story.has_flag("DefeatedGlitch") || flags.0.get("DefeatedGlitch").copied().unwrap_or(false);

    let intro_complete = flags.0.get("IntroComplete").copied().unwrap_or(false);
    let quest_accepted = flags
        .0
        .get("QuestAccepted_cellar")
        .copied()
        .unwrap_or(false);
    let quest_complete = flags
        .0
        .get("QuestComplete_cellar")
        .copied()
        .unwrap_or(false);
    let quest_turned_in = flags
        .0
        .get("QuestTurnedIn_cellar")
        .copied()
        .unwrap_or(false);

    for mut text in &mut query {
        if !intro_complete {
            text.0 = "Objective: Listen to the Narrator".to_string();
        } else if !met_hamster {
            text.0 = "Objective: Find the Narrator (East)".to_string();
        } else if quest_turned_in {
            text.0 = "Objective: Explore the village".to_string();
        } else if quest_complete {
            text.0 = "Objective: Return to Village Elder".to_string();
        } else if quest_accepted {
            text.0 = "Objective: Clear rats in the cellar (South)".to_string();
        } else if !defeated_glitch {
            text.0 = "Objective: Talk to Village Elder (North-East)".to_string();
        } else {
            text.0 = "Objective: Return to Narrator".to_string();
        }
    }
}

pub fn update_objective_markers(
    mut commands: Commands,
    story: Res<StoryState>,
    flags: Res<StoryFlags>,
    npc_query: Query<(Entity, &NPC), Without<MapTarget>>,
    target_query: Query<(Entity, &NPC), With<MapTarget>>,
) {
    if !story.is_changed() && !flags.is_changed() {
        return;
    }

    let met_hamster =
        story.has_flag("MetHamster") || flags.0.get("MetHamster").copied().unwrap_or(false);
    let defeated_glitch =
        story.has_flag("DefeatedGlitch") || flags.0.get("DefeatedGlitch").copied().unwrap_or(false);

    {
        let target_id = if !met_hamster {
            "hamster_narrator"
        } else if !defeated_glitch {
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
