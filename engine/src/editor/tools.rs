//! Editor tool implementations.
//!
//! Ported from Helix2000's tool strategy pattern (`tools/*.ts`).

use super::grid::{EditorTool, GridLevel, LayerType, TileType};
use std::collections::VecDeque;

// ---------------------------------------------------------------------------
// Brush tool — paint NxN area
// ---------------------------------------------------------------------------

pub fn brush_paint(
    grid: &mut GridLevel,
    layer: LayerType,
    tile: TileType,
    tx: i32,
    ty: i32,
    brush_size: u8,
) -> bool {
    let size = brush_size.max(1) as i32;
    let offset = size / 2;
    let mut changed = false;
    for dx in 0..size {
        for dy in 0..size {
            if grid.paint(layer, tx - offset + dx, ty - offset + dy, tile) {
                changed = true;
            }
        }
    }
    changed
}

// ---------------------------------------------------------------------------
// Eraser tool — erase NxN area
// ---------------------------------------------------------------------------

pub fn eraser_erase(
    grid: &mut GridLevel,
    layer: LayerType,
    tx: i32,
    ty: i32,
    brush_size: u8,
) -> bool {
    let size = brush_size.max(1) as i32;
    let offset = size / 2;
    let mut changed = false;
    for dx in 0..size {
        for dy in 0..size {
            if grid.erase(layer, tx - offset + dx, ty - offset + dy) {
                changed = true;
            }
        }
    }
    changed
}

// ---------------------------------------------------------------------------
// Fill tool — BFS flood fill (max 5000 tiles)
// ---------------------------------------------------------------------------

const MAX_FILL_TILES: usize = 5000;

pub fn flood_fill(
    grid: &mut GridLevel,
    layer: LayerType,
    tx: i32,
    ty: i32,
    tile: TileType,
) -> bool {
    let target = grid.get_tile(layer, tx, ty);
    if target == tile {
        return false; // already the target type
    }

    let mut queue = VecDeque::new();
    let mut visited = std::collections::HashSet::new();
    queue.push_back((tx, ty));
    visited.insert((tx, ty));
    let mut count = 0;

    while let Some((cx, cy)) = queue.pop_front() {
        if count >= MAX_FILL_TILES {
            break;
        }
        if grid.get_tile(layer, cx, cy) != target {
            continue;
        }
        grid.paint(layer, cx, cy, tile);
        count += 1;

        for (nx, ny) in [(cx - 1, cy), (cx + 1, cy), (cx, cy - 1), (cx, cy + 1)] {
            if !visited.contains(&(nx, ny)) {
                visited.insert((nx, ny));
                queue.push_back((nx, ny));
            }
        }
    }
    count > 0
}

// ---------------------------------------------------------------------------
// Rectangle tool — fill rect from corner to corner
// ---------------------------------------------------------------------------

pub fn paint_rectangle(
    grid: &mut GridLevel,
    layer: LayerType,
    tile: TileType,
    x1: i32,
    y1: i32,
    x2: i32,
    y2: i32,
) -> bool {
    let min_x = x1.min(x2);
    let max_x = x1.max(x2);
    let min_y = y1.min(y2);
    let max_y = y1.max(y2);
    let mut changed = false;
    for x in min_x..=max_x {
        for y in min_y..=max_y {
            if grid.paint(layer, x, y, tile) {
                changed = true;
            }
        }
    }
    changed
}

// ---------------------------------------------------------------------------
// Line tool — Bresenham's line algorithm
// ---------------------------------------------------------------------------

pub fn paint_line(
    grid: &mut GridLevel,
    layer: LayerType,
    tile: TileType,
    x1: i32,
    y1: i32,
    x2: i32,
    y2: i32,
) -> bool {
    let mut changed = false;
    let dx = (x2 - x1).abs();
    let dy = -(y2 - y1).abs();
    let sx = if x1 < x2 { 1 } else { -1 };
    let sy = if y1 < y2 { 1 } else { -1 };
    let mut err = dx + dy;
    let mut cx = x1;
    let mut cy = y1;

    loop {
        if grid.paint(layer, cx, cy, tile) {
            changed = true;
        }
        if cx == x2 && cy == y2 {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            if cx == x2 {
                break;
            }
            err += dy;
            cx += sx;
        }
        if e2 <= dx {
            if cy == y2 {
                break;
            }
            err += dx;
            cy += sy;
        }
    }
    changed
}

// ---------------------------------------------------------------------------
// Tool dispatch
// ---------------------------------------------------------------------------

/// Execute the primary action for the current tool at the given tile position.
/// Returns true if the grid was modified.
pub fn dispatch_tool(
    tool: EditorTool,
    grid: &mut GridLevel,
    layer: LayerType,
    tile: TileType,
    tx: i32,
    ty: i32,
    brush_size: u8,
) -> bool {
    match tool {
        EditorTool::Brush => brush_paint(grid, layer, tile, tx, ty, brush_size),
        EditorTool::Eraser => eraser_erase(grid, layer, tx, ty, brush_size),
        EditorTool::Fill => flood_fill(grid, layer, tx, ty, tile),
        EditorTool::Select
        | EditorTool::Rectangle
        | EditorTool::Line
        | EditorTool::EntityPlacer => false,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn brush_paints_3x3() {
        let mut grid = GridLevel::default();
        brush_paint(&mut grid, LayerType::Ground, TileType::Grass, 5, 5, 3);
        // Center and all 8 neighbors
        for dx in -1..=1 {
            for dy in -1..=1 {
                assert_eq!(
                    grid.get_tile(LayerType::Ground, 5 + dx, 5 + dy),
                    TileType::Grass
                );
            }
        }
    }

    #[test]
    fn eraser_clears() {
        let mut grid = GridLevel::default();
        brush_paint(&mut grid, LayerType::Ground, TileType::Stone, 0, 0, 1);
        assert_eq!(grid.get_tile(LayerType::Ground, 0, 0), TileType::Stone);
        eraser_erase(&mut grid, LayerType::Ground, 0, 0, 1);
        assert_eq!(grid.get_tile(LayerType::Ground, 0, 0), TileType::Empty);
    }

    #[test]
    fn flood_fill_fills_connected() {
        let mut grid = GridLevel::default();
        // Place a border
        for i in -2..=2 {
            grid.paint(LayerType::Ground, i, -2, TileType::Wall);
            grid.paint(LayerType::Ground, i, 2, TileType::Wall);
            grid.paint(LayerType::Ground, -2, i, TileType::Wall);
            grid.paint(LayerType::Ground, 2, i, TileType::Wall);
        }
        // Fill inside (0,0 is Empty surrounded by Wall)
        flood_fill(&mut grid, LayerType::Ground, 0, 0, TileType::Water);
        assert_eq!(grid.get_tile(LayerType::Ground, 0, 0), TileType::Water);
        assert_eq!(grid.get_tile(LayerType::Ground, 1, 1), TileType::Water);
        // Border should be unchanged
        assert_eq!(grid.get_tile(LayerType::Ground, 2, 2), TileType::Wall);
    }

    #[test]
    fn paint_rectangle_fills_area() {
        let mut grid = GridLevel::default();
        paint_rectangle(&mut grid, LayerType::Ground, TileType::Floor, 0, 0, 2, 2);
        for x in 0..=2 {
            for y in 0..=2 {
                assert_eq!(grid.get_tile(LayerType::Ground, x, y), TileType::Floor);
            }
        }
    }

    #[test]
    fn paint_line_diagonal() {
        let mut grid = GridLevel::default();
        paint_line(&mut grid, LayerType::Ground, TileType::Rope, 0, 0, 3, 3);
        for i in 0..=3 {
            assert_eq!(grid.get_tile(LayerType::Ground, i, i), TileType::Rope);
        }
    }
}
