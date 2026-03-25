use super::{player::Player, NPC};
use crate::state::GameState;
use crate::story::StoryState;
use bevy::prelude::*;
use dj_engine::data::InteractivityComponent;
use dj_engine::prelude::TriggerContacts;
use dj_engine::story_graph::{GraphExecutor, StoryGraph, StoryNode};

pub fn interaction_check(
    keys: Res<ButtonInput<KeyCode>>,
    trigger_contacts: Res<TriggerContacts>,
    mut next_state: ResMut<NextState<GameState>>,
    mut executor: ResMut<GraphExecutor>,
    mut _story: ResMut<StoryState>,
    player_query: Query<(Entity, &Transform), With<Player>>,
    interaction_query: Query<(
        Entity,
        &Transform,
        Option<&NPC>,
        Option<&InteractivityComponent>,
    )>,
) {
    if !keys.just_pressed(KeyCode::KeyE) {
        return;
    }

    let Ok((player_entity, player_transform)) = player_query.single() else {
        return;
    };

    let Some(target) = select_interaction_target(
        player_entity,
        player_transform,
        &trigger_contacts,
        &interaction_query,
    ) else {
        return;
    };

    let Ok((_entity, _transform, npc, _interaction)) = interaction_query.get(target) else {
        return;
    };
    let Some(npc) = npc else {
        return;
    };

    info!("Interacting with NPC: {}", npc.id);

    match npc.id.as_str() {
        "hamster_narrator" => {
            let mut graph = StoryGraph::new();

            let end = graph.add(StoryNode::End);

            let win2 = graph.add(StoryNode::Dialogue {
                speaker: "Hamster Narrator".to_string(),
                text: "But the corruption runs deeper...".to_string(),
                portrait: None,
                next: Some(end),
            });
            let win1 = graph.add(StoryNode::Dialogue {
                speaker: "Hamster Narrator".to_string(),
                text: "Incredible! You purged the glitch.".to_string(),
                portrait: None,
                next: Some(win2),
            });

            let quest2 = graph.add(StoryNode::Dialogue {
                speaker: "Hamster Narrator".to_string(),
                text: "Go investigate that purple puddle.".to_string(),
                portrait: None,
                next: Some(end),
            });
            let quest1 = graph.add(StoryNode::Dialogue {
                speaker: "Hamster Narrator".to_string(),
                text: "There is a corruption to the south-west.".to_string(),
                portrait: None,
                next: Some(quest2),
            });

            let branch_glitch = graph.add(StoryNode::Branch {
                flag: "DefeatedGlitch".to_string(),
                if_true: Some(win1),
                if_false: Some(quest1),
            });

            let set_met = graph.add(StoryNode::SetFlag {
                flag: "MetHamster".to_string(),
                value: true,
                next: Some(end),
            });
            let intro3 = graph.add(StoryNode::Dialogue {
                speaker: "Hamster Narrator".to_string(),
                text: "I am the Narrator. I will guide you.".to_string(),
                portrait: None,
                next: Some(set_met),
            });
            let intro2 = graph.add(StoryNode::Dialogue {
                speaker: "Hamster Narrator".to_string(),
                text:
                    "This prototype was scraped from the internet after it caused too much... doom."
                        .to_string(),
                portrait: None,
                next: Some(intro3),
            });
            let intro1 = graph.add(StoryNode::Dialogue {
                speaker: "Hamster Narrator".to_string(),
                text: "Oh you managed to find this lost exe.".to_string(),
                portrait: None,
                next: Some(intro2),
            });

            let root = graph.add(StoryNode::Branch {
                flag: "MetHamster".to_string(),
                if_true: Some(branch_glitch),
                if_false: Some(intro1),
            });

            graph.set_start(root);
            executor.start(graph);
            next_state.set(GameState::NarratorDialogue);
        }
        "glitch_puddle" => {
            let mut graph = StoryGraph::new();
            let end = graph.add(StoryNode::End);

            let inert = graph.add(StoryNode::Dialogue {
                speaker: "Glitch".to_string(),
                text: "The puddle is inert.".to_string(),
                portrait: None,
                next: Some(end),
            });

            let trigger_battle = graph.add(StoryNode::Event {
                event_id: "StartBattle".to_string(),
                payload: "".to_string(),
                next: None, // Don't continue to End — battle state change handles transition
            });
            let battle_warn = graph.add(StoryNode::Dialogue {
                speaker: "System".to_string(),
                text: "Initiating Battle Protocol...".to_string(),
                portrait: None,
                next: Some(trigger_battle),
            });
            let screech = graph.add(StoryNode::Dialogue {
                speaker: "Glitch".to_string(),
                text: "The glitch screeches!".to_string(),
                portrait: None,
                next: Some(battle_warn),
            });

            let branch_victory = graph.add(StoryNode::Branch {
                flag: "DefeatedGlitch".to_string(),
                if_true: Some(inert),
                if_false: Some(screech),
            });

            let warn2 = graph.add(StoryNode::Dialogue {
                speaker: "System".to_string(),
                text: "Talk to the Hamster Narrator first. He might know what to do."
                    .to_string(),
                portrait: None,
                next: Some(end),
            });
            let warn1 = graph.add(StoryNode::Dialogue {
                speaker: "Glitch".to_string(),
                text: "A writhing mass of corrupted data. Too dangerous to touch alone."
                    .to_string(),
                portrait: None,
                next: Some(warn2),
            });

            let root = graph.add(StoryNode::Branch {
                flag: "MetHamster".to_string(),
                if_true: Some(branch_victory),
                if_false: Some(warn1),
            });

            graph.set_start(root);
            executor.start(graph);
            next_state.set(GameState::NarratorDialogue);
        }
        _ => {}
    }
}

fn select_interaction_target(
    player_entity: Entity,
    player_transform: &Transform,
    trigger_contacts: &TriggerContacts,
    interaction_query: &Query<(
        Entity,
        &Transform,
        Option<&NPC>,
        Option<&InteractivityComponent>,
    )>,
) -> Option<Entity> {
    let candidates = trigger_contacts
        .contacts_for(player_entity)
        .iter()
        .filter_map(|entity| {
            let Ok((entity, transform, npc, interaction)) = interaction_query.get(*entity) else {
                return None;
            };
            if npc.is_none() && interaction.is_none() {
                return None;
            }

            Some(InteractionCandidate {
                entity,
                distance_squared: player_transform
                    .translation
                    .distance_squared(transform.translation),
                stable_key: stable_contact_key(entity, npc, interaction),
            })
        })
        .collect::<Vec<_>>();

    choose_best_candidate(candidates)
}

#[derive(Debug, Clone, PartialEq)]
struct InteractionCandidate {
    entity: Entity,
    distance_squared: f32,
    stable_key: String,
}

fn choose_best_candidate(mut candidates: Vec<InteractionCandidate>) -> Option<Entity> {
    candidates.sort_by(|left, right| {
        left.distance_squared
            .total_cmp(&right.distance_squared)
            .then_with(|| left.stable_key.cmp(&right.stable_key))
            .then_with(|| left.entity.to_bits().cmp(&right.entity.to_bits()))
    });
    candidates.first().map(|candidate| candidate.entity)
}

fn stable_contact_key(
    entity: Entity,
    npc: Option<&NPC>,
    interaction: Option<&InteractivityComponent>,
) -> String {
    if let Some(npc) = npc {
        return npc.id.clone();
    }
    if let Some(interaction) = interaction {
        return interaction.trigger_id.clone();
    }
    entity.to_bits().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_choose_best_candidate_prefers_nearest() {
        let candidates = vec![
            InteractionCandidate {
                entity: Entity::from_bits(1),
                distance_squared: 100.0,
                stable_key: "b".into(),
            },
            InteractionCandidate {
                entity: Entity::from_bits(2),
                distance_squared: 25.0,
                stable_key: "a".into(),
            },
        ];
        assert_eq!(
            choose_best_candidate(candidates),
            Some(Entity::from_bits(2))
        );
    }

    #[test]
    fn test_choose_best_candidate_tie_breaks_deterministically() {
        let candidates = vec![
            InteractionCandidate {
                entity: Entity::from_bits(5),
                distance_squared: 16.0,
                stable_key: "zeta".into(),
            },
            InteractionCandidate {
                entity: Entity::from_bits(7),
                distance_squared: 16.0,
                stable_key: "alpha".into(),
            },
        ];
        assert_eq!(
            choose_best_candidate(candidates),
            Some(Entity::from_bits(7))
        );
    }
}
