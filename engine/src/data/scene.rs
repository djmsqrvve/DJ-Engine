//! Scene data structures for maps and levels.
//!
//! A [`Scene`] represents a single map/level containing layers and entities.
//! Scenes can be JRPG maps, TD maps, or shared between both game types.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::components::{ColorData, EntityComponents, Vec3Data};

/// Scene type categorization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, Reflect)]
#[serde(rename_all = "snake_case")]
pub enum SceneType {
    /// JRPG-style map (tilemap, NPCs, story triggers)
    #[default]
    Jrpg,
    /// Tower Defense map (pathfinding grid, build zones)
    Td,
    /// Shared/generic scene usable by both game types
    Shared,
}

/// Entity type categorization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, Reflect)]
#[serde(rename_all = "snake_case")]
pub enum EntityType {
    /// Non-player character (friendly)
    Npc,
    /// Hostile enemy
    Enemy,
    /// Defensive tower (TD)
    Tower,
    /// Trigger zone
    Trigger,
    /// Static prop (interactive)
    Prop,
    /// Decoration (non-interactive)
    Deco,
    /// Enemy/unit spawner
    Spawner,
    /// UI element
    Ui,
    /// Generic/other
    #[default]
    Other,
}

/// 2D size with integer dimensions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, Reflect)]
pub struct TileSize {
    pub width: u32,
    pub height: u32,
}

/// Default spawn points for player and camera.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, Reflect)]
pub struct DefaultSpawn {
    /// Player spawn position
    #[serde(default)]
    pub player: Vec3Data,
    /// Camera initial position
    #[serde(default)]
    pub camera: Vec3Data,
}

/// Scene audio settings.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize, Reflect)]
pub struct SceneAudio {
    /// Background music track ID
    #[serde(default)]
    pub music_track_id: Option<String>,
    /// Whether to loop the music
    #[serde(default = "default_true")]
    pub loop_music: bool,
}

fn default_true() -> bool {
    true
}

/// Scene script hooks.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize, Reflect)]
pub struct SceneScripts {
    /// Script to run when entering the scene
    #[serde(default)]
    pub on_enter: Option<String>,
    /// Script to run when exiting the scene
    #[serde(default)]
    pub on_exit: Option<String>,
}

/// Layer in a scene (for organizing entities).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Reflect)]
pub struct Layer {
    /// Unique layer identifier
    pub id: String,
    /// Human-readable layer name
    pub name: String,
    /// Rendering order (higher = rendered later / on top)
    #[serde(default)]
    pub order: i32,
    /// Whether layer is visible in editor/game
    #[serde(default = "default_true")]
    pub visible: bool,
    /// Whether layer is locked (cannot select entities)
    #[serde(default)]
    pub locked: bool,
    /// Parallax scrolling factor
    #[serde(default = "default_parallax")]
    pub parallax: Vec3Data,
}

fn default_parallax() -> Vec3Data {
    Vec3Data::new(1.0, 1.0, 1.0)
}

impl Default for Layer {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: "Layer".to_string(),
            order: 0,
            visible: true,
            locked: false,
            parallax: default_parallax(),
        }
    }
}

impl Layer {
    /// Create a new layer with the given ID and name.
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            ..Default::default()
        }
    }

    /// Create a new layer with order.
    pub fn with_order(mut self, order: i32) -> Self {
        self.order = order;
        self
    }
}

/// Pathfinding cell for TD maps.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize, Reflect)]
pub struct PathfindingCell {
    /// Cell X coordinate
    pub x: i32,
    /// Cell Y coordinate
    pub y: i32,
    /// Whether units can walk through
    #[serde(default = "default_true")]
    pub walkable: bool,
    /// Whether towers can be built here
    #[serde(default)]
    pub buildable: bool,
}

/// Pathfinding grid for TD maps.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize, Reflect)]
pub struct PathfindingGrid {
    /// Grid width in cells
    pub width: u32,
    /// Grid height in cells
    pub height: u32,
    /// Individual cell data
    #[serde(default)]
    pub cells: Vec<PathfindingCell>,
}

/// Scene pathfinding configuration.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize, Reflect)]
pub struct ScenePathfinding {
    /// Whether pathfinding is enabled for this scene
    #[serde(default)]
    pub enabled: bool,
    /// Pathfinding grid
    #[serde(default)]
    pub grid: PathfindingGrid,
}

/// An entity in a scene.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Reflect)]
pub struct Entity {
    /// Unique entity identifier
    pub id: String,
    /// Human-readable entity name
    pub name: String,
    /// Entity type categorization
    #[serde(default)]
    pub entity_type: EntityType,
    /// Layer this entity belongs to
    #[serde(default)]
    pub layer_id: String,
    /// Parent entity ID (for hierarchy)
    #[serde(default)]
    pub parent_id: Option<String>,
    /// Prefab ID if this is a prefab instance
    #[serde(default)]
    pub prefab_id: Option<String>,
    /// Entity components
    pub components: EntityComponents,
}

impl Default for Entity {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: "Entity".to_string(),
            entity_type: EntityType::Other,
            layer_id: String::new(),
            parent_id: None,
            prefab_id: None,
            components: EntityComponents::default(),
        }
    }
}

impl Entity {
    /// Create a new entity with the given ID and name.
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            ..Default::default()
        }
    }

    /// Set the entity type.
    pub fn with_type(mut self, entity_type: EntityType) -> Self {
        self.entity_type = entity_type;
        self
    }

    /// Set the layer ID.
    pub fn with_layer(mut self, layer_id: impl Into<String>) -> Self {
        self.layer_id = layer_id.into();
        self
    }

    /// Set the components.
    pub fn with_components(mut self, components: EntityComponents) -> Self {
        self.components = components;
        self
    }
}

/// A complete scene/map.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Reflect)]
pub struct Scene {
    /// Unique scene identifier
    pub id: String,
    /// Human-readable scene name
    pub name: String,
    /// Scene type (JRPG, TD, or Shared)
    #[serde(default)]
    pub scene_type: SceneType,
    /// Scene size in tiles
    #[serde(default)]
    pub size_tiles: TileSize,
    /// Tile size in pixels
    #[serde(default = "default_tile_size")]
    pub tile_size: TileSize,
    /// Background color
    #[serde(default)]
    pub background_color: ColorData,
    /// Default spawn points
    #[serde(default)]
    pub default_spawn: DefaultSpawn,
    /// Audio settings
    #[serde(default)]
    pub audio: SceneAudio,
    /// Script hooks
    #[serde(default)]
    pub scripts: SceneScripts,
    /// Scene layers
    #[serde(default)]
    pub layers: Vec<Layer>,
    /// Scene entities
    #[serde(default)]
    pub entities: Vec<Entity>,
    /// Pathfinding configuration (TD)
    #[serde(default)]
    pub pathfinding: ScenePathfinding,
}

fn default_tile_size() -> TileSize {
    TileSize {
        width: 32,
        height: 32,
    }
}

impl Default for Scene {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: "New Scene".to_string(),
            scene_type: SceneType::default(),
            size_tiles: TileSize {
                width: 20,
                height: 15,
            },
            tile_size: default_tile_size(),
            background_color: ColorData::black(),
            default_spawn: DefaultSpawn::default(),
            audio: SceneAudio::default(),
            scripts: SceneScripts::default(),
            layers: vec![
                Layer::new("background", "Background").with_order(-10),
                Layer::new("main", "Main").with_order(0),
                Layer::new("foreground", "Foreground").with_order(10),
            ],
            entities: Vec::new(),
            pathfinding: ScenePathfinding::default(),
        }
    }
}

impl Scene {
    /// Create a new scene with the given ID and name.
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            ..Default::default()
        }
    }

    /// Create a new JRPG scene.
    pub fn new_jrpg(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            scene_type: SceneType::Jrpg,
            ..Self::new(id, name)
        }
    }

    /// Create a new TD scene.
    pub fn new_td(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            scene_type: SceneType::Td,
            pathfinding: ScenePathfinding {
                enabled: true,
                grid: PathfindingGrid::default(),
            },
            ..Self::new(id, name)
        }
    }

    /// Find an entity by ID.
    pub fn find_entity(&self, id: &str) -> Option<&Entity> {
        self.entities.iter().find(|e| e.id == id)
    }

    /// Find an entity by ID (mutable).
    pub fn find_entity_mut(&mut self, id: &str) -> Option<&mut Entity> {
        self.entities.iter_mut().find(|e| e.id == id)
    }

    /// Add an entity to the scene.
    pub fn add_entity(&mut self, entity: Entity) {
        self.entities.push(entity);
    }

    /// Remove an entity by ID. Returns the removed entity if found.
    pub fn remove_entity(&mut self, id: &str) -> Option<Entity> {
        if let Some(pos) = self.entities.iter().position(|e| e.id == id) {
            Some(self.entities.remove(pos))
        } else {
            None
        }
    }

    /// Find a layer by ID.
    pub fn find_layer(&self, id: &str) -> Option<&Layer> {
        self.layers.iter().find(|l| l.id == id)
    }

    /// Add a layer to the scene.
    pub fn add_layer(&mut self, layer: Layer) {
        self.layers.push(layer);
        self.layers.sort_by_key(|l| l.order);
    }

    /// Get all entities in a specific layer.
    pub fn entities_in_layer(&self, layer_id: &str) -> Vec<&Entity> {
        self.entities
            .iter()
            .filter(|e| e.layer_id == layer_id)
            .collect()
    }

    /// Get all entities of a specific type.
    pub fn entities_of_type(&self, entity_type: EntityType) -> Vec<&Entity> {
        self.entities
            .iter()
            .filter(|e| e.entity_type == entity_type)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scene_serialization() {
        let scene = Scene::new_jrpg("level_01", "Town Square");
        let json = serde_json::to_string_pretty(&scene).unwrap();
        let parsed: Scene = serde_json::from_str(&json).unwrap();
        assert_eq!(scene.id, parsed.id);
        assert_eq!(scene.scene_type, parsed.scene_type);
    }

    #[test]
    fn test_add_find_entity() {
        let mut scene = Scene::new("test", "Test Scene");
        scene.add_entity(Entity::new("npc_01", "Merchant").with_type(EntityType::Npc));
        scene.add_entity(Entity::new("enemy_01", "Goblin").with_type(EntityType::Enemy));

        assert!(scene.find_entity("npc_01").is_some());
        assert!(scene.find_entity("nonexistent").is_none());

        let npcs = scene.entities_of_type(EntityType::Npc);
        assert_eq!(npcs.len(), 1);
    }

    #[test]
    fn test_remove_entity() {
        let mut scene = Scene::new("test", "Test");
        scene.add_entity(Entity::new("e1", "Entity 1"));
        scene.add_entity(Entity::new("e2", "Entity 2"));

        let removed = scene.remove_entity("e1");
        assert!(removed.is_some());
        assert_eq!(scene.entities.len(), 1);
        assert!(scene.find_entity("e1").is_none());
    }
}
