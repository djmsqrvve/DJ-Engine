//! Undo/Redo system for the tile editor.
//!
//! Ported from Helix2000's `EditorHistorySystem.ts` + `EditorSnapshotAction.ts`.
//! Uses full GridLevel JSON snapshots (memento pattern).

use super::grid::GridLevel;
use bevy::prelude::*;

const MAX_HISTORY: usize = 50;

#[derive(Resource)]
pub struct EditorHistory {
    undo_stack: Vec<String>,
    redo_stack: Vec<String>,
    max_steps: usize,
}

impl Default for EditorHistory {
    fn default() -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_steps: MAX_HISTORY,
        }
    }
}

impl EditorHistory {
    /// Capture a snapshot of the current grid state before a destructive action.
    /// Call this BEFORE modifying the grid.
    pub fn push_snapshot(&mut self, grid: &GridLevel) {
        if let Ok(json) = serde_json::to_string(grid) {
            self.undo_stack.push(json);
            if self.undo_stack.len() > self.max_steps {
                self.undo_stack.remove(0);
            }
            // Any new action clears the redo stack
            self.redo_stack.clear();
        }
    }

    /// Undo: restore the most recent snapshot, pushing current state to redo.
    /// Returns true if undo was performed.
    pub fn undo(&mut self, grid: &mut GridLevel) -> bool {
        if let Some(snapshot) = self.undo_stack.pop() {
            // Save current state to redo
            if let Ok(current) = serde_json::to_string(grid) {
                self.redo_stack.push(current);
            }
            // Restore snapshot
            if let Ok(restored) = serde_json::from_str::<GridLevel>(&snapshot) {
                *grid = restored;
                return true;
            }
        }
        false
    }

    /// Redo: restore the most recent redo snapshot, pushing current state to undo.
    /// Returns true if redo was performed.
    pub fn redo(&mut self, grid: &mut GridLevel) -> bool {
        if let Some(snapshot) = self.redo_stack.pop() {
            // Save current state to undo
            if let Ok(current) = serde_json::to_string(grid) {
                self.undo_stack.push(current);
            }
            // Restore snapshot
            if let Ok(restored) = serde_json::from_str::<GridLevel>(&snapshot) {
                *grid = restored;
                return true;
            }
        }
        false
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::editor::grid::{LayerType, TileType};

    #[test]
    fn undo_restores_previous_state() {
        let mut history = EditorHistory::default();
        let mut grid = GridLevel::default();

        // Snapshot before paint
        history.push_snapshot(&grid);
        grid.paint(LayerType::Ground, 0, 0, TileType::Grass);

        assert!(history.can_undo());
        history.undo(&mut grid);
        assert_eq!(grid.get_tile(LayerType::Ground, 0, 0), TileType::Empty);
    }

    #[test]
    fn redo_reapplies_undone_state() {
        let mut history = EditorHistory::default();
        let mut grid = GridLevel::default();

        history.push_snapshot(&grid);
        grid.paint(LayerType::Ground, 0, 0, TileType::Stone);

        // Snapshot the painted state before we undo
        let painted_json = serde_json::to_string(&grid).unwrap();

        history.undo(&mut grid);
        assert_eq!(grid.get_tile(LayerType::Ground, 0, 0), TileType::Empty);

        assert!(history.can_redo());
        history.redo(&mut grid);
        assert_eq!(grid.get_tile(LayerType::Ground, 0, 0), TileType::Stone);
    }

    #[test]
    fn new_action_clears_redo() {
        let mut history = EditorHistory::default();
        let mut grid = GridLevel::default();

        history.push_snapshot(&grid);
        grid.paint(LayerType::Ground, 0, 0, TileType::Water);

        history.undo(&mut grid);
        assert!(history.can_redo());

        // New action
        history.push_snapshot(&grid);
        grid.paint(LayerType::Ground, 1, 1, TileType::Lava);

        assert!(!history.can_redo());
    }

    #[test]
    fn max_history_enforced() {
        let mut history = EditorHistory {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_steps: 3,
        };
        let grid = GridLevel::default();

        for _ in 0..5 {
            history.push_snapshot(&grid);
        }
        assert_eq!(history.undo_stack.len(), 3);
    }
}
