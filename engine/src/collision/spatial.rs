//! Grid-based spatial hash for fast neighbor queries.
//!
//! Divides 2D space into a uniform grid of cells. Each cell tracks which
//! entities occupy it, making it cheap to find nearby candidates for
//! collision checks instead of testing every pair.

use bevy::prelude::*;
use std::collections::HashMap;

/// Grid-based spatial index for fast neighbor queries.
#[derive(Resource)]
pub struct SpatialHash {
    cell_size: f32,
    inv_cell_size: f32,
    cells: HashMap<(i32, i32), Vec<Entity>>,
}

impl Default for SpatialHash {
    fn default() -> Self {
        Self::new(64.0)
    }
}

impl SpatialHash {
    /// Create a new spatial hash with the given cell size.
    pub fn new(cell_size: f32) -> Self {
        let cell_size = cell_size.max(1.0);
        Self {
            cell_size,
            inv_cell_size: 1.0 / cell_size,
            cells: HashMap::new(),
        }
    }

    /// Returns the configured cell size.
    pub fn cell_size(&self) -> f32 {
        self.cell_size
    }

    /// Clear and rebuild from entity positions.
    pub fn rebuild(&mut self, entities: impl Iterator<Item = (Entity, Vec2)>) {
        self.cells.clear();
        for (entity, pos) in entities {
            let coords = self.cell_coords(pos);
            self.cells.entry(coords).or_default().push(entity);
        }
    }

    /// Get all entities in the same cell and neighboring cells (3x3 grid).
    pub fn query_neighbors(&self, position: Vec2) -> Vec<Entity> {
        let (cx, cy) = self.cell_coords(position);
        let mut result = Vec::new();
        for dx in -1..=1 {
            for dy in -1..=1 {
                if let Some(entities) = self.cells.get(&(cx + dx, cy + dy)) {
                    result.extend(entities);
                }
            }
        }
        result
    }

    /// Convert world position to cell coordinates.
    fn cell_coords(&self, pos: Vec2) -> (i32, i32) {
        (
            (pos.x * self.inv_cell_size).floor() as i32,
            (pos.y * self.inv_cell_size).floor() as i32,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::world::World;

    fn spawn_entities(count: usize) -> (World, Vec<Entity>) {
        let mut world = World::new();
        let entities: Vec<Entity> = (0..count).map(|_| world.spawn_empty().id()).collect();
        (world, entities)
    }

    #[test]
    fn test_cell_coords() {
        let hash = SpatialHash::new(64.0);
        assert_eq!(hash.cell_coords(Vec2::new(0.0, 0.0)), (0, 0));
        assert_eq!(hash.cell_coords(Vec2::new(63.9, 63.9)), (0, 0));
        assert_eq!(hash.cell_coords(Vec2::new(64.0, 0.0)), (1, 0));
        assert_eq!(hash.cell_coords(Vec2::new(-1.0, -1.0)), (-1, -1));
        assert_eq!(hash.cell_coords(Vec2::new(128.5, -64.5)), (2, -2));
    }

    #[test]
    fn test_rebuild_and_query() {
        let (_world, entities) = spawn_entities(3);
        let e0 = entities[0];
        let e1 = entities[1];
        let e2 = entities[2];

        let mut hash = SpatialHash::new(64.0);
        hash.rebuild(
            vec![
                (e0, Vec2::new(10.0, 10.0)),
                (e1, Vec2::new(50.0, 50.0)),
                (e2, Vec2::new(500.0, 500.0)),
            ]
            .into_iter(),
        );

        // e0 and e1 are in the same cell (0,0), so querying near e0 returns both
        let neighbors = hash.query_neighbors(Vec2::new(10.0, 10.0));
        assert!(neighbors.contains(&e0));
        assert!(neighbors.contains(&e1));

        // e2 is far away and should not be a neighbor of e0
        assert!(!neighbors.contains(&e2));

        // Querying near e2 should return e2 but not e0/e1
        let neighbors = hash.query_neighbors(Vec2::new(500.0, 500.0));
        assert!(neighbors.contains(&e2));
        assert!(!neighbors.contains(&e0));
        assert!(!neighbors.contains(&e1));
    }
}
