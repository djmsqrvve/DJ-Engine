//! Bridge layer: convert helix-data typed entities to DJ Engine database rows.
//!
//! DJ Engine's `database.rs` types (ItemRow, EnemyRow, NpcRow, QuestRow) are the
//! engine's own game types with engine-specific fields. Helix types are MMORPG-domain
//! types with different fields. This module provides explicit conversions between them.

use crate::balance::BalanceOverlay;
use crate::registries::HelixRegistries;
use dj_engine::data::database::{
    Database, EnemyRow, ItemRow, ItemType, NpcRow, QuestRewards, QuestRow, Rarity,
};
use std::collections::HashMap;

/// Convert a helix-data `LocalizedString` to engine's `LocalizedString` (HashMap<String, String>).
fn convert_localized_string(
    helix_ls: &helix_data::types::LocalizedString,
) -> HashMap<String, String> {
    helix_ls.0.clone()
}

/// Map helix item_type string to engine's ItemType enum.
fn map_item_type(helix_type: &helix_data::item::ItemType) -> ItemType {
    match helix_type {
        helix_data::item::ItemType::Weapon => ItemType::Weapon,
        helix_data::item::ItemType::Armor => ItemType::Armor,
        helix_data::item::ItemType::Consumable => ItemType::Potion,
        helix_data::item::ItemType::Quest => ItemType::QuestItem,
        helix_data::item::ItemType::Reagent => ItemType::Misc,
        helix_data::item::ItemType::Container => ItemType::Misc,
        helix_data::item::ItemType::TradeGood => ItemType::Currency,
        helix_data::item::ItemType::Miscellaneous => ItemType::Misc,
    }
}

/// Map helix Rarity to engine's Rarity enum.
fn map_rarity(helix_rarity: &Option<helix_data::types::Rarity>) -> Rarity {
    match helix_rarity {
        Some(helix_data::types::Rarity::Common) => Rarity::Common,
        Some(helix_data::types::Rarity::Uncommon) => Rarity::Uncommon,
        Some(helix_data::types::Rarity::Rare) => Rarity::Rare,
        Some(helix_data::types::Rarity::Epic) => Rarity::Epic,
        Some(helix_data::types::Rarity::Legendary) | Some(helix_data::types::Rarity::Artifact) => {
            Rarity::Legendary
        }
        None => Rarity::Common,
    }
}

/// Convert a helix Item to a DJ Engine ItemRow.
pub fn item_to_item_row(
    id: &str,
    item: &helix_data::item::Item,
    balance: Option<&BalanceOverlay>,
) -> ItemRow {
    let damage = balance.and_then(|b| b.get_f64("damage")).unwrap_or(0.0) as i32;
    let defense = balance.and_then(|b| b.get_f64("defense")).unwrap_or(0.0) as i32;

    ItemRow {
        id: id.to_string(),
        name: convert_localized_string(&item.base.name),
        item_type: map_item_type(&item.item_type),
        damage,
        defense,
        heal_amount: 0,
        price: balance
            .and_then(|b| b.get_f64("buy_price"))
            .unwrap_or(item.buy_price as f64) as i32,
        sell_value: balance
            .and_then(|b| b.get_f64("sell_price"))
            .unwrap_or(item.sell_price as f64) as i32,
        max_stack: item.stack_size,
        rarity: map_rarity(&item.quality),
        sprite_id: String::new(),
        description: convert_localized_string(&item.base.description),
        scripts: Default::default(),
    }
}

/// Convert a helix Mob to a DJ Engine EnemyRow.
pub fn mob_to_enemy_row(
    id: &str,
    mob: &helix_data::mob::Mob,
    balance: Option<&BalanceOverlay>,
) -> EnemyRow {
    EnemyRow {
        id: id.to_string(),
        name: convert_localized_string(&mob.base.name),
        hp: balance
            .and_then(|b| b.get_f64("health"))
            .unwrap_or(mob.health as f64) as i32,
        damage: balance
            .and_then(|b| b.get_f64("damage_max"))
            .unwrap_or(mob.damage_max as f64) as i32,
        speed: balance
            .and_then(|b| b.get_f64("move_speed"))
            .unwrap_or(mob.move_speed) as f32,
        experience: balance
            .and_then(|b| b.get_f64("experience_value"))
            .unwrap_or(mob.experience_value as f64) as i32,
        loot_table_id: mob.loot_table_id.clone().unwrap_or_default(),
        behavior_profile_id: mob.ai_type.clone().unwrap_or_default(),
        faction: mob.faction.clone().unwrap_or_default(),
        respawn_time: mob.respawn_time as f32,
        attack_speed: balance
            .and_then(|b| b.get_f64("attack_speed"))
            .unwrap_or(mob.attack_speed) as f32,
    }
}

/// Convert a helix Npc to a DJ Engine NpcRow.
pub fn npc_to_npc_row(
    id: &str,
    npc: &helix_data::npc::Npc,
    _balance: Option<&BalanceOverlay>,
) -> NpcRow {
    NpcRow {
        id: id.to_string(),
        name: convert_localized_string(&npc.base.name),
        dialogue_set_id: String::new(),
        location_tags: npc.base.tags.clone(),
        default_faction: npc.faction.clone().unwrap_or_default(),
        default_quest_ids: npc.quests.clone(),
        loot_table_id: None,
        portrait_id: String::new(),
        vendor_items: npc.vendor_items.clone(),
    }
}

/// Convert a helix Quest to a DJ Engine QuestRow.
pub fn quest_to_quest_row(
    id: &str,
    quest: &helix_data::quest::Quest,
    balance: Option<&BalanceOverlay>,
) -> QuestRow {
    QuestRow {
        id: id.to_string(),
        name: convert_localized_string(&quest.base.name),
        description: convert_localized_string(&quest.base.description),
        start_conditions: quest
            .prerequisite_quests
            .iter()
            .map(|q| serde_json::Value::String(q.clone()))
            .collect(),
        completion_conditions: quest
            .objectives
            .iter()
            .map(|o| serde_json::Value::String(o.clone()))
            .collect(),
        rewards: QuestRewards {
            gold: balance
                .and_then(|b| b.get_f64("reward_gold"))
                .unwrap_or(quest.reward_gold as f64) as i32,
            experience: balance
                .and_then(|b| b.get_f64("reward_xp"))
                .unwrap_or(quest.reward_xp as f64) as i32,
            item_rewards: Vec::new(),
            flags: HashMap::new(),
        },
        is_daily: quest.is_daily,
        is_repeatable: quest.is_repeatable,
        sharable: quest.sharable,
    }
}

/// Convert a helix Ability → engine AbilityRow.
pub fn ability_to_ability_row(
    id: &str,
    ability: &helix_data::ability::Ability,
    _balance: Option<&BalanceOverlay>,
) -> dj_engine::data::AbilityRow {
    dj_engine::data::AbilityRow {
        id: id.to_string(),
        name: convert_localized_string(&ability.base.name),
        ability_type: format!("{:?}", ability.ability_type).to_lowercase(),
        school: ability
            .school
            .as_ref()
            .map(|s| format!("{s:?}").to_lowercase())
            .unwrap_or_default(),
        cooldown: ability.cooldown,
        mana_cost: ability.mana_cost,
        description: convert_localized_string(&ability.base.description),
    }
}

/// Convert a helix Zone → engine ZoneRow.
pub fn zone_to_zone_row(
    id: &str,
    zone: &helix_data::zone::Zone,
    _balance: Option<&BalanceOverlay>,
) -> dj_engine::data::ZoneRow {
    dj_engine::data::ZoneRow {
        id: id.to_string(),
        name: convert_localized_string(&zone.base.name),
        level_min: zone.level_range[0],
        level_max: zone.level_range[1],
        continent: zone.continent.clone(),
        description: convert_localized_string(&zone.base.description),
    }
}

/// Populate a DJ Engine `Database` from all typed Helix registries.
///
/// Converts abilities→abilities, zones→zones, mobs→enemies, items→items,
/// npcs→npcs, quests→quests. Balance overlays (if provided) are applied
/// during conversion.
pub fn populate_database_from_helix(
    registries: &HelixRegistries,
    balance: Option<&crate::balance::BalanceOverlays>,
) -> Database {
    let mut db = Database::new();

    for (id, ability) in registries.abilities.iter() {
        let overlay = balance.and_then(|b| b.get("abilities", id));
        db.abilities
            .push(ability_to_ability_row(id, ability, overlay));
    }

    for (id, zone) in registries.zones.iter() {
        let overlay = balance.and_then(|b| b.get("zones", id));
        db.zones.push(zone_to_zone_row(id, zone, overlay));
    }

    for (id, item) in registries.items.iter() {
        let overlay = balance.and_then(|b| b.get("items", id));
        db.items.push(item_to_item_row(id, item, overlay));
    }

    for (id, mob) in registries.mobs.iter() {
        let overlay = balance.and_then(|b| b.get("mobs", id));
        db.enemies.push(mob_to_enemy_row(id, mob, overlay));
    }

    for (id, npc) in registries.npcs.iter() {
        let overlay = balance.and_then(|b| b.get("npcs", id));
        db.npcs.push(npc_to_npc_row(id, npc, overlay));
    }

    for (id, quest) in registries.quests.iter() {
        let overlay = balance.and_then(|b| b.get("quests", id));
        db.quests.push(quest_to_quest_row(id, quest, overlay));
    }

    db
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_mob() -> helix_data::mob::Mob {
        helix_data::mob::Mob {
            base: helix_data::types::BaseEntity {
                name: "Wolf".into(),
                description: Default::default(),
                category: String::new(),
                tags: Vec::new(),
            },
            mob_type: helix_data::types::MobType::Normal,
            level_min: 1,
            level_max: 1,
            health: 50,
            mana: 0,
            damage_min: 5,
            damage_max: 10,
            armor: 2,
            abilities: Vec::new(),
            loot_table_id: Some("wolf_loot".into()),
            zone_ids: Vec::new(),
            faction: None,
            respawn_time: 0.0,
            experience_value: 25,
            aggro_range: 10.0,
            leash_range: 40.0,
            attack_speed: 2.0,
            creature_type: None,
            move_speed: 100.0,
            ai_type: Some("basic_aggro".into()),
            loot_ids: Vec::new(),
        }
    }

    fn make_test_item() -> helix_data::item::Item {
        helix_data::item::Item {
            base: helix_data::types::BaseEntity {
                name: "Health Potion".into(),
                description: Default::default(),
                category: String::new(),
                tags: Vec::new(),
            },
            item_type: helix_data::item::ItemType::Consumable,
            quality: Some(helix_data::types::Rarity::Common),
            level_requirement: 0,
            stats: Vec::new(),
            stack_size: 20,
            sell_price: 10,
            buy_price: 50,
            equip_slot: None,
            bind_type: Default::default(),
            required_class: None,
            set_id: None,
            durability: 0,
        }
    }

    #[test]
    fn mob_to_enemy_row_basic_conversion() {
        let mob = make_test_mob();
        let row = mob_to_enemy_row("wolf", &mob, None);
        assert_eq!(row.id, "wolf");
        assert_eq!(row.hp, 50);
        assert_eq!(row.damage, 10);
        assert_eq!(row.speed, 100.0);
        assert_eq!(row.experience, 25);
        assert_eq!(row.loot_table_id, "wolf_loot");
        assert_eq!(row.behavior_profile_id, "basic_aggro");
        assert_eq!(row.name.get("en").unwrap(), "Wolf");
    }

    #[test]
    fn mob_to_enemy_row_with_balance_override() {
        let mob = make_test_mob();

        let mut overlay = BalanceOverlay::default();
        overlay.set("health", 30.0);
        overlay.set("damage_max", 5.0);

        let row = mob_to_enemy_row("wolf", &mob, Some(&overlay));
        assert_eq!(row.hp, 30);
        assert_eq!(row.damage, 5);
    }

    #[test]
    fn item_to_item_row_basic_conversion() {
        let item = make_test_item();
        let row = item_to_item_row("health_potion", &item, None);
        assert_eq!(row.id, "health_potion");
        assert_eq!(row.item_type, ItemType::Potion);
        assert_eq!(row.rarity, Rarity::Common);
        assert_eq!(row.max_stack, 20);
        assert_eq!(row.sell_value, 10);
        assert_eq!(row.price, 50);
    }

    #[test]
    fn populate_database_from_real_helix_data() {
        let helix3d_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../../helix/helix_standardization/dist/helix3d");
        if !helix3d_dir.is_dir() {
            eprintln!("Skipping: helix3d dir not found");
            return;
        }

        let regs = crate::registries::load_helix_registries_lenient(&helix3d_dir).unwrap();
        let db = populate_database_from_helix(&regs, None);

        // Mobs, npcs, quests should always parse. Items may have schema mismatches.
        assert!(!db.enemies.is_empty(), "Expected enemies from helix mobs");
        assert!(!db.npcs.is_empty(), "Expected npcs from helix npcs");
        assert!(!db.quests.is_empty(), "Expected quests from helix quests");

        // Verify a known entity
        let wolf = db.enemies.iter().find(|e| e.id == "wolf");
        assert!(wolf.is_some(), "Expected wolf enemy");

        eprintln!(
            "Bridge populated: {} enemies, {} items, {} npcs, {} quests",
            db.enemies.len(),
            db.items.len(),
            db.npcs.len(),
            db.quests.len()
        );
    }
}
