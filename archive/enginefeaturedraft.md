Here’s a concise, schema-style checklist of everything your dj_engine editor needs to support JRPG + TD content creation. Think of this as the “full feature surface” your tools must expose.

1. Global Project / Editor Settings
Project Settings

Project name, ID, version

Target platforms (PC, console, web, mobile)

Default resolution & aspect ratio

Render settings (VSync, target FPS, pixel art snapping)

Input profile selection (JRPG, RTS, hybrid)

Localization languages enabled (EN, FR, JP, etc.)

Default save paths (scenes, story graphs, DB, assets)

Autosave interval / backup policy

Editor Preferences (Per-User)

Theme (light/dark)

Grid size (pixels)

Snap increment (position, rotation, scale)

Default gizmo mode (move/rotate/scale)

UI scale / font size

Keybinding customization (override default shortcuts)

Panel layout presets (JRPG mapping vs TD balancing)

Build / Export Settings

Export targets (dev build, release build)

Compression levels for assets

Scene/story/database packaging rules

Versioning metadata (build number, changelog notes)

2. Scene Editor (Maps & Layout)
Scene-Level Properties

Scene ID, name, description

Scene type: JRPG map / TD map / shared

Size (width, height in tiles/world units)

Default background color / skybox

Default player spawn / camera spawn

Ambient music track, loop settings

Scene-specific Lua scripts (on_enter, on_exit)

Tilemap Layers

Layer list with:

Name (Layer_Background, Layer_Interactive, Layer_NPCs, etc.)

Sort order

Visibility toggle

Lock toggle

Parallax factor (if you add parallax)

Tile size (width, height)

Tile blending / autotiling rules per tileset

Tile Painting Tools

Brush shape: single, line, rectangle, area (3x3, 5x5)

Brush patterns (saved stamps/prefabs)

Autotiling rules (WC3-style smoothing): terrain types, transitions

Eraser tool (tiles / objects)

Fill tool (flood fill a region)

Collision overlay toggle (show walkable vs blocked)

Pathfinding grid overlay for TD:

Walkable/unwalkable cells

Tower placement cells

Enemy path preview

Object Placement

Place entities from Palette:

Props: doors, chests, triggers, decorations

NPCs / enemies

Towers (TD only)

Spawners / waypoints / regions

Object snapping:

To grid

To other objects (align edges/centers)

Gizmos:

Move (X/Y, optional Z)

Rotate

Scale (with aspect lock)

Multi-select / box-select

Align / distribute tools (align left, center, etc.)

Scene Navigation / View

Zoom in/out

Pan (middle mouse / hotkey)

Zoom to fit

Focus on selection

Toggle grid

Toggle snapping

Toggle collision/pathfinding overlays

Scene Operations

New scene / duplicate scene

Save scene

Revert scene

Scene validation (broken references, missing scripts, etc.)

3. Hierarchy (Entities & Grouping)
Tree Structure

Root node, Layers folders

Nested entities (parent/child relationships)

Towers subtree (TD only)

Special folders (UI, Spawners, Regions, etc.)

Per-Entity Metadata

Name

Unique ID

Entity type (NPC, Enemy, Tower, Trigger, Deco, etc.)

Prefab reference (if instance of prefab)

Scene reference (for cross-scene links if needed)

Hierarchy Operations

Create / rename / delete entities

Create empty entity

Create from template (NPC, Enemy, Tower, Trigger Zone)

Duplicate (with new IDs)

Copy/paste entities

Drag-and-drop to reparent / reorder

Group / ungroup entities into folders/groups

Visibility / Locking

Per-entity:

Visible in editor

Locked (unselectable)

Bulk operations on selection

Search / Filter

Search by name / ID / type

Filters:

Only NPCs

Only Enemies

Only Towers

Only with errors/warnings

4. Inspector (Components & Properties)
Transform Component (All Entities)

Position (X, Y, Z)

Rotation (X, Y, Z)

Scale (X, Y, Z)

Lock aspect ratio (for scale)

Coordinate mode (world vs local, if you care)

Sprite / Appearance Component

Sprite asset reference

Sorting layer / order

Tint color (color picker)

Flip X / Flip Y

Animation:

Animation clip ID

Playback speed

Loop / play once

Collision / Physics Component

Enabled

Body type: Static / Dynamic / Kinematic

Shape: Box / Circle / Polygon

Shape dimensions, offset

Physics layer / collision mask

Bounciness, friction (if used)

Trigger vs solid

Interactivity Component

Trigger type: None / Door / Chest / NPC / Custom

Trigger ID (string)

Trigger parameters (e.g., door destination scene/map, spawn list)

Lua script reference

Events:

on_interact

on_enter / on_exit

on_death, etc.

NPC / Enemy Component

NPC / Enemy ID (links to database row)

Display name

Dialogue set ID

Quest ID(s)

Inventory preset ID

Faction / alignment

Patrol route / behavior preset

Combat Stats Component

HP / max HP

Mana / resource

Damage

Defense / armor

Attack speed / cooldown

Move speed

Aggro range

Loot table ID

Tower Component (TD Only)

Tower type ID

Damage

Range

Attack cooldown

Build cost

Upgrade path ID

Targeting mode (first, last, closest, strongest)

Projectile type / effect ID

Spawner Component (TD & JRPG)

Wave count

Enemies per wave

Enemy template IDs (with weights)

Spawn interval

Start delay

Path / waypoint IDs

Loop behavior

Misc Components

Audio source (for ambient SFX/music in scene):

Clip ID

Volume, loop, 2D/3D

Camera anchor / bounds:

Camera region

Follow target, limits

Inspector-Level Actions

Save / Apply changes

Reset component to defaults

Add component (list of all component types)

Remove component

Copy component / paste component values

5. Story Graph Editor
Graph-Level Settings

Graph ID, name, description

Graph type: dialogue / cutscene / mission logic

Root node ID

Default language / active language tab

Node Types & Fields

Dialogue Node

Node ID

Speaker ID (NPC/Party/ narrator)

Portrait asset

Dialogue text (per language)

Voice line asset (optional)

Auto-advance duration (seconds)

Next node (ID) or branching condition

Choice Node

Node ID

Prompt text (optional)

Choices list:

Choice text (per language)

Target node ID

Conditions (simple flags/variables)

Side effects (set flags, give items, etc.)

Action Node

Lua script reference or inline command ID

Parameters (JSON or key-value)

Next node ID

Conditional Node

Condition expression (simple flag/var system)

True target node ID

False target node ID

Camera Node

Target camera preset (close-up, wide, follow, static)

Position / zoom / angle (if relevant)

Ease type / duration

TimeControl Node

Pause gameplay: yes/no

Slow motion factor

Resume flag

End Node

End type (return to gameplay, load new scene, etc.)

Graph Editor Operations

Create/duplicate/delete nodes

Drag nodes in canvas

Connect nodes (edges)

Validate graph (missing targets, loops, unreachable)

Auto-layout

Search nodes by ID/text

Preview dialogue playback

Show variable usage (which nodes read/write which flags)

Localization

Language tabs per graph

Missing translation indicators

Export/import localization CSV/JSON

6. Database Editor
Global DB Controls

Search within table

Filters (by type, tag, etc.)

Add / duplicate / delete row

Import/export as CSV/JSON

Reference validation (IDs used in scenes/story graphs)

Tables & Fields

Items Table

ID

Name (per language)

Type (Weapon, Armor, Potion, Currency, QuestItem, etc.)

Stats: damage, defense, heal amount, buff duration, etc.

Price / sell value

Max stack size

Rarity

Sprite asset

Description (per language)

Script hooks (on_use, on_equip, on_unequip)

NPCs Table

ID

Name (per language)

Dialogue set ID

Location / region tags

Default faction

Default quest(s)

Loot table ID

Portrait sprite

Towers Table (TD)

ID

Name (per language)

Damage

Range

Cooldown

Cost

Build time

Upgrade path IDs (next tower IDs)

Projectile / FX ID

Description

Enemies Table

ID

Name (per language)

HP

Damage

Speed

Experience reward

Loot table ID

Behavior profile ID (AI)

Loot Tables

Table ID

Entries: item ID + chance %

Optional quantity ranges

Quests Table (if you surface it in editor)

Quest ID

Name / description (per language)

Start conditions

Completion conditions

Rewards (items, gold, flags)

Balance / Versioning

Change history per table (log edits)

“Mark row as deprecated” flag

Quick presets / templates for common items/enemies/towers

7. Asset Browser / Palette
Asset Types

Sprites (characters, enemies, tilesets, UI)

Audio (music, SFX, voice)

Prefabs (entity templates)

Scripts (Lua)

Story Graph assets (JSON graphs)

Scene files

Browser Features

Folder hierarchy

Grid view / list view

Search by name / tag / type

Sorting (name, type, modified date, size)

Favorites / pins

Recent assets

Asset Operations

Import (drag/drop, file picker)

Delete (with “used by” warning)

Duplicate

Rename

Open in viewer (sprite preview, waveform preview)

Show properties (size, format, length, tags)

Open containing folder

Prefab Handling

Create prefab from selected entity

Instantiate prefab into scene

Override prefab properties per instance

Apply instance overrides back to prefab

Revert to prefab defaults

8. Console / Diagnostics
Log Categories

Errors

Warnings

Info

Lua / scripting logs

Engine logs (loading, saving, performance)

Controls

Filter by category

Search text

Clear log

Export log

Click a log entry → select related entity/asset where possible

9. Play / Test Integration
Play Controls

Play from current scene

Play from specific spawn point

Pause, resume, stop

Hot-reload scene data while running (if feasible)

Quick restart

Debug Overlays (Optional but Valuable)

Entity IDs / names

Pathfinding debug (walk mesh/path lines)

Trigger volumes

Tower ranges

FPS counter

10. Validation & Tooling
Validation Checks

Broken asset references (missing sprite, audio, script)

Missing dialogue sets for NPCs

Loot tables referencing unknown items

Story graphs with missing nodes/edges

Scenes with no player spawn

TD maps with no valid path from spawn to goal

Guided Tooling

Wizard for creating a new JRPG map (tileset + NPCs + story links)

Wizard for creating a new TD map (paths + waves + towers)

Tutorial tooltips for each main tab