//! Loot drop system for DJ Engine.
//!
//! Listens for [`DamageEvent`] where `target_defeated` is true, looks up
//! the target's loot table, rolls drops, and adds items to the player's
//! [`Inventory`]. Fires [`LootDropEvent`] for each item dropped.

use bevy::prelude::*;

use crate::combat::DamageEvent;
use crate::data::components::CombatStatsComponent;
use crate::data::database::{Database, LootEntry, LootTableRow};
use crate::inventory::Inventory;

/// Fired when loot is dropped from a defeated entity.
#[derive(Message, Debug, Clone, PartialEq)]
pub struct LootDropEvent {
    pub source_entity: Entity,
    pub item_id: String,
    pub quantity: u32,
    pub added_to_inventory: bool,
}

/// Roll a single loot entry. Returns Some((item_id, quantity)) if the roll succeeds.
pub fn roll_loot_entry(entry: &LootEntry, rng_roll: f32) -> Option<(String, u32)> {
    if rng_roll >= entry.chance {
        return None;
    }

    let quantity = if entry.min_quantity == entry.max_quantity {
        entry.min_quantity
    } else {
        let range = entry.max_quantity - entry.min_quantity;
        entry.min_quantity + (rng_roll / entry.chance * range as f32) as u32
    };

    Some((entry.item_id.clone(), quantity.max(1)))
}

/// Roll all entries in a loot table. Returns vec of (item_id, quantity).
pub fn roll_loot_table(table: &LootTableRow) -> Vec<(String, u32)> {
    table
        .entries
        .iter()
        .filter_map(|entry| roll_loot_entry(entry, rand::random::<f32>()))
        .collect()
}

/// System that processes defeat events and generates loot drops.
pub fn process_loot_drops(
    mut damage_events: MessageReader<DamageEvent>,
    mut loot_events: MessageWriter<LootDropEvent>,
    stats_query: Query<&CombatStatsComponent>,
    database: Option<Res<Database>>,
    mut inventory: ResMut<Inventory>,
) {
    let Some(db) = database else {
        return;
    };

    for event in damage_events.read() {
        if !event.target_defeated {
            continue;
        }

        let Ok(target_stats) = stats_query.get(event.target) else {
            continue;
        };

        let Some(loot_table_id) = &target_stats.loot_table_id else {
            continue;
        };

        let Some(table) = db.find_loot_table(loot_table_id) else {
            warn!("Loot: table '{}' not found in database", loot_table_id);
            continue;
        };

        let drops = roll_loot_table(table);
        for (item_id, quantity) in drops {
            let leftover = inventory.add_item(&item_id, quantity, 99);
            let added = leftover == 0;

            loot_events.write(LootDropEvent {
                source_entity: event.target,
                item_id: item_id.clone(),
                quantity,
                added_to_inventory: added,
            });

            if added {
                info!("Loot: {} x{} added to inventory", item_id, quantity);
            } else {
                warn!(
                    "Loot: inventory full, {} x{} partially lost",
                    item_id, leftover
                );
            }
        }
    }
}

/// Plugin providing loot drop processing.
pub struct LootPlugin;

impl Plugin for LootPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<LootDropEvent>()
            .add_systems(Update, process_loot_drops);

        use crate::contracts::{AppContractExt, ContractEntry, PluginContract};
        app.register_contract(PluginContract {
            name: "LootPlugin".into(),
            description: "Loot table rolling and inventory integration on enemy defeat".into(),
            resources: vec![],
            components: vec![],
            events: vec![ContractEntry::of::<LootDropEvent>(
                "Item dropped from defeated entity",
            )],
            system_sets: vec![],
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::database::LootEntry;

    #[test]
    fn test_roll_loot_entry_guaranteed_drop() {
        let entry = LootEntry {
            item_id: "potion".into(),
            chance: 1.0,
            min_quantity: 1,
            max_quantity: 1,
        };
        // Any roll < 1.0 should succeed
        let result = roll_loot_entry(&entry, 0.5);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), ("potion".into(), 1));
    }

    #[test]
    fn test_roll_loot_entry_zero_chance() {
        let entry = LootEntry {
            item_id: "rare_gem".into(),
            chance: 0.0,
            min_quantity: 1,
            max_quantity: 1,
        };
        let result = roll_loot_entry(&entry, 0.5);
        assert!(result.is_none());
    }

    #[test]
    fn test_roll_loot_entry_partial_chance() {
        let entry = LootEntry {
            item_id: "herb".into(),
            chance: 0.3,
            min_quantity: 2,
            max_quantity: 2,
        };
        // Roll 0.1 < 0.3 chance → drops
        assert!(roll_loot_entry(&entry, 0.1).is_some());
        // Roll 0.5 >= 0.3 chance → no drop
        assert!(roll_loot_entry(&entry, 0.5).is_none());
    }

    #[test]
    fn test_roll_loot_table() {
        let mut table = LootTableRow::new("test_table");
        table.add_entry("gold_coin", 1.0, 5); // guaranteed
        table.add_entry("rare_sword", 0.0, 1); // never drops

        // Since we can't control rand, test the pure function
        let entry_guaranteed = &table.entries[0];
        assert!(roll_loot_entry(entry_guaranteed, 0.1).is_some());
        let entry_never = &table.entries[1];
        assert!(roll_loot_entry(entry_never, 0.1).is_none());
    }

    #[test]
    fn test_roll_quantity_range() {
        let entry = LootEntry {
            item_id: "coin".into(),
            chance: 1.0,
            min_quantity: 5,
            max_quantity: 10,
        };
        let (_, qty) = roll_loot_entry(&entry, 0.5).unwrap();
        assert!((5..=10).contains(&qty));
    }

    #[test]
    fn test_roll_loot_entry_at_boundary() {
        let entry = LootEntry {
            item_id: "gem".into(),
            chance: 0.5,
            min_quantity: 1,
            max_quantity: 1,
        };
        // Roll exactly at chance boundary — should NOT drop (roll < chance)
        assert!(roll_loot_entry(&entry, 0.5).is_none());
        // Roll just below — should drop
        assert!(roll_loot_entry(&entry, 0.49).is_some());
    }

    #[test]
    fn test_roll_loot_table_multiple_guaranteed() {
        let mut table = LootTableRow::new("multi");
        table.add_entry("gold", 1.0, 3);
        table.add_entry("silver", 1.0, 2);
        table.add_entry("bronze", 1.0, 1);

        let drops = roll_loot_table(&table);
        assert_eq!(drops.len(), 3);
    }

    #[test]
    fn test_empty_loot_table() {
        let table = LootTableRow::new("empty");
        let drops = roll_loot_table(&table);
        assert!(drops.is_empty());
    }

    #[test]
    fn test_loot_table_row_new() {
        let table = LootTableRow::new("test");
        assert_eq!(table.id, "test");
        assert!(table.entries.is_empty());
    }

    #[test]
    fn test_min_equals_max_quantity() {
        let entry = LootEntry {
            item_id: "exact".into(),
            chance: 1.0,
            min_quantity: 5,
            max_quantity: 5,
        };
        let (_, qty) = roll_loot_entry(&entry, 0.0).unwrap();
        assert_eq!(qty, 5);
    }
}
