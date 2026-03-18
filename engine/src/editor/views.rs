use super::grid::{self, EditorTool, GridLevel, PaintState};
use super::history::EditorHistory;
use super::tools;
use super::types::{ActiveStoryGraph, EditorUiState, COLOR_BG};
use crate::data::components::common::Vec3Data;
use crate::data::story::nodes::StoryNodeData;
use crate::data::story::types::StoryNodeType;
use bevy::prelude::*;
use bevy_egui::egui::{self, Color32};

const BASE_GRID_PX: f32 = 40.0;
const MIN_ZOOM: f32 = 0.25;
const MAX_ZOOM: f32 = 3.0;
const PAN_SPEED: f32 = 20.0;

/// Capture a history snapshot (clones grid to avoid borrow conflicts).
fn snapshot(world: &mut World) {
    let grid = world.resource::<GridLevel>().clone();
    world.resource_mut::<EditorHistory>().push_snapshot(&grid);
}

/// Convert a screen position to a tile coordinate with camera transform.
fn screen_to_tile(pointer: egui::Pos2, center: egui::Pos2, offset: egui::Vec2, zoom: f32) -> (i32, i32) {
    let gpx = BASE_GRID_PX * zoom;
    let wx = pointer.x - center.x - offset.x;
    let wy = center.y + offset.y - pointer.y;
    ((wx / gpx).round() as i32, (wy / gpx).round() as i32)
}

/// Convert a tile coordinate back to screen position with camera transform.
fn tile_to_screen(tx: i32, ty: i32, center: egui::Pos2, offset: egui::Vec2, zoom: f32) -> egui::Pos2 {
    let gpx = BASE_GRID_PX * zoom;
    egui::pos2(
        center.x + offset.x + tx as f32 * gpx,
        center.y + offset.y - ty as f32 * gpx,
    )
}

pub(crate) fn draw_grid(ui: &mut egui::Ui, world: &mut World) {
    let rect = ui.available_rect_before_wrap();

    // Allocate interactive rect (click + drag) before painter
    let response = ui.allocate_rect(rect, egui::Sense::click_and_drag());
    let painter = ui.painter();
    let center = rect.center();

    // Read current editor state
    let (tool, tile, layer, brush_size, cam_offset, zoom, grid_visible) = {
        let st = world.resource::<EditorUiState>();
        (
            st.current_tool, st.current_tile, st.current_layer,
            st.clamped_brush_size(), st.camera_offset, st.zoom, st.grid_visible,
        )
    };

    // --- 0. Keyboard shortcuts ---
    {
        let keys = ui.input(|i| {
            let ctrl = i.modifiers.command;
            let no_mod = !ctrl && !i.modifiers.alt;
            (
                // Undo/Redo
                ctrl && i.key_pressed(egui::Key::Z) && !i.modifiers.shift,
                ctrl && (i.key_pressed(egui::Key::Y)
                    || (i.key_pressed(egui::Key::Z) && i.modifiers.shift)),
                // Tool shortcuts (no modifier)
                no_mod && i.key_pressed(egui::Key::B),
                no_mod && i.key_pressed(egui::Key::E),
                no_mod && i.key_pressed(egui::Key::S),
                no_mod && i.key_pressed(egui::Key::F),
                no_mod && i.key_pressed(egui::Key::R),
                no_mod && i.key_pressed(egui::Key::L),
                no_mod && i.key_pressed(egui::Key::G),
                no_mod && i.key_pressed(egui::Key::N),
                // Brush size 1-5
                no_mod && i.key_pressed(egui::Key::Num1),
                no_mod && i.key_pressed(egui::Key::Num2),
                no_mod && i.key_pressed(egui::Key::Num3),
                no_mod && i.key_pressed(egui::Key::Num4),
                no_mod && i.key_pressed(egui::Key::Num5),
                // Camera: Shift+WASD
                i.modifiers.shift && i.key_down(egui::Key::W),
                i.modifiers.shift && i.key_down(egui::Key::A),
                i.modifiers.shift && i.key_down(egui::Key::S),
                i.modifiers.shift && i.key_down(egui::Key::D),
                // Scroll (zoom)
                i.smooth_scroll_delta.y,
                // Delete + Ctrl+S + C (collision) + Enter (fill region)
                no_mod && i.key_pressed(egui::Key::Delete),
                ctrl && i.key_pressed(egui::Key::S),
                no_mod && i.key_pressed(egui::Key::C),
                no_mod && i.key_pressed(egui::Key::Enter),
            )
        });

        // Undo/Redo
        if keys.0 {
            let mut grid = world.resource::<GridLevel>().clone();
            if world.resource_mut::<EditorHistory>().undo(&mut grid) {
                *world.resource_mut::<GridLevel>() = grid;
            }
        } else if keys.1 {
            let mut grid = world.resource::<GridLevel>().clone();
            if world.resource_mut::<EditorHistory>().redo(&mut grid) {
                *world.resource_mut::<GridLevel>() = grid;
            }
        }

        // Tool shortcuts
        let mut st = world.resource_mut::<EditorUiState>();
        if keys.2 { st.current_tool = EditorTool::Brush; }
        if keys.3 { st.current_tool = EditorTool::Eraser; }
        if keys.4 { st.current_tool = EditorTool::Select; }
        if keys.5 { st.current_tool = EditorTool::Fill; }
        if keys.6 { st.current_tool = EditorTool::Rectangle; }
        if keys.7 { st.current_tool = EditorTool::Line; }
        if keys.8 { st.grid_visible = !st.grid_visible; }
        if keys.9 { st.current_tool = EditorTool::EntityPlacer; }

        // Brush size
        if keys.10 { st.brush_size = 1; }
        if keys.11 { st.brush_size = 2; }
        if keys.12 { st.brush_size = 3; }
        if keys.13 { st.brush_size = 4; }
        if keys.14 { st.brush_size = 5; }

        // Camera pan (Shift+WASD) — key_down for continuous
        if keys.15 { st.camera_offset.y += PAN_SPEED; }  // W = pan up (offset Y+)
        if keys.16 { st.camera_offset.x -= PAN_SPEED; }  // A = pan left
        if keys.17 { st.camera_offset.y -= PAN_SPEED; }  // S = pan down
        if keys.18 { st.camera_offset.x += PAN_SPEED; }  // D = pan right

        // Mouse wheel zoom (around cursor)
        let scroll_delta = keys.19;
        if scroll_delta.abs() > 0.1 {
            let zoom_step = scroll_delta * 0.002;
            st.zoom = (st.zoom + zoom_step).clamp(MIN_ZOOM, MAX_ZOOM);
        }

        // Delete key — erase selected tile or remove selected entity
        let wants_delete = keys.20;
        let wants_save = keys.21;
        let wants_collision_toggle = keys.22;
        let wants_fill_region = keys.23;

        if wants_collision_toggle {
            st.show_collision = !st.show_collision;
        }

        // Drop mutable borrow on st before accessing world resources
        let sel_tile = st.selected_tile_pos;
        let sel_entity = st.selected_entity_id.clone();
        let sel_region = st.selection_region;
        let cur_layer = st.current_layer;
        let cur_tile = st.current_tile;
        drop(st);

        if wants_delete {
            if let Some(id) = &sel_entity {
                snapshot(world);
                let id = id.clone();
                world.resource_mut::<GridLevel>().remove_entity(&id);
                world.resource_mut::<EditorUiState>().selected_entity_id = None;
            } else if let Some((min_x, min_y, max_x, max_y)) = sel_region {
                // Bulk erase selection region
                snapshot(world);
                let mut grid = world.resource_mut::<GridLevel>();
                for x in min_x..=max_x {
                    for y in min_y..=max_y {
                        grid.erase(cur_layer, x, y);
                    }
                }
                world.resource_mut::<EditorUiState>().selection_region = None;
            } else if let Some((tx, ty)) = sel_tile {
                snapshot(world);
                world.resource_mut::<GridLevel>().erase(cur_layer, tx, ty);
            }
        }

        // Enter fills selection region with current tile
        if wants_fill_region {
            if let Some((min_x, min_y, max_x, max_y)) = sel_region {
                snapshot(world);
                let mut grid = world.resource_mut::<GridLevel>();
                for x in min_x..=max_x {
                    for y in min_y..=max_y {
                        grid.paint(cur_layer, x, y, cur_tile);
                    }
                }
                world.resource_mut::<EditorUiState>().selection_region = None;
            }
        }

        if wants_save {
            use super::scene_io::save_project_impl;
            if let Err(e) = save_project_impl(world) {
                warn!("Ctrl+S save failed: {}", e);
            }
        }
    }

    // Re-read state after shortcuts may have changed it
    let (tool, tile, layer, brush_size, cam_offset, zoom, grid_visible, show_collision) = {
        let st = world.resource::<EditorUiState>();
        (
            st.current_tool, st.current_tile, st.current_layer,
            st.clamped_brush_size(), st.camera_offset, st.zoom, st.grid_visible,
            st.show_collision,
        )
    };

    // Middle-click drag → camera pan
    if response.dragged_by(egui::PointerButton::Middle) {
        let delta = response.drag_delta();
        let mut st = world.resource_mut::<EditorUiState>();
        st.camera_offset += delta;
    }

    // --- 1. Handle Input (paint / erase / tool actions) ---
    let pointer_tile = ui
        .input(|i| i.pointer.hover_pos())
        .filter(|p| rect.contains(*p))
        .map(|p| screen_to_tile(p, center, cam_offset, zoom));

    // Eyedropper: Alt+Click samples tile under cursor (works with any tool)
    let alt_held = ui.input(|i| i.modifiers.alt);
    if alt_held && response.clicked() {
        if let Some((tx, ty)) = pointer_tile {
            let sampled = world.resource::<GridLevel>().get_tile(layer, tx, ty);
            if sampled != grid::TileType::Empty {
                let mut st = world.resource_mut::<EditorUiState>();
                st.current_tile = sampled;
                st.current_tool = EditorTool::Brush;
            }
        }
    }

    // Drag painting (brush / eraser)
    if tool.supports_drag() {
        if response.drag_started() {
            // Snapshot before starting a drag-paint stroke
            snapshot(world);
            let mut ps = world.resource_mut::<PaintState>();
            ps.is_painting = true;
            ps.last_paint_pos = None;
        }
        if response.drag_stopped() || (!response.dragged() && !response.drag_started()) {
            world.resource_mut::<PaintState>().is_painting = false;
        }

        let is_painting = world.resource::<PaintState>().is_painting;
        let last_pos = world.resource::<PaintState>().last_paint_pos;

        // Snapshot for single clicks (not drag start)
        let need_click_snapshot = response.clicked() && !is_painting;

        if is_painting || response.clicked() {
            if let Some((tx, ty)) = pointer_tile {
                if last_pos != Some((tx, ty)) || response.clicked() {
                    if need_click_snapshot {
                        snapshot(world);
                    }
                    let mut grid = world.resource_mut::<GridLevel>();
                    tools::dispatch_tool(tool, &mut grid, layer, tile, tx, ty, brush_size);
                    world.resource_mut::<PaintState>().last_paint_pos = Some((tx, ty));
                }
            }
        }
    } else if tool == EditorTool::Fill {
        if response.clicked() {
            if let Some((tx, ty)) = pointer_tile {
                snapshot(world);
                let mut grid = world.resource_mut::<GridLevel>();
                tools::dispatch_tool(tool, &mut grid, layer, tile, tx, ty, brush_size);
            }
        }
    } else if tool == EditorTool::Rectangle || tool == EditorTool::Line {
        if response.drag_started() {
            if let Some((tx, ty)) = pointer_tile {
                snapshot(world);
                world.resource_mut::<PaintState>().drag_start = Some((tx, ty));
            }
        }
        if response.drag_stopped() {
            let start = world.resource::<PaintState>().drag_start;
            if let (Some((sx, sy)), Some((ex, ey))) = (start, pointer_tile) {
                let mut grid = world.resource_mut::<GridLevel>();
                if tool == EditorTool::Rectangle {
                    tools::paint_rectangle(&mut grid, layer, tile, sx, sy, ex, ey);
                } else {
                    tools::paint_line(&mut grid, layer, tile, sx, sy, ex, ey);
                }
            }
            world.resource_mut::<PaintState>().drag_start = None;
        }
    } else if tool == EditorTool::Select {
        let shift_held = ui.input(|i| i.modifiers.shift);

        if shift_held {
            // Shift+drag → region selection
            if response.drag_started() {
                if let Some((tx, ty)) = pointer_tile {
                    world.resource_mut::<EditorUiState>().region_drag_start = Some((tx, ty));
                }
            }
            if response.drag_stopped() {
                let start = world.resource::<EditorUiState>().region_drag_start;
                if let (Some((sx, sy)), Some((ex, ey))) = (start, pointer_tile) {
                    let mut st = world.resource_mut::<EditorUiState>();
                    st.selection_region = Some((
                        sx.min(ex), sy.min(ey), sx.max(ex), sy.max(ey),
                    ));
                    st.region_drag_start = None;
                }
            }
        } else {
            // Normal select: click to select, drag to move entities
            if response.drag_started() {
                if let Some((tx, ty)) = pointer_tile {
                    let grid = world.resource::<GridLevel>();
                    if let Some(ent) = grid.entity_at(tx, ty) {
                        let id = ent.id.clone();
                        let mut st = world.resource_mut::<EditorUiState>();
                        st.selected_entity_id = Some(id.clone());
                        st.selected_tile_pos = None;
                        // Start drag
                        world.resource_mut::<PaintState>().drag_start = Some((tx, ty));
                        snapshot(world);
                    } else {
                        let mut st = world.resource_mut::<EditorUiState>();
                        st.selected_tile_pos = Some((tx, ty));
                        st.selected_entity_id = None;
                        st.selection_region = None; // clear region on normal click
                    }
                }
            }
            if response.clicked() && !response.drag_started() {
                // Simple click (no drag) — select
                if let Some((tx, ty)) = pointer_tile {
                    let grid = world.resource::<GridLevel>();
                    if let Some(ent) = grid.entity_at(tx, ty) {
                        let id = ent.id.clone();
                        let mut st = world.resource_mut::<EditorUiState>();
                        st.selected_entity_id = Some(id);
                        st.selected_tile_pos = None;
                    } else {
                        let mut st = world.resource_mut::<EditorUiState>();
                        st.selected_tile_pos = Some((tx, ty));
                        st.selected_entity_id = None;
                        st.selection_region = None; // clear region on normal click
                    }
                }
            }
            if response.drag_stopped() {
                // Move entity to new position
                let drag_start = world.resource::<PaintState>().drag_start;
                let sel_id = world.resource::<EditorUiState>().selected_entity_id.clone();
                if let (Some(_start), Some(id), Some((tx, ty))) = (drag_start, sel_id, pointer_tile) {
                    world.resource_mut::<GridLevel>().move_entity(&id, tx, ty);
                }
                world.resource_mut::<PaintState>().drag_start = None;
            }
        }
    } else if tool == grid::EditorTool::EntityPlacer {
        // Entity placer: click to place entity
        if response.clicked() {
            if let Some((tx, ty)) = pointer_tile {
                snapshot(world);
                let entity_type = world.resource::<EditorUiState>().current_entity_type.clone();
                let id = world.resource_mut::<GridLevel>().place_entity(&entity_type, tx, ty);
                world.resource_mut::<EditorUiState>().selected_entity_id = Some(id);
            }
        }
    }

    // --- 2. Draw Grid Background & Lines ---
    painter.rect_filled(rect, 0.0, COLOR_BG);

    let gpx = BASE_GRID_PX * zoom;
    let tile_draw = (BASE_GRID_PX - 2.0) * zoom;

    if grid_visible && gpx > 4.0 {
        let line_color = Color32::from_rgb(30, 30, 40);
        // Grid origin in screen space
        let origin_x = center.x + cam_offset.x;
        let origin_y = center.y + cam_offset.y;

        // Vertical lines
        let start_x = origin_x - ((origin_x - rect.left()) / gpx).ceil() * gpx;
        let mut x = start_x;
        while x < rect.right() {
            painter.line_segment(
                [egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())],
                (1.0, line_color),
            );
            x += gpx;
        }
        // Horizontal lines
        let start_y = origin_y - ((origin_y - rect.top()) / gpx).ceil() * gpx;
        let mut y = start_y;
        while y < rect.bottom() {
            painter.line_segment(
                [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)],
                (1.0, line_color),
            );
            y += gpx;
        }
        // Origin crosshair (subtle)
        if rect.contains(egui::pos2(origin_x, origin_y)) {
            let cross_color = Color32::from_rgba_unmultiplied(0, 255, 204, 40);
            painter.line_segment(
                [egui::pos2(origin_x, rect.top()), egui::pos2(origin_x, rect.bottom())],
                (1.0, cross_color),
            );
            painter.line_segment(
                [egui::pos2(rect.left(), origin_y), egui::pos2(rect.right(), origin_y)],
                (1.0, cross_color),
            );
        }
    }

    // --- 3. Draw placed tiles from GridLevel ---
    let min_tile = screen_to_tile(rect.left_top(), center, cam_offset, zoom);
    let max_tile = screen_to_tile(rect.right_bottom(), center, cam_offset, zoom);
    let vis_min_tx = min_tile.0 - 1;
    let vis_max_tx = max_tile.0 + 1;
    let vis_min_ty = max_tile.1 - 1; // Y flipped
    let vis_max_ty = min_tile.1 + 1;

    let grid = world.resource::<GridLevel>();
    let font_size = (14.0 * zoom).max(6.0);

    // Draw layers bottom to top
    for tile_layer in &grid.layers {
        if !tile_layer.visible {
            continue;
        }
        let alpha = (tile_layer.opacity * 255.0) as u8;
        for (&(tx, ty), &tt) in &tile_layer.tiles {
            if tx < vis_min_tx || tx > vis_max_tx || ty < vis_min_ty || ty > vis_max_ty {
                continue;
            }
            let screen_pos = tile_to_screen(tx, ty, center, cam_offset, zoom);
            let tile_rect = egui::Rect::from_center_size(
                screen_pos,
                egui::vec2(tile_draw, tile_draw),
            );

            let base_color = grid::tile_color(tt);
            let color = Color32::from_rgba_unmultiplied(base_color.r(), base_color.g(), base_color.b(), alpha);
            painter.rect_filled(tile_rect, 2.0, color);

            if font_size >= 8.0 {
                let label = grid::tile_label(tt);
                painter.text(
                    screen_pos,
                    egui::Align2::CENTER_CENTER,
                    label.to_string(),
                    egui::FontId::monospace(font_size),
                    Color32::from_rgba_unmultiplied(255, 255, 255, alpha),
                );
            }
        }
    }

    // --- 3b. Entity overlays (H2K EntityOverlayRenderer) ---
    let selected_entity_id = world.resource::<EditorUiState>().selected_entity_id.clone();
    for entity in &grid.entities {
        let ex = entity.x;
        let ey = entity.y;
        let ew = entity.width.max(1);
        let eh = entity.height.max(1);

        // Entity color by type
        let ent_color = match entity.entity_type.as_str() {
            "mob" | "training_dummy" => Color32::from_rgb(244, 67, 54),
            "npc" | "npc_spawn" => Color32::from_rgb(76, 175, 80),
            "spawn_point" => Color32::from_rgb(50, 205, 50),
            "teleporter" => Color32::from_rgb(147, 112, 219),
            "chest" => Color32::from_rgb(218, 165, 32),
            _ => Color32::from_rgb(135, 206, 235),
        };

        // Bounding box (centered on tile range)
        let top_left = tile_to_screen(ex, ey + eh - 1, center, cam_offset, zoom);
        let bot_right = tile_to_screen(ex + ew - 1, ey, center, cam_offset, zoom);
        let half = gpx / 2.0;
        let ent_rect = egui::Rect::from_two_pos(
            egui::pos2(top_left.x - half, top_left.y - half),
            egui::pos2(bot_right.x + half, bot_right.y + half),
        );

        // Filled bg
        let fill = Color32::from_rgba_unmultiplied(ent_color.r(), ent_color.g(), ent_color.b(), 60);
        painter.rect_filled(ent_rect, 3.0, fill);
        painter.rect_stroke(ent_rect, 3.0, (2.0, ent_color), egui::StrokeKind::Outside);

        // Label
        if font_size >= 8.0 {
            let label_pos = egui::pos2(ent_rect.center().x, ent_rect.bottom() + 2.0);
            painter.text(
                label_pos,
                egui::Align2::CENTER_TOP,
                &entity.entity_type,
                egui::FontId::monospace((10.0 * zoom).max(6.0)),
                ent_color,
            );
        }

        // Corner markers (4px squares)
        let cs = 3.0 * zoom;
        for corner in [
            ent_rect.left_top(), ent_rect.right_top(),
            ent_rect.left_bottom(), ent_rect.right_bottom(),
        ] {
            painter.rect_filled(
                egui::Rect::from_center_size(corner, egui::vec2(cs * 2.0, cs * 2.0)),
                0.0,
                ent_color,
            );
        }

        // Selection highlight (gold)
        if selected_entity_id.as_deref() == Some(&entity.id) {
            let sel = Color32::from_rgb(255, 215, 0);
            painter.rect_stroke(ent_rect, 3.0, (3.0, sel), egui::StrokeKind::Outside);
        }
    }

    // Teleporter link lines (dashed purple)
    let tp_color = Color32::from_rgba_unmultiplied(147, 112, 219, 180);
    for entity in &grid.entities {
        if let Some(link_id) = entity.properties.get("teleporter_link") {
            if let Some(target) = grid.entities.iter().find(|e| &e.id == link_id) {
                let from = tile_to_screen(entity.x, entity.y, center, cam_offset, zoom);
                let to = tile_to_screen(target.x, target.y, center, cam_offset, zoom);
                painter.line_segment([from, to], (2.0, tp_color));
                // Small diamond at midpoint
                let mid = egui::pos2((from.x + to.x) / 2.0, (from.y + to.y) / 2.0);
                painter.circle_filled(mid, 4.0 * zoom, tp_color);
            }
        }
    }

    // --- 3c. Collision visualization overlay (red X pattern) ---
    if show_collision {
        if let Some(coll_layer) = grid.layer(grid::LayerType::Collision) {
            for (&(tx, ty), &tt) in &coll_layer.tiles {
                if tt == grid::TileType::Empty { continue; }
                if tx < vis_min_tx || tx > vis_max_tx || ty < vis_min_ty || ty > vis_max_ty {
                    continue;
                }
                let screen_pos = tile_to_screen(tx, ty, center, cam_offset, zoom);
                let sz = tile_draw;
                let r = egui::Rect::from_center_size(screen_pos, egui::vec2(sz, sz));
                painter.rect_filled(r, 0.0, Color32::from_rgba_unmultiplied(255, 0, 0, 60));
                painter.line_segment(
                    [r.left_top(), r.right_bottom()],
                    (1.0, Color32::from_rgba_unmultiplied(255, 0, 0, 120)),
                );
                painter.line_segment(
                    [r.right_top(), r.left_bottom()],
                    (1.0, Color32::from_rgba_unmultiplied(255, 0, 0, 120)),
                );
            }
        }
    }

    // --- 3d. Selection region overlay (cyan) ---
    {
        let st = world.resource::<EditorUiState>();
        // Persistent selection region
        if let Some((min_x, min_y, max_x, max_y)) = st.selection_region {
            let tl = tile_to_screen(min_x, max_y, center, cam_offset, zoom);
            let br = tile_to_screen(max_x, min_y, center, cam_offset, zoom);
            let half = gpx / 2.0;
            let sel_rect = egui::Rect::from_two_pos(
                egui::pos2(tl.x - half, tl.y - half),
                egui::pos2(br.x + half, br.y + half),
            );
            painter.rect_filled(sel_rect, 0.0, Color32::from_rgba_unmultiplied(0, 255, 255, 25));
            painter.rect_stroke(sel_rect, 0.0, (2.0, Color32::from_rgba_unmultiplied(0, 255, 255, 180)), egui::StrokeKind::Outside);
            // Label
            painter.text(
                egui::pos2(sel_rect.left() + 4.0, sel_rect.top() + 2.0),
                egui::Align2::LEFT_TOP,
                format!("{}x{}", max_x - min_x + 1, max_y - min_y + 1),
                egui::FontId::monospace(10.0),
                Color32::from_rgba_unmultiplied(0, 255, 255, 200),
            );
        }
        // Live drag preview
        if let Some((sx, sy)) = st.region_drag_start {
            if let Some((ex, ey)) = pointer_tile {
                let min_x = sx.min(ex);
                let max_x = sx.max(ex);
                let min_y = sy.min(ey);
                let max_y = sy.max(ey);
                let tl = tile_to_screen(min_x, max_y, center, cam_offset, zoom);
                let br = tile_to_screen(max_x, min_y, center, cam_offset, zoom);
                let half = gpx / 2.0;
                let drag_rect = egui::Rect::from_two_pos(
                    egui::pos2(tl.x - half, tl.y - half),
                    egui::pos2(br.x + half, br.y + half),
                );
                painter.rect_filled(drag_rect, 0.0, Color32::from_rgba_unmultiplied(0, 255, 255, 30));
                painter.rect_stroke(drag_rect, 0.0, (2.0, Color32::from_rgba_unmultiplied(0, 255, 255, 150)), egui::StrokeKind::Outside);
            }
        }
    }

    // --- 3e. Map boundary visualization (red lines + out-of-bounds shading) ---
    {
        let bl = grid.boundary_left;
        let br_bound = grid.boundary_right;
        let left_x = tile_to_screen(bl, 0, center, cam_offset, zoom).x - gpx / 2.0;
        let right_x = tile_to_screen(br_bound, 0, center, cam_offset, zoom).x + gpx / 2.0;

        painter.vline(left_x, rect.y_range(), (2.0, Color32::RED));
        painter.vline(right_x, rect.y_range(), (2.0, Color32::RED));

        // Out-of-bounds shading
        if left_x > rect.left() {
            let oob = egui::Rect::from_min_max(rect.left_top(), egui::pos2(left_x, rect.bottom()));
            painter.rect_filled(oob, 0.0, Color32::from_rgba_unmultiplied(255, 0, 0, 20));
        }
        if right_x < rect.right() {
            let oob = egui::Rect::from_min_max(egui::pos2(right_x, rect.top()), rect.right_bottom());
            painter.rect_filled(oob, 0.0, Color32::from_rgba_unmultiplied(255, 0, 0, 20));
        }

        // Labels
        painter.text(
            egui::pos2(left_x + 4.0, rect.top() + 4.0),
            egui::Align2::LEFT_TOP, "LEFT BOUND",
            egui::FontId::proportional(10.0), Color32::RED,
        );
        painter.text(
            egui::pos2(right_x - 4.0, rect.top() + 4.0),
            egui::Align2::RIGHT_TOP, "RIGHT BOUND",
            egui::FontId::proportional(10.0), Color32::RED,
        );
    }

    // --- 4. Shape tool preview (rect/line) ---
    let drag_start = world.resource::<PaintState>().drag_start;
    if let Some((sx, sy)) = drag_start {
        if let Some((ex, ey)) = pointer_tile {
            let preview_color = Color32::from_rgba_unmultiplied(0, 255, 255, 50);
            let stroke_color = Color32::from_rgba_unmultiplied(0, 255, 255, 200);

            if tool == EditorTool::Rectangle {
                let min_x = sx.min(ex);
                let max_x = sx.max(ex);
                let min_y = sy.min(ey);
                let max_y = sy.max(ey);
                for px in min_x..=max_x {
                    for py in min_y..=max_y {
                        let pos = tile_to_screen(px, py, center, cam_offset, zoom);
                        let r = egui::Rect::from_center_size(pos, egui::vec2(tile_draw, tile_draw));
                        painter.rect_filled(r, 0.0, preview_color);
                    }
                }
                let tl = tile_to_screen(min_x, max_y, center, cam_offset, zoom);
                let br = tile_to_screen(max_x, min_y, center, cam_offset, zoom);
                let outline = egui::Rect::from_two_pos(
                    egui::pos2(tl.x - gpx / 2.0, tl.y - gpx / 2.0),
                    egui::pos2(br.x + gpx / 2.0, br.y + gpx / 2.0),
                );
                painter.rect_stroke(outline, 0.0, (2.0, stroke_color), egui::StrokeKind::Outside);
            } else if tool == EditorTool::Line {
                let s = tile_to_screen(sx, sy, center, cam_offset, zoom);
                let e = tile_to_screen(ex, ey, center, cam_offset, zoom);
                painter.line_segment([s, e], (2.0, stroke_color));
            }
        }
    }

    // --- 5. Coordinate tooltip + zoom indicator ---
    if let Some((tx, ty)) = pointer_tile {
        if let Some(pointer_pos) = ui.input(|i| i.pointer.hover_pos()) {
            let tool_label = tool.label();
            let tile_name = grid::palette_item(tile).map_or("?", |p| p.name);
            painter.text(
                egui::pos2(pointer_pos.x + 14.0, pointer_pos.y + 14.0),
                egui::Align2::LEFT_TOP,
                format!("({}, {}) [{}:{}] {:.1}x", tx, ty, tool_label, tile_name, zoom),
                egui::FontId::monospace(11.0),
                Color32::from_rgba_unmultiplied(255, 255, 255, 180),
            );
        }
    }

    // --- 6. Ghost cursor preview (grid-snapped, zoom-aware) ---
    if matches!(tool, EditorTool::Brush | EditorTool::Eraser) {
        if let Some((tx, ty)) = pointer_tile {
            let size = brush_size as i32;
            let boff = size / 2;
            let ghost_base = if tool == EditorTool::Eraser {
                Color32::from_rgba_unmultiplied(255, 80, 80, 80)
            } else {
                let base = grid::tile_color(tile);
                Color32::from_rgba_unmultiplied(base.r(), base.g(), base.b(), 80)
            };
            for dx in 0..size {
                for dy in 0..size {
                    let gx = tx - boff + dx;
                    let gy = ty - boff + dy;
                    let pos = tile_to_screen(gx, gy, center, cam_offset, zoom);
                    let r = egui::Rect::from_center_size(pos, egui::vec2(tile_draw, tile_draw));
                    painter.rect_filled(r, 2.0, ghost_base);
                    if tool == EditorTool::Brush && font_size >= 8.0 {
                        let label = grid::tile_label(tile);
                        painter.text(
                            pos,
                            egui::Align2::CENTER_CENTER,
                            label.to_string(),
                            egui::FontId::monospace(font_size),
                            Color32::from_rgba_unmultiplied(255, 255, 255, 120),
                        );
                    }
                }
            }
            let tl = tile_to_screen(tx - boff, ty + boff, center, cam_offset, zoom);
            let br = tile_to_screen(tx - boff + size - 1, ty + boff - size + 1, center, cam_offset, zoom);
            let outline = egui::Rect::from_two_pos(
                egui::pos2(tl.x - gpx / 2.0, tl.y - gpx / 2.0),
                egui::pos2(br.x + gpx / 2.0, br.y + gpx / 2.0),
            );
            let outline_color = if tool == EditorTool::Eraser {
                Color32::from_rgba_unmultiplied(255, 80, 80, 200)
            } else {
                let base = grid::tile_color(tile);
                Color32::from_rgba_unmultiplied(base.r(), base.g(), base.b(), 200)
            };
            painter.rect_stroke(outline, 0.0, (2.0, outline_color), egui::StrokeKind::Outside);
        }
    }
}

pub(crate) fn draw_story_graph(ui: &mut egui::Ui, world: &mut World) {
    let painter = ui.painter().clone(); // Clone painter to avoid borrow issues? No, ui.painter() returns reference.
                                        // We need to be careful with borrowing world and ui.

    let rect = ui.available_rect_before_wrap();

    // Darker background
    painter.rect_filled(rect, 0.0, Color32::from_rgb(10, 10, 15));

    // Context Menu
    let mut add_node_cmd = None;

    // We can't access world inside context_menu closure easily if we are borrowing it from outside?
    // Egui context menu runs immediately.
    let response = ui.allocate_rect(rect, egui::Sense::click());

    response.context_menu(|ui| {
        ui.label("Add Node");
        ui.separator();
        if ui.button("Start Node").clicked() {
            add_node_cmd = Some("Start");
            ui.close();
        }
        if ui.button("Dialogue Node").clicked() {
            add_node_cmd = Some("Dialogue");
            ui.close();
        }
        if ui.button("End Node").clicked() {
            add_node_cmd = Some("End");
            ui.close();
        }
    });

    if let Some(cmd) = add_node_cmd {
        world.resource_scope::<ActiveStoryGraph, _>(|_, mut graph| {
            let id = format!("node_{}", graph.0.nodes.len());
            let pos = response.interact_pointer_pos().unwrap_or(rect.center());
            // Adjust to be relative to panel if needed, but we store absolute screen coords for simpler drag?
            // Ideally relative to rect.min.
            let rel_pos = pos - rect.min;

            let mut node = match cmd {
                "Start" => StoryNodeData::start(id.clone(), None::<String>),
                "Dialogue" => StoryNodeData::dialogue(id.clone(), "Stranger", "Hello world"),
                "End" => StoryNodeData::end(id.clone()),
                _ => StoryNodeData::dialogue(id.clone(), "Err", "Err"),
            };

            // Set position
            node.position = Vec3Data::new(rel_pos.x, rel_pos.y, 0.0);

            // If start, set root
            if cmd == "Start" {
                graph.0.root_node_id = id.clone();
            }

            graph.0.add_node(node);
        });
    }

    // DRAW NODES AND LINES
    // We need to scope world to get graph
    world.resource_scope::<ActiveStoryGraph, _>(|world, mut graph| {
        let mut ui_state = world.resource_mut::<EditorUiState>();

        // 1. Draw Connections
        for node in &graph.0.nodes {
            let start_pos =
                rect.min + egui::vec2(node.position.x, node.position.y) + egui::vec2(100.0, 25.0); // Approx right side

            for next_id in node.next_node_ids() {
                if let Some(target) = graph.0.find_node(next_id) {
                    let end_pos = rect.min
                        + egui::vec2(target.position.x, target.position.y)
                        + egui::vec2(0.0, 25.0); // Approx left side
                    painter.line_segment([start_pos, end_pos], (2.0, Color32::GRAY));
                }
            }

            // Draw active drag line
            if let Some(start_id) = &ui_state.connection_start_id {
                if start_id == &node.id {
                    if let Some(pointer) = ui.input(|i| i.pointer.hover_pos()) {
                        painter.line_segment([start_pos, pointer], (2.0, Color32::YELLOW));
                    }
                }
            }
        }

        // 2. Draw Nodes
        let mut node_to_update_pos = None;
        let mut connection_established = None; // (from, to)

        for node in &mut graph.0.nodes {
            let node_rect = egui::Rect::from_min_size(
                rect.min + egui::vec2(node.position.x, node.position.y),
                egui::vec2(150.0, 80.0),
            );

            // Background
            let color = match node.node_type() {
                StoryNodeType::Start => Color32::from_rgb(50, 200, 100),
                StoryNodeType::End => Color32::from_rgb(200, 50, 50),
                StoryNodeType::Dialogue => Color32::from_rgb(50, 100, 200),
                _ => Color32::from_rgb(100, 100, 100),
            };

            painter.rect_filled(node_rect, 5.0, color);
            painter.rect_stroke(
                node_rect,
                5.0,
                (1.0, Color32::WHITE),
                egui::StrokeKind::Inside,
            );

            // Content
            painter.text(
                node_rect.min + egui::vec2(10.0, 10.0),
                egui::Align2::LEFT_TOP,
                &node.id,
                egui::FontId::proportional(14.0),
                Color32::WHITE,
            );
            painter.text(
                node_rect.min + egui::vec2(10.0, 30.0),
                egui::Align2::LEFT_TOP,
                format!("{:?}", node.node_type()),
                egui::FontId::proportional(12.0),
                Color32::BLACK,
            );

            // Interaction
            let response = ui.allocate_rect(node_rect, egui::Sense::drag());
            if response.dragged() {
                node_to_update_pos = Some((node.id.clone(), response.drag_delta()));
            }
            if response.clicked() {
                ui_state.selected_node_id = Some(node.id.clone());
            }

            // Connect Button (Little circle on right)
            let port_rect =
                egui::Rect::from_center_size(node_rect.right_center(), egui::vec2(12.0, 12.0));
            painter.circle_filled(port_rect.center(), 6.0, Color32::WHITE);
            let port_resp = ui.allocate_rect(port_rect, egui::Sense::click());

            if port_resp.clicked() {
                if let Some(start_id) = ui_state.connection_start_id.clone() {
                    // Complete connection
                    if start_id != node.id {
                        connection_established = Some((start_id, node.id.clone()));
                        ui_state.connection_start_id = None;
                    }
                } else {
                    // Start connection
                    ui_state.connection_start_id = Some(node.id.clone());
                }
            }

            // If clicking node body while connecting, also connect (easier target)
            if response.clicked() && ui_state.connection_start_id.is_some() {
                if let Some(start_id) = &ui_state.connection_start_id {
                    if start_id != &node.id {
                        connection_established = Some((start_id.clone(), node.id.clone()));
                        ui_state.connection_start_id = None;
                    }
                }
            }
        }

        // Apply position updates
        if let Some((id, delta)) = node_to_update_pos {
            if let Some(node) = graph.0.nodes.iter_mut().find(|n| n.id == id) {
                node.position.x += delta.x;
                node.position.y += delta.y;
            }
        }

        // Apply connection
        if let Some((from, to)) = connection_established {
            if let Some(node) = graph.0.nodes.iter_mut().find(|n| n.id == from) {
                node.data.set_next_node_id(to);
            }
        }
    });
}
