use bevy::prelude::*;

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GameState {
    #[default]
    TitleScreen,
    Overworld,

    // Narrative states
    NarratorDialogue,
    Battle,
}

impl GameState {
    /// Returns valid transition targets from this state.
    /// See docs/STATE_MACHINE.md for the full map.
    pub fn valid_targets(&self) -> &[GameState] {
        match self {
            GameState::TitleScreen => &[GameState::Overworld, GameState::Battle],
            GameState::Overworld => &[GameState::NarratorDialogue],
            GameState::NarratorDialogue => &[GameState::Overworld, GameState::Battle],
            GameState::Battle => &[GameState::Overworld],
        }
    }

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
        // Battle -> Overworld (victory or defeat)
        assert!(GameState::Battle.can_transition_to(GameState::Overworld));
    }

    #[test]
    fn test_invalid_transitions() {
        // Can't go from overworld directly to battle (must go through dialogue)
        assert!(!GameState::Overworld.can_transition_to(GameState::Battle));
        // Can't go from battle to dialogue
        assert!(!GameState::Battle.can_transition_to(GameState::NarratorDialogue));
        // Can't go from battle to title
        assert!(!GameState::Battle.can_transition_to(GameState::TitleScreen));
    }
}
