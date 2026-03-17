# Chapter 2: Board, Pieces, and Grid

Define the game's core data types and model the 10x10 board using the engine's reusable `Grid<T>`.

## What You'll Add

- `pieces.rs` -- Piece ranks, teams, army composition (137 lines, 4 tests)
- `board.rs` -- Board resource wrapping `Grid<Cell>`, terrain, placement (222 lines, 5 tests)

We create `pieces.rs` first because `board.rs` imports types from it.

## The Engine's Grid\<T\>

DJ Engine provides a generic 2D grid at `engine/src/data/grid.rs`. It's a flat `Vec<T>` in row-major order:

| Method | Description |
| --- | --- |
| `Grid::new(width, height)` | Creates grid filled with `T::default()` |
| `grid.get(x, y)` | Returns `Option<&T>` |
| `grid.get_mut(x, y)` | Returns `Option<&mut T>` |
| `grid.set(x, y, value)` | Returns `bool` (true if in bounds) |
| `grid.iter()` | Yields `(x, y, &T)` for every cell |
| `grid.neighbors(x, y)` | Returns `Vec<(usize, usize)>` (4-directional) |
| `grid.in_bounds(x, y)` | Bounds check |
| `grid.width()` / `grid.height()` | Dimensions |

It supports `Serialize`/`Deserialize` out of the box, so board state can be saved.

## Step 1: pieces.rs

> **File: `games/dev/stratego/src/pieces.rs`**

```rust
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
```

Key design decisions:

- `PieceRank` has numeric strength for combat comparison
- The Flag has strength 0 and `can_move() == false` -- it's your capture target
- `revealed: bool` tracks whether a piece has been exposed through combat
- 25 pieces per team fills exactly 4 rows of the 10-wide board (minus lakes)

## Step 2: board.rs

> **File: `games/dev/stratego/src/board.rs`**

```rust
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
```

Key points:

- `use crate::pieces::{army_composition, PlacedPiece, Team}` -- this is the cross-module import. `board.rs` depends on types defined in `pieces.rs`.
- `StrategoBoard` wraps `Grid<Cell>` and adds convenience methods (`get`, `get_mut`) that delegate to the grid.
- Lakes are 8 cells in two 2x2 blocks at the center. They block movement and placement.
- Red's setup zone is rows 0-3 (bottom), Blue's is rows 6-9 (top).
- `auto_fill_army` counts what's already placed, builds a remaining list, shuffles empty zone cells, and fills.

## Step 3: Update main.rs

> **File: `games/dev/stratego/src/main.rs`**

```rust
//! Stratego-lite — a turn-based board game tutorial for DJ Engine.

use bevy::prelude::*;
use bevy::window::WindowResolution;

mod board;
mod pieces;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "DJ Engine - Stratego".into(),
                        resolution: WindowResolution::new(800, 800)
                            .with_scale_factor_override(1.0),
                        position: WindowPosition::Centered(MonitorSelection::Primary),
                        present_mode: bevy::window::PresentMode::AutoVsync,
                        ..default()
                    }),
                    ..default()
                }),
        )
        .insert_resource(ClearColor(Color::srgb(0.15, 0.15, 0.2)))
        .init_resource::<board::StrategoBoard>()
        .add_systems(Startup, setup_camera)
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}
```

## Checkpoint

```sh
cargo test -p stratego
```

9 tests pass (4 from pieces, 5 from board). The window still shows nothing visual -- rendering comes in Chapter 4.

## Next

[Chapter 3: Rules and Combat](03-pieces-and-rules.md) -- Movement validation, combat resolution, and move execution.
