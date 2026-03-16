//! Piece definitions for Stratego-lite.

use serde::{Deserialize, Serialize};

/// Piece rank — higher numeric value wins in combat.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum PieceRank {
    Flag,       // 0 — capture target, cannot move
    Scout,      // 3 — moves any distance in a straight line
    Sergeant,   // 4
    Lieutenant, // 5
    Captain,    // 6
    Major,      // 7
    Colonel,    // 8
    General,    // 9
    Marshal,    // 10 — strongest
}

impl PieceRank {
    pub fn strength(self) -> u8 {
        match self {
            Self::Flag => 0,
            Self::Scout => 3,
            Self::Sergeant => 4,
            Self::Lieutenant => 5,
            Self::Captain => 6,
            Self::Major => 7,
            Self::Colonel => 8,
            Self::General => 9,
            Self::Marshal => 10,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Flag => "F",
            Self::Scout => "Sc",
            Self::Sergeant => "Sg",
            Self::Lieutenant => "Lt",
            Self::Captain => "Cp",
            Self::Major => "Mj",
            Self::Colonel => "Co",
            Self::General => "Gn",
            Self::Marshal => "Ma",
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Flag => "Flag",
            Self::Scout => "Scout",
            Self::Sergeant => "Sergeant",
            Self::Lieutenant => "Lieutenant",
            Self::Captain => "Captain",
            Self::Major => "Major",
            Self::Colonel => "Colonel",
            Self::General => "General",
            Self::Marshal => "Marshal",
        }
    }

    pub fn can_move(self) -> bool {
        self != Self::Flag
    }
}

/// Which team a piece belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Team {
    Red,
    Blue,
}

impl Team {
    pub fn opponent(self) -> Self {
        match self {
            Self::Red => Self::Blue,
            Self::Blue => Self::Red,
        }
    }
}

/// A piece placed on the board.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlacedPiece {
    pub rank: PieceRank,
    pub team: Team,
    pub revealed: bool,
}

/// How many of each piece rank each team gets.
pub fn army_composition() -> Vec<(PieceRank, usize)> {
    vec![
        (PieceRank::Marshal, 1),
        (PieceRank::General, 1),
        (PieceRank::Colonel, 2),
        (PieceRank::Major, 3),
        (PieceRank::Captain, 4),
        (PieceRank::Lieutenant, 4),
        (PieceRank::Sergeant, 4),
        (PieceRank::Scout, 5),
        (PieceRank::Flag, 1),
    ]
}

/// Total pieces per team.
pub fn army_size() -> usize {
    army_composition().iter().map(|(_, count)| count).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn army_has_25_pieces() {
        assert_eq!(army_size(), 25);
    }

    #[test]
    fn marshal_beats_general() {
        assert!(PieceRank::Marshal.strength() > PieceRank::General.strength());
    }

    #[test]
    fn flag_cannot_move() {
        assert!(!PieceRank::Flag.can_move());
        assert!(PieceRank::Scout.can_move());
        assert!(PieceRank::Marshal.can_move());
    }

    #[test]
    fn team_opponent_is_symmetric() {
        assert_eq!(Team::Red.opponent(), Team::Blue);
        assert_eq!(Team::Blue.opponent(), Team::Red);
    }
}
