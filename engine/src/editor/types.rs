use crate::data::story::graph::StoryGraphData;
use bevy::prelude::*;
use bevy_egui::egui::Color32;
use bevy_inspector_egui::bevy_inspector;
use std::{
    path::PathBuf,
    process::Child,
    sync::{Arc, Mutex},
};

pub(crate) const COLOR_PRIMARY: Color32 = Color32::from_rgb(0, 255, 204); // Cyberpunk Mint
pub(crate) const COLOR_SECONDARY: Color32 = Color32::from_rgb(255, 175, 200); // Pale Rose
pub(crate) const COLOR_BG: Color32 = Color32::from_rgb(15, 15, 20);

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum EditorState {
    #[default]
    Editor,
    GraphPreview,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum RuntimePreviewLaunchPhase {
    #[default]
    Idle,
    Launching,
    Running,
    Stopping,
    Failed,
}

#[derive(Resource, Default)]
pub struct RuntimePreviewLaunchState {
    pub phase: RuntimePreviewLaunchPhase,
    pub manifest_path: Option<PathBuf>,
    pub status_message: Option<String>,
    pub last_exit: Option<String>,
    pub process: Option<Arc<Mutex<Child>>>,
}

impl RuntimePreviewLaunchState {
    pub fn is_running(&self) -> bool {
        matches!(
            self.phase,
            RuntimePreviewLaunchPhase::Launching
                | RuntimePreviewLaunchPhase::Running
                | RuntimePreviewLaunchPhase::Stopping
        ) && self.process.is_some()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum EditorView {
    #[default]
    Level,
    StoryGraph,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum BrowserTab {
    #[default]
    Hierarchy,
    Assets,
    Palette,
}

#[derive(Resource, Default)]
pub struct ActiveStoryGraph(pub StoryGraphData);

#[derive(Resource, Default)]
pub struct EditorUiState {
    pub current_view: EditorView,
    pub browser_tab: BrowserTab,
    // We don't need Option<Entity> anymore, SelectedEntities handles it
    pub selected_entities: bevy_inspector::hierarchy::SelectedEntities,
    pub asset_search_query: String,
    pub selected_palette_item: Option<String>,
    pub console_open: bool,
    pub dragged_node_id: Option<String>,
    pub connection_start_id: Option<String>,
    pub selected_node_id: Option<String>,
}
