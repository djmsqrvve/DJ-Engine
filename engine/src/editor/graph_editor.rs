use bevy::prelude::*;
use bevy_egui::egui::{self, Color32, Pos2, RichText, Vec2 as EguiVec2};

use crate::data::custom::{DocumentLinkTarget, EditorDocumentRoute, LoadedCustomDocuments};

use super::types::{EditorUiState, COLOR_PRIMARY, COLOR_SECONDARY};

/// State for the generic document graph editor.
#[derive(Resource)]
pub struct GraphEditorState {
    /// Currently displayed document kind (e.g., "helix_talents").
    pub active_kind: Option<String>,
    /// Pan offset for the graph canvas.
    pub pan_offset: EguiVec2,
    /// Zoom level (1.0 = 100%).
    pub zoom: f32,
    /// Node positions keyed by document ID (persisted during session).
    pub node_positions: std::collections::HashMap<String, Pos2>,
    /// Currently selected node ID.
    pub selected_node: Option<String>,
}

impl Default for GraphEditorState {
    fn default() -> Self {
        Self {
            active_kind: None,
            pan_offset: EguiVec2::ZERO,
            zoom: 1.0,
            node_positions: std::collections::HashMap::new(),
            selected_node: None,
        }
    }
}

/// Compute grid positions for `count` nodes arranged in columns of 5 with
/// the given spacing.  Returns a vec of `Pos2` in row-major order.
pub(crate) fn auto_layout_positions(count: usize, spacing: f32) -> Vec<Pos2> {
    let cols = 5usize;
    (0..count)
        .map(|i| {
            let col = i % cols;
            let row = i / cols;
            Pos2::new(
                40.0 + col as f32 * spacing,
                40.0 + row as f32 * (spacing * 0.5),
            )
        })
        .collect()
}

/// Hash a kind string to a deterministic colour so each kind gets its own
/// tint.  This is intentionally simple (not cryptographic).
fn kind_color(kind: &str) -> Color32 {
    let mut h: u32 = 5381;
    for b in kind.bytes() {
        h = h.wrapping_mul(33).wrapping_add(b as u32);
    }
    let r = ((h & 0xFF) as u8).max(60);
    let g = (((h >> 8) & 0xFF) as u8).max(60);
    let b = (((h >> 16) & 0xFF) as u8).max(60);
    Color32::from_rgb(r, g, b)
}

/// Draw the graph editor for relationship-heavy document kinds.
pub(crate) fn draw_graph_editor(ui: &mut egui::Ui, world: &mut World) {
    // ── Collect graph-routed kinds ──────────────────────────────────────
    let loaded = world.resource::<LoadedCustomDocuments>().clone();
    let graph_kinds: Vec<String> = {
        let mut seen = std::collections::BTreeSet::new();
        for doc in &loaded.documents {
            if doc.resolved_route == EditorDocumentRoute::Graph {
                seen.insert(doc.entry.kind.clone());
            }
        }
        seen.into_iter().collect()
    };

    if graph_kinds.is_empty() {
        ui.centered_and_justified(|ui| {
            ui.label(
                RichText::new("No document kinds are routed to the graph editor.")
                    .color(Color32::GRAY),
            );
        });
        return;
    }

    // ── Kind selector bar ───────────────────────────────────────────────
    let mut state = world.resource_mut::<GraphEditorState>();

    // Default to first available kind if nothing selected or selection invalid.
    if state.active_kind.is_none()
        || !graph_kinds.contains(state.active_kind.as_ref().unwrap_or(&String::new()))
    {
        state.active_kind = graph_kinds.first().cloned();
    }

    ui.horizontal(|ui| {
        ui.label(RichText::new("Kind:").strong().color(COLOR_PRIMARY));
        let mut state = world.resource_mut::<GraphEditorState>();
        for kind in &graph_kinds {
            let selected = state.active_kind.as_deref() == Some(kind.as_str());
            let label = RichText::new(kind).color(if selected {
                COLOR_PRIMARY
            } else {
                Color32::GRAY
            });
            if ui.selectable_label(selected, label).clicked() {
                state.active_kind = Some(kind.clone());
                // Reset positions so the new kind gets auto-laid out.
                state.node_positions.clear();
                state.selected_node = None;
            }
        }
    });

    ui.separator();

    // ── Resolve active kind and filter documents ────────────────────────
    let active_kind = world
        .resource::<GraphEditorState>()
        .active_kind
        .clone()
        .unwrap_or_default();

    // Collect documents for the active kind.
    struct DocInfo {
        id: String,
        label: Option<String>,
        /// (target_kind, target_id) pairs from references.
        outgoing: Vec<(String, String)>,
    }

    let docs: Vec<DocInfo> = loaded
        .documents
        .iter()
        .filter(|d| d.entry.kind == active_kind)
        .map(|d| {
            let outgoing = d
                .document
                .as_ref()
                .map(|env| {
                    env.references
                        .iter()
                        .filter_map(|link| match &link.target {
                            DocumentLinkTarget::Document { kind, id } => {
                                Some((kind.clone(), id.clone()))
                            }
                            _ => None,
                        })
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            DocInfo {
                id: d.entry.id.clone(),
                label: d.document.as_ref().and_then(|env| env.label.clone()),
                outgoing,
            }
        })
        .collect();

    if docs.is_empty() {
        ui.centered_and_justified(|ui| {
            ui.label(
                RichText::new(format!("No documents loaded for kind '{active_kind}'."))
                    .color(Color32::GRAY),
            );
        });
        return;
    }

    // ── Auto-layout if no positions saved for these docs ────────────────
    {
        let mut state = world.resource_mut::<GraphEditorState>();
        let needs_layout = docs
            .iter()
            .any(|d| !state.node_positions.contains_key(&d.id));
        if needs_layout {
            let positions = auto_layout_positions(docs.len(), 200.0);
            for (i, doc) in docs.iter().enumerate() {
                state
                    .node_positions
                    .entry(doc.id.clone())
                    .or_insert(positions[i]);
            }
        }
    }

    // ── Canvas ──────────────────────────────────────────────────────────
    let rect = ui.available_rect_before_wrap();
    let painter = ui.painter().clone();

    // Background
    painter.rect_filled(rect, 0.0, Color32::from_rgb(10, 10, 15));

    let state_snap = {
        let s = world.resource::<GraphEditorState>();
        (s.pan_offset, s.zoom, s.selected_node.clone())
    };
    let (pan, zoom, selected_node) = state_snap;

    let node_color = kind_color(&active_kind);
    let node_w = 160.0 * zoom;
    let node_h = 60.0 * zoom;

    // Build a lookup of id -> screen position (centre).
    let id_to_center: std::collections::HashMap<&str, Pos2> = {
        let state = world.resource::<GraphEditorState>();
        docs.iter()
            .filter_map(|d| {
                state.node_positions.get(&d.id).map(|pos| {
                    let screen = Pos2::new(
                        rect.min.x + pos.x * zoom + pan.x,
                        rect.min.y + pos.y * zoom + pan.y,
                    );
                    (d.id.as_str(), screen)
                })
            })
            .collect()
    };

    // ── Draw edges ──────────────────────────────────────────────────────
    for doc in &docs {
        let Some(&src_center) = id_to_center.get(doc.id.as_str()) else {
            continue;
        };
        let src_bottom = Pos2::new(src_center.x, src_center.y + node_h * 0.5);
        for (_target_kind, target_id) in &doc.outgoing {
            if let Some(&dst_center) = id_to_center.get(target_id.as_str()) {
                let dst_top = Pos2::new(dst_center.x, dst_center.y - node_h * 0.5);
                painter.line_segment(
                    [src_bottom, dst_top],
                    (1.5, Color32::from_rgb(120, 120, 120)),
                );
                // Small arrowhead at target
                let dir = (dst_top.to_vec2() - src_bottom.to_vec2()).normalized();
                let perp = EguiVec2::new(-dir.y, dir.x) * 5.0;
                let tip = dst_top;
                let base = tip - dir * 10.0;
                painter.line_segment([tip, base + perp], (1.5, Color32::from_rgb(120, 120, 120)));
                painter.line_segment([tip, base - perp], (1.5, Color32::from_rgb(120, 120, 120)));
            }
        }
    }

    // ── Draw nodes & handle interaction ─────────────────────────────────
    let mut clicked_node: Option<String> = None;
    let mut dragged_node: Option<(String, EguiVec2)> = None;

    for doc in &docs {
        let Some(&center) = id_to_center.get(doc.id.as_str()) else {
            continue;
        };

        let node_rect = egui::Rect::from_center_size(center, egui::vec2(node_w, node_h));

        if !rect.intersects(node_rect) {
            continue; // off-screen culling
        }

        let is_selected = selected_node.as_deref() == Some(&doc.id);

        // Background fill
        let fill = if is_selected {
            node_color.linear_multiply(1.4)
        } else {
            node_color
        };
        painter.rect_filled(node_rect, 4.0, fill);

        // Border
        let stroke_color = if is_selected {
            COLOR_PRIMARY
        } else {
            Color32::from_rgb(80, 80, 80)
        };
        painter.rect_stroke(
            node_rect,
            4.0,
            (if is_selected { 2.0 } else { 1.0 }, stroke_color),
            egui::StrokeKind::Inside,
        );

        // ID text
        let font_size = (13.0 * zoom).max(9.0);
        painter.text(
            node_rect.min + egui::vec2(8.0, 8.0),
            egui::Align2::LEFT_TOP,
            &doc.id,
            egui::FontId::proportional(font_size),
            Color32::WHITE,
        );

        // Label text (if present)
        if let Some(label) = &doc.label {
            painter.text(
                node_rect.min + egui::vec2(8.0, 8.0 + font_size + 4.0),
                egui::Align2::LEFT_TOP,
                label,
                egui::FontId::proportional((font_size - 2.0).max(8.0)),
                COLOR_SECONDARY,
            );
        }

        // Reference count badge
        if !doc.outgoing.is_empty() {
            let badge_text = format!("{} ref", doc.outgoing.len());
            painter.text(
                Pos2::new(node_rect.right() - 8.0, node_rect.bottom() - 8.0),
                egui::Align2::RIGHT_BOTTOM,
                badge_text,
                egui::FontId::proportional((font_size - 3.0).max(7.0)),
                Color32::from_rgb(180, 180, 180),
            );
        }

        // Interaction
        let resp = ui.allocate_rect(node_rect, egui::Sense::click_and_drag());
        if resp.clicked() {
            clicked_node = Some(doc.id.clone());
        }
        if resp.dragged() {
            dragged_node = Some((doc.id.clone(), resp.drag_delta()));
        }
    }

    // ── Canvas pan (middle-mouse or Ctrl+drag on background) ────────────
    let bg_resp = ui.allocate_rect(rect, egui::Sense::click_and_drag());
    let ctrl_held = ui.input(|i| i.modifiers.ctrl);
    let middle_dragged = ui.input(|i| i.pointer.middle_down()) && bg_resp.dragged();
    if bg_resp.dragged() && (ctrl_held || middle_dragged) && dragged_node.is_none() {
        let mut state = world.resource_mut::<GraphEditorState>();
        state.pan_offset += bg_resp.drag_delta();
    }

    // ── Apply node drag ─────────────────────────────────────────────────
    if let Some((id, delta)) = dragged_node {
        let mut state = world.resource_mut::<GraphEditorState>();
        if let Some(pos) = state.node_positions.get_mut(&id) {
            let inv_zoom = if zoom.abs() > f32::EPSILON {
                1.0 / zoom
            } else {
                1.0
            };
            pos.x += delta.x * inv_zoom;
            pos.y += delta.y * inv_zoom;
        }
    }

    // ── Apply selection ─────────────────────────────────────────────────
    if let Some(id) = clicked_node {
        let kind = active_kind.clone();
        {
            let mut state = world.resource_mut::<GraphEditorState>();
            state.selected_node = Some(id.clone());
        }
        // Also update the main editor selection so the inspector shows this doc.
        let mut ui_state = world.resource_mut::<EditorUiState>();
        ui_state.selected_custom_document = Some(crate::data::DocumentRef { kind, id });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graph_editor_state_default() {
        let state = GraphEditorState::default();
        assert!(state.active_kind.is_none());
        assert_eq!(state.zoom, 1.0);
        assert_eq!(state.pan_offset, EguiVec2::ZERO);
        assert!(state.node_positions.is_empty());
        assert!(state.selected_node.is_none());
    }

    #[test]
    fn test_auto_layout_positions() {
        let positions = auto_layout_positions(7, 200.0);
        assert_eq!(positions.len(), 7);

        // First row: 5 nodes
        assert_eq!(positions[0], Pos2::new(40.0, 40.0));
        assert_eq!(positions[1], Pos2::new(240.0, 40.0));
        assert_eq!(positions[4], Pos2::new(840.0, 40.0));
        // Second row: 2 nodes
        assert_eq!(positions[5], Pos2::new(40.0, 140.0));
        assert_eq!(positions[6], Pos2::new(240.0, 140.0));
    }

    #[test]
    fn test_node_positions_persist() {
        let mut state = GraphEditorState::default();
        state
            .node_positions
            .insert("warrior".into(), Pos2::new(100.0, 200.0));
        state
            .node_positions
            .insert("mage".into(), Pos2::new(300.0, 200.0));

        assert_eq!(state.node_positions.len(), 2);
        assert_eq!(
            state.node_positions.get("warrior"),
            Some(&Pos2::new(100.0, 200.0))
        );
        assert_eq!(
            state.node_positions.get("mage"),
            Some(&Pos2::new(300.0, 200.0))
        );
    }
}
