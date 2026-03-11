use crate::data::story::graph::StoryGraphData;
use bevy::prelude::*;
use bevy_egui::egui::Color32;
use bevy_inspector_egui::bevy_inspector;

pub(crate) const COLOR_PRIMARY: Color32 = Color32::from_rgb(0, 255, 204); // Cyberpunk Mint
pub(crate) const COLOR_SECONDARY: Color32 = Color32::from_rgb(255, 175, 200); // Pale Rose
pub(crate) const COLOR_BG: Color32 = Color32::from_rgb(15, 15, 20);

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum EditorState {
    #[default]
    Editor,
    Playing,
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
