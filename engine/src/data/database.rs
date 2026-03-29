//! Game database structures for items, NPCs, enemies, etc.
//!
//! The database contains static game data that is referenced by entities
//! and story graphs. Data is stored in JSON and loaded at startup.

use bevy::prelude::*;
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
    Container,
    Reagent,
    TradeGood,
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
    Artifact,
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
    /// Item IDs this NPC sells as a vendor
    #[serde(default)]
    pub vendor_items: Vec<String>,
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
    /// Defense value
    #[serde(default)]
    pub defense: i32,
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
    /// Faction affiliation
    #[serde(default)]
    pub faction: String,
    /// Respawn time in seconds
    #[serde(default)]
    pub respawn_time: f32,
    /// Attack speed multiplier
    #[serde(default)]
    pub attack_speed: f32,
    /// Zone IDs this enemy spawns in
    #[serde(default)]
    pub zone_ids: Vec<String>,
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
            defense: 0,
            speed: 100.0,
            experience: 50,
            loot_table_id: String::new(),
            behavior_profile_id: String::new(),
            faction: String::new(),
            respawn_time: 0.0,
            attack_speed: 0.0,
            zone_ids: Vec::new(),
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
    /// Whether this quest resets daily
    #[serde(default)]
    pub is_daily: bool,
    /// Whether this quest can be repeated
    #[serde(default)]
    pub is_repeatable: bool,
    /// Whether this quest can be shared with party members
    #[serde(default)]
    pub sharable: bool,
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

/// Ability definition.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct AbilityRow {
    pub id: String,
    #[serde(default)]
    pub name: LocalizedString,
    #[serde(default)]
    pub ability_type: String,
    #[serde(default)]
    pub school: String,
    #[serde(default)]
    pub cooldown: f64,
    #[serde(default)]
    pub mana_cost: f64,
    #[serde(default)]
    pub description: LocalizedString,
}

/// Zone definition.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct ZoneRow {
    pub id: String,
    #[serde(default)]
    pub name: LocalizedString,
    #[serde(default)]
    pub level_min: u32,
    #[serde(default)]
    pub level_max: u32,
    #[serde(default)]
    pub continent: String,
    #[serde(default)]
    pub description: LocalizedString,
}

/// Aura (buff/debuff/passive) definition.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct AuraRow {
    pub id: String,
    #[serde(default)]
    pub name: LocalizedString,
    /// buff, debuff, passive
    #[serde(default)]
    pub aura_type: String,
    #[serde(default)]
    pub duration: f64,
    #[serde(default)]
    pub max_stacks: u32,
    #[serde(default)]
    pub description: LocalizedString,
}

/// Playable class definition.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct ClassDataRow {
    pub id: String,
    #[serde(default)]
    pub name: LocalizedString,
    /// tank, healer, melee_dps, ranged_dps, hybrid
    #[serde(default)]
    pub role: String,
    /// mana, rage, energy, focus, runic_power
    #[serde(default)]
    pub resource_type: String,
    #[serde(default)]
    pub abilities: Vec<String>,
    #[serde(default)]
    pub talent_trees: Vec<String>,
}

/// Raid/dungeon instance definition.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct RaidRow {
    pub id: String,
    #[serde(default)]
    pub name: LocalizedString,
    #[serde(default)]
    pub zone_id: String,
    #[serde(default)]
    pub size: u32,
    /// normal, heroic, mythic
    #[serde(default)]
    pub difficulty: String,
    #[serde(default)]
    pub bosses: Vec<String>,
    #[serde(default)]
    pub description: LocalizedString,
}

/// Talent tree node definition.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct TalentRow {
    pub id: String,
    #[serde(default)]
    pub name: LocalizedString,
    #[serde(default)]
    pub class_id: String,
    #[serde(default)]
    pub tree: String,
    #[serde(default)]
    pub tier: u32,
    #[serde(default)]
    pub column: u32,
    #[serde(default)]
    pub max_rank: u32,
    #[serde(default)]
    pub prerequisite_talent: Option<String>,
    #[serde(default)]
    pub description: LocalizedString,
}

/// Profession (crafting/gathering) definition.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct ProfessionRow {
    pub id: String,
    #[serde(default)]
    pub name: LocalizedString,
    /// primary, gathering, crafting, secondary
    #[serde(default)]
    pub profession_type: String,
    #[serde(default)]
    pub max_skill: u32,
    #[serde(default)]
    pub description: LocalizedString,
}

/// PvP content definition.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct PvpRow {
    pub id: String,
    #[serde(default)]
    pub name: LocalizedString,
    /// battleground, arena, world_pvp, object
    #[serde(default)]
    pub pvp_type: String,
    #[serde(default)]
    pub team_size: u32,
    #[serde(default)]
    pub description: LocalizedString,
}

/// Achievement definition.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct AchievementRow {
    pub id: String,
    #[serde(default)]
    pub name: LocalizedString,
    #[serde(default)]
    pub points: u32,
    #[serde(default)]
    pub criteria: Vec<String>,
    #[serde(default)]
    pub description: LocalizedString,
}

/// Mount definition.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct MountRow {
    pub id: String,
    #[serde(default)]
    pub name: LocalizedString,
    /// ground, flying, aquatic
    #[serde(default)]
    pub mount_type: String,
    #[serde(default)]
    pub speed_modifier: f64,
    #[serde(default)]
    pub description: LocalizedString,
}

/// Guild template definition.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct GuildRow {
    pub id: String,
    #[serde(default)]
    pub name: LocalizedString,
    #[serde(default)]
    pub max_members: u32,
    #[serde(default)]
    pub description: LocalizedString,
}

/// Consumable item definition.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct ConsumableRow {
    pub id: String,
    #[serde(default)]
    pub name: LocalizedString,
    /// potion, food, drink, elixir, flask, bandage, scroll, consumable
    #[serde(default)]
    pub consumable_type: String,
    #[serde(default)]
    pub stack_size: u32,
    #[serde(default)]
    pub cooldown: f32,
    #[serde(default)]
    pub description: LocalizedString,
}

/// Currency definition.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct CurrencyRow {
    pub id: String,
    #[serde(default)]
    pub name: LocalizedString,
    #[serde(default)]
    pub max_amount: u32,
    #[serde(default)]
    pub description: LocalizedString,
}

/// Equipment piece definition (armor/weapon with stats).
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct EquipmentRow {
    pub id: String,
    #[serde(default)]
    pub name: LocalizedString,
    #[serde(default)]
    pub slot: String,
    #[serde(default)]
    pub armor_value: i32,
    /// Stat bonuses as (stat_name, value) pairs.
    #[serde(default)]
    pub stats: Vec<(String, f64)>,
    #[serde(default)]
    pub level_requirement: u32,
    #[serde(default)]
    pub rarity: Rarity,
    #[serde(default)]
    pub description: LocalizedString,
}

/// Inventory slot/container definition.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct InventoryRow {
    pub id: String,
    #[serde(default)]
    pub name: LocalizedString,
    #[serde(default)]
    pub slot_type: String,
    #[serde(default)]
    pub capacity: u32,
    #[serde(default)]
    pub description: LocalizedString,
}

/// Display title definition.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct TitleRow {
    pub id: String,
    #[serde(default)]
    pub name: LocalizedString,
    /// prefix or suffix
    #[serde(default)]
    pub style: String,
    #[serde(default)]
    pub source: String,
    #[serde(default)]
    pub description: LocalizedString,
}

/// Trade good / crafting material definition.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct TradeGoodRow {
    pub id: String,
    #[serde(default)]
    pub name: LocalizedString,
    #[serde(default)]
    pub stack_size: u32,
    #[serde(default)]
    pub vendor_price: i32,
    #[serde(default)]
    pub description: LocalizedString,
}

/// Weapon skill proficiency definition.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct WeaponSkillRow {
    pub id: String,
    #[serde(default)]
    pub name: LocalizedString,
    #[serde(default)]
    pub weapon_type: String,
    #[serde(default)]
    pub classes: Vec<String>,
    #[serde(default)]
    pub max_skill: u32,
    #[serde(default)]
    pub description: LocalizedString,
}

/// HashMap indices for O(1) lookups by entity ID.
#[derive(Debug, Clone, Default)]
pub struct DatabaseIndices {
    pub items: HashMap<String, usize>,
    pub npcs: HashMap<String, usize>,
    pub towers: HashMap<String, usize>,
    pub enemies: HashMap<String, usize>,
    pub loot_tables: HashMap<String, usize>,
    pub quests: HashMap<String, usize>,
    pub abilities: HashMap<String, usize>,
    pub zones: HashMap<String, usize>,
    pub auras: HashMap<String, usize>,
    pub class_data: HashMap<String, usize>,
    pub raids: HashMap<String, usize>,
    pub talents: HashMap<String, usize>,
    pub professions: HashMap<String, usize>,
    pub pvp: HashMap<String, usize>,
    pub achievements: HashMap<String, usize>,
    pub mounts: HashMap<String, usize>,
    pub guilds: HashMap<String, usize>,
    pub consumables: HashMap<String, usize>,
    pub currencies: HashMap<String, usize>,
    pub equipment: HashMap<String, usize>,
    pub inventory: HashMap<String, usize>,
    pub titles: HashMap<String, usize>,
    pub trade_goods: HashMap<String, usize>,
    pub weapon_skills: HashMap<String, usize>,
}

impl PartialEq for DatabaseIndices {
    fn eq(&self, _other: &Self) -> bool {
        // Indices are derived from data; two databases with the same data
        // always produce the same indices, so we treat them as equal.
        true
    }
}

/// The complete game database.
#[derive(Resource, Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
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
    /// Ability definitions
    #[serde(default)]
    pub abilities: Vec<AbilityRow>,
    /// Zone definitions
    #[serde(default)]
    pub zones: Vec<ZoneRow>,
    /// Aura definitions
    #[serde(default)]
    pub auras: Vec<AuraRow>,
    /// Class definitions
    #[serde(default)]
    pub class_data: Vec<ClassDataRow>,
    /// Raid definitions
    #[serde(default)]
    pub raids: Vec<RaidRow>,
    /// Talent definitions
    #[serde(default)]
    pub talents: Vec<TalentRow>,
    /// Profession definitions
    #[serde(default)]
    pub professions: Vec<ProfessionRow>,
    /// PvP content definitions
    #[serde(default)]
    pub pvp: Vec<PvpRow>,
    /// Achievement definitions
    #[serde(default)]
    pub achievements: Vec<AchievementRow>,
    /// Mount definitions
    #[serde(default)]
    pub mounts: Vec<MountRow>,
    /// Guild definitions
    #[serde(default)]
    pub guilds: Vec<GuildRow>,
    /// Consumable definitions
    #[serde(default)]
    pub consumables: Vec<ConsumableRow>,
    /// Currency definitions
    #[serde(default)]
    pub currencies: Vec<CurrencyRow>,
    /// Equipment definitions
    #[serde(default)]
    pub equipment: Vec<EquipmentRow>,
    /// Inventory slot definitions
    #[serde(default)]
    pub inventory: Vec<InventoryRow>,
    /// Title definitions
    #[serde(default)]
    pub titles: Vec<TitleRow>,
    /// Trade good definitions
    #[serde(default)]
    pub trade_goods: Vec<TradeGoodRow>,
    /// Weapon skill definitions
    #[serde(default)]
    pub weapon_skills: Vec<WeaponSkillRow>,
    /// Lookup indices (not serialized)
    #[serde(skip)]
    pub indices: DatabaseIndices,
}

impl Database {
    /// Create a new empty database.
    pub fn new() -> Self {
        Self::default()
    }

    /// Builder that populates indices after construction/deserialization.
    pub fn with_indices(mut self) -> Self {
        self.rebuild_indices();
        self
    }

    /// Rebuild all HashMap indices from the current Vec contents.
    pub fn rebuild_indices(&mut self) {
        self.indices = DatabaseIndices::default();
        for (i, row) in self.items.iter().enumerate() {
            self.indices.items.insert(row.id.clone(), i);
        }
        for (i, row) in self.npcs.iter().enumerate() {
            self.indices.npcs.insert(row.id.clone(), i);
        }
        for (i, row) in self.towers.iter().enumerate() {
            self.indices.towers.insert(row.id.clone(), i);
        }
        for (i, row) in self.enemies.iter().enumerate() {
            self.indices.enemies.insert(row.id.clone(), i);
        }
        for (i, row) in self.loot_tables.iter().enumerate() {
            self.indices.loot_tables.insert(row.id.clone(), i);
        }
        for (i, row) in self.quests.iter().enumerate() {
            self.indices.quests.insert(row.id.clone(), i);
        }
        for (i, row) in self.abilities.iter().enumerate() {
            self.indices.abilities.insert(row.id.clone(), i);
        }
        for (i, row) in self.zones.iter().enumerate() {
            self.indices.zones.insert(row.id.clone(), i);
        }
        for (i, row) in self.auras.iter().enumerate() {
            self.indices.auras.insert(row.id.clone(), i);
        }
        for (i, row) in self.class_data.iter().enumerate() {
            self.indices.class_data.insert(row.id.clone(), i);
        }
        for (i, row) in self.raids.iter().enumerate() {
            self.indices.raids.insert(row.id.clone(), i);
        }
        for (i, row) in self.talents.iter().enumerate() {
            self.indices.talents.insert(row.id.clone(), i);
        }
        for (i, row) in self.professions.iter().enumerate() {
            self.indices.professions.insert(row.id.clone(), i);
        }
        for (i, row) in self.pvp.iter().enumerate() {
            self.indices.pvp.insert(row.id.clone(), i);
        }
        for (i, row) in self.achievements.iter().enumerate() {
            self.indices.achievements.insert(row.id.clone(), i);
        }
        for (i, row) in self.mounts.iter().enumerate() {
            self.indices.mounts.insert(row.id.clone(), i);
        }
        for (i, row) in self.guilds.iter().enumerate() {
            self.indices.guilds.insert(row.id.clone(), i);
        }
        for (i, row) in self.consumables.iter().enumerate() {
            self.indices.consumables.insert(row.id.clone(), i);
        }
        for (i, row) in self.currencies.iter().enumerate() {
            self.indices.currencies.insert(row.id.clone(), i);
        }
        for (i, row) in self.equipment.iter().enumerate() {
            self.indices.equipment.insert(row.id.clone(), i);
        }
        for (i, row) in self.inventory.iter().enumerate() {
            self.indices.inventory.insert(row.id.clone(), i);
        }
        for (i, row) in self.titles.iter().enumerate() {
            self.indices.titles.insert(row.id.clone(), i);
        }
        for (i, row) in self.trade_goods.iter().enumerate() {
            self.indices.trade_goods.insert(row.id.clone(), i);
        }
        for (i, row) in self.weapon_skills.iter().enumerate() {
            self.indices.weapon_skills.insert(row.id.clone(), i);
        }
    }

    /// Find an item by ID (O(1) with index, O(n) fallback).
    pub fn find_item(&self, id: &str) -> Option<&ItemRow> {
        if let Some(&idx) = self.indices.items.get(id) {
            self.items.get(idx)
        } else {
            self.items.iter().find(|i| i.id == id)
        }
    }

    /// Find an NPC by ID (O(1) with index, O(n) fallback).
    pub fn find_npc(&self, id: &str) -> Option<&NpcRow> {
        if let Some(&idx) = self.indices.npcs.get(id) {
            self.npcs.get(idx)
        } else {
            self.npcs.iter().find(|n| n.id == id)
        }
    }

    /// Find a tower by ID (O(1) with index, O(n) fallback).
    pub fn find_tower(&self, id: &str) -> Option<&TowerRow> {
        if let Some(&idx) = self.indices.towers.get(id) {
            self.towers.get(idx)
        } else {
            self.towers.iter().find(|t| t.id == id)
        }
    }

    /// Find an enemy by ID (O(1) with index, O(n) fallback).
    pub fn find_enemy(&self, id: &str) -> Option<&EnemyRow> {
        if let Some(&idx) = self.indices.enemies.get(id) {
            self.enemies.get(idx)
        } else {
            self.enemies.iter().find(|e| e.id == id)
        }
    }

    /// Find a loot table by ID (O(1) with index, O(n) fallback).
    pub fn find_loot_table(&self, id: &str) -> Option<&LootTableRow> {
        if let Some(&idx) = self.indices.loot_tables.get(id) {
            self.loot_tables.get(idx)
        } else {
            self.loot_tables.iter().find(|l| l.id == id)
        }
    }

    /// Find a quest by ID (O(1) with index, O(n) fallback).
    pub fn find_quest(&self, id: &str) -> Option<&QuestRow> {
        if let Some(&idx) = self.indices.quests.get(id) {
            self.quests.get(idx)
        } else {
            self.quests.iter().find(|q| q.id == id)
        }
    }

    /// Find an ability by ID (O(1) with index, O(n) fallback).
    pub fn find_ability(&self, id: &str) -> Option<&AbilityRow> {
        if let Some(&idx) = self.indices.abilities.get(id) {
            self.abilities.get(idx)
        } else {
            self.abilities.iter().find(|a| a.id == id)
        }
    }

    /// Find a zone by ID (O(1) with index, O(n) fallback).
    pub fn find_zone(&self, id: &str) -> Option<&ZoneRow> {
        if let Some(&idx) = self.indices.zones.get(id) {
            self.zones.get(idx)
        } else {
            self.zones.iter().find(|z| z.id == id)
        }
    }

    /// Find an aura by ID (O(1) with index, O(n) fallback).
    pub fn find_aura(&self, id: &str) -> Option<&AuraRow> {
        if let Some(&idx) = self.indices.auras.get(id) {
            self.auras.get(idx)
        } else {
            self.auras.iter().find(|a| a.id == id)
        }
    }

    /// Find a class by ID (O(1) with index, O(n) fallback).
    pub fn find_class_data(&self, id: &str) -> Option<&ClassDataRow> {
        if let Some(&idx) = self.indices.class_data.get(id) {
            self.class_data.get(idx)
        } else {
            self.class_data.iter().find(|c| c.id == id)
        }
    }

    /// Find a raid by ID (O(1) with index, O(n) fallback).
    pub fn find_raid(&self, id: &str) -> Option<&RaidRow> {
        if let Some(&idx) = self.indices.raids.get(id) {
            self.raids.get(idx)
        } else {
            self.raids.iter().find(|r| r.id == id)
        }
    }

    /// Find a talent by ID (O(1) with index, O(n) fallback).
    pub fn find_talent(&self, id: &str) -> Option<&TalentRow> {
        if let Some(&idx) = self.indices.talents.get(id) {
            self.talents.get(idx)
        } else {
            self.talents.iter().find(|t| t.id == id)
        }
    }

    /// Find a profession by ID (O(1) with index, O(n) fallback).
    pub fn find_profession(&self, id: &str) -> Option<&ProfessionRow> {
        if let Some(&idx) = self.indices.professions.get(id) {
            self.professions.get(idx)
        } else {
            self.professions.iter().find(|p| p.id == id)
        }
    }

    /// Find a PvP entry by ID (O(1) with index, O(n) fallback).
    pub fn find_pvp(&self, id: &str) -> Option<&PvpRow> {
        if let Some(&idx) = self.indices.pvp.get(id) {
            self.pvp.get(idx)
        } else {
            self.pvp.iter().find(|p| p.id == id)
        }
    }

    /// Find an achievement by ID (O(1) with index, O(n) fallback).
    pub fn find_achievement(&self, id: &str) -> Option<&AchievementRow> {
        if let Some(&idx) = self.indices.achievements.get(id) {
            self.achievements.get(idx)
        } else {
            self.achievements.iter().find(|a| a.id == id)
        }
    }

    /// Find a mount by ID (O(1) with index, O(n) fallback).
    pub fn find_mount(&self, id: &str) -> Option<&MountRow> {
        if let Some(&idx) = self.indices.mounts.get(id) {
            self.mounts.get(idx)
        } else {
            self.mounts.iter().find(|m| m.id == id)
        }
    }

    /// Find a guild by ID (O(1) with index, O(n) fallback).
    pub fn find_guild(&self, id: &str) -> Option<&GuildRow> {
        if let Some(&idx) = self.indices.guilds.get(id) {
            self.guilds.get(idx)
        } else {
            self.guilds.iter().find(|g| g.id == id)
        }
    }

    /// Find a consumable by ID (O(1) with index, O(n) fallback).
    pub fn find_consumable(&self, id: &str) -> Option<&ConsumableRow> {
        if let Some(&idx) = self.indices.consumables.get(id) {
            self.consumables.get(idx)
        } else {
            self.consumables.iter().find(|c| c.id == id)
        }
    }

    /// Find a currency by ID (O(1) with index, O(n) fallback).
    pub fn find_currency(&self, id: &str) -> Option<&CurrencyRow> {
        if let Some(&idx) = self.indices.currencies.get(id) {
            self.currencies.get(idx)
        } else {
            self.currencies.iter().find(|c| c.id == id)
        }
    }

    /// Find equipment by ID (O(1) with index, O(n) fallback).
    pub fn find_equipment(&self, id: &str) -> Option<&EquipmentRow> {
        if let Some(&idx) = self.indices.equipment.get(id) {
            self.equipment.get(idx)
        } else {
            self.equipment.iter().find(|e| e.id == id)
        }
    }

    /// Find an inventory slot by ID (O(1) with index, O(n) fallback).
    pub fn find_inventory(&self, id: &str) -> Option<&InventoryRow> {
        if let Some(&idx) = self.indices.inventory.get(id) {
            self.inventory.get(idx)
        } else {
            self.inventory.iter().find(|i| i.id == id)
        }
    }

    /// Find a title by ID (O(1) with index, O(n) fallback).
    pub fn find_title(&self, id: &str) -> Option<&TitleRow> {
        if let Some(&idx) = self.indices.titles.get(id) {
            self.titles.get(idx)
        } else {
            self.titles.iter().find(|t| t.id == id)
        }
    }

    /// Find a trade good by ID (O(1) with index, O(n) fallback).
    pub fn find_trade_good(&self, id: &str) -> Option<&TradeGoodRow> {
        if let Some(&idx) = self.indices.trade_goods.get(id) {
            self.trade_goods.get(idx)
        } else {
            self.trade_goods.iter().find(|t| t.id == id)
        }
    }

    /// Find a weapon skill by ID (O(1) with index, O(n) fallback).
    pub fn find_weapon_skill(&self, id: &str) -> Option<&WeaponSkillRow> {
        if let Some(&idx) = self.indices.weapon_skills.get(id) {
            self.weapon_skills.get(idx)
        } else {
            self.weapon_skills.iter().find(|w| w.id == id)
        }
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

    // --- Index tests ---

    #[test]
    fn test_index_lookup_works() {
        let mut db = Database::new();
        db.items
            .push(ItemRow::new("sword_01", "Iron Sword").with_type(ItemType::Weapon));
        db.npcs.push(NpcRow::new("merchant_01", "Merchant"));
        db.enemies.push(EnemyRow::new("goblin_01", "Goblin"));
        db.quests.push(QuestRow::new("quest_01", "First Quest"));
        db.towers.push(TowerRow::new("tower_01", "Arrow Tower"));
        db.loot_tables.push(LootTableRow::new("loot_01"));
        db.abilities.push(AbilityRow {
            id: "fireball".to_string(),
            ..Default::default()
        });
        db.zones.push(ZoneRow {
            id: "zone_01".to_string(),
            ..Default::default()
        });
        db.rebuild_indices();

        assert_eq!(db.find_item("sword_01").unwrap().id, "sword_01");
        assert_eq!(db.find_npc("merchant_01").unwrap().id, "merchant_01");
        assert_eq!(db.find_enemy("goblin_01").unwrap().id, "goblin_01");
        assert_eq!(db.find_quest("quest_01").unwrap().id, "quest_01");
        assert_eq!(db.find_tower("tower_01").unwrap().id, "tower_01");
        assert_eq!(db.find_loot_table("loot_01").unwrap().id, "loot_01");
        assert_eq!(db.find_ability("fireball").unwrap().id, "fireball");
        assert_eq!(db.find_zone("zone_01").unwrap().id, "zone_01");
    }

    #[test]
    fn test_index_missing_returns_none() {
        let mut db = Database::new();
        db.items.push(ItemRow::new("sword_01", "Iron Sword"));
        db.rebuild_indices();

        assert!(db.find_item("nonexistent").is_none());
        assert!(db.find_npc("nonexistent").is_none());
        assert!(db.find_enemy("nonexistent").is_none());
        assert!(db.find_quest("nonexistent").is_none());
    }

    #[test]
    fn test_rebuild_after_push() {
        let mut db = Database::new();
        db.items.push(ItemRow::new("sword_01", "Iron Sword"));
        db.rebuild_indices();

        assert!(db.find_item("sword_01").is_some());
        assert!(db.find_item("shield_01").is_none());

        // Push a new item and rebuild
        db.items.push(ItemRow::new("shield_01", "Wooden Shield"));
        db.rebuild_indices();

        assert!(db.find_item("shield_01").is_some());
        assert_eq!(db.indices.items.len(), 2);
    }

    #[test]
    fn test_serialization_roundtrip_preserves_data() {
        let mut db = Database::new();
        db.items
            .push(ItemRow::new("sword_01", "Iron Sword").with_type(ItemType::Weapon));
        db.npcs.push(NpcRow::new("merchant_01", "Merchant"));
        db.enemies.push(EnemyRow::new("goblin_01", "Goblin"));
        db.quests.push(QuestRow::new("quest_01", "First Quest"));
        db.rebuild_indices();

        // Serialize (indices are skipped)
        let json = serde_json::to_string_pretty(&db).unwrap();

        // Deserialize and rebuild indices
        let parsed: Database = serde_json::from_str::<Database>(&json)
            .unwrap()
            .with_indices();

        // Data preserved
        assert_eq!(db.items.len(), parsed.items.len());
        assert_eq!(db.npcs.len(), parsed.npcs.len());
        assert_eq!(db.enemies.len(), parsed.enemies.len());
        assert_eq!(db.quests.len(), parsed.quests.len());

        // Indices work after roundtrip
        assert_eq!(parsed.find_item("sword_01").unwrap().id, "sword_01");
        assert_eq!(parsed.find_npc("merchant_01").unwrap().id, "merchant_01");
        assert_eq!(parsed.find_enemy("goblin_01").unwrap().id, "goblin_01");
        assert_eq!(parsed.find_quest("quest_01").unwrap().id, "quest_01");

        // JSON doesn't contain index fields
        assert!(!json.contains("indices"));
    }

    #[test]
    fn test_new_row_types_serialize() {
        let rows: Vec<Box<dyn std::fmt::Debug>> = vec![
            Box::new(AuraRow {
                id: "blessing".into(),
                aura_type: "buff".into(),
                duration: 1800.0,
                max_stacks: 1,
                ..Default::default()
            }),
            Box::new(ClassDataRow {
                id: "warrior".into(),
                role: "tank".into(),
                resource_type: "rage".into(),
                abilities: vec!["charge".into()],
                talent_trees: vec!["arms".into(), "fury".into()],
                ..Default::default()
            }),
            Box::new(RaidRow {
                id: "molten_core".into(),
                zone_id: "mc_zone".into(),
                size: 40,
                difficulty: "normal".into(),
                bosses: vec!["ragnaros".into()],
                ..Default::default()
            }),
            Box::new(TalentRow {
                id: "mortal_strike".into(),
                class_id: "warrior".into(),
                tree: "arms".into(),
                tier: 6,
                column: 1,
                max_rank: 1,
                prerequisite_talent: Some("deep_wounds".into()),
                ..Default::default()
            }),
            Box::new(ProfessionRow {
                id: "blacksmithing".into(),
                profession_type: "crafting".into(),
                max_skill: 300,
                ..Default::default()
            }),
            Box::new(PvpRow {
                id: "warsong_gulch".into(),
                pvp_type: "battleground".into(),
                team_size: 10,
                ..Default::default()
            }),
            Box::new(AchievementRow {
                id: "explore_world".into(),
                points: 50,
                criteria: vec!["visit_all_zones".into()],
                ..Default::default()
            }),
            Box::new(MountRow {
                id: "epic_horse".into(),
                mount_type: "ground".into(),
                speed_modifier: 2.0,
                ..Default::default()
            }),
            Box::new(GuildRow {
                id: "default_guild".into(),
                max_members: 500,
                ..Default::default()
            }),
        ];

        // Roundtrip each through JSON
        let aura = AuraRow {
            id: "blessing".into(),
            aura_type: "buff".into(),
            duration: 1800.0,
            max_stacks: 1,
            ..Default::default()
        };
        let json = serde_json::to_string(&aura).unwrap();
        let parsed: AuraRow = serde_json::from_str(&json).unwrap();
        assert_eq!(aura, parsed);

        let class = ClassDataRow {
            id: "warrior".into(),
            role: "tank".into(),
            resource_type: "rage".into(),
            abilities: vec!["charge".into()],
            talent_trees: vec!["arms".into()],
            ..Default::default()
        };
        let json = serde_json::to_string(&class).unwrap();
        let parsed: ClassDataRow = serde_json::from_str(&json).unwrap();
        assert_eq!(class, parsed);

        let raid = RaidRow {
            id: "mc".into(),
            size: 40,
            bosses: vec!["rag".into()],
            ..Default::default()
        };
        let json = serde_json::to_string(&raid).unwrap();
        let parsed: RaidRow = serde_json::from_str(&json).unwrap();
        assert_eq!(raid, parsed);

        // Verify all rows are constructible (type check)
        assert_eq!(rows.len(), 9);
    }

    #[test]
    fn test_new_row_find_methods() {
        let mut db = Database::new();

        db.auras.push(AuraRow {
            id: "blessing".into(),
            aura_type: "buff".into(),
            ..Default::default()
        });
        db.class_data.push(ClassDataRow {
            id: "warrior".into(),
            role: "tank".into(),
            ..Default::default()
        });
        db.raids.push(RaidRow {
            id: "molten_core".into(),
            size: 40,
            ..Default::default()
        });
        db.talents.push(TalentRow {
            id: "mortal_strike".into(),
            class_id: "warrior".into(),
            tree: "arms".into(),
            ..Default::default()
        });
        db.professions.push(ProfessionRow {
            id: "blacksmithing".into(),
            profession_type: "crafting".into(),
            ..Default::default()
        });
        db.pvp.push(PvpRow {
            id: "warsong_gulch".into(),
            pvp_type: "battleground".into(),
            ..Default::default()
        });
        db.achievements.push(AchievementRow {
            id: "explore_world".into(),
            points: 50,
            ..Default::default()
        });
        db.mounts.push(MountRow {
            id: "epic_horse".into(),
            mount_type: "ground".into(),
            ..Default::default()
        });
        db.guilds.push(GuildRow {
            id: "default_guild".into(),
            max_members: 500,
            ..Default::default()
        });

        db.rebuild_indices();

        assert_eq!(db.find_aura("blessing").unwrap().aura_type, "buff");
        assert_eq!(db.find_class_data("warrior").unwrap().role, "tank");
        assert_eq!(db.find_raid("molten_core").unwrap().size, 40);
        assert_eq!(db.find_talent("mortal_strike").unwrap().class_id, "warrior");
        assert_eq!(
            db.find_profession("blacksmithing").unwrap().profession_type,
            "crafting"
        );
        assert_eq!(
            db.find_pvp("warsong_gulch").unwrap().pvp_type,
            "battleground"
        );
        assert_eq!(db.find_achievement("explore_world").unwrap().points, 50);
        assert_eq!(db.find_mount("epic_horse").unwrap().mount_type, "ground");
        assert_eq!(db.find_guild("default_guild").unwrap().max_members, 500);

        // Missing returns None
        assert!(db.find_aura("nonexistent").is_none());
        assert!(db.find_class_data("nonexistent").is_none());
        assert!(db.find_raid("nonexistent").is_none());
        assert!(db.find_talent("nonexistent").is_none());
        assert!(db.find_profession("nonexistent").is_none());
        assert!(db.find_pvp("nonexistent").is_none());
        assert!(db.find_achievement("nonexistent").is_none());
        assert!(db.find_mount("nonexistent").is_none());
        assert!(db.find_guild("nonexistent").is_none());
    }
}
