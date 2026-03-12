use crate::data::DocumentKindId;
use bevy::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegisteredCustomEditorPanel {
    pub kind: DocumentKindId,
    pub panel_id: String,
    pub title: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegisteredToolbarAction {
    pub action_id: String,
    pub title: String,
    pub kind_filter: Option<DocumentKindId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegisteredValidationView {
    pub view_id: String,
    pub title: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegisteredPreviewPreset {
    pub preset_id: String,
    pub title: String,
    pub profile_id: Option<String>,
}

#[derive(Resource, Default, Debug, Clone)]
pub struct EditorExtensionRegistry {
    pub custom_panels: Vec<RegisteredCustomEditorPanel>,
    pub toolbar_actions: Vec<RegisteredToolbarAction>,
    pub validation_views: Vec<RegisteredValidationView>,
    pub preview_presets: Vec<RegisteredPreviewPreset>,
}

pub trait AppEditorExtensionExt {
    fn register_custom_editor_panel(&mut self, panel: RegisteredCustomEditorPanel) -> &mut Self;
    fn register_toolbar_action(&mut self, action: RegisteredToolbarAction) -> &mut Self;
    fn register_validation_view(&mut self, view: RegisteredValidationView) -> &mut Self;
    fn register_preview_preset(&mut self, preset: RegisteredPreviewPreset) -> &mut Self;
}

impl AppEditorExtensionExt for App {
    fn register_custom_editor_panel(&mut self, panel: RegisteredCustomEditorPanel) -> &mut Self {
        self.init_resource::<EditorExtensionRegistry>();
        self.world_mut()
            .resource_mut::<EditorExtensionRegistry>()
            .custom_panels
            .push(panel);
        self
    }

    fn register_toolbar_action(&mut self, action: RegisteredToolbarAction) -> &mut Self {
        self.init_resource::<EditorExtensionRegistry>();
        self.world_mut()
            .resource_mut::<EditorExtensionRegistry>()
            .toolbar_actions
            .push(action);
        self
    }

    fn register_validation_view(&mut self, view: RegisteredValidationView) -> &mut Self {
        self.init_resource::<EditorExtensionRegistry>();
        self.world_mut()
            .resource_mut::<EditorExtensionRegistry>()
            .validation_views
            .push(view);
        self
    }

    fn register_preview_preset(&mut self, preset: RegisteredPreviewPreset) -> &mut Self {
        self.init_resource::<EditorExtensionRegistry>();
        self.world_mut()
            .resource_mut::<EditorExtensionRegistry>()
            .preview_presets
            .push(preset);
        self
    }
}
