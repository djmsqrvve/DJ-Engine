//! Character progression systems for DJ Engine.
//!
//! Consumes the remaining Helix Row types:
//! - [`InventoryRow`] → bag capacity management
//! - [`TitleRow`] → display title prefix/suffix
//! - [`WeaponSkillRow`] → weapon proficiency gating

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::data::database::Database;

// ---------------------------------------------------------------------------
// Titles
// ---------------------------------------------------------------------------

/// Player's active title.
#[derive(Resource, Default, Debug, Clone, Serialize, Deserialize, Reflect)]
#[reflect(Resource)]
pub struct PlayerTitle {
    /// Currently equipped title ID (from TitleRow).
    pub active_title_id: Option<String>,
    /// Earned title IDs.
    pub earned_titles: Vec<String>,
}

impl PlayerTitle {
    /// Grant a title to the player.
    pub fn earn(&mut self, title_id: impl Into<String>) {
        let id = title_id.into();
        if !self.earned_titles.contains(&id) {
            self.earned_titles.push(id);
        }
    }

    /// Set the active display title. Returns false if not earned.
    pub fn equip(&mut self, title_id: &str) -> bool {
        if self.earned_titles.iter().any(|t| t == title_id) {
            self.active_title_id = Some(title_id.to_string());
            true
        } else {
            false
        }
    }

    /// Clear the active title.
    pub fn unequip(&mut self) {
        self.active_title_id = None;
    }

    /// Get the formatted display name with title applied.
    pub fn format_name(&self, base_name: &str, database: Option<&Database>) -> String {
        let Some(title_id) = &self.active_title_id else {
            return base_name.to_string();
        };

        let Some(db) = database else {
            return base_name.to_string();
        };

        let Some(title) = db.find_title(title_id) else {
            return base_name.to_string();
        };

        let title_text = title
            .name
            .get("en")
            .cloned()
            .unwrap_or_else(|| title_id.clone());

        match title.style.as_str() {
            "prefix" => format!("{} {}", title_text, base_name),
            "suffix" => format!("{}, {}", base_name, title_text),
            _ => format!("{} {}", title_text, base_name),
        }
    }
}

/// Events for title changes.
#[derive(Message, Debug, Clone, PartialEq)]
pub enum TitleEvent {
    Earned { title_id: String },
    Equipped { title_id: String },
    Unequipped,
}

// ---------------------------------------------------------------------------
// Weapon Skills
// ---------------------------------------------------------------------------

/// Player's weapon skill proficiency levels.
#[derive(Resource, Default, Debug, Clone, Serialize, Deserialize, Reflect)]
#[reflect(Resource)]
pub struct WeaponProficiencies {
    /// Skill level per weapon type (weapon_skill_id -> current_skill).
    #[reflect(ignore)]
    pub skills: HashMap<String, u32>,
}

impl WeaponProficiencies {
    /// Get the skill level for a weapon type.
    pub fn skill_level(&self, weapon_skill_id: &str) -> u32 {
        self.skills.get(weapon_skill_id).copied().unwrap_or(0)
    }

    /// Increase skill from use. Returns new level. Caps at max_skill from Database.
    pub fn gain_skill(
        &mut self,
        weapon_skill_id: &str,
        amount: u32,
        database: Option<&Database>,
    ) -> u32 {
        let max = database
            .and_then(|db| db.find_weapon_skill(weapon_skill_id))
            .map(|ws| ws.max_skill)
            .unwrap_or(300);

        let current = self.skills.entry(weapon_skill_id.to_string()).or_insert(0);
        *current = (*current + amount).min(max);
        *current
    }

    /// Check if a class can use a weapon type (from WeaponSkillRow.classes).
    pub fn can_class_use_weapon(
        &self,
        weapon_skill_id: &str,
        class_id: &str,
        database: Option<&Database>,
    ) -> bool {
        let Some(db) = database else {
            return true; // permissive without data
        };

        let Some(ws) = db.find_weapon_skill(weapon_skill_id) else {
            return true;
        };

        ws.classes.is_empty() || ws.classes.iter().any(|c| c == class_id)
    }
}

/// Events for weapon skill changes.
#[derive(Message, Debug, Clone, PartialEq)]
pub struct WeaponSkillGainEvent {
    pub weapon_skill_id: String,
    pub new_level: u32,
    pub amount: u32,
}

// ---------------------------------------------------------------------------
// Inventory Bag Management
// ---------------------------------------------------------------------------

/// Initialize inventory capacity from InventoryRow definitions in Database.
pub fn initialize_inventory_from_database(
    database: Option<Res<Database>>,
    mut inventory: ResMut<crate::inventory::Inventory>,
) {
    let Some(db) = database else {
        return;
    };

    // Find the "backpack" or first inventory definition
    let bag = db
        .find_inventory("backpack")
        .or_else(|| db.inventory.first());

    if let Some(bag) = bag {
        let new_capacity = bag.capacity as usize;
        if new_capacity > 0 && new_capacity != inventory.capacity {
            info!(
                "Character: resizing inventory to {} slots (from '{}')",
                new_capacity, bag.id
            );
            inventory.capacity = new_capacity;
            inventory.slots.resize(new_capacity, None);
        }
    }
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct CharacterPlugin;

impl Plugin for CharacterPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerTitle>()
            .init_resource::<WeaponProficiencies>()
            .register_type::<PlayerTitle>()
            .register_type::<WeaponProficiencies>()
            .add_message::<TitleEvent>()
            .add_message::<WeaponSkillGainEvent>()
            .add_systems(Startup, initialize_inventory_from_database);

        use crate::contracts::{AppContractExt, ContractEntry, PluginContract};
        app.register_contract(PluginContract {
            name: "CharacterPlugin".into(),
            description: "Titles, weapon skills, inventory bags — from Database Row types".into(),
            resources: vec![
                ContractEntry::of::<PlayerTitle>("Active title and earned titles"),
                ContractEntry::of::<WeaponProficiencies>("Weapon skill levels"),
            ],
            components: vec![],
            events: vec![
                ContractEntry::of::<TitleEvent>("Title change events"),
                ContractEntry::of::<WeaponSkillGainEvent>("Weapon skill gain events"),
            ],
            system_sets: vec![],
        });
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::database::{TitleRow, WeaponSkillRow};

    #[test]
    fn test_earn_and_equip_title() {
        let mut pt = PlayerTitle::default();
        assert!(pt.active_title_id.is_none());

        pt.earn("champion");
        assert_eq!(pt.earned_titles.len(), 1);

        assert!(pt.equip("champion"));
        assert_eq!(pt.active_title_id.as_deref(), Some("champion"));

        // Can't equip unearned title
        assert!(!pt.equip("warlord"));
    }

    #[test]
    fn test_title_format_name_prefix() {
        let mut db = Database::default();
        db.titles.push(TitleRow {
            id: "champion".into(),
            name: [("en".into(), "Champion".into())].into(),
            style: "prefix".into(),
            ..default()
        });

        let mut pt = PlayerTitle::default();
        pt.earn("champion");
        pt.equip("champion");

        let formatted = pt.format_name("PlayerOne", Some(&db));
        assert_eq!(formatted, "Champion PlayerOne");
    }

    #[test]
    fn test_title_format_name_suffix() {
        let mut db = Database::default();
        db.titles.push(TitleRow {
            id: "the_brave".into(),
            name: [("en".into(), "the Brave".into())].into(),
            style: "suffix".into(),
            ..default()
        });

        let mut pt = PlayerTitle::default();
        pt.earn("the_brave");
        pt.equip("the_brave");

        let formatted = pt.format_name("PlayerOne", Some(&db));
        assert_eq!(formatted, "PlayerOne, the Brave");
    }

    #[test]
    fn test_weapon_skill_gain_caps_at_max() {
        let mut db = Database::default();
        db.weapon_skills.push(WeaponSkillRow {
            id: "swords".into(),
            max_skill: 300,
            classes: vec!["warrior".into(), "paladin".into()],
            ..default()
        });

        let mut wp = WeaponProficiencies::default();
        let level = wp.gain_skill("swords", 50, Some(&db));
        assert_eq!(level, 50);

        let level = wp.gain_skill("swords", 400, Some(&db));
        assert_eq!(level, 300); // capped
    }

    #[test]
    fn test_weapon_skill_class_restriction() {
        let mut db = Database::default();
        db.weapon_skills.push(WeaponSkillRow {
            id: "swords".into(),
            classes: vec!["warrior".into(), "paladin".into()],
            ..default()
        });

        let wp = WeaponProficiencies::default();
        assert!(wp.can_class_use_weapon("swords", "warrior", Some(&db)));
        assert!(!wp.can_class_use_weapon("swords", "mage", Some(&db)));
    }

    #[test]
    fn test_weapon_skill_empty_classes_allows_all() {
        let mut db = Database::default();
        db.weapon_skills.push(WeaponSkillRow {
            id: "daggers".into(),
            classes: vec![], // no restriction
            ..default()
        });

        let wp = WeaponProficiencies::default();
        assert!(wp.can_class_use_weapon("daggers", "mage", Some(&db)));
        assert!(wp.can_class_use_weapon("daggers", "warrior", Some(&db)));
    }

    #[test]
    fn test_inventory_resize_from_database() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(crate::inventory::Inventory::new(10));
        app.add_systems(Startup, initialize_inventory_from_database);

        let mut db = Database::default();
        use crate::data::database::InventoryRow;
        db.inventory.push(InventoryRow {
            id: "backpack".into(),
            capacity: 30,
            ..default()
        });
        app.insert_resource(db);

        app.update();

        let inv = app.world().resource::<crate::inventory::Inventory>();
        assert_eq!(inv.capacity, 30);
        assert_eq!(inv.slots.len(), 30);
    }
}
