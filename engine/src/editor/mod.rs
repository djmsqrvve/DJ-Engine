//! Editor UI for DJ Engine.
//!
//! Provides a professional game development environment using Egui.

mod extensions;
pub(crate) mod panels;
mod plugin;
pub(crate) mod scene_io;
pub(crate) mod types;
pub mod validation;
pub(crate) mod views;

pub use crate::project_mount::MountedProject;
pub use extensions::{
    AppEditorExtensionExt, EditorExtensionRegistry, RegisteredCustomEditorPanel,
    RegisteredPreviewPreset, RegisteredToolbarAction, RegisteredValidationView,
};
pub use plugin::EditorPlugin;
pub use types::ActiveStoryGraph;
pub use types::{
    BrowserTab, EditorDirtyState, EditorSnapshotBaseline, EditorState, EditorUiState, EditorView,
    PendingProjectAction, PendingProjectActionResolution, RuntimePreviewLaunchPhase,
    RuntimePreviewLaunchState,
};

use bevy::prelude::*;
use bevy_egui::{egui, PrimaryEguiContext};

pub(crate) fn editor_ui_system(world: &mut World) {
    let egui_ctx = {
        let mut egui_query =
            world.query_filtered::<&mut bevy_egui::EguiContext, With<PrimaryEguiContext>>();
        let Ok(mut egui_context) = egui_query.single_mut(world) else {
            warn!("Editor UI: primary Egui context unavailable, skipping frame");
            return;
        };
        egui_context.get_mut().clone()
    };

    egui::TopBottomPanel::top("top_panel").show(&egui_ctx, |ui| {
        panels::draw_top_menu(ui, world);
    });

    if world.resource::<types::EditorUiState>().console_open {
        panels::draw_console_window(&egui_ctx, world);
    }

    panels::draw_pending_project_action_window(&egui_ctx, world);

    egui::SidePanel::left("left_panel")
        .default_width(250.0)
        .show(&egui_ctx, |ui| {
            panels::draw_left_panel(ui, world);
        });

    egui::SidePanel::right("right_panel")
        .default_width(300.0)
        .show(&egui_ctx, |ui| {
            panels::draw_right_panel(ui, world);
        });

    let current_state = world.resource::<State<types::EditorState>>().get();
    let central_frame = if *current_state == types::EditorState::GraphPreview {
        egui::Frame::NONE
    } else {
        egui::Frame::central_panel(&egui_ctx.style())
    };

    egui::CentralPanel::default()
        .frame(central_frame)
        .show(&egui_ctx, |ui| {
            panels::draw_central_panel(ui, world);
        });
}
