use super::{player::Player, NPC};
use crate::state::GameState;
use crate::story::StoryState;
use bevy::prelude::*;
use dj_engine::story_graph::{GraphExecutor, StoryGraph, StoryNode};

pub fn interaction_check(
    keys: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut executor: ResMut<GraphExecutor>,
    mut _story: ResMut<StoryState>,
    player_query: Query<&Transform, With<Player>>,
    npc_query: Query<(&Transform, &NPC)>,
) {
    if keys.just_pressed(KeyCode::KeyE) {
        if let Ok(player_transform) = player_query.single() {
            for (npc_transform, npc) in &npc_query {
                let distance = player_transform
                    .translation
                    .distance(npc_transform.translation);
                if distance < 50.0 {
                    info!("Interacting with NPC: {}", npc.id);

                    match npc.id.as_str() {
                        "hamster_narrator" => {
                            let mut graph = StoryGraph::new();

                            // 1. End Node
                            let end = graph.add(StoryNode::End);

                            // 2. Branch: Defeated Glitch Handling
                            // Path A: Already Won
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

                            // Path B: Need to fight
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

                            // 3. Intro Path (if not MetHamster)
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
                                text: "This prototype was scraped from the internet after it caused too much... doom.".to_string(),
                                portrait: None,
                                next: Some(intro3),
                            });
                            let intro1 = graph.add(StoryNode::Dialogue {
                                speaker: "Hamster Narrator".to_string(),
                                text: "Oh you managed to find this lost exe.".to_string(),
                                portrait: None,
                                next: Some(intro2),
                            });

                            // 4. Root Branch (MetHamster?)
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

                            // Path C: Already Defeated
                            let inert = graph.add(StoryNode::Dialogue {
                                speaker: "Glitch".to_string(),
                                text: "The puddle is inert.".to_string(),
                                portrait: None,
                                next: Some(end),
                            });

                            // Path B: Fight! (Trigger Event)
                            let trigger_battle = graph.add(StoryNode::Event {
                                event_id: "StartBattle".to_string(),
                                payload: "".to_string(),
                                next: Some(end),
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

                            // Path A: Not Met Hamster (Warning)
                            let warn2 = graph.add(StoryNode::Dialogue {
                                speaker: "Glitch".to_string(),
                                text: "It seems dangerous to touch without guidance.".to_string(),
                                portrait: None,
                                next: Some(end),
                            });
                            let warn1 = graph.add(StoryNode::Dialogue {
                                speaker: "Glitch".to_string(),
                                text: "It's a writhing mass of corrupted data.".to_string(),
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
            }
        }
    }
}
