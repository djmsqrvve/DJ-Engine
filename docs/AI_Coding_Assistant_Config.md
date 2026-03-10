# AI Coding Assistant Configuration & Workflows (2026)

## 1. RECOMMENDED AI ASSISTANTS FOR dj_engine

### Top Tier (Recommended for Game Engine Work)

| Tool | Cost | Best For | Rust Support | Integration |
|------|------|----------|--------------|-------------|
| **Cursor** | $20/mo | Whole-project refactoring, architecture | ⭐⭐⭐⭐⭐ | Native (VS Code fork) |
| **Claude 3.5 Sonnet (via API)** | $20/mo + usage | Complex ECS reasoning, system design | ⭐⭐⭐⭐⭐ | Continue.dev / Windsurf |
| **GitHub Copilot** | $10/mo | Quick boilerplate, inline suggestions | ⭐⭐⭐⭐ | VS Code native |
| **GPT-4o (ChatGPT Plus)** | $20/mo | High-level architecture, debugging | ⭐⭐⭐⭐ | Web + API |

### Why Cursor + Claude for dj_engine?

**Cursor** is a VS Code fork with Claude 3.5 Sonnet built-in. This is ideal because:

1. **Codebase Context:** Cursor understands your entire Bevy/ECS codebase
2. **Multi-file edits:** Can refactor across components, systems, and resources simultaneously
3. **ECS-Aware:** Can suggest proper archetype changes when you ask for architectural modifications
4. **No context window limit (with Claude):** Your 50K-line engine stays in scope

**Example workflow:**
```
You: "Refactor StoryGraph execution so it respects TimeScale and GameState changes"

Cursor/Claude:
- Scans director_system.rs, story_graph.rs, time_systems.rs
- Identifies 4 places that need updating
- Provides diffs with explanations
- Tests that event ordering is preserved
```

---

## 2. CURSOR-SPECIFIC SETUP FOR BEVY/ECS

### Installation

```bash
# macOS
brew install cursor

# Windows
# Download from https://www.cursor.sh/

# Linux
# Download from https://www.cursor.sh/
```

### `.cursor/config.json`

```json
{
  "model": "claude-3.5-sonnet",
  "language": "rust",
  "codebaseContextEnabled": true,
  "contextWindow": {
    "maxTokens": 200000,
    "strategy": "intelligent-sampling"
  },
  "rust": {
    "enableFullAnalysis": true,
    "checkOnSave": true,
    "clippy": true,
    "format": true
  },
  "editor": {
    "tabSize": 4,
    "insertSpaces": true,
    "rulers": [100, 120],
    "defaultFormatter": "rust-analyzer"
  }
}
```

### Key Cursor Commands for ECS Work

```
Cmd+K (macOS) / Ctrl+K (Windows/Linux): Edit Mode
  - Highlight code → Ask Claude to refactor
  - Example: Select `query<(&Transform, &mut Velocity)>` → "Optimize this query for cache locality"
  - Cursor shows diff, you approve/reject

Cmd+L / Ctrl+L: Chat Mode (context-aware)
  - Ask architectural questions without editing
  - "How should I structure the Director system to support pausing during dialogue?"
  - Claude scans your codebase, gives specific advice

Cmd+Shift+P / Ctrl+Shift+P: Command Palette
  - "Generate test for component"
  - "Create archetype analysis report"
  - "Find inefficient queries"
```

---

## 3. GITHUB COPILOT SETUP (Cheaper Alternative)

### Installation & Configuration

```bash
# 1. Install extension in VS Code
# 2. Authenticate with GitHub account
# 3. Configure in settings.json:
```

```json
{
  "github.copilot.enable": {
    "rust": true,
    "markdown": true,
    "*": true
  },
  
  "github.copilot.advanced": {
    "inlineCompletions.count": 5,
    "debug.overrideBehavior": false,
    "listMaxResults": 10,
    "panelSelectionText": false,
    "skipCloudFeatures": false
  },
  
  // Quality suggestions
  "github.copilot.codeActions.enabled": true
}
```

### Copilot Prompt Patterns for Game Dev

**Pattern 1: Component Boilerplate**
```rust
// Type: #[derive(Component)]
// Press Ctrl+Shift+L, then Tab to autocomplete
#[derive(Component, Reflect)]
pub struct |  // Cursor suggests: AnimationState, Stats, etc.
```

**Pattern 2: System Generation**
```rust
/// System to handle [feature description]
pub fn |  // Copilot generates full system stub with proper signature
```

**Pattern 3: Test Generation**
```rust
#[test]
fn test_|  // Copilot suggests test cases from your component
```

---

## 4. CLAUDE + CONTINUE.DEV INTEGRATION

### Why Continue.dev?

Continue.dev is an open-source IDE plugin that connects any LLM (Claude, ChatGPT, etc.) to your editor. It's **perfect for dj_engine** because:

1. **Use Claude 3.5 Sonnet** (better at Rust than Copilot)
2. **Use GPT-4o** (better at system design than Copilot)
3. **Self-hosted option** (private on your machine)
4. **Works with VS Code, JetBrains, and Vim**

### Installation

```bash
# VS Code
# 1. Install "Continue" extension from marketplace
# 2. Configure ~/.continue/config.json:
```

```json
{
  "models": [
    {
      "title": "claude-3.5-sonnet",
      "provider": "anthropic",
      "model": "claude-3-5-sonnet-20241022",
      "apiKey": "YOUR_ANTHROPIC_API_KEY"
    },
    {
      "title": "gpt-4o",
      "provider": "openai",
      "model": "gpt-4o",
      "apiKey": "YOUR_OPENAI_API_KEY"
    }
  ],
  
  "slashCommands": [
    {
      "name": "test",
      "description": "Generate unit tests for selected code",
      "prompt": "Write comprehensive unit tests for this Bevy component using bevy::prelude::*"
    },
    {
      "name": "ecs-analyze",
      "description": "Analyze ECS archetype efficiency",
      "prompt": "Analyze this Bevy query for cache locality and archetype fragmentation issues. Suggest optimizations."
    },
    {
      "name": "story-graph",
      "description": "Generate story node traversal logic",
      "prompt": "For this story graph structure, generate a system that advances through nodes correctly handling branches and merges."
    }
  ],
  
  "contextProviders": [
    {
      "name": "codebase",
      "params": {}
    },
    {
      "name": "docs",
      "params": {
        "urls": [
          "https://bevyengine.org/learn/quick-start/",
          "https://docs.rs/bevy/latest/bevy/"
        ]
      }
    }
  ]
}
```

### Continue.dev Workflow

```
1. Select problematic code
2. Press Cmd+Shift+M (macOS) to open Continue chat
3. Type: "/ecs-analyze" 
4. Claude scans your selection + entire codebase
5. Returns specific optimization suggestions
6. Press Tab to auto-apply suggested changes
```

---

## 5. AI PROMPTS FOR dj_engine ARCHITECTURE

### Prompt 1: Generate Story Graph Execution System

```
You are a Bevy game engine expert specializing in ECS architecture.

I'm building a narrative/story system called "StoryGraph" that needs to:
1. Execute a directed graph of dialogue/action nodes
2. Support branching based on player choices
3. Handle dialogue UI display, camera transitions, and Lua script execution
4. Pause/resume game time during cutscenes
5. Work identically in both a JRPG (DoomExe) and RTS (RTS-TBD)

Current components:
- StoryNode enum with variants: Dialogue, Choice, Action, CameraTransition, TimeControl, End
- StoryGraph resource containing node map and root node ID
- StoryDirector component tracking current node and playback state

Generate:
1. A complete `story_advancement_system` that handles node transitions
2. An `on_choice_selected` event handler for branching
3. A system to execute Lua code in Action nodes
4. Proper event ordering to prevent TLE (tight loop errors)

Constraints:
- Use Bevy 0.18 syntax
- Must work with `bevy_picking` for choice button clicks
- TimeScale must be respected by all systems
- No mutable borrow conflicts

Context files to scan: src/systems/story.rs, src/components.rs, src/resources.rs
```

### Prompt 2: Universal Unit Archetype Optimization

```
I have a "Unit" component that represents both JRPG heroes and RTS units. 

Current components included:
- Actor (base), Stats, Inventory, AbilitySet, Transform, GlobalTransform, Visibility
- JRPG adds: DirectInput, PartyLeader
- RTS adds: RTSUnit, PathfindingAgent, AutoAttack

Problem: Queries are slow because every unit loads even unused components.

Solution: Use Bevy's `without()` filter and Required Components pattern.

Generate:
1. Separate query examples optimized for each game mode
2. A `unit_type_filter_system` that skips unnecessary components
3. Suggestions for splitting components into additional archetypes
4. Performance comparison (# of components loaded per query)

Consider:
- Cache locality impact
- Added query overhead from extra `without()` clauses
- Whether some components should use markers (zero-size) instead
```

### Prompt 3: Lua Integration for Unit Actions

```
I need to expose Bevy units to Lua scripts (via mlua crate).

My Lua API should support:
unit:move_to(x, y)                    -- Works in JRPG cutscene AND RTS
unit:play_animation("attack")
unit:cast_spell(spell_id, target_id)
unit:take_damage(amount)
unit:add_item(item_id)               -- JRPG only, but RTS doesn't error
party:add_member(unit_id)
trigger_dialogue("scene_id")
director:play_sequence(sequence_json)

Generate:
1. mlua table definitions for `Unit`, `Party`, `Director`
2. Wrapper functions that dispatch to Bevy ECS systems
3. Error handling (e.g., if unit doesn't exist)
4. Example Lua script using the full API
5. Type marshalling for Rust <-> Lua (e.g., u64 -> EntityId)

Test case: A script that plays 5 seconds of dialogue then spawns an enemy.
```

### Prompt 4: Story Graph + Director Integration

```
I need to connect two systems:

1. StoryGraph: Linear progression through dialogue/action nodes
2. Director: Sequences complex events (camera movement, animations, timing)

My goal: When a story node contains "show dialogue", the Director should:
1. Pause game time
2. Transition camera to conversation view
3. Display dialogue UI
4. Wait for user click
5. Resume game time

Generate:
1. An event: `StoryNodeTriggered` that bridges both systems
2. A system that converts story nodes to Director commands
3. Timing logic to ensure camera finishes before dialogue shows
4. Test scenario: "Opening cutscene with dialogue + character animation"

Dependencies: Story nodes define WHAT to show; Director handles HOW and WHEN.
```

### Prompt 5: Performance Profiling Template

```
I'm hitting frame drops (60 FPS target) in my Bevy game.

Generate a profiling setup that:
1. Measures time spent in each system
2. Tracks archetype cache misses
3. Reports per-query overhead
4. Identifies bottleneck systems
5. Exports results as CSV

Add to main.rs:
- Diagnostic plugins
- Custom timer system
- Post-frame report system

Then provide example output showing:
- System timings (story_advancement_system: 0.3ms, pathfinding: 2.1ms, rendering: 11.4ms)
- Query efficiency (full scan vs. filtered)
- Bottleneck diagnosis
```

---

## 6. CUSTOM SLASH COMMANDS FOR CONTINUE.DEV

Save as `~/.continue/commands.json`:

```json
{
  "slashCommands": [
    {
      "name": "bevy-query",
      "description": "Analyze Bevy query for efficiency",
      "prompt": "This Bevy query may be inefficient. Analyze for:\n1. Unnecessary component loads\n2. Archetype fragmentation\n3. Cache locality issues\n4. Suggest optimizations with without() filters\n\nCode: {selection}"
    },
    
    {
      "name": "story-test",
      "description": "Generate story graph traversal tests",
      "prompt": "Generate unit tests for StoryGraph traversal:\n1. Test branching (player chooses left/right)\n2. Test critical path merging\n3. Test Lua action execution\n4. Test dialogue display timing\n\nCode: {selection}"
    },
    
    {
      "name": "unit-refactor",
      "description": "Optimize universal unit archetype",
      "prompt": "Refactor this unit/actor component for both JRPG and RTS:\n1. Use Required Components pattern\n2. Minimize component loads in queries\n3. Add performance comments\n4. Suggest archetype-specific queries\n\nCode: {selection}"
    },
    
    {
      "name": "ecs-antipattern",
      "description": "Find ECS anti-patterns",
      "prompt": "Review for ECS anti-patterns:\n1. Monolithic God components\n2. Unnecessary component cloning\n3. Large/expensive query iterations\n4. Missing archetype optimization\n\nCode: {selection}"
    },
    
    {
      "name": "lua-binding",
      "description": "Generate Lua FFI bindings",
      "prompt": "Generate mlua Lua bindings for this Rust struct:\n1. Create Lua table representation\n2. Add methods as Lua functions\n3. Handle type marshalling (Rust <-> Lua)\n4. Include error handling\n5. Test example in Lua\n\nCode: {selection}"
    }
  ]
}
```

---

## 7. BEST PRACTICES: PROMPT ENGINEERING FOR RUST/BEVY

### Effective Prompts

✅ **DO:**
```
"I have a Bevy system that queries (Transform, Velocity) for all moving entities.
It's iterating over 10,000 entities per frame even though only 100 are actually moving.
How can I use Bevy's archetype system to skip static entities?"
```

```
"Generate a test for this component that verifies:
1. Initial state is correct
2. State transitions are valid
3. Edge cases (zero values, None options)"
```

❌ **DON'T:**
```
"Write me a game in Rust"  (too vague)
```

```
"Fix this" (no context)
```

### Prompt Structure for Best Results

```
CONTEXT:
  Project: dj_engine (Bevy 0.18 game engine)
  Goal: Unified JRPG/RTS architecture
  Language: Rust
  Constraint: Single-player only

PROBLEM:
  [2-3 sentences describing what's broken/missing]

CURRENT CODE:
  [Paste relevant functions, 20-50 lines]

REQUIREMENTS:
  - Requirement 1
  - Requirement 2
  - Requirement 3

GENERATE:
  [What you want Claude to produce: code, tests, docs, etc.]

CONSTRAINTS:
  - Performance: 60 FPS on mid-range hardware
  - Compatibility: Must work with existing systems
  - Safety: No unsafe blocks unless justified
```

---

## 8. WORKFLOW: FROM ARCHITECT PROMPT TO IMPLEMENTATION

### Example: Building Story Graph Executor

**Step 1: Send Architect Prompt to Claude (via website)**

```
[Use Prompt 1 from Section 5]
Claude response: 400-line Rust code with full implementation
```

**Step 2: Copy code into project, open in Cursor**

```bash
# Create new file
mkdir -p src/systems
touch src/systems/story.rs

# Paste Claude's code, then:
```

**Step 3: Use Cursor to refine**

```
You: "Add detailed comments explaining the event ordering. 
Why must choice_selected_system run before story_advancement_system?"

Cursor scans your codebase, adds inline comments explaining:
- Why event ordering matters
- How Bevy's schedule ensures causality
- Specific line numbers where order is critical
```

**Step 4: Use Copilot to generate tests**

```rust
#[test]
fn test_  // Press Tab
// Copilot auto-completes: test_story_graph_executes_nodes_in_order()
```

**Step 5: Use Continue.dev for optimization**

```
Select your story_advancement_system
Press Cmd+Shift+M
Type: "/ecs-analyze"
Claude identifies: Query could use without() to skip invisible actors
Shows exact diff, you approve in 1 click
```

---

## 9. COST OPTIMIZATION STRATEGY

| Budget | Recommended Setup | Total Cost |
|--------|-------------------|-----------|
| **$0/mo (free)** | GitHub Copilot (free tier) | $0 |
| **$10/mo** | GitHub Copilot ($10) | $10 |
| **$20/mo** | Cursor ($20) = best value | $20 |
| **$40/mo** | Cursor ($20) + Claude API ($20) | $40 |
| **$60/mo** | Cursor + Claude + ChatGPT+ | $60 |

**Recommendation for teams:**
- **Solo dev:** Cursor ($20) - pays for itself in time saved
- **2-3 devs:** Cursor + Copilot (split cost ~$10/person)
- **Studio:** Cursor + Continue.dev + self-hosted LLM (llama.cpp)

---

## 10. REAL-WORLD EXAMPLE: DEBUGGING WITH AI

### Scenario: Story graph not advancing

**Your error log:**
```
thread 'main' panicked at 'called Option::unwrap() on a None value', 
src/systems/story.rs:87
```

**Cursor workflow:**
```
1. Cursor -> Cmd+K (edit mode)
2. Highlight line 87 and surrounding function
3. Ask: "Why is current_node_id.unwrap() panicking? 
   What should happen when current_node_id is None?"
4. Cursor suggests: Add check before unwrap, handle None case
5. Shows diff: panics avoided, story advances correctly
```

**Result:** Bug fixed in 30 seconds vs. 10 minutes of manual debugging

---

## 11. TEAM GUIDELINES: AI USAGE POLICY

```markdown
# dj_engine AI Coding Guidelines

## ✅ ENCOURAGED USE

- [ ] Generating component boilerplate
- [ ] Creating unit tests from components
- [ ] Optimizing slow queries
- [ ] Refactoring ECS systems
- [ ] Finding performance bottlenecks
- [ ] Generating documentation
- [ ] Translating code patterns to Bevy 0.18
- [ ] Exploring alternative architectures

## ⚠️ CAUTION AREAS

- [ ] Always review AI-generated code line-by-line
- [ ] Don't accept architectural suggestions without team review
- [ ] Test all generated tests (they can miss edge cases)
- [ ] Don't use AI for security-sensitive code (auth, encryption)
- [ ] Verify performance claims with actual profiling

## ❌ NEVER USE AI FOR

- [ ] Game design decisions (story, mechanics)
- [ ] Credential management
- [ ] Network security code
- [ ] Accessibility features (use humans for UX testing)

## CODE REVIEW CHECKLIST FOR AI CODE

- [ ] Does it compile without warnings?
- [ ] Does it follow project style guide?
- [ ] Are error cases handled?
- [ ] Are there edge case tests?
- [ ] Is performance acceptable?
- [ ] Can a human maintain this?
```

---

**Configuration Version:** 2026-01-21  
**Recommended Tool:** Cursor + Claude 3.5 Sonnet  
**Fallback:** GitHub Copilot ($10/mo)
