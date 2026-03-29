use super::{player::Player, NPC};
use crate::state::GameState;
use crate::story::StoryState;
use bevy::prelude::*;
use dj_engine::data::InteractivityComponent;
use dj_engine::input::{ActionState, InputAction};
use dj_engine::prelude::TriggerContacts;
use dj_engine::story_graph::{GraphChoice, GraphExecutor, StoryGraph, StoryNode};

pub fn interaction_check(
    actions: Res<ActionState>,
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
    if !actions.just_pressed(InputAction::Confirm) {
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

            // -- Demo complete outro --
            let set_demo = graph.add(StoryNode::Event {
                event_id: "DemoComplete".to_string(),
                payload: "".to_string(),
                next: Some(end),
            });
            let outro3 = graph.add(StoryNode::Dialogue {
                speaker: "Hamster Narrator".to_string(),
                text: "To be continued... probably. If someone doesn't delete this exe first."
                    .to_string(),
                portrait: Some("hamster".to_string()),
                next: Some(set_demo),
            });
            let outro2 = graph.add(StoryNode::Dialogue {
                speaker: "Hamster Narrator".to_string(),
                text: "The corruption runs deeper than a cellar full of rats. But that's a story for another build.".to_string(),
                portrait: Some("hamster".to_string()),
                next: Some(outro3),
            });
            let outro1 = graph.add(StoryNode::Dialogue {
                speaker: "Hamster Narrator".to_string(),
                text: "Not bad for a prototype! You cleared the cellar AND got a fancy title. I'm almost impressed.".to_string(),
                portrait: Some("hamster".to_string()),
                next: Some(outro2),
            });

            // -- Already completed outro (replay) --
            let replay = graph.add(StoryNode::Dialogue {
                speaker: "Hamster Narrator".to_string(),
                text: "You already beat the demo. Go touch grass. ...Or wait for the next build."
                    .to_string(),
                portrait: Some("hamster".to_string()),
                next: Some(end),
            });

            // -- Glitch victory branch --
            let win2 = graph.add(StoryNode::Dialogue {
                speaker: "Hamster Narrator".to_string(),
                text: "But the corruption runs deeper...".to_string(),
                portrait: Some("hamster".to_string()),
                next: Some(end),
            });
            let win1 = graph.add(StoryNode::Dialogue {
                speaker: "Hamster Narrator".to_string(),
                text: "Incredible! You purged the glitch.".to_string(),
                portrait: Some("hamster".to_string()),
                next: Some(win2),
            });

            // -- Quest direction --
            let quest2 = graph.add(StoryNode::Dialogue {
                speaker: "Hamster Narrator".to_string(),
                text: "Talk to the Village Elder to the north-east. He's got a job for you."
                    .to_string(),
                portrait: Some("hamster".to_string()),
                next: Some(end),
            });
            let quest1 = graph.add(StoryNode::Dialogue {
                speaker: "Hamster Narrator".to_string(),
                text: "This village has problems. Rats, corruption, the usual.".to_string(),
                portrait: Some("hamster".to_string()),
                next: Some(quest2),
            });

            let branch_glitch = graph.add(StoryNode::Branch {
                flag: "DefeatedGlitch".to_string(),
                if_true: Some(win1),
                if_false: Some(quest1),
            });

            // -- First meeting --
            let set_met = graph.add(StoryNode::SetFlag {
                flag: "MetHamster".to_string(),
                value: true,
                next: Some(end),
            });
            let intro3 = graph.add(StoryNode::Dialogue {
                speaker: "Hamster Narrator".to_string(),
                text: "I am the Narrator. I will guide you.".to_string(),
                portrait: Some("hamster".to_string()),
                next: Some(set_met),
            });
            let intro2 = graph.add(StoryNode::Dialogue {
                speaker: "Hamster Narrator".to_string(),
                text:
                    "This prototype was scraped from the internet after it caused too much... doom."
                        .to_string(),
                portrait: Some("hamster".to_string()),
                next: Some(intro3),
            });
            let intro1 = graph.add(StoryNode::Dialogue {
                speaker: "Hamster Narrator".to_string(),
                text: "Oh you managed to find this lost exe.".to_string(),
                portrait: Some("hamster".to_string()),
                next: Some(intro2),
            });

            // -- Root: check quest state first, then met hamster --
            let branch_met = graph.add(StoryNode::Branch {
                flag: "MetHamster".to_string(),
                if_true: Some(branch_glitch),
                if_false: Some(intro1),
            });
            let branch_turned_in = graph.add(StoryNode::Branch {
                flag: "QuestTurnedIn_cellar".to_string(),
                if_true: Some(outro1),
                if_false: Some(branch_met),
            });
            let root = graph.add(StoryNode::Branch {
                flag: "DemoComplete".to_string(),
                if_true: Some(replay),
                if_false: Some(branch_turned_in),
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
                portrait: Some("system".to_string()),
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
                text: "Talk to the Hamster Narrator first. He might know what to do.".to_string(),
                portrait: Some("system".to_string()),
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
        "vendor" => {
            let mut graph = StoryGraph::new();
            let end = graph.add(StoryNode::End);

            let bought = graph.add(StoryNode::Dialogue {
                speaker: "Old Ratcatcher".to_string(),
                text: "Here's your potion. Don't drink it all at once.".to_string(),
                portrait: Some("vendor".to_string()),
                next: Some(end),
            });
            let buy = graph.add(StoryNode::Event {
                event_id: "VendorBuy_health_potion".to_string(),
                payload: "".to_string(),
                next: Some(bought),
            });
            let no_gold = graph.add(StoryNode::Dialogue {
                speaker: "Old Ratcatcher".to_string(),
                text: "You can't afford that. Come back when you've got coin.".to_string(),
                portrait: Some("vendor".to_string()),
                next: Some(end),
            });
            let leave = graph.add(StoryNode::Dialogue {
                speaker: "Old Ratcatcher".to_string(),
                text: "Come back when you need supplies.".to_string(),
                portrait: Some("vendor".to_string()),
                next: Some(end),
            });

            // Sell flow
            let sold_tail = graph.add(StoryNode::Dialogue {
                speaker: "Old Ratcatcher".to_string(),
                text: "Rat tail, eh? Here's 3 gold. I know what these are worth.".to_string(),
                portrait: Some("vendor".to_string()),
                next: Some(end),
            });
            let sell_tail = graph.add(StoryNode::Event {
                event_id: "VendorSell_rat_tail".to_string(),
                payload: "".to_string(),
                next: Some(sold_tail),
            });
            let sold_all_tails = graph.add(StoryNode::Dialogue {
                speaker: "Old Ratcatcher".to_string(),
                text: "I'll take the lot. Here's your coin.".to_string(),
                portrait: Some("vendor".to_string()),
                next: Some(end),
            });
            let sell_all_tails = graph.add(StoryNode::Event {
                event_id: "VendorSellAll_rat_tail".to_string(),
                payload: "".to_string(),
                next: Some(sold_all_tails),
            });

            let greeting = graph.add(StoryNode::Choice {
                speaker: "Old Ratcatcher".to_string(),
                prompt: "What'll it be?".to_string(),
                options: vec![
                    GraphChoice {
                        text: "Buy Health Potion (10 gold)".into(),
                        next: Some(buy),
                        flag_required: None,
                    },
                    GraphChoice {
                        text: "Sell Rat Tail (3 gold)".into(),
                        next: Some(sell_tail),
                        flag_required: None,
                    },
                    GraphChoice {
                        text: "Sell All Rat Tails".into(),
                        next: Some(sell_all_tails),
                        flag_required: None,
                    },
                    GraphChoice {
                        text: "Leave".into(),
                        next: Some(leave),
                        flag_required: None,
                    },
                ],
            });
            let intro = graph.add(StoryNode::Dialogue {
                speaker: "Old Ratcatcher".to_string(),
                text: "Potions, supplies, the usual. I used to catch rats for a living, but my knees gave out.".to_string(),
                portrait: Some("vendor".to_string()),
                next: Some(greeting),
            });

            graph.set_start(intro);
            executor.start(graph);
            next_state.set(GameState::NarratorDialogue);
        }
        "village_elder" => {
            let mut graph = StoryGraph::new();
            let end = graph.add(StoryNode::End);

            // After quest complete — turn-in dialogue
            let reward_done = graph.add(StoryNode::Dialogue {
                speaker: "Village Elder".to_string(),
                text: "The village is safe. You've earned your rest, hero.".to_string(),
                portrait: Some("elder".to_string()),
                next: Some(end),
            });
            let branch_turned_in = graph.add(StoryNode::Branch {
                flag: "QuestTurnedIn_cellar".to_string(),
                if_true: Some(reward_done),
                if_false: Some(end), // placeholder — filled below
            });

            let reward_text = graph.add(StoryNode::Dialogue {
                speaker: "Village Elder".to_string(),
                text: "Excellent work! Here — 50 gold and a new title. You've earned it."
                    .to_string(),
                portrait: Some("elder".to_string()),
                next: Some(end),
            });
            let turn_in_event = graph.add(StoryNode::Event {
                event_id: "QuestTurnIn_cellar".to_string(),
                payload: "".to_string(),
                next: Some(reward_text),
            });
            let turn_in_dialog = graph.add(StoryNode::Dialogue {
                speaker: "Village Elder".to_string(),
                text: "You cleared them all? Let me see... yes! The cellar is clean!".to_string(),
                portrait: Some("elder".to_string()),
                next: Some(turn_in_event),
            });
            let branch_complete = graph.add(StoryNode::Branch {
                flag: "QuestComplete_cellar".to_string(),
                if_true: Some(turn_in_dialog),
                if_false: Some(end), // placeholder
            });

            // Quest in progress
            let in_progress = graph.add(StoryNode::Dialogue {
                speaker: "Village Elder".to_string(),
                text: "The cellar still has rats. Get down there and finish the job!".to_string(),
                portrait: Some("elder".to_string()),
                next: Some(end),
            });

            // Quest accept
            let accept_event = graph.add(StoryNode::Event {
                event_id: "QuestAccept_cellar".to_string(),
                payload: "".to_string(),
                next: Some(end),
            });
            let accept2 = graph.add(StoryNode::Dialogue {
                speaker: "Village Elder".to_string(),
                text: "The cellar entrance is to the south. Be careful down there.".to_string(),
                portrait: Some("elder".to_string()),
                next: Some(accept_event),
            });
            let accept1 = graph.add(StoryNode::Dialogue {
                speaker: "Village Elder".to_string(),
                text:
                    "Rats have infested the cellar! Please, clear them out. I'll pay you 50 gold."
                        .to_string(),
                portrait: Some("elder".to_string()),
                next: Some(accept2),
            });

            // Root: branch on quest state
            let branch_accepted = graph.add(StoryNode::Branch {
                flag: "QuestAccepted_cellar".to_string(),
                if_true: Some(branch_complete),
                if_false: Some(accept1),
            });

            // If already turned in, show that. Otherwise check accepted.
            let root = graph.add(StoryNode::Branch {
                flag: "QuestTurnedIn_cellar".to_string(),
                if_true: Some(reward_done),
                if_false: Some(branch_accepted),
            });

            graph.set_start(root);
            executor.start(graph);
            next_state.set(GameState::NarratorDialogue);
        }
        "cellar_entrance" => {
            let mut graph = StoryGraph::new();
            let end = graph.add(StoryNode::End);

            let enter = graph.add(StoryNode::Event {
                event_id: "EnterCellar".to_string(),
                payload: "".to_string(),
                next: None,
            });
            let enter_text = graph.add(StoryNode::Dialogue {
                speaker: "System".to_string(),
                text: "You descend into the dark cellar...".to_string(),
                portrait: Some("system".to_string()),
                next: Some(enter),
            });

            let locked = graph.add(StoryNode::Dialogue {
                speaker: "System".to_string(),
                text: "The cellar is locked. Maybe someone in the village knows what's down there."
                    .to_string(),
                portrait: Some("system".to_string()),
                next: Some(end),
            });

            let root = graph.add(StoryNode::Branch {
                flag: "QuestAccepted_cellar".to_string(),
                if_true: Some(enter_text),
                if_false: Some(locked),
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
