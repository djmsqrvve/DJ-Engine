//! Bridge layer: convert helix-data typed entities to DJ Engine database rows.
//!
//! DJ Engine's `database.rs` types (ItemRow, EnemyRow, NpcRow, QuestRow) are the
//! engine's own game types with engine-specific fields. Helix types are MMORPG-domain
//! types with different fields. This module provides explicit conversions between them.

use crate::balance::BalanceOverlay;
use crate::registries::HelixRegistries;
use dj_engine::data::database::{
    AchievementRow, AuraRow, ClassDataRow, ConsumableRow, CurrencyRow, Database, EnemyRow,
    EquipmentRow, GuildRow, InventoryRow, ItemRow, ItemType, MountRow, NpcRow, ProfessionRow,
    PvpRow, QuestRewards, QuestRow, RaidRow, Rarity, TalentRow, TitleRow, TradeGoodRow,
    WeaponSkillRow,
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

/// Convert a helix Aura to a DJ Engine AuraRow.
pub fn aura_to_aura_row(
    id: &str,
    aura: &helix_data::aura::Aura,
    _balance: Option<&BalanceOverlay>,
) -> AuraRow {
    AuraRow {
        id: id.to_string(),
        name: convert_localized_string(&aura.base.name),
        aura_type: format!("{:?}", aura.aura_type).to_lowercase(),
        duration: aura.duration,
        max_stacks: aura.max_stacks,
        description: convert_localized_string(&aura.base.description),
    }
}

/// Convert a helix ClassData to a DJ Engine ClassDataRow.
pub fn class_data_to_class_data_row(
    id: &str,
    class: &helix_data::class_data::ClassData,
    _balance: Option<&BalanceOverlay>,
) -> ClassDataRow {
    ClassDataRow {
        id: id.to_string(),
        name: convert_localized_string(&class.base.name),
        role: format!("{:?}", class.role).to_lowercase(),
        resource_type: format!("{:?}", class.resource_type).to_lowercase(),
        abilities: class.abilities.clone(),
        talent_trees: class.talent_trees.clone(),
    }
}

/// Convert a helix Raid to a DJ Engine RaidRow.
pub fn raid_to_raid_row(
    id: &str,
    raid: &helix_data::raid::Raid,
    _balance: Option<&BalanceOverlay>,
) -> RaidRow {
    RaidRow {
        id: id.to_string(),
        name: convert_localized_string(&raid.base.name),
        zone_id: raid.zone_id.clone().unwrap_or_default(),
        size: raid.size,
        difficulty: format!("{:?}", raid.difficulty).to_lowercase(),
        bosses: raid.bosses.clone(),
        description: convert_localized_string(&raid.base.description),
    }
}

/// Convert a helix Talent to a DJ Engine TalentRow.
pub fn talent_to_talent_row(
    id: &str,
    talent: &helix_data::talent::Talent,
    _balance: Option<&BalanceOverlay>,
) -> TalentRow {
    TalentRow {
        id: id.to_string(),
        name: convert_localized_string(&talent.base.name),
        class_id: talent.class_id.clone(),
        tree: talent.tree.clone(),
        tier: talent.tier,
        column: talent.column,
        max_rank: talent.max_rank,
        prerequisite_talent: talent.prerequisite_talent.clone(),
        description: convert_localized_string(&talent.base.description),
    }
}

/// Convert a helix Profession to a DJ Engine ProfessionRow.
pub fn profession_to_profession_row(
    id: &str,
    prof: &helix_data::profession::Profession,
    _balance: Option<&BalanceOverlay>,
) -> ProfessionRow {
    ProfessionRow {
        id: id.to_string(),
        name: convert_localized_string(&prof.base.name),
        profession_type: format!("{:?}", prof.profession_type).to_lowercase(),
        max_skill: prof.max_skill,
        description: convert_localized_string(&prof.base.description),
    }
}

/// Convert a helix PvpData to a DJ Engine PvpRow.
pub fn pvp_to_pvp_row(
    id: &str,
    pvp: &helix_data::pvp::PvpData,
    _balance: Option<&BalanceOverlay>,
) -> PvpRow {
    PvpRow {
        id: id.to_string(),
        name: convert_localized_string(&pvp.base.name),
        pvp_type: format!("{:?}", pvp.pvp_type).to_lowercase(),
        team_size: pvp.team_size[1], // max team size
        description: convert_localized_string(&pvp.base.description),
    }
}

/// Convert a helix Achievement to a DJ Engine AchievementRow.
pub fn achievement_to_achievement_row(
    id: &str,
    ach: &helix_data::achievement::Achievement,
    _balance: Option<&BalanceOverlay>,
) -> AchievementRow {
    AchievementRow {
        id: id.to_string(),
        name: convert_localized_string(&ach.base.name),
        points: ach.points,
        criteria: ach.criteria.clone(),
        description: convert_localized_string(&ach.base.description),
    }
}

/// Convert a helix Mount to a DJ Engine MountRow.
pub fn mount_to_mount_row(
    id: &str,
    mount: &helix_data::mount::Mount,
    _balance: Option<&BalanceOverlay>,
) -> MountRow {
    MountRow {
        id: id.to_string(),
        name: convert_localized_string(&mount.base.name),
        mount_type: format!("{:?}", mount.mount_type).to_lowercase(),
        speed_modifier: mount.speed_modifier,
        description: convert_localized_string(&mount.base.description),
    }
}

/// Convert a helix Guild to a DJ Engine GuildRow.
pub fn guild_to_guild_row(
    id: &str,
    guild: &helix_data::guild::Guild,
    _balance: Option<&BalanceOverlay>,
) -> GuildRow {
    GuildRow {
        id: id.to_string(),
        name: convert_localized_string(&guild.base.name),
        max_members: guild.max_members,
        description: convert_localized_string(&guild.base.description),
    }
}

/// Convert a helix Consumable to a DJ Engine ConsumableRow.
pub fn consumable_to_consumable_row(
    id: &str,
    consumable: &helix_data::consumable::Consumable,
    _balance: Option<&BalanceOverlay>,
) -> ConsumableRow {
    ConsumableRow {
        id: id.to_string(),
        name: convert_localized_string(&consumable.base.name),
        consumable_type: format!("{:?}", consumable.consumable_type).to_lowercase(),
        stack_size: consumable.stack_size,
        cooldown: consumable.cooldown as f32,
        description: convert_localized_string(&consumable.base.description),
    }
}

/// Convert a helix Currency to a DJ Engine CurrencyRow.
pub fn currency_to_currency_row(
    id: &str,
    currency: &helix_data::currency::Currency,
    _balance: Option<&BalanceOverlay>,
) -> CurrencyRow {
    CurrencyRow {
        id: id.to_string(),
        name: convert_localized_string(&currency.base.name),
        max_amount: currency.max_amount,
        description: convert_localized_string(&currency.base.description),
    }
}

/// Convert a helix Equipment to a DJ Engine EquipmentRow.
pub fn equipment_to_equipment_row(
    id: &str,
    equip: &helix_data::equipment::Equipment,
    _balance: Option<&BalanceOverlay>,
) -> EquipmentRow {
    EquipmentRow {
        id: id.to_string(),
        name: convert_localized_string(&equip.base.name),
        slot: format!("{:?}", equip.slot).to_lowercase(),
        armor_value: equip.durability as i32,
        stats: equip
            .stats
            .iter()
            .map(|s| (s.stat.clone(), s.value))
            .collect(),
        level_requirement: equip.level_requirement,
        rarity: map_rarity(&equip.quality),
        description: convert_localized_string(&equip.base.description),
    }
}

/// Convert a helix Inventory to a DJ Engine InventoryRow.
pub fn inventory_to_inventory_row(
    id: &str,
    inv: &helix_data::inventory::Inventory,
    _balance: Option<&BalanceOverlay>,
) -> InventoryRow {
    InventoryRow {
        id: id.to_string(),
        name: convert_localized_string(&inv.base.name),
        slot_type: format!("{:?}", inv.slot_type).to_lowercase(),
        capacity: inv.capacity,
        description: convert_localized_string(&inv.base.description),
    }
}

/// Convert a helix Title to a DJ Engine TitleRow.
pub fn title_to_title_row(
    id: &str,
    title: &helix_data::title::Title,
    _balance: Option<&BalanceOverlay>,
) -> TitleRow {
    TitleRow {
        id: id.to_string(),
        name: convert_localized_string(&title.base.name),
        style: format!("{:?}", title.style).to_lowercase(),
        source: title.source.clone(),
        description: convert_localized_string(&title.base.description),
    }
}

/// Convert a helix TradeGood to a DJ Engine TradeGoodRow.
pub fn trade_good_to_trade_good_row(
    id: &str,
    tg: &helix_data::trade_good::TradeGood,
    _balance: Option<&BalanceOverlay>,
) -> TradeGoodRow {
    TradeGoodRow {
        id: id.to_string(),
        name: convert_localized_string(&tg.base.name),
        stack_size: tg.stack_size,
        vendor_price: tg.vendor_price as i32,
        description: convert_localized_string(&tg.base.description),
    }
}

/// Convert a helix WeaponSkill to a DJ Engine WeaponSkillRow.
pub fn weapon_skill_to_weapon_skill_row(
    id: &str,
    ws: &helix_data::weapon_skill::WeaponSkill,
    _balance: Option<&BalanceOverlay>,
) -> WeaponSkillRow {
    WeaponSkillRow {
        id: id.to_string(),
        name: convert_localized_string(&ws.base.name),
        weapon_type: ws.weapon_type.clone(),
        classes: ws.classes.clone(),
        max_skill: ws.max_skill,
        description: convert_localized_string(&ws.base.description),
    }
}

/// Populate a DJ Engine `Database` from all 22 typed Helix registries.
///
/// Balance overlays (if provided) are applied during conversion.
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

    for (id, aura) in registries.auras.iter() {
        let overlay = balance.and_then(|b| b.get("auras", id));
        db.auras.push(aura_to_aura_row(id, aura, overlay));
    }

    for (id, class) in registries.class_data.iter() {
        let overlay = balance.and_then(|b| b.get("class_data", id));
        db.class_data
            .push(class_data_to_class_data_row(id, class, overlay));
    }

    for (id, raid) in registries.raids.iter() {
        let overlay = balance.and_then(|b| b.get("raids", id));
        db.raids.push(raid_to_raid_row(id, raid, overlay));
    }

    for (id, talent) in registries.talents.iter() {
        let overlay = balance.and_then(|b| b.get("talents", id));
        db.talents.push(talent_to_talent_row(id, talent, overlay));
    }

    for (id, prof) in registries.professions.iter() {
        let overlay = balance.and_then(|b| b.get("professions", id));
        db.professions
            .push(profession_to_profession_row(id, prof, overlay));
    }

    for (id, pvp) in registries.pvp.iter() {
        let overlay = balance.and_then(|b| b.get("pvp", id));
        db.pvp.push(pvp_to_pvp_row(id, pvp, overlay));
    }

    for (id, ach) in registries.achievements.iter() {
        let overlay = balance.and_then(|b| b.get("achievements", id));
        db.achievements
            .push(achievement_to_achievement_row(id, ach, overlay));
    }

    for (id, mount) in registries.mounts.iter() {
        let overlay = balance.and_then(|b| b.get("mounts", id));
        db.mounts.push(mount_to_mount_row(id, mount, overlay));
    }

    for (id, guild) in registries.guilds.iter() {
        let overlay = balance.and_then(|b| b.get("guilds", id));
        db.guilds.push(guild_to_guild_row(id, guild, overlay));
    }

    for (id, consumable) in registries.consumables.iter() {
        let overlay = balance.and_then(|b| b.get("consumables", id));
        db.consumables
            .push(consumable_to_consumable_row(id, consumable, overlay));
    }

    for (id, currency) in registries.currencies.iter() {
        let overlay = balance.and_then(|b| b.get("currencies", id));
        db.currencies
            .push(currency_to_currency_row(id, currency, overlay));
    }

    for (id, equip) in registries.equipment.iter() {
        let overlay = balance.and_then(|b| b.get("equipment", id));
        db.equipment
            .push(equipment_to_equipment_row(id, equip, overlay));
    }

    for (id, inv) in registries.inventory.iter() {
        let overlay = balance.and_then(|b| b.get("inventory", id));
        db.inventory
            .push(inventory_to_inventory_row(id, inv, overlay));
    }

    for (id, title) in registries.titles.iter() {
        let overlay = balance.and_then(|b| b.get("titles", id));
        db.titles.push(title_to_title_row(id, title, overlay));
    }

    for (id, tg) in registries.trade_goods.iter() {
        let overlay = balance.and_then(|b| b.get("trade_goods", id));
        db.trade_goods
            .push(trade_good_to_trade_good_row(id, tg, overlay));
    }

    for (id, ws) in registries.weapon_skills.iter() {
        let overlay = balance.and_then(|b| b.get("weapon_skills", id));
        db.weapon_skills
            .push(weapon_skill_to_weapon_skill_row(id, ws, overlay));
    }

    db
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_mob() -> helix_data::mob::Mob {
        helix_data::mob::Mob {
            base: helix_data::types::BaseEntity {
                schema_version: None,
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
                schema_version: None,
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
    fn consumable_to_consumable_row_basic() {
        let c = helix_data::consumable::Consumable {
            base: helix_data::types::BaseEntity {
                schema_version: None,
                name: "Health Potion".into(),
                description: Default::default(),
                category: String::new(),
                tags: Vec::new(),
            },
            consumable_type: helix_data::consumable::ConsumableType::Potion,
            effects: vec!["heal_50".into()],
            duration: 0.0,
            cooldown: 30.0,
            stack_size: 20,
            level_requirement: 0,
        };
        let row = consumable_to_consumable_row("health_potion", &c, None);
        assert_eq!(row.id, "health_potion");
        assert_eq!(row.consumable_type, "potion");
        assert_eq!(row.stack_size, 20);
        assert_eq!(row.cooldown, 30.0);
    }

    #[test]
    fn currency_to_currency_row_basic() {
        let c = helix_data::currency::Currency {
            base: helix_data::types::BaseEntity {
                schema_version: None,
                name: "Gold".into(),
                description: Default::default(),
                category: String::new(),
                tags: Vec::new(),
            },
            max_amount: 9999,
            cap_per_week: Some(500),
        };
        let row = currency_to_currency_row("gold", &c, None);
        assert_eq!(row.id, "gold");
        assert_eq!(row.max_amount, 9999);
    }

    #[test]
    fn equipment_to_equipment_row_basic() {
        let e = helix_data::equipment::Equipment {
            base: helix_data::types::BaseEntity {
                schema_version: None,
                name: "Iron Helm".into(),
                description: Default::default(),
                category: String::new(),
                tags: Vec::new(),
            },
            slot: helix_data::item::EquipSlot::Head,
            armor_type: Some(helix_data::equipment::ArmorType::Plate),
            stats: vec![helix_data::types::StatModifier {
                stat: "stamina".into(),
                value: 10.0,
            }],
            level_requirement: 5,
            quality: Some(helix_data::types::Rarity::Uncommon),
            durability: 100,
            set_id: None,
            required_class: None,
        };
        let row = equipment_to_equipment_row("iron_helm", &e, None);
        assert_eq!(row.id, "iron_helm");
        assert_eq!(row.slot, "head");
        assert_eq!(row.stats.len(), 1);
        assert_eq!(row.level_requirement, 5);
    }

    #[test]
    fn inventory_to_inventory_row_basic() {
        let inv = helix_data::inventory::Inventory {
            base: helix_data::types::BaseEntity {
                schema_version: None,
                name: "Backpack".into(),
                description: Default::default(),
                category: String::new(),
                tags: Vec::new(),
            },
            slot_type: helix_data::inventory::SlotType::Bag,
            capacity: 16,
            allowed_types: Vec::new(),
            default_unlocked: true,
        };
        let row = inventory_to_inventory_row("backpack", &inv, None);
        assert_eq!(row.id, "backpack");
        assert_eq!(row.slot_type, "bag");
        assert_eq!(row.capacity, 16);
    }

    #[test]
    fn title_to_title_row_basic() {
        let t = helix_data::title::Title {
            base: helix_data::types::BaseEntity {
                schema_version: None,
                name: "Champion".into(),
                description: Default::default(),
                category: String::new(),
                tags: Vec::new(),
            },
            style: helix_data::title::TitleStyle::Prefix,
            source: "achievement".into(),
            source_id: Some("arena_100".into()),
            display_format: None,
        };
        let row = title_to_title_row("champion", &t, None);
        assert_eq!(row.id, "champion");
        assert_eq!(row.style, "prefix");
        assert_eq!(row.source, "achievement");
    }

    #[test]
    fn trade_good_to_trade_good_row_basic() {
        let tg = helix_data::trade_good::TradeGood {
            base: helix_data::types::BaseEntity {
                schema_version: None,
                name: "Copper Ore".into(),
                description: Default::default(),
                category: String::new(),
                tags: Vec::new(),
            },
            stack_size: 20,
            vendor_price: 50,
        };
        let row = trade_good_to_trade_good_row("copper_ore", &tg, None);
        assert_eq!(row.id, "copper_ore");
        assert_eq!(row.stack_size, 20);
        assert_eq!(row.vendor_price, 50);
    }

    #[test]
    fn weapon_skill_to_weapon_skill_row_basic() {
        let ws = helix_data::weapon_skill::WeaponSkill {
            base: helix_data::types::BaseEntity {
                schema_version: None,
                name: "Swords".into(),
                description: Default::default(),
                category: String::new(),
                tags: Vec::new(),
            },
            weapon_type: "sword".into(),
            classes: vec!["warrior".into(), "paladin".into()],
            max_skill: 300,
        };
        let row = weapon_skill_to_weapon_skill_row("swords", &ws, None);
        assert_eq!(row.id, "swords");
        assert_eq!(row.weapon_type, "sword");
        assert_eq!(row.classes, vec!["warrior", "paladin"]);
        assert_eq!(row.max_skill, 300);
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
