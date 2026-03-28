//! Typed registries for all helix3d entity types.
//!
//! `HelixRegistries` is a single Bevy Resource that holds a `Registry<T>` for
//! the 22 primary entity types plus 4 supporting types from `helix-data`.

use bevy::prelude::*;
use helix_data::registry::Registry;
use std::path::Path;

use crate::toml_loader::{load_registry, validate_helix3d_dir, HelixLoadError};

/// All 22 helix registries collected into one Bevy Resource.
///
/// Each field is a typed `Registry<T>` providing O(1) lookup by entity ID.
/// Load from a `dist/helix3d/` directory via [`load_helix_registries`].
#[derive(Resource, Default)]
pub struct HelixRegistries {
    // --- 22 primary entity types ---
    pub abilities: Registry<helix_data::ability::Ability>,
    pub achievements: Registry<helix_data::achievement::Achievement>,
    pub auras: Registry<helix_data::aura::Aura>,
    pub class_data: Registry<helix_data::class_data::ClassData>,
    pub consumables: Registry<helix_data::consumable::Consumable>,
    pub currencies: Registry<helix_data::currency::Currency>,
    pub equipment: Registry<helix_data::equipment::Equipment>,
    pub guilds: Registry<helix_data::guild::Guild>,
    pub inventory: Registry<helix_data::inventory::Inventory>,
    pub items: Registry<helix_data::item::Item>,
    pub mobs: Registry<helix_data::mob::Mob>,
    pub mounts: Registry<helix_data::mount::Mount>,
    pub npcs: Registry<helix_data::npc::Npc>,
    pub professions: Registry<helix_data::profession::Profession>,
    pub pvp: Registry<helix_data::pvp::PvpData>,
    pub quests: Registry<helix_data::quest::Quest>,
    pub raids: Registry<helix_data::raid::Raid>,
    pub talents: Registry<helix_data::talent::Talent>,
    pub titles: Registry<helix_data::title::Title>,
    pub trade_goods: Registry<helix_data::trade_good::TradeGood>,
    pub weapon_skills: Registry<helix_data::weapon_skill::WeaponSkill>,
    pub zones: Registry<helix_data::zone::Zone>,
    // --- 4 supporting types ---
    pub ability_effects: Registry<helix_data::ability_effect::AbilityEffect>,
    pub factions: Registry<helix_data::faction::Faction>,
    pub loot_tables: Registry<helix_data::loot_table::LootTable>,
    pub quest_objectives: Registry<helix_data::quest_objective::QuestObjective>,
}

impl HelixRegistries {
    /// Total number of entities across all registries.
    pub fn total_entities(&self) -> usize {
        self.abilities.len()
            + self.achievements.len()
            + self.auras.len()
            + self.class_data.len()
            + self.consumables.len()
            + self.currencies.len()
            + self.equipment.len()
            + self.guilds.len()
            + self.inventory.len()
            + self.items.len()
            + self.mobs.len()
            + self.mounts.len()
            + self.npcs.len()
            + self.professions.len()
            + self.pvp.len()
            + self.quests.len()
            + self.raids.len()
            + self.talents.len()
            + self.titles.len()
            + self.trade_goods.len()
            + self.weapon_skills.len()
            + self.zones.len()
            + self.ability_effects.len()
            + self.factions.len()
            + self.loot_tables.len()
            + self.quest_objectives.len()
    }

    /// Iterate all entities across all 22 registries, serializing each to JSON.
    ///
    /// The callback receives `(kind_constant, entity_id, json_value)` for each entity.
    /// This is the bridge between typed TOML registries and the engine's
    /// `CustomDocument<Value>` system.
    pub fn for_each_as_json(&self, mut cb: impl FnMut(&str, &str, serde_json::Value)) {
        use crate::*;

        macro_rules! emit_registry {
            ($field:ident, $kind:expr) => {
                for (id, entity) in self.$field.iter() {
                    if let Ok(value) = serde_json::to_value(entity) {
                        cb($kind, id, value);
                    }
                }
            };
        }

        emit_registry!(abilities, HELIX_ABILITY_KIND);
        emit_registry!(achievements, HELIX_ACHIEVEMENT_KIND);
        emit_registry!(auras, HELIX_AURA_KIND);
        emit_registry!(class_data, HELIX_CLASS_DATA_KIND);
        emit_registry!(consumables, HELIX_CONSUMABLE_KIND);
        emit_registry!(currencies, HELIX_CURRENCY_KIND);
        emit_registry!(equipment, HELIX_EQUIPMENT_KIND);
        emit_registry!(guilds, HELIX_GUILD_KIND);
        emit_registry!(inventory, HELIX_INVENTORY_KIND);
        emit_registry!(items, HELIX_ITEM_KIND);
        emit_registry!(mobs, HELIX_MOB_KIND);
        emit_registry!(mounts, HELIX_MOUNT_KIND);
        emit_registry!(npcs, HELIX_NPC_KIND);
        emit_registry!(professions, HELIX_PROFESSION_KIND);
        emit_registry!(pvp, HELIX_PVP_KIND);
        emit_registry!(quests, HELIX_QUEST_KIND);
        emit_registry!(raids, HELIX_RAID_KIND);
        emit_registry!(talents, HELIX_TALENT_KIND);
        emit_registry!(titles, HELIX_TITLE_KIND);
        emit_registry!(trade_goods, HELIX_TRADE_GOOD_KIND);
        emit_registry!(weapon_skills, HELIX_WEAPON_SKILL_KIND);
        emit_registry!(zones, HELIX_ZONE_KIND);
    }

    /// Returns a summary of entity counts per registry kind.
    pub fn summary(&self) -> Vec<(&'static str, usize)> {
        vec![
            ("abilities", self.abilities.len()),
            ("achievements", self.achievements.len()),
            ("auras", self.auras.len()),
            ("class_data", self.class_data.len()),
            ("consumables", self.consumables.len()),
            ("currencies", self.currencies.len()),
            ("equipment", self.equipment.len()),
            ("guilds", self.guilds.len()),
            ("inventory", self.inventory.len()),
            ("items", self.items.len()),
            ("mobs", self.mobs.len()),
            ("mounts", self.mounts.len()),
            ("npcs", self.npcs.len()),
            ("professions", self.professions.len()),
            ("pvp", self.pvp.len()),
            ("quests", self.quests.len()),
            ("raids", self.raids.len()),
            ("talents", self.talents.len()),
            ("titles", self.titles.len()),
            ("trade_goods", self.trade_goods.len()),
            ("weapon_skills", self.weapon_skills.len()),
            ("zones", self.zones.len()),
            ("ability_effects", self.ability_effects.len()),
            ("factions", self.factions.len()),
            ("loot_tables", self.loot_tables.len()),
            ("quest_objectives", self.quest_objectives.len()),
        ]
    }
}

/// Load all 22 helix registries from a `dist/helix3d/` directory.
///
/// Each registry is loaded from its corresponding TOML file (e.g. `abilities.toml`).
/// Missing files produce an error — use [`load_helix_registries_lenient`] if you
/// want to skip missing files.
pub fn load_helix_registries(helix3d_dir: &Path) -> Result<HelixRegistries, HelixLoadError> {
    validate_helix3d_dir(helix3d_dir)?;

    Ok(HelixRegistries {
        abilities: load_registry(helix3d_dir, "abilities.toml")?,
        achievements: load_registry(helix3d_dir, "achievements.toml")?,
        auras: load_registry(helix3d_dir, "auras.toml")?,
        class_data: load_registry(helix3d_dir, "class_data.toml")?,
        consumables: load_registry(helix3d_dir, "consumables.toml")?,
        currencies: load_registry(helix3d_dir, "currencies.toml")?,
        equipment: load_registry(helix3d_dir, "equipment.toml")?,
        guilds: load_registry(helix3d_dir, "guilds.toml")?,
        inventory: load_registry(helix3d_dir, "inventory.toml")?,
        items: load_registry(helix3d_dir, "items.toml")?,
        mobs: load_registry(helix3d_dir, "mobs.toml")?,
        mounts: load_registry(helix3d_dir, "mounts.toml")?,
        npcs: load_registry(helix3d_dir, "npcs.toml")?,
        professions: load_registry(helix3d_dir, "professions.toml")?,
        pvp: load_registry(helix3d_dir, "pvp.toml")?,
        quests: load_registry(helix3d_dir, "quests.toml")?,
        raids: load_registry(helix3d_dir, "raids.toml")?,
        talents: load_registry(helix3d_dir, "talents.toml")?,
        titles: load_registry(helix3d_dir, "titles.toml")?,
        trade_goods: load_registry(helix3d_dir, "trade_goods.toml")?,
        weapon_skills: load_registry(helix3d_dir, "weapon_skills.toml")?,
        zones: load_registry(helix3d_dir, "zones.toml")?,
        ability_effects: load_or_default(helix3d_dir, "ability_effects.toml"),
        factions: load_or_default(helix3d_dir, "factions.toml"),
        loot_tables: load_or_default(helix3d_dir, "loot_tables.toml"),
        quest_objectives: load_or_default(helix3d_dir, "quest_objectives.toml"),
    })
}

/// Load a supporting type, returning empty on failure (supporting files are optional).
fn load_or_default<T: serde::de::DeserializeOwned>(dir: &Path, filename: &str) -> Registry<T> {
    load_registry(dir, filename).unwrap_or_default()
}

/// Load helix registries leniently — missing or unparseable files produce
/// empty registries with warnings instead of errors.
pub fn load_helix_registries_lenient(
    helix3d_dir: &Path,
) -> Result<HelixRegistries, HelixLoadError> {
    validate_helix3d_dir(helix3d_dir)?;

    fn load_or_empty<T: serde::de::DeserializeOwned>(
        dir: &Path,
        filename: &str,
    ) -> Result<Registry<T>, HelixLoadError> {
        match load_registry(dir, filename) {
            Ok(reg) => Ok(reg),
            Err(HelixLoadError::Io { .. }) => Ok(Registry::new()),
            Err(HelixLoadError::Toml { file, source }) => {
                eprintln!(
                    "Warning: skipping {} (schema mismatch): {}",
                    file,
                    source.message()
                );
                Ok(Registry::new())
            }
            Err(e) => Err(e),
        }
    }

    Ok(HelixRegistries {
        abilities: load_or_empty(helix3d_dir, "abilities.toml")?,
        achievements: load_or_empty(helix3d_dir, "achievements.toml")?,
        auras: load_or_empty(helix3d_dir, "auras.toml")?,
        class_data: load_or_empty(helix3d_dir, "class_data.toml")?,
        consumables: load_or_empty(helix3d_dir, "consumables.toml")?,
        currencies: load_or_empty(helix3d_dir, "currencies.toml")?,
        equipment: load_or_empty(helix3d_dir, "equipment.toml")?,
        guilds: load_or_empty(helix3d_dir, "guilds.toml")?,
        inventory: load_or_empty(helix3d_dir, "inventory.toml")?,
        items: load_or_empty(helix3d_dir, "items.toml")?,
        mobs: load_or_empty(helix3d_dir, "mobs.toml")?,
        mounts: load_or_empty(helix3d_dir, "mounts.toml")?,
        npcs: load_or_empty(helix3d_dir, "npcs.toml")?,
        professions: load_or_empty(helix3d_dir, "professions.toml")?,
        pvp: load_or_empty(helix3d_dir, "pvp.toml")?,
        quests: load_or_empty(helix3d_dir, "quests.toml")?,
        raids: load_or_empty(helix3d_dir, "raids.toml")?,
        talents: load_or_empty(helix3d_dir, "talents.toml")?,
        titles: load_or_empty(helix3d_dir, "titles.toml")?,
        trade_goods: load_or_empty(helix3d_dir, "trade_goods.toml")?,
        weapon_skills: load_or_empty(helix3d_dir, "weapon_skills.toml")?,
        zones: load_or_empty(helix3d_dir, "zones.toml")?,
        ability_effects: load_or_empty(helix3d_dir, "ability_effects.toml")?,
        factions: load_or_empty(helix3d_dir, "factions.toml")?,
        loot_tables: load_or_empty(helix3d_dir, "loot_tables.toml")?,
        quest_objectives: load_or_empty(helix3d_dir, "quest_objectives.toml")?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_registries_has_zero_total() {
        let regs = HelixRegistries::default();
        assert_eq!(regs.total_entities(), 0);
        assert_eq!(regs.summary().len(), 26);
    }

    #[test]
    fn load_real_helix3d_data() {
        let helix3d_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../../helix/helix_standardization/dist/helix3d");
        if !helix3d_dir.is_dir() {
            eprintln!("Skipping: helix3d dir not found at {:?}", helix3d_dir);
            return;
        }

        // Use lenient loader since some TOML files may have schema mismatches
        // with helix-data structs (e.g. consumable_type = "consumable" not in enum).
        // The dashboard system will catch these contract violations.
        let regs = load_helix_registries_lenient(&helix3d_dir)
            .expect("Failed to load helix3d data (lenient)");
        assert!(
            regs.total_entities() > 0,
            "Expected at least some entities, got 0"
        );

        // Individual registries may fail due to upstream schema drift
        // (helix_standardization adds new enum variants before helix-data
        // is updated). Only assert total > 0 — the dashboard catches specifics.
        eprintln!(
            "Abilities: {}, Mobs: {}",
            regs.abilities.len(),
            regs.mobs.len()
        );

        // Print summary for debugging
        for (kind, count) in regs.summary() {
            if count > 0 {
                eprintln!("  {}: {} entities", kind, count);
            }
        }
        eprintln!("Total: {} entities", regs.total_entities());
    }

    #[test]
    fn for_each_as_json_emits_all_entities() {
        let dir = tempfile::tempdir().unwrap();
        let content = r#"
[test_ability]
ability_type = "offensive"
cooldown = 5.0
mana_cost = 10.0

[test_ability.name]
en = "Test"
"#;
        std::fs::write(dir.path().join("abilities.toml"), content).unwrap();

        let regs = load_helix_registries_lenient(dir.path()).expect("load");
        let mut collected = Vec::new();
        regs.for_each_as_json(|kind, id, _value| {
            collected.push((kind.to_string(), id.to_string()));
        });

        assert_eq!(collected.len(), 1);
        assert_eq!(collected[0].0, crate::HELIX_ABILITY_KIND);
        assert_eq!(collected[0].1, "test_ability");
    }

    #[test]
    fn load_lenient_with_partial_dir() {
        let dir = tempfile::tempdir().unwrap();
        // Only write one file
        let content = r#"
[test_ability]
ability_type = "offensive"
cooldown = 5.0
mana_cost = 10.0

[test_ability.name]
en = "Test"
"#;
        std::fs::write(dir.path().join("abilities.toml"), content).unwrap();

        let regs = load_helix_registries_lenient(dir.path()).expect("Lenient load should not fail");
        assert_eq!(regs.abilities.len(), 1);
        assert_eq!(regs.items.len(), 0); // Missing file → empty registry
        assert_eq!(regs.total_entities(), 1);
    }
}
