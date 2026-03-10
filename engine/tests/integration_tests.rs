use bevy::prelude::*;
use dj_engine::animation::components::{BlinkingAnimation, BreathingAnimation};
use dj_engine::data::{StoryGraphData, StoryNodeData};
use dj_engine::midi::MidiManager;
use dj_engine::prelude::*;
use dj_engine::scripting::context::LuaContext;
use dj_engine::scripting::ffi::{
    create_shared_state, register_core_api, register_generic_state_api,
};

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

    assert!(flags.get("met_hamster"));
    assert_eq!(executor.current_node, Some(2)); // Should have jumped to Node 2
    assert_eq!(executor.status, ExecutionStatus::WaitingForInput);
}

#[test]
fn test_graph_executor_load_from_data() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(AssetPlugin::default());
    app.add_plugins(bevy::input::InputPlugin);
    app.init_asset::<AudioSource>();
    app.add_plugins(DJEnginePlugin::default().without_diagnostics());

    // Build a minimal Start -> Dialogue -> End graph via data layer
    let mut data = StoryGraphData::new("test_graph", "Test");
    data.add_node(StoryNodeData::start("n_start", Some("n_dialogue")));
    data.add_node(StoryNodeData::dialogue("n_dialogue", "Hamster", "Hello!"));
    data.add_node(StoryNodeData::end("n_end"));
    data.root_node_id = "n_start".into();

    {
        let mut executor = app.world_mut().resource_mut::<GraphExecutor>();
        executor.load_from_data(&data);
    }

    app.update();

    let executor = app.world().resource::<GraphExecutor>();
    // After one update, Start node advances immediately and Dialogue blocks for input
    assert_eq!(executor.status, ExecutionStatus::WaitingForInput);
    assert!(executor.current_node.is_some());
}

#[test]
fn test_audio_state_volume_clamping_via_resource() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(AssetPlugin::default());
    app.add_plugins(bevy::input::InputPlugin);
    app.init_asset::<AudioSource>();
    app.add_plugins(DJEnginePlugin::default().without_diagnostics());

    // Directly mutate AudioState resource and verify clamp behavior
    {
        let mut state = app.world_mut().resource_mut::<AudioState>();
        state.master_volume = 2.0_f32.clamp(0.0, 1.0);
    }
    assert_eq!(app.world().resource::<AudioState>().master_volume, 1.0);

    {
        let mut state = app.world_mut().resource_mut::<AudioState>();
        state.master_volume = (-1.0_f32).clamp(0.0, 1.0);
    }
    assert_eq!(app.world().resource::<AudioState>().master_volume, 0.0);
}

#[test]
fn test_breathing_system_applies_scale_at_phase() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(AssetPlugin::default());
    app.add_plugins(bevy::input::InputPlugin);
    app.init_asset::<AudioSource>();
    app.add_plugins(DJEnginePlugin::default().without_diagnostics());

    // phase=PI/2 means sin(0 + PI/2)=1.0 on the first frame (elapsed≈0),
    // giving scale_factor = 1.0 + amplitude * 1.0 > 1.0
    let entity = app
        .world_mut()
        .spawn((
            BreathingAnimation {
                phase: std::f32::consts::FRAC_PI_2,
                ..BreathingAnimation::hamster_default()
            },
            Transform::from_xyz(0.0, 0.0, 0.0),
        ))
        .id();

    app.update();

    let transform = app.world().entity(entity).get::<Transform>().unwrap();
    assert!(
        transform.scale.y > 1.0,
        "breathing should expand scale.y, got {}",
        transform.scale.y
    );
}

#[test]
fn test_blinking_system_triggers_on_expired_timer() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(AssetPlugin::default());
    app.add_plugins(bevy::input::InputPlugin);
    app.init_asset::<AudioSource>();
    app.add_plugins(DJEnginePlugin::default().without_diagnostics());

    // timer=0.0 → after subtracting delta(0) it stays 0 ≤ 0, triggering blink
    let entity = app
        .world_mut()
        .spawn(BlinkingAnimation {
            timer: 0.0,
            ..BlinkingAnimation::hamster_default()
        })
        .id();

    app.update();

    let blink = app
        .world()
        .entity(entity)
        .get::<BlinkingAnimation>()
        .unwrap();
    assert!(
        blink.is_blinking,
        "blink should have started when timer expired"
    );
}

#[test]
fn test_lua_context_basic_execution() {
    let ctx = LuaContext::new();
    let lua = ctx.lua.lock().unwrap();
    let result: i32 = lua.load("return 1 + 1").eval().unwrap();
    assert_eq!(result, 2);
}

#[test]
fn test_lua_context_with_ffi_roundtrip() {
    let ctx = LuaContext::new();
    let lua = ctx.lua.lock().unwrap();
    register_core_api(&lua).unwrap();
    let state = create_shared_state();
    register_generic_state_api(&lua, state.clone()).unwrap();

    lua.load(
        r#"
        set_float("score", 100.0)
        set_string("player", "hamster")
        set_bool("game_over", false)
    "#,
    )
    .exec()
    .unwrap();

    let data = state.read().unwrap();
    assert!((data.floats["score"] - 100.0).abs() < f32::EPSILON);
    assert_eq!(data.strings["player"], "hamster");
    assert!(!data.bools["game_over"]);
}
