use super::types::{ActiveStoryGraph, EditorUiState, COLOR_BG, COLOR_PRIMARY};
use crate::data::components::Vec3Data;
use crate::data::story::{StoryNodeData, StoryNodeType, StoryNodeVariant};
use bevy::prelude::*;
use bevy_egui::egui::{self, Color32};

pub(crate) fn draw_grid(ui: &mut egui::Ui, world: &mut World) {
    let rect = ui.available_rect_before_wrap();

    // 1. Handle Input (Placement)
    // We do this before drawing so the new item appears immediately (or next frame)
    let response = ui.allocate_rect(rect, egui::Sense::click());

    // Now valid to create painter after mutable borrow is done (or rather, we don't hold the painter while mutating ui via allocate_rect if we scope it,
    // but ui.painter() borrows ui. allocate_rect borrows ui mutably.
    // So we must call allocate_rect first, THEN get painter.
    let painter = ui.painter();

    if response.clicked() {
        if let Some(pointer_pos) = ui.input(|i| i.pointer.interact_pos()) {
            // Convert UI coordinates to "World" coordinates relative to the panel
            // For this 2D editor prototype, we treat the top-left of the panel as (0,0) world space for simplicity,
            // or we center it. Let's map it simply for now.
            let _relative_pos = pointer_pos - rect.min;

            // Snap to grid
            let grid_size = 40.0;
            // let grid_x = (relative_pos.x / grid_size).floor() * grid_size;
            // let grid_y = (relative_pos.y / grid_size).floor() * grid_size;

            // Bevy coordinates: Y is up, X is right. Center is (0,0).
            // Egui coordinates: Y is down, X is right. Top-left is (0,0).
            // We need a translation. For this visual prototype, we'll just spawn at a transform
            // that roughly aligns with where we clicked visually if we assume a standard 2D camera.
            // But since we aren't rendering the world *in* the egui panel yet (just a grid overlay),
            // this is a "blind" spawn into the world.
            // However, the Hierarchy will update, confirming the action.

            // Let's spawn at a 3D position assuming Z=0 plane.
            // We'll map the panel center to World (0,0).
            let center = rect.center();
            let world_x = pointer_pos.x - center.x;
            let world_y = center.y - pointer_pos.y; // Flip Y for Bevy

            let snap_x = (world_x / grid_size).round() * grid_size;
            let snap_y = (world_y / grid_size).round() * grid_size;

            let selected_item = world
                .resource::<EditorUiState>()
                .selected_palette_item
                .clone();

            if let Some(item) = selected_item {
                info!("Editor: Spawning {} at ({}, {})", item, snap_x, snap_y);

                // Determine color based on item
                let color = match item.as_str() {
                    "Grass" => Color::srgb(0.2, 0.8, 0.2),
                    "Wall" => Color::srgb(0.5, 0.5, 0.5),
                    "Hamster" => Color::srgb(0.8, 0.5, 0.2),
                    "Chest" => Color::srgb(0.8, 0.8, 0.1),
                    _ => Color::WHITE,
                };

                world.spawn((
                    Name::new(format!("{} [{:.0}, {:.0}]", item, snap_x, snap_y)),
                    Sprite {
                        color,
                        custom_size: Some(Vec2::new(30.0, 30.0)),
                        ..default()
                    },
                    Transform::from_xyz(snap_x, snap_y, 0.0),
                ));
            }
        }
    }

    // 2. Draw Grid Visuals
    painter.rect_filled(rect, 0.0, COLOR_BG);

    let grid_size = 40.0;
    let color = Color32::from_rgb(30, 30, 40);

    let mut x = rect.left();
    while x < rect.right() {
        painter.line_segment(
            [egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())],
            (1.0, color),
        );
        x += grid_size;
    }

    let mut y = rect.top();
    while y < rect.bottom() {
        painter.line_segment(
            [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)],
            (1.0, color),
        );
        y += grid_size;
    }

    // Draw ghost of selected item at mouse cursor
    if let Some(_item) = &world.resource::<EditorUiState>().selected_palette_item {
        if let Some(pointer_pos) = ui.input(|i| i.pointer.hover_pos()) {
            if rect.contains(pointer_pos) {
                painter.circle_filled(pointer_pos, 5.0, COLOR_PRIMARY);
            }
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
                // Ugly mutation manually based on type
                // TODO: Add helper 'set_next' to StoryNodeData
                match &mut node.data {
                    StoryNodeVariant::Start(d) => d.next_node_id = Some(to),
                    StoryNodeVariant::Dialogue(d) => d.next_node_id = Some(to),
                    StoryNodeVariant::Action(a) => a.next_node_id = Some(to),
                    _ => {}
                }
            }
        }
    });
}
