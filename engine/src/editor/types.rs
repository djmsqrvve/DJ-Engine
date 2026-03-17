use crate::data::story::graph::StoryGraphData;
use crate::data::DocumentRef;
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PendingProjectAction {
    LoadMountedProject,
    ReloadProject,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PendingProjectActionResolution {
    SaveAndContinue,
    DiscardChanges,
    Cancel,
}

#[derive(Resource, Default, Debug, Clone, PartialEq, Eq)]
pub struct EditorSnapshotBaseline {
    pub scene_json: Option<String>,
    pub story_graph_json: Option<String>,
    pub project_json: Option<String>,
    pub custom_documents_json: Option<String>,
}

#[derive(Resource, Default, Debug, Clone, PartialEq, Eq)]
pub struct EditorDirtyState {
    pub is_dirty: bool,
    pub snapshot_error: Option<String>,
    pub pending_project_action: Option<PendingProjectAction>,
}

#[derive(Resource, Default)]
pub struct RuntimePreviewLaunchState {
    pub phase: RuntimePreviewLaunchPhase,
    pub manifest_path: Option<PathBuf>,
    pub status_message: Option<String>,
    pub last_exit: Option<String>,
    /// Last stderr output from a failed preview process (for diagnostics).
    pub last_error: Option<String>,
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

/// Configurable launch settings for the Helix 3D Renderer.
#[derive(Resource, Debug, Clone)]
pub struct Helix3DLaunchConfig {
    pub model_preset: String,
    pub play_mode: bool,
    pub data_dir: Option<String>,
    pub terrain_dir: Option<String>,
    pub extra_args: Vec<String>,
}

impl Default for Helix3DLaunchConfig {
    fn default() -> Self {
        Self {
            model_preset: "drow".to_string(),
            play_mode: true,
            data_dir: None,
            terrain_dir: None,
            extra_args: Vec::new(),
        }
    }
}

/// Tracks a launched Helix 3D Renderer subprocess.
#[derive(Resource, Default)]
pub struct Helix3DViewerState {
    pub process: Option<Arc<Mutex<Child>>>,
    pub status: Option<String>,
}

impl Helix3DViewerState {
    pub fn is_running(&self) -> bool {
        let Some(ref proc) = self.process else {
            return false;
        };
        let Ok(mut child) = proc.lock() else {
            return false;
        };
        // Check if still running (non-blocking)
        matches!(child.try_wait(), Ok(None))
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
    Documents,
    Palette,
    Contracts,
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
    pub custom_document_search_query: String,
    pub custom_document_kind_filter: String,
    pub selected_custom_document: Option<DocumentRef>,
    pub selected_palette_item: Option<String>,
    pub console_open: bool,
    pub dragged_node_id: Option<String>,
    pub connection_start_id: Option<String>,
    pub selected_node_id: Option<String>,
}
