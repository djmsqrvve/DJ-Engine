//! Isometric grid data model wrapping the engine's Grid<T>.

use bevy::prelude::*;
use dj_engine::data::Grid;
use serde::{Deserialize, Serialize};

pub const GRID_WIDTH: usize = 16;
pub const GRID_HEIGHT: usize = 16;

/// Terrain type for an isometric cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum IsoTerrain {
    #[default]
    Grass,
    Stone,
    Water,
    Sand,
}

impl IsoTerrain {
    /// Cycle to the next terrain type.
    pub fn next(self) -> Self {
        match self {
            Self::Grass => Self::Stone,
            Self::Stone => Self::Water,
            Self::Water => Self::Sand,
            Self::Sand => Self::Grass,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Grass => "Grass",
            Self::Stone => "Stone",
            Self::Water => "Water",
            Self::Sand => "Sand",
        }
    }
}

/// What kind of entity is placed on a tile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntityKind {
    Actor,
    Prop,
    Blocker,
    Spawn,
}

impl EntityKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Actor => "A",
            Self::Prop => "P",
            Self::Blocker => "B",
            Self::Spawn => "S",
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Actor => "Actor",
            Self::Prop => "Prop",
            Self::Blocker => "Blocker",
            Self::Spawn => "Spawn",
        }
    }
}

/// An entity placed on the isometric grid.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlacedEntity {
    pub kind: EntityKind,
}

/// A single cell on the isometric grid.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct IsoCell {
    pub terrain: IsoTerrain,
    pub entity: Option<PlacedEntity>,
}

/// The isometric grid as a Bevy Resource.
#[derive(Resource, Debug, Clone)]
pub struct IsoGrid {
    pub grid: Grid<IsoCell>,
}

impl Default for IsoGrid {
    fn default() -> Self {
        Self::new()
    }
}

impl IsoGrid {
    pub fn new() -> Self {
        Self {
            grid: Grid::new(GRID_WIDTH, GRID_HEIGHT),
        }
    }

    pub fn get(&self, x: usize, y: usize) -> Option<&IsoCell> {
        self.grid.get(x, y)
    }

    pub fn get_mut(&mut self, x: usize, y: usize) -> Option<&mut IsoCell> {
        self.grid.get_mut(x, y)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_grid_has_correct_dimensions() {
        let grid = IsoGrid::new();
        assert_eq!(grid.grid.width(), GRID_WIDTH);
        assert_eq!(grid.grid.height(), GRID_HEIGHT);
    }

    #[test]
    fn default_terrain_is_grass() {
        let grid = IsoGrid::new();
        assert_eq!(grid.get(0, 0).unwrap().terrain, IsoTerrain::Grass);
    }

    #[test]
    fn terrain_cycles() {
        assert_eq!(IsoTerrain::Grass.next(), IsoTerrain::Stone);
        assert_eq!(IsoTerrain::Sand.next(), IsoTerrain::Grass);
    }

    #[test]
    fn place_and_remove_entity() {
        let mut grid = IsoGrid::new();
        let entity = PlacedEntity {
            kind: EntityKind::Actor,
        };
        grid.get_mut(5, 5).unwrap().entity = Some(entity);
        assert!(grid.get(5, 5).unwrap().entity.is_some());
        grid.get_mut(5, 5).unwrap().entity = None;
        assert!(grid.get(5, 5).unwrap().entity.is_none());
    }
}
