use bevy::prelude::*;

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GameState {
    #[default]
    TitleScreen,
    Overworld,
    Cellar,
    CorruptedGrove,

    // Narrative states
    NarratorDialogue,
    Battle,
    GameOver,
}

impl GameState {
    /// Returns valid transition targets from this state.
    /// See docs/STATE_MACHINE.md for the full map.
    #[cfg(test)]
    pub fn valid_targets(&self) -> &[GameState] {
        match self {
            GameState::TitleScreen => &[GameState::Overworld, GameState::Battle],
            GameState::Overworld => &[GameState::NarratorDialogue],
            GameState::Cellar => &[GameState::Overworld, GameState::Battle, GameState::GameOver],
            GameState::CorruptedGrove => {
                &[GameState::Overworld, GameState::Battle, GameState::GameOver]
            }
            GameState::NarratorDialogue => &[
                GameState::Overworld,
                GameState::Battle,
                GameState::Cellar,
                GameState::CorruptedGrove,
            ],
            GameState::Battle => &[GameState::Overworld, GameState::GameOver],
            GameState::GameOver => &[GameState::TitleScreen],
        }
    }

    #[cfg(test)]
    pub fn can_transition_to(&self, target: GameState) -> bool {
        self.valid_targets().contains(&target)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_state_is_title() {
        assert_eq!(GameState::default(), GameState::TitleScreen);
    }

    #[test]
    fn test_valid_transitions() {
        // Title -> Overworld (new game)
        assert!(GameState::TitleScreen.can_transition_to(GameState::Overworld));
        // Title -> Battle (loading a saved battle)
        assert!(GameState::TitleScreen.can_transition_to(GameState::Battle));
        // Overworld -> Dialogue
        assert!(GameState::Overworld.can_transition_to(GameState::NarratorDialogue));
        // Dialogue -> Battle (via story event)
        assert!(GameState::NarratorDialogue.can_transition_to(GameState::Battle));
        // Dialogue -> Overworld (graph complete)
        assert!(GameState::NarratorDialogue.can_transition_to(GameState::Overworld));
        // Battle -> Overworld (victory)
        assert!(GameState::Battle.can_transition_to(GameState::Overworld));
        // Battle -> GameOver (defeat)
        assert!(GameState::Battle.can_transition_to(GameState::GameOver));
        // GameOver -> TitleScreen (restart)
        assert!(GameState::GameOver.can_transition_to(GameState::TitleScreen));
    }

    #[test]
    fn test_invalid_transitions() {
        // Can't go from overworld directly to battle (must go through dialogue)
        assert!(!GameState::Overworld.can_transition_to(GameState::Battle));
        // Can't go from battle to dialogue
        assert!(!GameState::Battle.can_transition_to(GameState::NarratorDialogue));
        // Can't go from battle to title
        assert!(!GameState::Battle.can_transition_to(GameState::TitleScreen));
        // Can't go from game over to overworld (must go to title)
        assert!(!GameState::GameOver.can_transition_to(GameState::Overworld));
    }
}
