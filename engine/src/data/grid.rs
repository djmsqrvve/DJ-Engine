//! Generic 2D grid data structure.
//!
//! `Grid<T>` is a reusable width×height grid backed by a flat `Vec<T>`.
//! Useful for tilemaps, board games, tower defense maps, pathfinding, etc.

use serde::{Deserialize, Serialize};

/// A generic 2D grid of cells stored in row-major order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Grid<T> {
    width: usize,
    height: usize,
    cells: Vec<T>,
}

impl<T: Default + Clone> Grid<T> {
    /// Create a new grid filled with `T::default()`.
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            cells: vec![T::default(); width * height],
        }
    }
}

impl<T> Grid<T> {
    /// Create a grid from an existing vec of cells (row-major).
    ///
    /// Returns `None` if `cells.len() != width * height`.
    pub fn from_vec(width: usize, height: usize, cells: Vec<T>) -> Option<Self> {
        if cells.len() == width * height {
            Some(Self {
                width,
                height,
                cells,
            })
        } else {
            None
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn in_bounds(&self, x: usize, y: usize) -> bool {
        x < self.width && y < self.height
    }

    fn index(&self, x: usize, y: usize) -> Option<usize> {
        if self.in_bounds(x, y) {
            Some(y * self.width + x)
        } else {
            None
        }
    }

    pub fn get(&self, x: usize, y: usize) -> Option<&T> {
        self.index(x, y).map(|i| &self.cells[i])
    }

    pub fn get_mut(&mut self, x: usize, y: usize) -> Option<&mut T> {
        self.index(x, y).map(|i| &mut self.cells[i])
    }

    /// Set a cell value. Returns `true` if in bounds, `false` otherwise.
    pub fn set(&mut self, x: usize, y: usize, value: T) -> bool {
        if let Some(i) = self.index(x, y) {
            self.cells[i] = value;
            true
        } else {
            false
        }
    }

    /// Iterate all cells as `(x, y, &T)`.
    pub fn iter(&self) -> impl Iterator<Item = (usize, usize, &T)> {
        let w = self.width;
        self.cells
            .iter()
            .enumerate()
            .map(move |(i, cell)| (i % w, i / w, cell))
    }

    /// Iterate all cells as `(x, y, &mut T)`.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (usize, usize, &mut T)> {
        let w = self.width;
        self.cells
            .iter_mut()
            .enumerate()
            .map(move |(i, cell)| (i % w, i / w, cell))
    }

    /// Return 4-directional neighbor coordinates that are in bounds.
    pub fn neighbors(&self, x: usize, y: usize) -> Vec<(usize, usize)> {
        let mut result = Vec::with_capacity(4);
        if x > 0 {
            result.push((x - 1, y));
        }
        if x + 1 < self.width {
            result.push((x + 1, y));
        }
        if y > 0 {
            result.push((x, y - 1));
        }
        if y + 1 < self.height {
            result.push((x, y + 1));
        }
        result
    }

    /// Total number of cells.
    pub fn len(&self) -> usize {
        self.cells.len()
    }

    /// Whether the grid is empty (0 width or 0 height).
    pub fn is_empty(&self) -> bool {
        self.cells.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_grid_filled_with_defaults() {
        let grid: Grid<i32> = Grid::new(3, 2);
        assert_eq!(grid.width(), 3);
        assert_eq!(grid.height(), 2);
        assert_eq!(grid.len(), 6);
        assert_eq!(grid.get(0, 0), Some(&0));
        assert_eq!(grid.get(2, 1), Some(&0));
    }

    #[test]
    fn get_set_in_bounds() {
        let mut grid: Grid<i32> = Grid::new(4, 4);
        assert!(grid.set(2, 3, 42));
        assert_eq!(grid.get(2, 3), Some(&42));
        assert_eq!(grid.get(0, 0), Some(&0));
    }

    #[test]
    fn out_of_bounds_returns_none() {
        let grid: Grid<i32> = Grid::new(3, 3);
        assert_eq!(grid.get(3, 0), None);
        assert_eq!(grid.get(0, 3), None);
        assert_eq!(grid.get(100, 100), None);
    }

    #[test]
    fn set_out_of_bounds_returns_false() {
        let mut grid: Grid<i32> = Grid::new(3, 3);
        assert!(!grid.set(3, 0, 1));
        assert!(!grid.set(0, 3, 1));
    }

    #[test]
    fn from_vec_validates_length() {
        let grid = Grid::from_vec(2, 2, vec![1, 2, 3, 4]);
        assert!(grid.is_some());
        let grid = grid.unwrap();
        assert_eq!(grid.get(0, 0), Some(&1));
        assert_eq!(grid.get(1, 0), Some(&2));
        assert_eq!(grid.get(0, 1), Some(&3));
        assert_eq!(grid.get(1, 1), Some(&4));

        assert!(Grid::<i32>::from_vec(2, 2, vec![1, 2, 3]).is_none());
    }

    #[test]
    fn iter_yields_all_cells_with_coordinates() {
        let grid = Grid::from_vec(2, 2, vec![10, 20, 30, 40]).unwrap();
        let cells: Vec<_> = grid.iter().collect();
        assert_eq!(cells, vec![(0, 0, &10), (1, 0, &20), (0, 1, &30), (1, 1, &40)]);
    }

    #[test]
    fn neighbors_at_corner_and_center() {
        let grid: Grid<i32> = Grid::new(3, 3);

        let corner = grid.neighbors(0, 0);
        assert_eq!(corner.len(), 2);
        assert!(corner.contains(&(1, 0)));
        assert!(corner.contains(&(0, 1)));

        let center = grid.neighbors(1, 1);
        assert_eq!(center.len(), 4);
    }

    #[test]
    fn serialization_round_trip() {
        let mut grid: Grid<String> = Grid::new(2, 2);
        grid.set(0, 0, "a".into());
        grid.set(1, 1, "d".into());

        let json = serde_json::to_string(&grid).unwrap();
        let deserialized: Grid<String> = serde_json::from_str(&json).unwrap();
        assert_eq!(grid, deserialized);
    }
}
