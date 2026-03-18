//! Grid-based tile editor data model.
//!
//! Ported from the Helix2000 tile editor (`TileEditorTypes.ts`).
//! Sparse storage per layer using `HashMap<(i32, i32), TileType>`.

use bevy::prelude::*;
use bevy_egui::egui::Color32;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Tile types
// ---------------------------------------------------------------------------

/// All tile types available in the editor. Mirrors Helix2000's TileType enum.
#[derive(Default, Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TileType {
    #[default]
    Empty,
    // Ground (8)
    Floor,
    Grass,
    Stone,
    Wood,
    Water,
    Lava,
    Ice,
    Rope,
    // Collision (4)
    Wall,
    Door,
    Window,
    HalfWall,
    // Objects (2)
    Chest,
    Lever,
    // Entities (4)
    NpcSpawn,
    TrainingDummy,
    SpawnPoint,
    Teleporter,
    // Triggers (3)
    Trap,
    SafeZone,
    ExitZone,
}

// ---------------------------------------------------------------------------
// Layer types
// ---------------------------------------------------------------------------

/// Ordered layer types. Ground is bottom, Entities is top.
#[derive(Default, Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LayerType {
    #[default]
    Ground,
    Collision,
    Objects,
    Triggers,
    Entities,
}

impl LayerType {
    pub const ALL: &[LayerType] = &[
        LayerType::Ground,
        LayerType::Collision,
        LayerType::Objects,
        LayerType::Triggers,
        LayerType::Entities,
    ];

    pub fn display_name(self) -> &'static str {
        match self {
            LayerType::Ground => "Ground",
            LayerType::Collision => "Collision",
            LayerType::Objects => "Objects",
            LayerType::Triggers => "Triggers",
            LayerType::Entities => "Entities",
        }
    }

    pub fn icon(self) -> &'static str {
        match self {
            LayerType::Ground => "🌍",
            LayerType::Collision => "🛡",
            LayerType::Objects => "📦",
            LayerType::Triggers => "⚡",
            LayerType::Entities => "👤",
        }
    }

    pub fn color(self) -> Color32 {
        match self {
            LayerType::Ground => Color32::from_rgb(76, 175, 80),
            LayerType::Collision => Color32::from_rgb(244, 67, 54),
            LayerType::Objects => Color32::from_rgb(255, 152, 0),
            LayerType::Triggers => Color32::from_rgb(156, 39, 176),
            LayerType::Entities => Color32::from_rgb(33, 150, 243),
        }
    }

    /// Layers this can be placed on top of (H2K stacking rules).
    pub fn can_stack_on(self) -> &'static [LayerType] {
        match self {
            LayerType::Ground => &[],
            LayerType::Collision => &[LayerType::Ground],
            LayerType::Objects => &[LayerType::Ground, LayerType::Collision],
            LayerType::Triggers => &[LayerType::Ground, LayerType::Collision, LayerType::Objects],
            LayerType::Entities => &[LayerType::Ground, LayerType::Collision, LayerType::Objects],
        }
    }
}

// ---------------------------------------------------------------------------
// Editor tool enum
// ---------------------------------------------------------------------------

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EditorTool {
    Select,
    #[default]
    Brush,
    Eraser,
    Fill,
    Rectangle,
    Line,
    EntityPlacer,
}

impl EditorTool {
    pub const ALL: &[EditorTool] = &[
        EditorTool::Select,
        EditorTool::Brush,
        EditorTool::Eraser,
        EditorTool::Fill,
        EditorTool::Rectangle,
        EditorTool::Line,
        EditorTool::EntityPlacer,
    ];

    pub fn label(self) -> &'static str {
        match self {
            EditorTool::Select => "Select",
            EditorTool::Brush => "Brush",
            EditorTool::Eraser => "Eraser",
            EditorTool::Fill => "Fill",
            EditorTool::Rectangle => "Rect",
            EditorTool::Line => "Line",
            EditorTool::EntityPlacer => "Entity",
        }
    }

    pub fn icon(self) -> &'static str {
        match self {
            EditorTool::Select => "🔍",
            EditorTool::Brush => "🖌",
            EditorTool::Eraser => "🧹",
            EditorTool::Fill => "🪣",
            EditorTool::Rectangle => "▬",
            EditorTool::Line => "╱",
            EditorTool::EntityPlacer => "👤",
        }
    }

    pub fn supports_drag(self) -> bool {
        matches!(self, EditorTool::Brush | EditorTool::Eraser)
    }
}

// ---------------------------------------------------------------------------
// Tile layer (sparse)
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TileLayer {
    pub name: LayerType,
    #[serde(
        serialize_with = "serialize_tile_map",
        deserialize_with = "deserialize_tile_map"
    )]
    pub tiles: HashMap<(i32, i32), TileType>,
    pub visible: bool,
    pub locked: bool,
    pub opacity: f32,
}

/// Serialize tile map as `{"x,y": TileType}` (JSON-compatible string keys).
fn serialize_tile_map<S: serde::Serializer>(
    map: &HashMap<(i32, i32), TileType>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    use serde::ser::SerializeMap;
    let mut m = serializer.serialize_map(Some(map.len()))?;
    for ((x, y), tt) in map {
        m.serialize_entry(&format!("{},{}", x, y), tt)?;
    }
    m.end()
}

/// Deserialize tile map from `{"x,y": TileType}` string keys.
fn deserialize_tile_map<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<HashMap<(i32, i32), TileType>, D::Error> {
    let raw: HashMap<String, TileType> = HashMap::deserialize(deserializer)?;
    let mut result = HashMap::with_capacity(raw.len());
    for (key, tt) in raw {
        let parts: Vec<&str> = key.split(',').collect();
        if parts.len() == 2 {
            let x = parts[0]
                .parse::<i32>()
                .map_err(serde::de::Error::custom)?;
            let y = parts[1]
                .parse::<i32>()
                .map_err(serde::de::Error::custom)?;
            result.insert((x, y), tt);
        }
    }
    Ok(result)
}

impl TileLayer {
    pub fn new(name: LayerType) -> Self {
        Self {
            name,
            tiles: HashMap::new(),
            visible: true,
            locked: false,
            opacity: 1.0,
        }
    }
}

// ---------------------------------------------------------------------------
// Level entity
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LevelEntity {
    pub id: String,
    pub entity_type: String,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub properties: HashMap<String, String>,
}

// ---------------------------------------------------------------------------
// Grid level (the whole map)
// ---------------------------------------------------------------------------

#[derive(Resource, Clone, Debug, Serialize, Deserialize)]
pub struct GridLevel {
    pub width: i32,
    pub height: i32,
    pub tile_size: f32,
    pub layers: Vec<TileLayer>,
    pub entities: Vec<LevelEntity>,
    pub boundary_left: i32,
    pub boundary_right: i32,
}

impl Default for GridLevel {
    fn default() -> Self {
        Self {
            width: 60,
            height: 30,
            tile_size: 32.0,
            layers: LayerType::ALL.iter().map(|&lt| TileLayer::new(lt)).collect(),
            boundary_left: 0,
            boundary_right: 60,
            entities: Vec::new(),
        }
    }
}

impl GridLevel {
    pub fn layer(&self, lt: LayerType) -> Option<&TileLayer> {
        self.layers.iter().find(|l| l.name == lt)
    }

    pub fn layer_mut(&mut self, lt: LayerType) -> Option<&mut TileLayer> {
        self.layers.iter_mut().find(|l| l.name == lt)
    }

    /// Paint a tile on the given layer. Returns true if actually changed.
    /// Validates stacking rules: non-ground layers require a tile on a valid layer below.
    pub fn paint(&mut self, lt: LayerType, x: i32, y: i32, tile: TileType) -> bool {
        // Check locked
        if self.layer(lt).map_or(true, |l| l.locked) {
            return false;
        }
        // Stacking validation: require a tile on a supporting layer below
        let required = lt.can_stack_on();
        if !required.is_empty() {
            let has_support = required.iter().any(|&below| {
                self.layer(below)
                    .map_or(false, |l| l.tiles.contains_key(&(x, y)))
            });
            if !has_support {
                return false;
            }
        }
        // Apply
        if let Some(layer) = self.layer_mut(lt) {
            let old = layer.tiles.insert((x, y), tile);
            old != Some(tile)
        } else {
            false
        }
    }

    /// Erase a tile on the given layer. Returns true if actually changed.
    /// Rejects if a higher layer has content at the same position (content-above check).
    pub fn erase(&mut self, lt: LayerType, x: i32, y: i32) -> bool {
        // Check locked
        if self.layer(lt).map_or(true, |l| l.locked) {
            return false;
        }
        // Content-above check: don't remove foundation if higher layers depend on it
        let layer_idx = self.layers.iter().position(|l| l.name == lt);
        if let Some(idx) = layer_idx {
            for higher in &self.layers[idx + 1..] {
                if higher.tiles.contains_key(&(x, y)) {
                    return false; // Can't erase — something above depends on this
                }
            }
        }
        if let Some(layer) = self.layer_mut(lt) {
            layer.tiles.remove(&(x, y)).is_some()
        } else {
            false
        }
    }

    /// Get tile at position on a given layer.
    pub fn get_tile(&self, lt: LayerType, x: i32, y: i32) -> TileType {
        self.layer(lt)
            .and_then(|l| l.tiles.get(&(x, y)).copied())
            .unwrap_or(TileType::Empty)
    }

    // --- Entity CRUD ---

    /// Place an entity, returning its generated ID.
    /// Teleporters are auto-linked to the most recent teleporter (H2K behavior).
    pub fn place_entity(&mut self, entity_type: &str, tx: i32, ty: i32) -> String {
        // Remove any existing entity at that position
        self.entities.retain(|e| e.x != tx || e.y != ty);
        let id = format!("{}_{}", entity_type, self.entities.len());

        // Auto-link teleporters: find most recent teleporter to link with
        let mut properties = HashMap::new();
        if entity_type == "teleporter" {
            if let Some(last_tp) = self
                .entities
                .iter_mut()
                .rev()
                .find(|e| e.entity_type == "teleporter")
            {
                // Bidirectional link
                properties.insert("teleporter_link".to_string(), last_tp.id.clone());
                last_tp
                    .properties
                    .insert("teleporter_link".to_string(), id.clone());
            }
        }

        self.entities.push(LevelEntity {
            id: id.clone(),
            entity_type: entity_type.to_string(),
            x: tx,
            y: ty,
            width: 1,
            height: 1,
            properties,
        });
        id
    }

    /// Remove entity by ID.
    pub fn remove_entity(&mut self, id: &str) -> bool {
        let before = self.entities.len();
        self.entities.retain(|e| e.id != id);
        self.entities.len() < before
    }

    /// Move entity to new position.
    pub fn move_entity(&mut self, id: &str, new_x: i32, new_y: i32) -> bool {
        if let Some(e) = self.entities.iter_mut().find(|e| e.id == id) {
            e.x = new_x;
            e.y = new_y;
            true
        } else {
            false
        }
    }

    /// Find entity at tile position (hit-test).
    pub fn entity_at(&self, tx: i32, ty: i32) -> Option<&LevelEntity> {
        self.entities.iter().find(|e| {
            tx >= e.x && tx < e.x + e.width && ty >= e.y && ty < e.y + e.height
        })
    }
}

// ---------------------------------------------------------------------------
// Tile palette item
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct TilePaletteItem {
    pub tile_type: TileType,
    pub name: &'static str,
    pub color: Color32,
    pub layer: LayerType,
    pub label: char,
}

/// The default palette. Colors match Helix2000's `DEFAULT_TILE_PALETTE`.
pub const DEFAULT_PALETTE: &[TilePaletteItem] = &[
    // Ground (8)
    TilePaletteItem { tile_type: TileType::Floor,  name: "Floor",  color: Color32::from_rgb(139, 115, 85),  layer: LayerType::Ground, label: 'F' },
    TilePaletteItem { tile_type: TileType::Grass,  name: "Grass",  color: Color32::from_rgb(124, 252, 0),   layer: LayerType::Ground, label: 'G' },
    TilePaletteItem { tile_type: TileType::Stone,  name: "Stone",  color: Color32::from_rgb(128, 128, 128), layer: LayerType::Ground, label: 'S' },
    TilePaletteItem { tile_type: TileType::Wood,   name: "Wood",   color: Color32::from_rgb(210, 105, 30),  layer: LayerType::Ground, label: 'W' },
    TilePaletteItem { tile_type: TileType::Water,  name: "Water",  color: Color32::from_rgb(30, 144, 255),  layer: LayerType::Ground, label: '~' },
    TilePaletteItem { tile_type: TileType::Lava,   name: "Lava",   color: Color32::from_rgb(255, 69, 0),    layer: LayerType::Ground, label: '!' },
    TilePaletteItem { tile_type: TileType::Ice,    name: "Ice",    color: Color32::from_rgb(224, 255, 255), layer: LayerType::Ground, label: 'I' },
    TilePaletteItem { tile_type: TileType::Rope,   name: "Rope",   color: Color32::from_rgb(210, 180, 140), layer: LayerType::Ground, label: '|' },
    // Collision (4)
    TilePaletteItem { tile_type: TileType::Wall,     name: "Wall",      color: Color32::from_rgb(74, 74, 74),    layer: LayerType::Collision, label: '#' },
    TilePaletteItem { tile_type: TileType::Door,     name: "Door",      color: Color32::from_rgb(139, 69, 19),   layer: LayerType::Collision, label: 'D' },
    TilePaletteItem { tile_type: TileType::Window,   name: "Window",    color: Color32::from_rgb(135, 206, 235), layer: LayerType::Collision, label: 'O' },
    TilePaletteItem { tile_type: TileType::HalfWall, name: "Half Wall", color: Color32::from_rgb(105, 105, 105), layer: LayerType::Collision, label: '=' },
    // Objects (2)
    TilePaletteItem { tile_type: TileType::Chest, name: "Chest", color: Color32::from_rgb(218, 165, 32), layer: LayerType::Objects, label: 'C' },
    TilePaletteItem { tile_type: TileType::Lever, name: "Lever", color: Color32::from_rgb(160, 82, 45),  layer: LayerType::Objects, label: 'L' },
    // Entities (4)
    TilePaletteItem { tile_type: TileType::NpcSpawn,      name: "NPC Spawn",     color: Color32::from_rgb(0, 255, 0),    layer: LayerType::Entities, label: 'N' },
    TilePaletteItem { tile_type: TileType::TrainingDummy, name: "Training Dummy",color: Color32::from_rgb(255, 215, 0),  layer: LayerType::Entities, label: 'T' },
    TilePaletteItem { tile_type: TileType::SpawnPoint,    name: "Player Spawn",  color: Color32::from_rgb(50, 205, 50),  layer: LayerType::Entities, label: '@' },
    TilePaletteItem { tile_type: TileType::Teleporter,    name: "Teleporter",    color: Color32::from_rgb(147, 112, 219),layer: LayerType::Entities, label: 'T' },
    // Triggers (3)
    TilePaletteItem { tile_type: TileType::Trap,     name: "Trap",      color: Color32::from_rgb(255, 0, 0),     layer: LayerType::Triggers, label: 'X' },
    TilePaletteItem { tile_type: TileType::SafeZone, name: "Safe Zone", color: Color32::from_rgb(0, 255, 255),   layer: LayerType::Triggers, label: 'Z' },
    TilePaletteItem { tile_type: TileType::ExitZone, name: "Exit Zone", color: Color32::from_rgb(255, 0, 255),   layer: LayerType::Triggers, label: 'E' },
];

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Get the palette item for a tile type.
pub fn palette_item(tt: TileType) -> Option<&'static TilePaletteItem> {
    DEFAULT_PALETTE.iter().find(|p| p.tile_type == tt)
}

/// Get the display color for a tile type.
pub fn tile_color(tt: TileType) -> Color32 {
    palette_item(tt).map_or(Color32::WHITE, |p| p.color)
}

/// Get the display label char for a tile type.
pub fn tile_label(tt: TileType) -> char {
    palette_item(tt).map_or('?', |p| p.label)
}

/// Get the natural layer for a tile type.
pub fn tile_layer(tt: TileType) -> LayerType {
    palette_item(tt).map_or(LayerType::Ground, |p| p.layer)
}

// ---------------------------------------------------------------------------
// Painting state (tracked per-frame in the viewport)
// ---------------------------------------------------------------------------

/// Transient painting state for the viewport (not serialized).
#[derive(Resource, Default, Debug)]
pub struct PaintState {
    /// Whether the mouse is currently held down and painting.
    pub is_painting: bool,
    /// Last tile coordinate we painted to (for debounce during drag).
    pub last_paint_pos: Option<(i32, i32)>,
    /// Start position for rect/line tools.
    pub drag_start: Option<(i32, i32)>,
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_grid_has_five_layers() {
        let grid = GridLevel::default();
        assert_eq!(grid.layers.len(), 5);
        assert_eq!(grid.layers[0].name, LayerType::Ground);
        assert_eq!(grid.layers[4].name, LayerType::Entities);
    }

    #[test]
    fn paint_and_erase() {
        let mut grid = GridLevel::default();
        assert!(grid.paint(LayerType::Ground, 5, 3, TileType::Grass));
        assert_eq!(grid.get_tile(LayerType::Ground, 5, 3), TileType::Grass);
        assert!(grid.erase(LayerType::Ground, 5, 3));
        assert_eq!(grid.get_tile(LayerType::Ground, 5, 3), TileType::Empty);
    }

    #[test]
    fn paint_on_locked_layer_fails() {
        let mut grid = GridLevel::default();
        grid.layer_mut(LayerType::Ground).unwrap().locked = true;
        assert!(!grid.paint(LayerType::Ground, 0, 0, TileType::Floor));
    }

    #[test]
    fn palette_has_21_entries() {
        assert_eq!(DEFAULT_PALETTE.len(), 21);
    }

    #[test]
    fn tile_color_lookup() {
        assert_eq!(tile_color(TileType::Grass), Color32::from_rgb(124, 252, 0));
        assert_eq!(tile_color(TileType::Empty), Color32::WHITE);
    }

    #[test]
    fn tile_layer_lookup() {
        assert_eq!(tile_layer(TileType::Wall), LayerType::Collision);
        assert_eq!(tile_layer(TileType::Trap), LayerType::Triggers);
    }

    #[test]
    fn grid_serialization_roundtrip() {
        let mut grid = GridLevel::default();
        grid.paint(LayerType::Ground, 1, 2, TileType::Stone);
        // Collision requires ground below
        grid.paint(LayerType::Ground, -3, 4, TileType::Floor);
        grid.paint(LayerType::Collision, -3, 4, TileType::Wall);
        let json = serde_json::to_string(&grid).unwrap();
        let restored: GridLevel = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.get_tile(LayerType::Ground, 1, 2), TileType::Stone);
        assert_eq!(
            restored.get_tile(LayerType::Collision, -3, 4),
            TileType::Wall
        );
    }

    #[test]
    fn stacking_collision_requires_ground() {
        let mut grid = GridLevel::default();
        // No ground → collision should fail
        assert!(!grid.paint(LayerType::Collision, 0, 0, TileType::Wall));
        // Add ground → collision succeeds
        assert!(grid.paint(LayerType::Ground, 0, 0, TileType::Floor));
        assert!(grid.paint(LayerType::Collision, 0, 0, TileType::Wall));
    }

    #[test]
    fn stacking_triggers_on_ground_or_collision() {
        let mut grid = GridLevel::default();
        // Trigger on empty → fail
        assert!(!grid.paint(LayerType::Triggers, 1, 1, TileType::Trap));
        // Trigger on ground → ok
        grid.paint(LayerType::Ground, 1, 1, TileType::Floor);
        assert!(grid.paint(LayerType::Triggers, 1, 1, TileType::Trap));
    }

    #[test]
    fn ground_has_no_stacking_requirement() {
        let mut grid = GridLevel::default();
        assert!(grid.paint(LayerType::Ground, 99, 99, TileType::Grass));
    }

    #[test]
    fn entity_place_and_remove() {
        let mut grid = GridLevel::default();
        let id = grid.place_entity("npc", 5, 5);
        assert!(grid.entity_at(5, 5).is_some());
        assert!(grid.remove_entity(&id));
        assert!(grid.entity_at(5, 5).is_none());
    }

    #[test]
    fn entity_move() {
        let mut grid = GridLevel::default();
        let id = grid.place_entity("mob", 0, 0);
        assert!(grid.move_entity(&id, 3, 3));
        assert!(grid.entity_at(0, 0).is_none());
        assert!(grid.entity_at(3, 3).is_some());
    }

    #[test]
    fn entity_hit_test() {
        let mut grid = GridLevel::default();
        grid.place_entity("chest", 2, 2);
        assert!(grid.entity_at(2, 2).is_some());
        assert!(grid.entity_at(3, 3).is_none());
    }

    #[test]
    fn erase_blocked_by_content_above() {
        let mut grid = GridLevel::default();
        grid.paint(LayerType::Ground, 0, 0, TileType::Floor);
        grid.paint(LayerType::Collision, 0, 0, TileType::Wall);
        // Can't erase ground — collision tile is above
        assert!(!grid.erase(LayerType::Ground, 0, 0));
        // Can erase collision (nothing above it)
        assert!(grid.erase(LayerType::Collision, 0, 0));
        // Now can erase ground
        assert!(grid.erase(LayerType::Ground, 0, 0));
    }

    #[test]
    fn teleporter_auto_link() {
        let mut grid = GridLevel::default();
        let tp1 = grid.place_entity("teleporter", 0, 0);
        let tp2 = grid.place_entity("teleporter", 5, 5);
        // tp2 should link to tp1 and tp1 should link to tp2
        let tp2_ent = grid.entities.iter().find(|e| e.id == tp2).unwrap();
        assert_eq!(
            tp2_ent.properties.get("teleporter_link"),
            Some(&tp1.clone())
        );
        let tp1_ent = grid.entities.iter().find(|e| e.id == tp1).unwrap();
        assert_eq!(
            tp1_ent.properties.get("teleporter_link"),
            Some(&tp2)
        );
    }
}
