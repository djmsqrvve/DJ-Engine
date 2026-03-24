# dj_engine: Complete Editor Specification
## All Tabs & Features for Playable JRPG + Tower Defense RTS
**Date:** 2026-01-21  
**Status:** Detailed Architecture Ready for Implementation  
**Research Base:** Unity 6.3, Godot 4.5, Unreal 5, Warcraft 3, Tiled

---

## 🎯 EDITOR VISION

Your dj_engine editor should enable **non-programmers** to create:
1. **DoomExe (JRPG):** Playable 2D story-driven adventure with dialogue, inventory, turn-based combat
2. **Tower Defense RTS:** Builder units placing towers, defending against waves, simple AI

**Reference:** WC3 World Editor's philosophy—hub-and-spoke design with clear data flow between tabs.

---

## 📊 EDITOR LAYOUT (Inspired by Modern Engines)

```
┌─────────────────────────────────────────────────────────────────────┐
│ dj_engine EDITOR - [Project Name] - [Map Name]                     │
├─────────────────────────────────────────────────────────────────────┤
│ FILE EDIT VIEW HELP                                     [◼ Max]    │
├─────────────────────────────────────────────────────────────────────┤
│ [🎮 Main Tab Bar]                                                   │
│ [Scene] [Palette] [Hierarchy] [Inspector] [Story Graph] [Database] │
├───────────────────────┬─────────────────────────┬───────────────────┤
│                       │                         │                   │
│   LEFT PANEL          │   CENTER VIEWPORT       │   RIGHT PANEL     │
│   (Depends on Tab)    │   (2D/3D Preview)       │   (Properties)    │
│                       │                         │                   │
│  • Palette: Assets    │   Main Scene View:      │  • Inspector:     │
│  • Hierarchy: Tree    │   ├─ Tilemap (JRPG)    │    Properties     │
│  • Story: Nodes       │   ├─ Pathfinding Map    │  • Stats:         │
│  • Database: Tables   │   │   (TD)              │    Selected Item  │
│                       │   ├─ Grid overlay       │  • Details:       │
│                       │   ├─ Gizmos (move,      │    Components     │
│                       │   │   rotate, scale)    │                   │
│                       │   └─ Camera view        │                   │
│                       │                         │                   │
├───────────────────────┴─────────────────────────┴───────────────────┤
│ [Console / Output Log] - Errors & Warnings                          │
└───────────────────────────────────────────────────────────────────────┘
```

---

## 📑 MAIN TAB STRUCTURE (6 Core Tabs)

### **TAB 1: SCENE EDITOR** ✅
**Purpose:** Build maps/levels (2D tilemap for JRPG, pathfinding grid for TD)

**Left Panel: Palette**
```
🔍 Search: [_______]

📂 Tilesets
  ├─ Grass
  ├─ Stone
  ├─ Water
  └─ Forest

📂 Props (Interactive)
  ├─ Door
  ├─ Chest
  ├─ NPC spawner
  └─ Trigger zone

📂 Towers (TD Only)
  ├─ Tower base
  ├─ Gun turret
  └─ Defense structure

[Details: Tile Size, Layer Info]
```

**Center Viewport: Interactive Map**
- **Left-click:** Place tile/object
- **Right-click:** Delete/erase
- **Scroll wheel:** Zoom in/out
- **Middle drag:** Pan around map
- **Shift+Click:** Multi-select objects
- **Gizmos:**
  - Move tool (W)
  - Rotate tool (E)
  - Scale tool (R)

**Right Panel: Tile Properties**
```
📌 Selected: Grass Tile (X: 10, Y: 15)

🔧 Properties:
  ├─ Type: Ground
  ├─ Walkable: ✓
  ├─ Collision: None
  ├─ Layer: 0 (base)
  └─ Sprite: grass_01.png

🎨 Appearance:
  ├─ Sprite: [grass_01.png] [Browse]
  ├─ Tint: [Color picker]
  └─ Animation: [None v]

⚙️ Interactive (if applicable):
  ├─ Trigger Type: [None v]
  ├─ Lua Script: [_______]
  └─ Event: [_______]
```

**Key Features:**
- ✅ Tile brush (single, 3x3, 5x5 patterns)
- ✅ Autotiling (WC3-style terrain smoothing)
- ✅ Layer system (background, tiles, props, NPCs)
- ✅ Collision overlay (show/hide walkable areas)
- ✅ Pathfinding grid preview (for TD towers)
- ✅ Grid snapping (toggle with G)
- ✅ Undo/Redo (Ctrl+Z / Ctrl+Y)
- ✅ Export map as JSON

---

### **TAB 2: HIERARCHY** ✅
**Purpose:** Tree view of all entities in scene (like Unreal Outliner or Unity Hierarchy)

**Structure:**
```
🌳 Scene Hierarchy

📁 [Root]
  📁 [Layers]
  │   📁 Layer_Background
  │  ├─ bg_sky
  │  ├─ bg_mountains
  │  └─ bg_forest
  │
  │  📁 Layer_Interactive
  │  ├─ door_entrance [🔒] [👁]
  │  ├─ chest_loot
  │  └─ npc_merchant
  │
  │  📁 Layer_NPCs
  │  ├─ hero_spawn [👁]
  │  ├─ enemy_goblin_1
  │  └─ enemy_orc_1
  │
  └─ 📁 Towers (TD only)
     ├─ tower_base_1 [🔒]
     ├─ tower_gun_1
     └─ tower_cannon_1

🔍 Search: [_______] (find by name)
```

**Controls:**
- **Click name:** Select in scene + show in Inspector
- **Double-click name:** Focus camera on object
- **Right-click:** Context menu (copy, delete, duplicate, group)
- **👁 icon:** Toggle visibility (hide/show in editor)
- **🔒 icon:** Lock/unlock (prevent accidental selection)
- **Drag:** Reparent object or reorder in hierarchy

**Right-Click Context Menu:**
```
├─ Create Child
│  ├─ Empty Entity
│  ├─ NPC
│  ├─ Enemy
│  ├─ Trigger Zone
│  └─ Tower (TD only)
│
├─ Copy (Ctrl+C)
├─ Paste (Ctrl+V)
├─ Duplicate (Ctrl+D)
├─ Delete (Delete)
│
├─ Group Selected (Ctrl+G)
├─ Lock/Unlock (Alt+L)
├─ Visibility Toggle (Alt+H)
│
└─ Properties (Alt+P)
```

**Key Features:**
- ✅ Hierarchical organization (parents/children)
- ✅ Visibility toggling (eye icon)
- ✅ Selection locking (padlock icon)
- ✅ Multi-select (Ctrl+Click)
- ✅ Group/ungroup actors
- ✅ Search/filter by name
- ✅ Drag-and-drop reordering

---

### **TAB 3: INSPECTOR** ✅
**Purpose:** Detailed properties of selected object (like Unity Inspector or UE Details Panel)

**Layout:**
```
┌──────────────────────────────────┐
│ INSPECTOR                   [↔]  │ (↔ = collapse details)
├──────────────────────────────────┤
│ 📌 object_name [Edit Name]       │
│ 🎯 ID: 12345                      │
├──────────────────────────────────┤
│
│ 🔧 TRANSFORM
│ ├─ Position X: [100] Y: [50] Z: [0]
│ ├─ Rotation X: [0°] Y: [0°] Z: [45°]
│ ├─ Scale X: [1.0] Y: [1.0] Z: [1.0]
│ └─ [Lock X] [Lock Y] [Lock Z] (lock aspect ratio)
│
│ 🎨 SPRITE / APPEARANCE
│ ├─ Sprite: [grass_01.png v] [Browse]
│ ├─ Tint: [Color Picker]
│ ├─ Layer: [0 v] (sorting order)
│ └─ Flip: [X] [Y]
│
│ 🎯 COLLISION / PHYSICS
│ ├─ Enabled: [✓]
│ ├─ Type: [Static v] (Static / Dynamic / Kinematic)
│ ├─ Shape: [Box v] (Box / Circle / Polygon)
│ │  ├─ Width: [64] Height: [64]
│ │  └─ Offset: X[0] Y[0]
│ └─ Layer: [Default v]
│
│ 🔗 INTERACTIVITY
│ ├─ Trigger: [None v] (None / Door / Chest / NPC)
│ ├─ Trigger ID: [_______]
│ └─ Lua Script: [_______] [Browse]
│
│ 💻 CUSTOM PROPERTIES (if NPC/Enemy)
│ ├─ NPC Name: [Merchant]
│ ├─ Dialogue: [dialogue_merchant_01] [Browse]
│ ├─ Quest: [quest_find_sword] [Browse]
│ └─ Inventory: [Add Item] [Edit]
│
│ ⚔️ COMBAT STATS (if Enemy/Boss)
│ ├─ HP: [100]
│ ├─ Damage: [15]
│ ├─ Defense: [5]
│ ├─ Speed: [2.0]
│ └─ Loot: [gold_50, potion_01]
│
│ 🏰 TOWER PROPERTIES (TD only)
│ ├─ Tower Type: [Gun v]
│ ├─ Damage: [25]
│ ├─ Range: [200px]
│ ├─ Cooldown: [1.5s]
│ ├─ Cost: [100]
│ └─ Upgrade Path: [None v]
│
└──────────────────────────────────┘

[💾 Save] [🔄 Reset] [➕ Add Component]
```

**Features:**
- ✅ Transform manipulation (position, rotation, scale)
- ✅ Live editing (changes reflect in scene immediately)
- ✅ Copy/paste values between components
- ✅ Reset to defaults
- ✅ Color picker for tints
- ✅ File browser for sprites/scripts
- ✅ Component add/remove (+ button)
- ✅ Sections collapse/expand (+ arrows)

---

### **TAB 4: STORY GRAPH EDITOR** ✅
**Purpose:** Visual branching dialogue/cutscene designer (core from your roadmap)

**Viewport:**
```
                    ┌─────────────────────────────┐
                    │ START_SCENE_01              │
                    │ (Dialogue)                  │
                    │ "Welcome hero..."           │
                    └──────────┬────────────────┘
                               │
                    ┌──────────┴──────────┐
                    │                     │
         ┌──────────▼──────────┐ ┌────────▼──────────┐
         │ CHOICE              │ │ ACTION            │
         │ "Go left/right?"    │ │ (Spawn enemy)     │
         │                     │ │                   │
         │ [Left] → NODE_2     │ └───────────────────┘
         │ [Right] → NODE_3    │
         └─────────────────────┘

Nodes are draggable, clickable, connectable by edges
```

**Left Panel: Node Palette**
```
🔍 Search: [_______]

📂 Node Types
  ├─ [+ Dialogue]      (show text)
  ├─ [+ Choice]        (branch paths)
  ├─ [+ Action]        (execute Lua)
  ├─ [+ Conditional]   (if/then)
  ├─ [+ Camera]        (camera transition)
  ├─ [+ TimeControl]   (pause/resume)
  └─ [+ End]           (end branch)

[Recent Graphs: intro, mission_01, battle]
```

**Center: Node Editor**
- **Left-click + drag:** Pan viewport
- **Scroll wheel:** Zoom in/out
- **Left-click node:** Select
- **Right-click node:** Edit node
- **Drag from node output:** Create connection
- **Delete node:** Press Delete key
- **Duplicate:** Ctrl+D
- **Auto-arrange:** Layout button

**Right Panel: Node Details**
```
┌──────────────────────────────┐
│ NODE EDITOR                  │
├──────────────────────────────┤
│ 📌 Node: DIALOGUE_01         │
│ 🔗 ID: 42                     │
├──────────────────────────────┤
│
│ Speaker: [NPC Merchant v]
│ Portrait: [portrait_merchant.png]
│
│ Dialogue Text:
│ ┌────────────────────────────┐
│ │ "Greetings, adventurer! I  │
│ │ have a quest for you..."   │
│ └────────────────────────────┘
│
│ Voice Line: [choose_file]
│ Duration: [3.5] seconds
│
│ [Audio Preview ▶]
│
│ Next Node: NODE_02 (auto)
│
│ [Cancel] [Save] [Delete]
│
└──────────────────────────────┘
```

**Key Features:**
- ✅ Node types: Dialogue, Choice, Action, Conditional, Camera, TimeControl
- ✅ Live preview (hover shows what text displays)
- ✅ Connection validation (no invalid edges)
- ✅ Auto-layout (arrange nodes neatly)
- ✅ Variable tracking (show which story vars used)
- ✅ Export as JSON
- ✅ Search/replace dialogue
- ✅ Localization support (EN/FR/JP tabs)

---

### **TAB 5: DATABASE EDITOR** ✅
**Purpose:** Edit static game data (items, NPCs, towers, dialogue sets)

**Tabs in Database:**

#### **5A: ITEMS TABLE**
```
🔍 Search: [_______] [+ New Item]

| ID          | Name      | Type      | Damage | Defense | Price |
|─────────────┼───────────┼───────────┼────────┼─────────┼───────|
| sword_01    | Iron Sword| Weapon    | 15     | 0       | 50    |
| shield_01   | Wood Shld | Armor     | 0      | 5       | 30    |
| potion_heal | Health Po | Potion    | 0      | 0       | 10    |
| gold_50     | 50 Gold   | Currency  | 0      | 0       | 50    |

[Click row to edit]
```

**Details Panel (when item selected):**
```
Item: Sword_01
├─ Name: [Iron Sword]
├─ Type: [Weapon v]
├─ Damage: [15]
├─ Defense: [0]
├─ Price: [50]
├─ Sprite: [sword.png] [Browse]
├─ Description: [Sharp iron blade...]
└─ Script: [_______]
```

#### **5B: NPCS TABLE**
```
| ID           | Name     | Dialogue Set | Location | Loot Table |
|──────────────┼──────────┼──────────────┼──────────┼────────────|
| npc_merchant | Merchant | dialogue_001 | Town     | none       |
| npc_guard    | Guard    | dialogue_002 | Gate     | gold_10    |
| boss_dragon  | Dragon   | dialogue_003 | Tower    | rare_loot  |

[Add NPC] [Delete] [Edit]
```

#### **5C: TOWERS TABLE** (TD only)
```
| ID          | Name         | Damage | Range | Cost | Cooldown |
|─────────────┼──────────────┼────────┼───────┼──────┼──────────|
| tower_gun   | Gun Turret   | 25     | 200px | 100  | 1.5s     |
| tower_bomb  | Bomb Tower   | 50     | 150px | 200  | 3.0s     |
| tower_slow  | Slow Field   | 0      | 100px | 80   | 0.5s     |
```

#### **5D: LOOT TABLES**
```
| Table ID    | Item 1      | Chance | Item 2     | Chance |
|─────────────┼─────────────┼────────┼────────────┼────────|
| loot_common | gold_50     | 80%    | potion_hp  | 20%    |
| loot_rare   | rare_sword  | 40%    | rare_armor | 40%    |
|             | gold_200    | 20%    |            |        |
```

#### **5E: ENEMY TEMPLATES**
```
| ID         | Name      | HP | Damage | Experience | Loot Table |
|────────────┼───────────┼────┼────────┼────────────┼────────────|
| goblin_01  | Goblin    | 20 | 5      | 50         | loot_common|
| orc_01     | Orc       | 50 | 10     | 150        | loot_rare  |
| boss_troll | Troll     | 200| 20     | 500        | loot_boss  |
```

**Key Features:**
- ✅ Add/Edit/Delete rows
- ✅ Search and filter
- ✅ Import/export as CSV/JSON
- ✅ Validation (check for missing references)
- ✅ Version history (git-like commits)
- ✅ Multi-language support (columns for different languages)

---

### **TAB 6: PALETTE / ASSET BROWSER** ✅
**Purpose:** Browse and manage all game assets (sprites, audio, tilesets, prefabs)

**Structure:**
```
📂 ASSETS
  📁 Sprites
  │  ├─ Characters
  │  │  ├─ hero_idle.png
  │  │  ├─ hero_walk_01.png
  │  │  └─ hero_walk_02.png
  │  ├─ Enemies
  │  │  ├─ goblin_idle.png
  │  │  └─ orc_attack.png
  │  ├─ Tilesets
  │  │  ├─ grass_tileset.png
  │  │  └─ stone_tileset.png
  │  └─ UI
  │     ├─ dialogue_box.png
  │     └─ buttons.png
  │
  📁 Audio
  │  ├─ Music
  │  │  ├─ bgm_intro.mid
  │  │  └─ bgm_combat.mid
  │  ├─ SFX
  │  │  ├─ sword_slash.wav
  │  │  └─ potion_drink.wav
  │  └─ Voice
  │     ├─ merchant_greet.wav
  │     └─ hero_ow.wav
  │
  📁 Prefabs (reusable entities)
  │  ├─ enemy_goblin
  │  ├─ tower_gun
  │  └─ npc_merchant
  │
  📁 Scripts
  │  ├─ on_player_death.lua
  │  └─ tower_ai.lua
  │
  └─ 📁 Story Graphs
     ├─ dialogue_intro.json
     ├─ mission_01_brief.json
     └─ boss_cutscene.json

🔍 Search: [_______] [Sort: Name v] [View: Grid v]
```

**Views:**
- **Grid view:** Thumbnails (good for visual browsing)
- **List view:** Detailed with file sizes, dates
- **Favorites:** Pin frequently-used assets
- **Recent:** Last 20 accessed assets

**Right-click Asset:**
```
├─ Open (in viewer)
├─ Duplicate
├─ Rename
├─ Delete
├─ Properties (size, format, etc.)
├─ Pin to Favorites
└─ Open Folder (in file manager)
```

---

## 🎮 ADDITIONAL PANELS (Always Visible)

### **TOOLBAR (Top)**
```
[File] [Edit] [View] [Help]
[◄ Back] [► Forward] [🔄 Undo] [↻ Redo]
[🎮 Play] [⏸ Pause] [⏹ Stop]
[💾 Save] [📤 Export] [📥 Import]
[⚙️ Settings] [🔍 Zoom Fit] [📏 Grid] [🔗 Snap]
```

### **CONSOLE / OUTPUT (Bottom)**
```
┌─────────────────────────────────────────────────┐
│ CONSOLE / OUTPUT LOG                            │
├─────────────────────────────────────────────────┤
│ [❌ Errors (5)] [⚠️ Warnings (12)] [ℹ️ Info (3)]│
│                                                  │
│ ℹ️ [15:32] Map "level_01" loaded successfully   │
│ ⚠️ [15:32] NPC "merchant" has no dialogue set   │
│ ❌ [15:33] Entity 42: Unknown sprite "foo.png"  │
│ ⚠️ [15:33] Tower "gun_01" range exceeds map     │
│                                                  │
│ [🔄 Refresh] [🗑 Clear] [💾 Export Log]         │
│                                                  │
└─────────────────────────────────────────────────┘
```

### **PROPERTIES PANEL (Right)**
Always shows selected object's detailed properties (Inspector content).

---

## 🛠️ KEYBOARD SHORTCUTS (Essential)

```
GENERAL
├─ Ctrl+S          Save
├─ Ctrl+Shift+S    Save As
├─ Ctrl+Z          Undo
├─ Ctrl+Y          Redo
├─ Ctrl+Q          Quit Editor
└─ F1              Help

SCENE / VIEWPORT
├─ W               Move Tool
├─ E               Rotate Tool
├─ R               Scale Tool
├─ G               Toggle Grid Snap
├─ V               Pan (hold + drag)
├─ Z               Zoom to Fit
├─ F               Focus on Selection
├─ H               Toggle Visibility
├─ Alt+H           Toggle All Hidden Objects
└─ Delete          Delete Selected

HIERARCHY
├─ Ctrl+D          Duplicate
├─ Ctrl+C          Copy
├─ Ctrl+V          Paste
├─ Ctrl+G          Group
├─ Alt+L           Lock/Unlock
└─ Alt+Shift+H     Hide All Except Selected

STORY GRAPH
├─ Ctrl+N          New Node
├─ Ctrl+L          Auto-Layout
├─ Space           Preview Node
└─ Tab             Next Node (during playback)

GENERAL NAVIGATION
├─ Tab             Cycle through open tabs
├─ Shift+Tab       Cycle tabs backwards
├─ Alt+1-6         Jump to Tab 1-6
└─ F11             Toggle Fullscreen
```

---

## 📋 WORKFLOW EXAMPLES

### **Creating a JRPG Level**

1. **Open SCENE TAB**
   - Select grass tileset from Palette
   - Paint 20x20 map of grass tiles
   - Add stone path tiles for walkable area

2. **Add INTERACTABLES**
   - Place door sprite (from Palette → Props → Door)
   - In Inspector, set Trigger Type: "Door"
   - Set Destination: "next_map"

3. **Add NPCS**
   - Hierarchy: Right-click → Create → NPC
   - Inspector: Set NPC ID to "merchant_1"
   - Inspector: Set Dialogue Set to "dialogue_merchant"
   - Set position in scene (drag icon in viewport)

4. **Test**
   - Click Play button
   - Walk around (JRPG: WASD, arrow keys)
   - Talk to NPC (E key)
   - See dialogue from Story Graph

---

### **Creating a Tower Defense Level**

1. **Open SCENE TAB**
   - Paint pathfinding grid (shows where enemies walk)
   - Place towers strategically

2. **Configure TOWERS**
   - Each tower → Inspector → Set Cost, Range, Damage
   - Database Tab → Towers Table → Verify stats

3. **Add SPAWNER**
   - Hierarchy: Create → Enemy Spawner
   - Inspector: Set Wave Count = 5
   - Set Enemy Type = "goblin_01"
   - Set Spawn Rate = 2 per second

4. **Test**
   - Play button
   - Place towers (click, drag cost from UI)
   - Watch waves spawn and be defeated
   - Monitor gold/score

---

## 🔄 DATA FLOW (How Everything Connects)

```
Scene Editor (visual placement)
    ↓ [Saves as JSON]
    ↓
Scene JSON (tilemap + entities)
    ↓ [Bevy loads]
    ↓
Game World (entities spawned)
    ↓ [Player interacts]
    ↓
Story Graph (dialogue plays)
    ↓ [Lua executed]
    ↓
Database (NPC/item data referenced)
    ↓ [Bevy systems update]
    ↓
Game State (inventory, quests, gold)
    ↓ [Loop continues]
```

---

## 📦 EXPORT/IMPORT FORMATS

### **Scene Export (JSON)**
```json
{
  "map_name": "level_01",
  "tilemap": {
    "width": 20,
    "height": 15,
    "tiles": [
      {"x": 0, "y": 0, "sprite": "grass", "layer": 0},
      {"x": 1, "y": 0, "sprite": "grass", "layer": 0}
    ]
  },
  "entities": [
    {
      "id": "door_1",
      "type": "interactive",
      "x": 10,
      "y": 5,
      "trigger": "door",
      "destination": "level_02"
    },
    {
      "id": "npc_merchant",
      "type": "npc",
      "x": 5,
      "y": 8,
      "dialogue_set": "dialogue_merchant"
    }
  ]
}
```

### **Story Graph Export (JSON)**
```json
{
  "graph_id": "intro_scene",
  "root_node": 1,
  "nodes": {
    "1": {
      "type": "dialogue",
      "speaker": "Hero",
      "text": "Where am I?",
      "next": 2
    },
    "2": {
      "type": "choice",
      "options": [
        {"text": "Go left", "target": 3},
        {"text": "Go right", "target": 4}
      ]
    }
  }
}
```

---

## ⚡ PERFORMANCE CONSIDERATIONS

**Editor Responsiveness:**
- Load maps < 500ms
- Pan/zoom smooth (60 FPS)
- Undo/redo instant (no recompile)
- Save < 100ms

**Memory:**
- Keep unpacked sprites in RAM (fast access)
- Cache thumbnails
- Stream large tilemaps

---

## 🎯 SUCCESS CRITERIA: EDITOR COMPLETENESS

✅ **Can create playable JRPG level** (walk around, talk to NPC)  
✅ **Can create playable TD map** (place towers, see enemies spawn)  
✅ **Non-programmers can create content** (no code editing needed)  
✅ **All data persists** (save/load works)  
✅ **Hot-reload** (edit + re-run game, see changes)  

---

## 📊 IMPLEMENTATION PRIORITY

### **Phase 5 Editor Work (Weeks 17-20)**

**Week 17-18: Core Panels**
- [ ] Scene viewport (tilemap rendering + placement)
- [ ] Hierarchy tree view
- [ ] Inspector properties panel
- [ ] Basic toolbar + shortcuts

**Week 19: Visual Editors**
- [ ] Story Graph visual node editor
- [ ] Palette asset browser
- [ ] Save/load maps as JSON

**Week 20: Database + Polish**
- [ ] Database editor (items, NPCs, towers)
- [ ] Export/import CSV
- [ ] Validation (warn if broken references)
- [ ] Tutorial tooltips

---

**Editor Version:** 1.0  
**Target:** Production-ready for content creators  
**Research Base:** 2026 industry standards (Unity 6.3, Godot 4.5, UE5)
