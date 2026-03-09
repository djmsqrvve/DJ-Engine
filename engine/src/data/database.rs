//! Game database structures for items, NPCs, enemies, etc.
//!
//! The database contains static game data that is referenced by entities
//! and story graphs. Data is stored in JSON and loaded at startup.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Localized string (text in multiple languages).
pub type LocalizedString = HashMap<String, String>;

/// Item type categorization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ItemType {
    Weapon,
    Armor,
    Potion,
    Currency,
    QuestItem,
    #[default]
    Misc,
}

/// Item rarity tier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Rarity {
    #[default]
    Common,
    Uncommon,
    Rare,
    Epic,
    Legendary,
}

/// Script hooks for items.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ItemScripts {
    /// Script to run when item is used
    #[serde(default)]
    pub on_use: Option<String>,
    /// Script to run when item is equipped
    #[serde(default)]
    pub on_equip: Option<String>,
    /// Script to run when item is unequipped
    #[serde(default)]
    pub on_unequip: Option<String>,
}

/// An item definition in the database.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ItemRow {
    /// Unique item identifier
    pub id: String,
    /// Display name per language
    pub name: LocalizedString,
    /// Item type
    #[serde(default)]
    pub item_type: ItemType,
    /// Attack damage bonus
    #[serde(default)]
    pub damage: i32,
    /// Defense bonus
    #[serde(default)]
    pub defense: i32,
    /// Healing amount (for potions)
    #[serde(default)]
    pub heal_amount: i32,
    /// Buy price
    #[serde(default)]
    pub price: i32,
    /// Sell value
    #[serde(default)]
    pub sell_value: i32,
    /// Maximum stack size
    #[serde(default = "default_max_stack")]
    pub max_stack: u32,
    /// Rarity tier
    #[serde(default)]
    pub rarity: Rarity,
    /// Sprite asset ID
    #[serde(default)]
    pub sprite_id: String,
    /// Description per language
    #[serde(default)]
    pub description: LocalizedString,
    /// Script hooks
    #[serde(default)]
    pub scripts: ItemScripts,
}

fn default_max_stack() -> u32 {
    99
}

impl Default for ItemRow {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: HashMap::new(),
            item_type: ItemType::Misc,
            damage: 0,
            defense: 0,
            heal_amount: 0,
            price: 0,
            sell_value: 0,
            max_stack: 99,
            rarity: Rarity::Common,
            sprite_id: String::new(),
            description: HashMap::new(),
            scripts: ItemScripts::default(),
        }
    }
}

impl ItemRow {
    /// Create a new item with the given ID and English name.
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        let mut name_map = HashMap::new();
        name_map.insert("en".to_string(), name.into());
        Self {
            id: id.into(),
            name: name_map,
            ..Default::default()
        }
    }

    /// Set the item type.
    pub fn with_type(mut self, item_type: ItemType) -> Self {
        self.item_type = item_type;
        self
    }
}

/// An NPC definition in the database.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct NpcRow {
    /// Unique NPC identifier
    pub id: String,
    /// Display name per language
    pub name: LocalizedString,
    /// Dialogue set ID
    #[serde(default)]
    pub dialogue_set_id: String,
    /// Location tags for filtering
    #[serde(default)]
    pub location_tags: Vec<String>,
    /// Default faction/alignment
    #[serde(default)]
    pub default_faction: String,
    /// Associated quest IDs
    #[serde(default)]
    pub default_quest_ids: Vec<String>,
    /// Loot table ID (for killable NPCs)
    #[serde(default)]
    pub loot_table_id: Option<String>,
    /// Portrait sprite ID
    #[serde(default)]
    pub portrait_id: String,
}

impl NpcRow {
    /// Create a new NPC with the given ID and English name.
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        let mut name_map = HashMap::new();
        name_map.insert("en".to_string(), name.into());
        Self {
            id: id.into(),
            name: name_map,
            ..Default::default()
        }
    }
}

/// A tower definition in the database (TD-specific).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TowerRow {
    /// Unique tower identifier
    pub id: String,
    /// Display name per language
    pub name: LocalizedString,
    /// Attack damage
    #[serde(default)]
    pub damage: i32,
    /// Attack range in pixels
    #[serde(default = "default_tower_range")]
    pub range: f32,
    /// Attack cooldown in seconds
    #[serde(default = "default_tower_cooldown")]
    pub cooldown: f32,
    /// Build cost (resources)
    #[serde(default)]
    pub cost: i32,
    /// Build time in seconds
    #[serde(default)]
    pub build_time: f32,
    /// Upgrade target tower ID
    #[serde(default)]
    pub upgrade_to_id: Option<String>,
    /// Projectile asset ID
    #[serde(default)]
    pub projectile_id: String,
    /// Effect/VFX ID
    #[serde(default)]
    pub effect_id: Option<String>,
    /// Description per language
    #[serde(default)]
    pub description: LocalizedString,
}

fn default_tower_range() -> f32 {
    200.0
}
fn default_tower_cooldown() -> f32 {
    1.0
}

impl Default for TowerRow {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: HashMap::new(),
            damage: 25,
            range: 200.0,
            cooldown: 1.0,
            cost: 100,
            build_time: 0.0,
            upgrade_to_id: None,
            projectile_id: String::new(),
            effect_id: None,
            description: HashMap::new(),
        }
    }
}

impl TowerRow {
    /// Create a new tower with the given ID and English name.
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        let mut name_map = HashMap::new();
        name_map.insert("en".to_string(), name.into());
        Self {
            id: id.into(),
            name: name_map,
            ..Default::default()
        }
    }
}

/// An enemy definition in the database.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EnemyRow {
    /// Unique enemy identifier
    pub id: String,
    /// Display name per language
    pub name: LocalizedString,
    /// Hit points
    #[serde(default = "default_hp")]
    pub hp: i32,
    /// Attack damage
    #[serde(default)]
    pub damage: i32,
    /// Movement speed
    #[serde(default = "default_speed")]
    pub speed: f32,
    /// Experience reward on kill
    #[serde(default)]
    pub experience: i32,
    /// Loot table ID
    #[serde(default)]
    pub loot_table_id: String,
    /// AI behavior profile ID
    #[serde(default)]
    pub behavior_profile_id: String,
}

fn default_hp() -> i32 {
    100
}
fn default_speed() -> f32 {
    100.0
}

impl Default for EnemyRow {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: HashMap::new(),
            hp: 100,
            damage: 10,
            speed: 100.0,
            experience: 50,
            loot_table_id: String::new(),
            behavior_profile_id: String::new(),
        }
    }
}

impl EnemyRow {
    /// Create a new enemy with the given ID and English name.
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        let mut name_map = HashMap::new();
        name_map.insert("en".to_string(), name.into());
        Self {
            id: id.into(),
            name: name_map,
            ..Default::default()
        }
    }
}

/// A loot table entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LootEntry {
    /// Item ID to drop
    pub item_id: String,
    /// Drop chance (0.0 - 1.0)
    #[serde(default = "default_chance")]
    pub chance: f32,
    /// Minimum quantity
    #[serde(default = "default_min_qty")]
    pub min_quantity: u32,
    /// Maximum quantity
    #[serde(default = "default_max_qty")]
    pub max_quantity: u32,
}

fn default_chance() -> f32 {
    1.0
}
fn default_min_qty() -> u32 {
    1
}
fn default_max_qty() -> u32 {
    1
}

/// A loot table definition.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct LootTableRow {
    /// Unique loot table identifier
    pub id: String,
    /// Loot entries
    #[serde(default)]
    pub entries: Vec<LootEntry>,
}

impl LootTableRow {
    /// Create a new loot table with the given ID.
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            entries: Vec::new(),
        }
    }

    /// Add an entry to the loot table.
    pub fn add_entry(&mut self, item_id: impl Into<String>, chance: f32, quantity: u32) {
        self.entries.push(LootEntry {
            item_id: item_id.into(),
            chance,
            min_quantity: quantity,
            max_quantity: quantity,
        });
    }
}

/// Item reward for quests.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ItemReward {
    /// Item ID
    pub item_id: String,
    /// Quantity
    pub quantity: u32,
}

/// Quest rewards.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct QuestRewards {
    /// Gold reward
    #[serde(default)]
    pub gold: i32,
    /// Experience reward
    #[serde(default)]
    pub experience: i32,
    /// Item rewards
    #[serde(default)]
    pub item_rewards: Vec<ItemReward>,
    /// Flags to set on completion
    #[serde(default)]
    pub flags: HashMap<String, serde_json::Value>,
}

/// A quest definition.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct QuestRow {
    /// Unique quest identifier
    pub id: String,
    /// Display name per language
    pub name: LocalizedString,
    /// Description per language
    #[serde(default)]
    pub description: LocalizedString,
    /// Conditions to start the quest
    #[serde(default)]
    pub start_conditions: Vec<serde_json::Value>,
    /// Conditions to complete the quest
    #[serde(default)]
    pub completion_conditions: Vec<serde_json::Value>,
    /// Rewards on completion
    #[serde(default)]
    pub rewards: QuestRewards,
}

impl QuestRow {
    /// Create a new quest with the given ID and English name.
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        let mut name_map = HashMap::new();
        name_map.insert("en".to_string(), name.into());
        Self {
            id: id.into(),
            name: name_map,
            ..Default::default()
        }
    }
}

/// The complete game database.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Database {
    /// Item definitions
    #[serde(default)]
    pub items: Vec<ItemRow>,
    /// NPC definitions
    #[serde(default)]
    pub npcs: Vec<NpcRow>,
    /// Tower definitions (TD)
    #[serde(default)]
    pub towers: Vec<TowerRow>,
    /// Enemy definitions
    #[serde(default)]
    pub enemies: Vec<EnemyRow>,
    /// Loot table definitions
    #[serde(default)]
    pub loot_tables: Vec<LootTableRow>,
    /// Quest definitions
    #[serde(default)]
    pub quests: Vec<QuestRow>,
}

impl Database {
    /// Create a new empty database.
    pub fn new() -> Self {
        Self::default()
    }

    /// Find an item by ID.
    pub fn find_item(&self, id: &str) -> Option<&ItemRow> {
        self.items.iter().find(|i| i.id == id)
    }

    /// Find an NPC by ID.
    pub fn find_npc(&self, id: &str) -> Option<&NpcRow> {
        self.npcs.iter().find(|n| n.id == id)
    }

    /// Find a tower by ID.
    pub fn find_tower(&self, id: &str) -> Option<&TowerRow> {
        self.towers.iter().find(|t| t.id == id)
    }

    /// Find an enemy by ID.
    pub fn find_enemy(&self, id: &str) -> Option<&EnemyRow> {
        self.enemies.iter().find(|e| e.id == id)
    }

    /// Find a loot table by ID.
    pub fn find_loot_table(&self, id: &str) -> Option<&LootTableRow> {
        self.loot_tables.iter().find(|l| l.id == id)
    }

    /// Find a quest by ID.
    pub fn find_quest(&self, id: &str) -> Option<&QuestRow> {
        self.quests.iter().find(|q| q.id == id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_serialization() {
        let mut db = Database::new();
        db.items
            .push(ItemRow::new("sword_01", "Iron Sword").with_type(ItemType::Weapon));
        db.npcs.push(NpcRow::new("merchant_01", "Merchant"));
        db.enemies.push(EnemyRow::new("goblin_01", "Goblin"));

        let json = serde_json::to_string_pretty(&db).unwrap();
        let parsed: Database = serde_json::from_str(&json).unwrap();

        assert_eq!(db.items.len(), parsed.items.len());
        assert!(parsed.find_item("sword_01").is_some());
    }

    #[test]
    fn test_loot_table() {
        let mut loot = LootTableRow::new("common_loot");
        loot.add_entry("gold", 1.0, 10);
        loot.add_entry("potion_hp", 0.5, 1);

        assert_eq!(loot.entries.len(), 2);
    }
}
