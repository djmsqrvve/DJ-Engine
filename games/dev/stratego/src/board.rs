//! Stratego board using the engine's Grid<T>.

use bevy::prelude::*;
use dj_engine::data::Grid;

use crate::pieces::{army_composition, PlacedPiece, Team};

pub const BOARD_WIDTH: usize = 10;
pub const BOARD_HEIGHT: usize = 10;

/// Terrain type for a cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum CellTerrain {
    #[default]
    Open,
    Lake,
}

/// A single cell on the board.
#[derive(Debug, Clone, PartialEq, Default, serde::Serialize, serde::Deserialize)]
pub struct Cell {
    pub terrain: CellTerrain,
    pub piece: Option<PlacedPiece>,
}

impl Cell {
    pub fn is_open(&self) -> bool {
        self.terrain == CellTerrain::Open
    }

    pub fn is_empty(&self) -> bool {
        self.piece.is_none()
    }

    pub fn is_placeable(&self) -> bool {
        self.is_open() && self.is_empty()
    }
}

/// The Stratego board as a Bevy Resource.
#[derive(Resource, Debug, Clone)]
pub struct StrategoBoard {
    pub grid: Grid<Cell>,
}

impl Default for StrategoBoard {
    fn default() -> Self {
        Self::new()
    }
}

impl StrategoBoard {
    /// Create a fresh 10x10 board with lakes placed.
    pub fn new() -> Self {
        let mut grid = Grid::new(BOARD_WIDTH, BOARD_HEIGHT);

        // Two 2x2 lake zones in the center rows (4-5).
        for &(lx, ly) in &[(2, 4), (3, 4), (2, 5), (3, 5), (6, 4), (7, 4), (6, 5), (7, 5)] {
            grid.set(
                lx,
                ly,
                Cell {
                    terrain: CellTerrain::Lake,
                    piece: None,
                },
            );
        }

        Self { grid }
    }

    pub fn get(&self, x: usize, y: usize) -> Option<&Cell> {
        self.grid.get(x, y)
    }

    pub fn get_mut(&mut self, x: usize, y: usize) -> Option<&mut Cell> {
        self.grid.get_mut(x, y)
    }

    /// Check if a cell is within a team's setup zone.
    pub fn is_setup_zone(&self, x: usize, y: usize, team: Team) -> bool {
        if !self.grid.in_bounds(x, y) {
            return false;
        }
        match team {
            Team::Red => y < 4,
            Team::Blue => y >= 6,
        }
    }

    /// Place a piece during setup.
    pub fn place_piece(&mut self, x: usize, y: usize, piece: PlacedPiece) -> bool {
        if !self.is_setup_zone(x, y, piece.team) {
            return false;
        }
        let Some(cell) = self.grid.get_mut(x, y) else {
            return false;
        };
        if !cell.is_placeable() {
            return false;
        }
        cell.piece = Some(piece);
        true
    }

    /// Auto-fill remaining army pieces randomly into the setup zone.
    pub fn auto_fill_army(&mut self, team: Team) {
        use rand::seq::SliceRandom;

        let mut remaining = Vec::new();
        for (rank, count) in army_composition() {
            let already_placed = self
                .grid
                .iter()
                .filter(|(_, _, cell)| {
                    cell.piece
                        .as_ref()
                        .map(|p| p.team == team && p.rank == rank)
                        .unwrap_or(false)
                })
                .count();
            for _ in already_placed..count {
                remaining.push(rank);
            }
        }

        let mut empty_cells: Vec<(usize, usize)> = self
            .grid
            .iter()
            .filter(|(x, y, cell)| self.is_setup_zone(*x, *y, team) && cell.is_placeable())
            .map(|(x, y, _)| (x, y))
            .collect();

        let mut rng = rand::thread_rng();
        empty_cells.shuffle(&mut rng);

        for (rank, (x, y)) in remaining.into_iter().zip(empty_cells) {
            self.grid.set(
                x,
                y,
                Cell {
                    terrain: CellTerrain::Open,
                    piece: Some(PlacedPiece {
                        rank,
                        team,
                        revealed: false,
                    }),
                },
            );
        }
    }

    /// Count pieces for a team.
    pub fn piece_count(&self, team: Team) -> usize {
        self.grid
            .iter()
            .filter(|(_, _, cell)| {
                cell.piece
                    .as_ref()
                    .map(|p| p.team == team)
                    .unwrap_or(false)
            })
            .count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pieces::PieceRank;

    #[test]
    fn new_board_has_correct_dimensions() {
        let board = StrategoBoard::new();
        assert_eq!(board.grid.width(), 10);
        assert_eq!(board.grid.height(), 10);
    }

    #[test]
    fn lakes_are_placed_correctly() {
        let board = StrategoBoard::new();
        assert_eq!(board.get(2, 4).unwrap().terrain, CellTerrain::Lake);
        assert_eq!(board.get(3, 5).unwrap().terrain, CellTerrain::Lake);
        assert_eq!(board.get(6, 4).unwrap().terrain, CellTerrain::Lake);
        assert_eq!(board.get(7, 5).unwrap().terrain, CellTerrain::Lake);
        // Non-lake cells are open.
        assert_eq!(board.get(0, 0).unwrap().terrain, CellTerrain::Open);
    }

    #[test]
    fn setup_zones_are_correct() {
        let board = StrategoBoard::new();
        assert!(board.is_setup_zone(0, 0, Team::Red));
        assert!(board.is_setup_zone(9, 3, Team::Red));
        assert!(!board.is_setup_zone(0, 4, Team::Red));
        assert!(board.is_setup_zone(0, 6, Team::Blue));
        assert!(board.is_setup_zone(9, 9, Team::Blue));
        assert!(!board.is_setup_zone(0, 5, Team::Blue));
    }

    #[test]
    fn place_piece_enforces_setup_zone() {
        let mut board = StrategoBoard::new();
        let piece = PlacedPiece {
            rank: PieceRank::Marshal,
            team: Team::Red,
            revealed: false,
        };

        assert!(board.place_piece(0, 0, piece));
        assert!(!board.place_piece(0, 6, piece)); // Blue's zone
        assert!(!board.place_piece(0, 0, piece)); // Already occupied
    }

    #[test]
    fn auto_fill_places_correct_count() {
        let mut board = StrategoBoard::new();
        board.auto_fill_army(Team::Red);
        assert_eq!(board.piece_count(Team::Red), 25);
        assert_eq!(board.piece_count(Team::Blue), 0);
    }
}
