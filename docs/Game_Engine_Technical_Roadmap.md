# dj_engine: Technical Roadmap for Unified JRPG/RTS Architecture
## Shared Support Systems Blueprint (2026)

### Executive Summary

This roadmap provides a **practical, phased approach** to building dj_engine as a "Warcraft 3 Custom Map" style architecture where Unit/Character, Story, and Event systems are 90% identical between JRPG (DoomExe) and RTS (RTS-TBD) implementations. The key insight is that **controller input and camera perspective are the only genre-specific layers**; everything else can be unified.

**Current Status:**
- ✅ Bevy 0.18 with ECS foundation
- ✅ Lua scripting (mlua) for game logic
- ✅ Egui editor with palette, hierarchy/inspector
- ✅ Story Graph node editor (visual only)
- ⚠️ **Missing:** Story execution, Universal Unit archetype, Director system, advanced editor tooling

---

## 1. THE "VISUAL NOVEL" SYSTEM: Story Graph Architecture

### 1.1 Problem Statement

Currently:
- Story Graph is a **visual editor only** (no execution)
- No unified way to trigger dialogue in JRPG vs RTS
- No standardized way to serialize branching logic

Desired State:
- One StoryGraph → executes in both games
- Lua layer can trigger narrative events: `trigger_dialogue("intro_cutscene")`
- Node types: Dialogue, Conditional, Action, Camera Transition, Pause/Resume

### 1.2 Story Graph Node Architecture

```rust
/// Core node types that work for both JRPG and RTS
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum StoryNodeType {
    /// Display dialogue with speaker name and text
    Dialogue {
        speaker: String,
        text: String,
        voice_line: Option<String>, // MIDI file path
    },
    
    /// Branch based on player choice
    Choice {
        options: Vec<ChoiceOption>,
        /// If true, critical path: all branches merge here
        is_critical_path_merge: bool,
    },
    
    /// Execute Lua code (e.g., spawn unit, update quest)
    Action {
        lua_code: String,
        description: String,
    },
    
    /// Conditional branch (e.g., "if player has sword")
    Conditional {
        lua_condition: String,
        true_node_id: EntityId,
        false_node_id: EntityId,
    },
    
    /// Camera transition (RTS god-view ↔ JRPG follow-cam)
    CameraTransition {
        target_mode: CameraMode,
        duration_ms: u32,
    },
    
    /// Pause/Resume game time (for visual novel segments)
    TimeControl {
        action: TimeAction, // Pause, Resume, or SetTimeScale(f32)
    },
    
    /// End this story branch
    End,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StoryNode {
    pub id: EntityId,
    pub node_type: StoryNodeType,
    pub next_node_id: Option<EntityId>,
    pub metadata: StoryNodeMetadata,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChoiceOption {
    pub text: String,
    pub target_node_id: EntityId,
    pub lua_condition: Option<String>, // Hide option if condition fails
}
```

### 1.3 Story Graph Resource (Serializable)

```rust
/// The complete story graph, serialized to JSON
#[derive(Resource, Clone, Debug, Serialize, Deserialize)]
pub struct StoryGraph {
    pub id: String, // "intro_cutscene", "mission_briefing_01"
    pub version: u32,
    pub root_node_id: EntityId,
    pub nodes: HashMap<EntityId, StoryNode>,
    pub metadata: StoryGraphMetadata,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StoryGraphMetadata {
    pub title: String,
    pub author: String,
    pub created_date: String,
    pub tags: Vec<String>, // "intro", "cutscene", "gameplay", etc.
    pub locale: String, // "en_US" for localization
}
```

### 1.4 Story Graph Execution: The Director System

```rust
/// Active story playback state
#[derive(Component)]
pub struct StoryDirector {
    pub story_id: String,
    pub current_node_id: EntityId,
    pub state: DirectorState,
    pub variables: HashMap<String, LuaValue>, // Track player choices
}

pub enum DirectorState {
    Playing,
    WaitingForChoice,
    Paused,
    Finished,
}

/// System to advance story nodes
pub fn story_advancement_system(
    mut directors: Query<&mut StoryDirector>,
    story_graphs: Res<AssetServer>, // or custom StoryGraphAssets
    mut events: EventReader<StoryNodeCompleted>,
    mut lua: ResMut<Lua>,
) {
    for mut director in &mut directors {
        for event in events.iter() {
            if event.director_id == director.id {
                let graph = story_graphs.get(&director.story_id);
                let current_node = graph.nodes.get(&director.current_node_id);
                
                match current_node.node_type {
                    StoryNodeType::Action { lua_code, .. } => {
                        lua.execute_code(&lua_code, &mut director.variables);
                    }
                    StoryNodeType::Choice { .. } => {
                        director.state = DirectorState::WaitingForChoice;
                    }
                    StoryNodeType::Dialogue { .. } => {
                        // UI renders the dialogue
                        // Wait for next node trigger
                    }
                    _ => {}
                }
                
                // Advance if auto-advance
                if let Some(next_id) = current_node.next_node_id {
                    director.current_node_id = next_id;
                }
            }
        }
    }
}
```

### 1.5 Lua Integration: Standardized API

```lua
-- From both JRPG and RTS game scripts
trigger_dialogue("intro_cutscene")
-- Result: Pauses game (if TimeScale support), displays story UI

-- Within a Lua action node:
function on_mission_start()
    local unit_id = spawn_unit("hero", 100, 200)
    unit:add_item("sword_of_destiny")
    camera:transition_to_unit(unit_id, 1.5) -- 1.5s transition
    play_audio("bgm_intro", 0.8)
end

-- Story variable tracking (bidirectional)
set_story_var("player_chose_good", true)
if get_story_var("has_magic_sword") then
    -- Show special dialogue
end
```

### 1.6 Editor Workflow: Story Graph Editor Integration

**Next 3 Critical Tools:**

1. **Story Graph Visual Editor Enhancement**
   - **Drag-and-drop node creation** (Dialogue, Choice, Action, Camera, etc.)
   - **Live preview pane** (show what dialogue/choice looks like)
   - **Critical Path validator** (highlight which branches merge back)
   - **Lua syntax highlighting** in Action/Conditional nodes

2. **Story Variable Inspector**
   - **Track variables** set/read by story nodes
   - **Define variable types** (bool, string, number, array)
   - **Show dependency graph** (which nodes depend on which variables)

3. **Story Asset Manager**
   - **Localization** (store dialogue in multiple languages)
   - **Voice line mapping** (associate MIDI files with dialogue)
   - **Import/export** story graphs as JSON
   - **Version control integration** (mark changes per node)

---

## 2. UNIVERSAL UNIT DATA STRUCTURE: The Hero/Actor Component

### 2.1 Problem Statement

Currently:
- No unified "character" entity type
- JRPG needs: inventory, complex stats, party system
- RTS needs: pathfinding, auto-attack, selection circle
- **Solution:** Use Bevy's **Composition** pattern; build both from shared base

### 2.2 Core Unit/Character Components (Bevy 0.18 Required Components)

```rust
/// Base component—**required** for any playable unit
#[derive(Component, Reflect)]
pub struct Actor {
    pub id: u64,
    pub name: String,
    pub archetype: ActorArchetype, // Hero, Minion, NPC
}

#[derive(Debug, Clone, Reflect)]
pub enum ActorArchetype {
    Hero,
    Companion,
    Enemy,
    NPC,
}

/// Stats component—used by both games
#[derive(Component, Default, Reflect)]
#[require(Actor, Transform, GlobalTransform, Visibility)]
pub struct Stats {
    pub max_hp: i32,
    pub current_hp: i32,
    pub mana: i32,
    pub max_mana: i32,
    pub strength: i32,
    pub intelligence: i32,
    pub dexterity: i32,
    pub constitution: i32,
    pub level: u32,
    pub experience: u64,
    pub speed: f32, // Movement speed
}

/// Animation states—used by both games
#[derive(Component, Debug, Clone, Reflect)]
pub enum ActorAnimationState {
    Idle,
    Moving { direction: Vec2 },
    Attacking { target_id: u64 },
    Dead,
    CastingSpell { spell_id: String },
}

/// Inventory—JRPG-focused but RTS can ignore
#[derive(Component, Default, Reflect)]
pub struct Inventory {
    pub slots: Vec<Option<InventoryItem>>,
    pub capacity: usize,
}

#[derive(Clone, Debug, Reflect, Serialize, Deserialize)]
pub struct InventoryItem {
    pub item_id: String,
    pub quantity: u32,
    pub equipped: bool,
}

/// Ability/Skill system—both games
#[derive(Component, Default, Reflect)]
pub struct AbilitySet {
    pub abilities: Vec<Ability>,
}

#[derive(Clone, Debug, Reflect, Serialize, Deserialize)]
pub struct Ability {
    pub id: String,
    pub name: String,
    pub cooldown_ms: u32,
    pub last_used: u64, // timestamp
    pub lua_execution: String, // Lua code to run
}
```

### 2.3 JRPG-Specific Additions

```rust
/// Party system—JRPG only
#[derive(Component, Default)]
pub struct PartyLeader {
    pub party_members: Vec<u64>, // Actor IDs
    pub active_order: Vec<u64>,  // Turn order in combat
}

#[derive(Component)]
pub struct PartyMember {
    pub leader_id: u64,
    pub position_in_party: usize,
}

/// Direct input controller
#[derive(Component)]
pub struct DirectInput {
    pub input_map: HashMap<KeyCode, ActionCommand>,
}

pub enum ActionCommand {
    Move(Vec2),
    UseAbility(String),
    UseItem(String),
    Menu,
}

/// System: JRPG Input Handler
pub fn jrpg_input_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut actors: Query<(&DirectInput, &mut Transform), With<PartyLeader>>,
) {
    for (input, mut transform) in &mut actors {
        for (key, cmd) in &input.input_map {
            if keyboard_input.just_pressed(*key) {
                match cmd {
                    ActionCommand::Move(direction) => {
                        transform.translation += direction.extend(0.0) * 5.0;
                    }
                    ActionCommand::UseAbility(ability_id) => {
                        // Fire ability event
                    }
                    _ => {}
                }
            }
        }
    }
}
```

### 2.4 RTS-Specific Additions

```rust
/// Selection and interaction circle (RTS only)
#[derive(Component)]
pub struct RTSUnit {
    pub is_selected: bool,
    pub selection_circle_entity: Option<Entity>,
}

/// Pathfinding integration
#[derive(Component)]
pub struct PathfindingAgent {
    pub target: Option<Vec3>,
    pub path: Vec<Vec3>,
    pub current_waypoint: usize,
}

/// Automatic attack behavior
#[derive(Component)]
pub struct AutoAttack {
    pub range: f32,
    pub cooldown_ms: u32,
    pub last_attack: u64,
    pub preferred_target: Option<u64>,
}

/// System: RTS Click-to-Move
pub fn rts_click_to_move_system(
    mouse_input: Res<ButtonInput<MouseButton>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mut units: Query<(&mut PathfindingAgent, &RTSUnit)>,
) {
    if mouse_input.just_pressed(MouseButton::Right) {
        let (camera, camera_transform) = camera_query.single();
        if let Some(world_pos) = camera.viewport_to_world_2d(camera_transform, mouse_pos) {
            for (mut pathfinding, unit) in &mut units {
                if unit.is_selected {
                    pathfinding.target = Some(world_pos.extend(0.0));
                }
            }
        }
    }
}

/// System: Pathfinding Movement
pub fn pathfinding_movement_system(
    mut units: Query<(&mut Transform, &mut PathfindingAgent)>,
    time: Res<Time>,
) {
    for (mut transform, mut agent) in &mut units {
        if let Some(target) = agent.target {
            if agent.path.is_empty() {
                // Compute path (simplified—use a proper pathfinding crate)
                agent.path = vec![target];
                agent.current_waypoint = 0;
            }
            
            if agent.current_waypoint < agent.path.len() {
                let waypoint = agent.path[agent.current_waypoint];
                let direction = (waypoint - transform.translation).normalize();
                transform.translation += direction * 100.0 * time.delta_secs();
                
                if transform.translation.distance(waypoint) < 5.0 {
                    agent.current_waypoint += 1;
                }
            } else {
                agent.target = None;
            }
        }
    }
}
```

### 2.5 Camera Perspective System (Genre Bridge)

```rust
#[derive(Component)]
pub struct CameraMode {
    pub mode_type: CameraModeType,
}

pub enum CameraModeType {
    RPGFollowHero {
        follow_target: u64,
        distance_behind: f32,
        look_ahead_offset: f32,
    },
    RTSGodView {
        center: Vec3,
        zoom: f32,
    },
    FixedCutscene {
        position: Vec3,
        look_at: Vec3,
    },
}

/// System: Smooth camera transitions
pub fn camera_transition_system(
    mut cameras: Query<(&mut Transform, &mut Projection, &CameraMode)>,
    time: Res<Time>,
) {
    // Implements smooth Lerp between camera modes
    // Triggered by StoryNodeType::CameraTransition
}
```

---

## 3. THE "DIRECTOR" SYSTEM: Scene/Event Sequencing

### 3.1 Problem Statement

We need a **standardized way to sequence events:**
- Move camera (2s)
- Play animation (1s)
- Show text (dialogue UI)
- Execute Lua (trigger something)
- Wait for input (player clicks)

### 3.2 Director Component & Event Sequencing

```rust
/// Director: orchestrates sequences of events
#[derive(Component)]
pub struct Director {
    pub sequence: Vec<DirectorCommand>,
    pub current_command_index: usize,
    pub state: DirectorState,
    pub time_elapsed_ms: f32,
}

pub enum DirectorCommand {
    /// Transition camera smoothly
    CameraTransition {
        target_mode: CameraModeType,
        duration_ms: u32,
    },
    
    /// Play animation on actor
    PlayAnimation {
        actor_id: u64,
        animation: ActorAnimationState,
        duration_ms: u32,
    },
    
    /// Display UI element (dialogue, text box, etc.)
    ShowUI {
        ui_id: String,
        duration_ms: Option<u32>, // None = wait for user input
        lua_data: Option<String>, // JSON data to pass to UI
    },
    
    /// Execute arbitrary Lua code
    ExecuteLua {
        code: String,
    },
    
    /// Delay/wait
    Delay {
        duration_ms: u32,
    },
    
    /// Wait for specific condition
    WaitFor {
        lua_condition: String,
        timeout_ms: Option<u32>,
    },
    
    /// Pause/Resume game time
    TimeControl {
        action: TimeAction,
    },
    
    /// Branch based on condition
    Branch {
        condition: String,
        true_branch: Vec<DirectorCommand>,
        false_branch: Vec<DirectorCommand>,
    },
}

pub enum TimeAction {
    Pause,
    Resume,
    SetScale(f32),
}

/// System: Director execution
pub fn director_system(
    mut directors: Query<&mut Director>,
    time: Res<Time>,
    mut lua: ResMut<Lua>,
    mut events: EventWriter<DirectorEventTriggered>,
) {
    for mut director in &mut directors {
        director.time_elapsed_ms += time.delta_secs() * 1000.0;
        
        if director.current_command_index < director.sequence.len() {
            let cmd = &director.sequence[director.current_command_index];
            
            match cmd {
                DirectorCommand::Delay { duration_ms } => {
                    if director.time_elapsed_ms >= *duration_ms as f32 {
                        director.current_command_index += 1;
                        director.time_elapsed_ms = 0.0;
                    }
                }
                
                DirectorCommand::ExecuteLua { code } => {
                    lua.execute_code(code, &mut HashMap::new());
                    director.current_command_index += 1;
                }
                
                DirectorCommand::ShowUI { ui_id, duration_ms, .. } => {
                    events.send(DirectorEventTriggered {
                        event: DirectorEvent::ShowUI(ui_id.clone()),
                    });
                    
                    if let Some(dur) = duration_ms {
                        if director.time_elapsed_ms >= *dur as f32 {
                            director.current_command_index += 1;
                            director.time_elapsed_ms = 0.0;
                        }
                    } else {
                        // Wait for input (handled by UI system)
                    }
                }
                
                _ => {
                    // Handle other commands...
                    director.current_command_index += 1;
                }
            }
        } else {
            director.state = DirectorState::Finished;
        }
    }
}
```

### 3.3 Director as JSON (Serializable Sequences)

```json
{
  "director_id": "intro_sequence",
  "commands": [
    {
      "type": "CameraTransition",
      "target_mode": {
        "type": "FixedCutscene",
        "position": [0, 5, 10],
        "look_at": [0, 1, 0]
      },
      "duration_ms": 2000
    },
    {
      "type": "PlayAnimation",
      "actor_id": 1,
      "animation": "Idle",
      "duration_ms": 1000
    },
    {
      "type": "ShowUI",
      "ui_id": "dialogue_box",
      "lua_data": {
        "speaker": "Narrator",
        "text": "Welcome to the adventure!"
      }
    },
    {
      "type": "WaitFor",
      "lua_condition": "player_clicked_continue",
      "timeout_ms": null
    },
    {
      "type": "ExecuteLua",
      "code": "spawn_unit('hero', 0, 0)"
    }
  ]
}
```

---

## 4. SHARED SUPPORT SYSTEMS (Month-by-Month Roadmap)

### Phase 1: Foundation (Weeks 1-4)

**Goal:** Make Story Graph executable

- [ ] **Week 1-2:** Implement StoryNode ECS component + StoryGraph resource
  - Serialize/deserialize from JSON
  - Create story_advancement_system
  
- [ ] **Week 3:** Implement Story Graph UI rendering (show dialogue, choices)
  - Use Bevy UI (not Egui) for gameplay UI
  - Display speaker name, dialogue text, choice buttons
  
- [ ] **Week 4:** Lua integration for story triggers
  - `trigger_dialogue("scene_id")` → spawns Director with story sequence
  - Test with DoomExe first

### Phase 2: Director System (Weeks 5-8)

**Goal:** Sequence complex events (camera, animations, dialogue)

- [ ] **Week 5:** Implement Director component + DirectorCommand enum
  - CameraTransition, PlayAnimation, ShowUI, ExecuteLua
  
- [ ] **Week 6:** Camera transition system
  - Smooth Lerp between RPGFollowHero ↔ RTSGodView ↔ FixedCutscene
  
- [ ] **Week 7:** Time control integration
  - Pause/resume game time during cutscenes
  - Ensure RTS pathfinding respects TimeScale
  
- [ ] **Week 8:** Test Director with complex sequences
  - 5+ minute cutscene with camera, animations, dialogue

### Phase 3: Universal Unit (Weeks 9-12)

**Goal:** One unit archetype works for both JRPG and RTS

- [ ] **Week 9:** Define core Actor + Stats components
  - Spawn Heroes, Companions, NPCs with identical data
  
- [ ] **Week 10:** JRPG layer (DirectInput, Inventory, PartyLeader)
  - Test DoomExe with keyboard-controlled hero
  
- [ ] **Week 11:** RTS layer (RTSUnit, PathfindingAgent, AutoAttack)
  - Test RTS-TBD with mouse selection and pathfinding
  
- [ ] **Week 12:** Cross-game testing
  - Spawn same unit in both games, only controller differs

### Phase 4: Lua Standardization (Weeks 13-16)

**Goal:** Unified Lua API for unit/story/event manipulation

- [ ] **Week 13:** Define standard Lua bindings
  ```lua
  unit:move_to(x, y)
  unit:add_item(item_id)
  unit:cast_spell(spell_id)
  trigger_dialogue(scene_id)
  director:play_sequence(sequence_id)
  ```
  
- [ ] **Week 14-15:** Implement mlua bindings for each function
  - Wrap Bevy ECS queries in Lua-friendly APIs
  
- [ ] **Week 16:** Test in both games
  - Same Lua script runs in DoomExe + RTS

### Phase 5: Editor Enhancements (Weeks 17-20)

**Goal:** The next 3 critical tools for content creators

**Tool 1: Story Graph Visual Editor** (Week 17-18)
- Node creation (drag-and-drop or button+context menu)
- Live preview pane
- Lua syntax highlighting in Action nodes
- Export to JSON

**Tool 2: Story Variable Inspector** (Week 19)
- UI to define variables (name, type, default value)
- Dependency graph (which nodes use which variables)
- Validation (warn if variable is undefined)

**Tool 3: Story Asset Manager** (Week 20)
- Import voice lines (MIDI files)
- Localization editor (dialogue in EN/FR/JP)
- Version control UI (show changes per node)

---

## 5. CRITICAL ARCHITECTURAL DECISIONS

### 5.1 Time Management During Visual Novel Segments

**Question:** Should the game world pause while dialogue plays?

**Answer:** **Yes, with caveats**

```rust
/// TimeScale resource—controls all game time
#[derive(Resource)]
pub struct GameTimeScale(pub f32); // Default 1.0, 0.0 = paused

/// During dialogue:
pub fn on_dialogue_start(mut time_scale: ResMut<GameTimeScale>) {
    time_scale.0 = 0.0; // Pause world
}

pub fn on_dialogue_end(mut time_scale: ResMut<GameTimeScale>) {
    time_scale.0 = 1.0; // Resume
}

/// Systems respect TimeScale:
pub fn movement_system(
    mut transforms: Query<&mut Transform>,
    time: Res<Time>,
    time_scale: Res<GameTimeScale>,
) {
    let delta = time.delta_secs() * time_scale.0;
    // Apply delta to movement
}
```

**Exception:** RTS gameplay (pathfinding, unit attacks) should continue. Use a **layer-based time scale**:

```rust
pub struct LayeredTimeScale {
    pub global: f32,
    pub ui_layer: f32,
    pub combat_layer: f32,
}

// Only UI/story respects global scale; combat ignores it
let delta = if in_story_cutscene {
    time.delta_secs() * time_scale.global
} else {
    time.delta_secs() * time_scale.combat_layer
};
```

### 5.2 Lua Unit API Proposal

```lua
-- Standardized Unit object (works for both JRPG and RTS)
local unit = get_unit(hero_id)

-- Universal methods:
unit:move_to(x, y)              -- Both: JRPG cutscene or RTS pathfinding
unit:play_animation("attack")   -- Both: visual consistency
unit:cast_spell("fireball", target_id)  -- Both: execute ability
unit:take_damage(50)            -- Both: health update
unit:die()                       -- Both: death sequence

-- JRPG-specific (RTS can ignore):
unit:add_item("potion")
unit:equip_weapon("sword_of_destiny")
unit:gain_experience(100)
party:add_member(unit)

-- RTS-specific (JRPG can ignore):
unit:set_selection(true)  -- Show selection circle
unit:set_stance("aggressive") -- Auto-attack behavior
unit:set_rally_point(x, y)    -- Troops move here
```

### 5.3 Story Graph + Lua Hybrid Approach

**Pattern:** Use Story Graph for structure; Lua for logic

```
Story Graph (JSON) → "intro_cutscene"
├─ Node 1: Dialogue ("Welcome!")
├─ Node 2: Choice ("Go left or right?")
├─ Node 3a (left): ExecuteLua { code: "spawn_enemy_left()" }
├─ Node 3b (right): ExecuteLua { code: "spawn_enemy_right()" }
└─ Node 4: End

Lua script (doomexe_intro.lua):
function spawn_enemy_left()
    local goblin = unit:spawn("goblin", 100, 0)
    goblin:set_ai("aggressive")
end

function spawn_enemy_right()
    local orc = unit:spawn("orc", -100, 0)
    orc:set_ai("patrol")
end
```

**Benefit:** Non-programmers can design story in Graph editor; programmers write Lua logic.

### 5.4 Database View for Static Data (bevy_inspector_egui Extension)

Current pain point: Editing item stats requires code changes + recompile.

**Solution:** Extend bevy_inspector_egui with a "Database" plugin

```rust
// In editor code:
#[derive(Resource, Reflect, Serialize, Deserialize)]
pub struct ItemDatabase {
    pub items: Vec<ItemStats>,
}

#[derive(Reflect, Serialize, Deserialize)]
pub struct ItemStats {
    pub id: String,
    pub name: String,
    pub damage: i32,
    pub rarity: String,
}

// Editor automatically shows:
// - Table view of all items
// - Add/Remove/Edit buttons
// - Export to JSON

// Game loads at startup:
let db: ItemDatabase = serde_json::from_str(include_str!("../assets/items.json"))?;
```

**Implementation:** Use Egui table widget in editor plugin

---

## 6. NEXT STEPS & DEPENDENCIES

### Immediate Actions

1. **Clarify input requirements:** How much customization do you need for JRPG vs RTS controls?
2. **Pathfinding choice:** Use a crate like `bevy_pathfinding` or build your own?
3. **Audio requirements:** MIDI support is critical for voice lines?
4. **Localization scope:** Will you support multiple languages from day 1?

### Recommended Rust Crates to Add

```toml
[dependencies]
bevy = "0.18"
mlua = { version = "0.9", features = ["lua54", "serde"] }
serde_json = "1.0"
serde = { version = "1.0", features = ["derive"] }
uuid = { version = "1.0", features = ["v4", "serde"] }

# Pathfinding (RTS)
bevy_rapier3d = "0.28"  # Physics + collision
# OR
navmesh = "0.2"  # Simpler 2D pathfinding

# Animation
bevy_asset_loader = "0.20"
```

### Performance Considerations

- **Story Graph serialization:** Pre-load common stories at startup
- **Lua execution:** Cache compiled Lua chunks (mlua supports this)
- **Director sequences:** Use a bitset archetype for fast queries
- **Time scaling:** Ensure Physics engine respects TimeScale

---

## 7. SUCCESS METRICS

By end of Phase 5, you should have:

✅ One Story Graph that executes in both DoomExe and RTS  
✅ Camera smoothly transitions between RPG follow-view and RTS god-view  
✅ Hero unit spawned in both games using identical data  
✅ Lua API standardized so one script works in both games  
✅ Egui editor with Story Graph visual editor + Variable inspector + Asset manager  
✅ No code recompilation needed to edit story/dialogue/item stats  

**Estimated time to MVP:** 20 weeks (~5 months) with one developer  
**Team scaling:** With 2 developers, can parallelize Phases 3-4

---

## 8. REFERENCES & INSPIRATIONS

- **Warcraft 3 Editor:** Hub-and-spoke dialogue, critical path merging
- **Baldur's Gate 3:** Layered branching (consequential + thematic + character-driven)
- **Steins;Gate:** Phone-as-controller for narrative (alternative input metaphor)
- **Bevy 0.18:** Required Components pattern, Remote Protocol for editing
- **Lua in RTS:** Spring Engine (open-source RTS with full Lua scripting)

---

**Document Version:** 2026-01-21  
**Author:** AI Architect (guided by your dj_engine vision)  
**Status:** Ready for implementation
