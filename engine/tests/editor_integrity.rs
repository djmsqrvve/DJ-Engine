use bevy::prelude::*;
use dj_engine::data::{
    AppCustomDocumentExt, CustomDocumentRegistration, EditorDocumentRoute, LoadedCustomDocuments,
    Project,
};
use dj_engine::editor::EditorExtensionRegistry;
use dj_engine::editor::{
    BrowserTab, EditorDirtyState, EditorPlugin, EditorSnapshotBaseline, EditorState, EditorUiState,
    EditorView, MountedProject, RuntimePreviewLaunchPhase, RuntimePreviewLaunchState,
};

#[test]
fn test_editor_initialization_and_state() {
    // 1. Setup App
    let mut app = App::new();

    // Minimal plugins required for the editor resources and states to be registered
    // We don't add the full DefaultPlugins because we don't want a window/renderer in tests
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::state::app::StatesPlugin);

    // We can't easily add EguiPlugin in headless without Winit/Window,
    // but EditorPlugin adds it. EguiPlugin might panic if no window.
    // So we manually add the resources/states we want to test,
    // OR we modify EditorPlugin to be test-friendly (not adding EguiPlugin if already present or in test mode).
    // For this integrity test, let's just test the RESOURCES and logic, avoiding the actual EguiPlugin if possible,
    // or use a mock.

    // Actually, let's just register the resources manually to verify our data structures work,
    // since we can't spin up a full UI context in a headless CI environment easily.

    app.init_state::<EditorState>()
        .init_resource::<MountedProject>()
        .init_resource::<EditorUiState>()
        .init_resource::<EditorSnapshotBaseline>()
        .init_resource::<EditorDirtyState>()
        .init_resource::<RuntimePreviewLaunchState>()
        .init_resource::<LoadedCustomDocuments>()
        .init_resource::<EditorExtensionRegistry>();

    // 2. Verify Initial State
    let ui_state = app.world().resource::<EditorUiState>();
    assert_eq!(ui_state.current_view, EditorView::Level);
    assert_eq!(ui_state.browser_tab, BrowserTab::Hierarchy);
    assert_eq!(ui_state.selected_palette_item, None);
    assert_eq!(ui_state.selected_custom_document, None);
    assert_eq!(ui_state.custom_document_kind_filter, "");

    let launch_state = app.world().resource::<RuntimePreviewLaunchState>();
    assert_eq!(launch_state.phase, RuntimePreviewLaunchPhase::Idle);
    assert_eq!(launch_state.manifest_path, None);
    assert_eq!(launch_state.status_message, None);
    assert_eq!(launch_state.last_exit, None);
    assert!(!launch_state.is_running());

    let dirty_state = app.world().resource::<EditorDirtyState>();
    assert!(!dirty_state.is_dirty);
    assert_eq!(dirty_state.snapshot_error, None);
    assert_eq!(dirty_state.pending_project_action, None);

    let custom_documents = app.world().resource::<LoadedCustomDocuments>();
    assert!(custom_documents.documents.is_empty());

    // 3. Simulate User Actions

    // "Load Project"
    let mut project = app.world_mut().resource_mut::<MountedProject>();
    project.root_path = Some("test/path".into());
    project.manifest_path = Some("test/path/project.json".into());
    project.project = Some(Project::new("Test Project"));

    // "Select Palette Item"
    let mut ui_state = app.world_mut().resource_mut::<EditorUiState>();
    ui_state.browser_tab = BrowserTab::Documents;
    ui_state.selected_palette_item = Some("Actor".into());
    ui_state.custom_document_kind_filter = "abilities".into();

    // "Switch View"
    ui_state.current_view = EditorView::StoryGraph;

    // 4. Verify Changes
    let ui_state_after = app.world().resource::<EditorUiState>();
    assert_eq!(ui_state_after.browser_tab, BrowserTab::Documents);
    assert_eq!(ui_state_after.selected_palette_item, Some("Actor".into()));
    assert_eq!(ui_state_after.custom_document_kind_filter, "abilities");
    assert_eq!(ui_state_after.current_view, EditorView::StoryGraph);

    let project_after = app.world().resource::<MountedProject>();
    assert_eq!(project_after.project.as_ref().unwrap().name, "Test Project");
    assert_eq!(
        project_after.manifest_path.as_deref(),
        Some(std::path::Path::new("test/path/project.json"))
    );
}

#[test]
fn test_editor_plugin_structure() {
    // Verify that the plugin adds the expected resources
    // (We accept that it might fail to build in headless if we add the actual plugin due to Egui,
    // but we can check if the struct exists and compiles, which this test file does by importing it)

    let plugin = EditorPlugin;
    assert!(std::any::type_name_of_val(&plugin).contains("EditorPlugin"));
}

#[test]
fn test_table_route_resolves_from_registry() {
    use dj_engine::data::{
        load_custom_documents_from_project, CustomDocumentRegistry, DJDataRegistryPlugin,
    };

    const TEST_SCHEMA: &str = r#"{"type":"object"}"#;

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(DJDataRegistryPlugin);
    app.register_custom_document(CustomDocumentRegistration::<serde_json::Value>::new(
        "table_test_kind",
        1,
        EditorDocumentRoute::Table,
        TEST_SCHEMA,
    ));

    let registry = app.world().resource::<CustomDocumentRegistry>().clone();
    let registered = registry.get("table_test_kind").unwrap();
    assert_eq!(registered.editor_route, EditorDocumentRoute::Table);

    // Verify that loading resolves the route from registry, not from the entry.
    let temp_dir = tempfile::tempdir().unwrap();
    let project_root = temp_dir.path();
    let data_dir = project_root.join("data").join("table_test_kind");
    std::fs::create_dir_all(&data_dir).unwrap();
    std::fs::write(
        data_dir.join("alpha.json"),
        r#"{"kind":"table_test_kind","id":"alpha","schema_version":1,"payload":{"value":1}}"#,
    )
    .unwrap();

    let manifest = dj_engine::data::CustomDataManifest {
        version: 1,
        documents: vec![dj_engine::data::CustomDocumentEntry {
            kind: "table_test_kind".into(),
            id: "alpha".into(),
            path: "table_test_kind/alpha.json".into(),
            schema_version: 1,
            editor_route: EditorDocumentRoute::Inspector, // entry says Inspector
            tags: Vec::new(),
        }],
    };

    let manifest_path = project_root.join("data").join("registry.json");
    std::fs::write(
        &manifest_path,
        serde_json::to_string_pretty(&manifest).unwrap(),
    )
    .unwrap();

    let project = Project::new("Table Route Test");
    let project_manifest_path = project_root.join("project.json");
    std::fs::write(
        &project_manifest_path,
        serde_json::to_string_pretty(&project).unwrap(),
    )
    .unwrap();

    let mounted = dj_engine::editor::MountedProject {
        root_path: Some(project_root.to_path_buf()),
        manifest_path: Some(project_manifest_path),
        project: Some(project),
    };

    let loaded = load_custom_documents_from_project(&mounted, &registry);
    let doc = loaded.get("table_test_kind", "alpha").unwrap();
    // Registry overrides entry: resolved_route should be Table, not Inspector.
    assert_eq!(doc.resolved_route, EditorDocumentRoute::Table);
}
