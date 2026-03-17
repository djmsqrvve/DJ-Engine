//! Contracts browser panel for the editor.
//!
//! Renders a collapsible tree of all registered plugin contracts showing
//! resources, components, events, and system sets.

use crate::contracts::{format_contracts_text, ContractRegistry};
use crate::editor::types::{COLOR_PRIMARY, COLOR_SECONDARY};
use bevy::prelude::*;
use bevy_egui::egui::{self, Color32, RichText};

/// Tracks the last-read snapshot so the Refresh button can re-read.
#[derive(Resource, Default)]
pub(crate) struct ContractsPanelState {
    pub cached: Option<ContractRegistry>,
    pub copy_flash: f32,
}

pub(crate) fn draw_contracts_browser(ui: &mut egui::Ui, world: &mut World) {
    let Some(registry) = world.get_resource::<ContractRegistry>() else {
        ui.label(
            RichText::new("ContractRegistry not available.")
                .italics()
                .color(Color32::GRAY),
        );
        return;
    };
    let registry = registry.clone();

    // Tick copy flash timer
    let dt = world.resource::<Time>().delta_secs();
    world.init_resource::<ContractsPanelState>();
    let mut panel_state = world.resource_mut::<ContractsPanelState>();
    if panel_state.copy_flash > 0.0 {
        panel_state.copy_flash = (panel_state.copy_flash - dt).max(0.0);
    }
    let show_copied = panel_state.copy_flash > 0.0;

    ui.add_space(5.0);
    ui.horizontal(|ui| {
        ui.label(
            RichText::new("ENGINE CONTRACTS")
                .strong()
                .color(COLOR_PRIMARY),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui
                .small_button(RichText::new("Copy").color(COLOR_PRIMARY))
                .clicked()
            {
                let text = format_contracts_text(&registry);
                ui.ctx().copy_text(text);
                world.resource_mut::<ContractsPanelState>().copy_flash = 1.5;
            }
            if ui
                .small_button(RichText::new("Refresh").color(COLOR_SECONDARY))
                .clicked()
            {
                // Registry is populated at build time, so refresh just
                // forces the panel to re-read (useful after hot-reload).
                world
                    .resource_mut::<ContractsPanelState>()
                    .cached
                    .replace(registry.clone());
            }
        });
    });
    ui.add_space(2.0);
    if show_copied {
        ui.label(
            RichText::new("Copied to clipboard!")
                .small()
                .color(Color32::from_rgb(100, 255, 150)),
        );
    }
    ui.label(
        RichText::new(format!(
            "{} plugins  |  {} resources  |  {} components  |  {} events",
            registry.contracts.len(),
            registry.total_resources(),
            registry.total_components(),
            registry.total_events(),
        ))
        .small()
        .color(Color32::GRAY),
    );
    ui.add_space(4.0);
    ui.separator();

    egui::ScrollArea::vertical().show(ui, |ui| {
        for contract in &registry.contracts {
            let total = contract.resources.len()
                + contract.components.len()
                + contract.events.len()
                + contract.system_sets.len();

            let header = format!("{} ({})", contract.name, total);
            egui::CollapsingHeader::new(RichText::new(header).strong())
                .default_open(false)
                .show(ui, |ui| {
                    ui.label(
                        RichText::new(&contract.description)
                            .italics()
                            .color(Color32::LIGHT_GRAY),
                    );
                    ui.add_space(2.0);

                    if !contract.resources.is_empty() {
                        ui.label(
                            RichText::new("Resources").color(Color32::from_rgb(100, 200, 255)),
                        );
                        for entry in &contract.resources {
                            ui.horizontal(|ui| {
                                ui.add_space(12.0);
                                ui.label(&entry.name);
                                ui.label(
                                    RichText::new(&entry.description)
                                        .small()
                                        .color(Color32::GRAY),
                                );
                            });
                        }
                        ui.add_space(2.0);
                    }

                    if !contract.components.is_empty() {
                        ui.label(
                            RichText::new("Components").color(Color32::from_rgb(100, 255, 150)),
                        );
                        for entry in &contract.components {
                            ui.horizontal(|ui| {
                                ui.add_space(12.0);
                                ui.label(&entry.name);
                                ui.label(
                                    RichText::new(&entry.description)
                                        .small()
                                        .color(Color32::GRAY),
                                );
                            });
                        }
                        ui.add_space(2.0);
                    }

                    if !contract.events.is_empty() {
                        ui.label(RichText::new("Events").color(Color32::from_rgb(255, 200, 100)));
                        for entry in &contract.events {
                            ui.horizontal(|ui| {
                                ui.add_space(12.0);
                                ui.label(&entry.name);
                                ui.label(
                                    RichText::new(&entry.description)
                                        .small()
                                        .color(Color32::GRAY),
                                );
                            });
                        }
                        ui.add_space(2.0);
                    }

                    if !contract.system_sets.is_empty() {
                        ui.label(
                            RichText::new("System Sets").color(Color32::from_rgb(200, 150, 255)),
                        );
                        for set in &contract.system_sets {
                            ui.horizontal(|ui| {
                                ui.add_space(12.0);
                                ui.label(&set.name);
                                ui.label(
                                    RichText::new(format!("({})", set.schedule))
                                        .small()
                                        .color(Color32::GRAY),
                                );
                            });
                        }
                    }
                });
        }
    });
}
