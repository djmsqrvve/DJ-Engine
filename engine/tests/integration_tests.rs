use bevy::prelude::*;
use dj_engine::midi::MidiManager;
use dj_engine::prelude::*;

#[test]
fn test_engine_initialization() {
    let mut app = App::new();

    // Use MinimalPlugins for headless testing
    app.add_plugins(MinimalPlugins);
    app.add_plugins(AssetPlugin::default());
    app.add_plugins(bevy::input::InputPlugin);
    app.init_asset::<AudioSource>();

    // Add our engine plugin (without diagnostics to avoid window requirement issues if any)
    app.add_plugins(DJEnginePlugin::default().without_diagnostics());

    // Run one update cycle
    app.update();

    // Verify core resources exist
    assert!(app.world().contains_resource::<AudioState>());
    assert!(app.world().contains_resource::<MidiManager>());
    assert!(app.world().contains_resource::<GraphExecutor>());
    assert!(app.world().contains_resource::<StoryFlags>());
}
#[test]
fn test_story_graph_branching() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(AssetPlugin::default());
    app.add_plugins(bevy::input::InputPlugin);
    app.init_asset::<AudioSource>();
    app.add_plugins(DJEnginePlugin::default().without_diagnostics());

    let mut graph = StoryGraph::new();

    // Node 0: Set flag 'met_hamster' to true
    let n0 = graph.add(StoryNode::SetFlag {
        flag: "met_hamster".to_string(),
        value: true,
        next: Some(1),
    });

    // Node 1: Branch based on 'met_hamster'
    let _n1 = graph.add(StoryNode::Branch {
        flag: "met_hamster".to_string(),
        if_true: Some(2),
        if_false: Some(3),
    });

    // Node 2: Dialogue for true branch
    let _n2 = graph.add(StoryNode::Dialogue {
        speaker: "Hamster".to_string(),
        text: "Hello again!".to_string(),
        portrait: None,
        next: Some(4),
    });

    // Node 3: Dialogue for false branch
    let _n3 = graph.add(StoryNode::Dialogue {
        speaker: "Hamster".to_string(),
        text: "Who are you?".to_string(),
        portrait: None,
        next: Some(4),
    });

    // Node 4: End
    let _n4 = graph.add(StoryNode::End);

    graph.set_start(n0);

    let mut executor = app.world_mut().resource_mut::<GraphExecutor>();
    executor.start(graph);

    // Run updates to process SetFlag and Branch (should take 0 frames to process intermediate logic)
    // But Dialogue blocks execution until input.
    app.update();

    let executor = app.world().resource::<GraphExecutor>();
    let flags = app.world().resource::<StoryFlags>();

    assert_eq!(flags.get("met_hamster"), true);
    assert_eq!(executor.current_node, Some(2)); // Should have jumped to Node 2
    assert_eq!(executor.status, ExecutionStatus::WaitingForInput);
}
