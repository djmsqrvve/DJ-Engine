use crate::data::scene::{Entity, EntityType, Scene};
use crate::data::story::{SceneValidationError, StoryGraphData};
use bevy::prelude::*;
use bevy_egui::egui;

/// Resource for managing editor validation state.
#[derive(Resource, Default)]
pub struct ValidationState {
    pub errors: Vec<SceneValidationError>,
    pub last_validation_time: f64,
}

/// Panel for displaying validation errors and missing items.
pub fn draw_validation_panel(
    ui: &mut egui::Ui,
    validation_state: &mut ValidationState,
    active_graph: Option<&StoryGraphData>,
    active_scene: Option<&mut Scene>,
) {
    ui.heading("Validation & Missing Items");
    ui.separator();

    if active_graph.is_none() || active_scene.is_none() {
        ui.label("Open a Scene and a Story Graph to validate.");
        return;
    }

    let graph = active_graph.unwrap();
    let scene = active_scene.unwrap(); // We need mutable access for fixing issues

    // Validate if needed (e.g. periodically or on change, for now we assume errors are populated)
    // In a real implementation, we might trigger this on events.
    // For this immediate mode UI, let's re-validate every frame for simplicity,
    // or rely on the system calling this to update `validation_state`.
    // Let's re-validate here for the "Missing Items" feature demonstration.
    validation_state.errors = graph.validate_against_scene(scene);

    if validation_state.errors.is_empty() {
        ui.label(egui::RichText::new("✓ All checks passed").color(egui::Color32::GREEN));
    } else {
        ui.label(
            egui::RichText::new(format!("⚠ Found {} issues", validation_state.errors.len()))
                .color(egui::Color32::YELLOW),
        );
        ui.separator();

        egui::ScrollArea::vertical().show(ui, |ui| {
            // Iterate over errors. Note: fixing modifies scene, so we need to be careful with borrowing.
            // We collect fixes to apply after the loop.
            let mut fixes_to_apply = Vec::new();

            for (index, error) in validation_state.errors.iter().enumerate() {
                ui.group(|ui| {
                    match error {
                        SceneValidationError::MissingRequiredEntity { node_id, entity_id } => {
                            ui.horizontal(|ui| {
                                ui.label(
                                    egui::RichText::new("Missing Entity")
                                        .strong()
                                        .color(egui::Color32::RED),
                                );
                                ui.label(format!(
                                    "Node '{}' requires entity '{}'",
                                    node_id, entity_id
                                ));
                            });

                            // Feature: Drag and drop / Auto-fix
                            // We simulate "dragging" by clicking a button that spawns the entity.
                            if ui.button(format!("Fix: Spawn {}", entity_id)).clicked() {
                                fixes_to_apply.push((index, "spawn_entity", entity_id.clone()));
                            }
                        }
                        SceneValidationError::WrongEntityType {
                            node_id,
                            entity_id,
                            expected,
                            found,
                        } => {
                            ui.horizontal(|ui| {
                                ui.label(
                                    egui::RichText::new("Type Mismatch")
                                        .strong()
                                        .color(egui::Color32::ORANGE),
                                );
                                ui.label(format!(
                                    "Node '{}' expects '{}' to be {:?}, found {:?}",
                                    node_id, entity_id, expected, found
                                ));
                            });

                            if ui.button("Fix: Update Type").clicked() {
                                fixes_to_apply.push((index, "update_type", entity_id.clone()));
                            }
                        }
                    }
                });
            }

            // Apply fixes
            for (_index, action, id) in fixes_to_apply {
                if action == "spawn_entity" {
                    // Logic to spawn a placeholder entity
                    // In a real editor, this might start a drag payload
                    let mut new_entity = Entity::new(id.clone(), id.clone());
                    // Try to guess type from the error? For Simplicity, default to NPC if unknown
                    new_entity.entity_type = EntityType::Npc;
                    // Verify if we can find the specific requirement to set the correct type
                    if let Some(node) = graph
                        .nodes
                        .iter()
                        .find(|n| n.required_entities.iter().any(|r| r.entity_id == id))
                    {
                        if let Some(req) = node.required_entities.iter().find(|r| r.entity_id == id)
                        {
                            if let Some(etype) = req.entity_type {
                                new_entity.entity_type = etype;
                            }
                        }
                    }

                    scene.add_entity(new_entity);
                } else if action == "update_type" {
                    if let Some(entity) = scene.find_entity_mut(&id) {
                        // Find expected type again
                        if let Some(node) = graph
                            .nodes
                            .iter()
                            .find(|n| n.required_entities.iter().any(|r| r.entity_id == id))
                        {
                            if let Some(req) =
                                node.required_entities.iter().find(|r| r.entity_id == id)
                            {
                                if let Some(etype) = req.entity_type {
                                    entity.entity_type = etype;
                                }
                            }
                        }
                    }
                }
            }
        });
    }
}
