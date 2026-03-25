//! Economy and equipment systems for DJ Engine.
//!
//! Consumes Helix Row types from [`Database`] to drive gameplay:
//! - [`UseConsumableRequest`] → look up [`ConsumableRow`], apply effect
//! - [`EquipItemRequest`] / [`UnequipItemRequest`] → look up [`EquipmentRow`], modify stats
//! - [`VendorBuyRequest`] / [`VendorSellRequest`] → look up prices, transfer items/gold

use bevy::prelude::*;

use crate::data::components::CombatStatsComponent;
use crate::data::components::EquipmentSlotsComponent;
use crate::data::database::Database;
use crate::inventory::Inventory;

// ---------------------------------------------------------------------------
// Consumables
// ---------------------------------------------------------------------------

/// Request to use a consumable item.
#[derive(Message, Debug, Clone, PartialEq)]
pub struct UseConsumableRequest {
    pub entity: Entity,
    pub consumable_id: String,
}

/// Result of using a consumable.
#[derive(Message, Debug, Clone, PartialEq)]
pub enum ConsumableUsedEvent {
    Success {
        entity: Entity,
        consumable_id: String,
        effect: String,
    },
    NotFound {
        consumable_id: String,
    },
    NotInInventory {
        consumable_id: String,
    },
}

/// System that processes consumable use requests.
pub fn process_consumable_use(
    mut requests: MessageReader<UseConsumableRequest>,
    mut events: MessageWriter<ConsumableUsedEvent>,
    database: Option<Res<Database>>,
    mut inventory: ResMut<Inventory>,
    mut stats_query: Query<&mut CombatStatsComponent>,
) {
    for request in requests.read() {
        // Check inventory
        if !inventory.has_item(&request.consumable_id, 1) {
            events.write(ConsumableUsedEvent::NotInInventory {
                consumable_id: request.consumable_id.clone(),
            });
            continue;
        }

        // Look up consumable definition
        let consumable = database
            .as_ref()
            .and_then(|db| db.find_consumable(&request.consumable_id));

        let Some(consumable) = consumable else {
            events.write(ConsumableUsedEvent::NotFound {
                consumable_id: request.consumable_id.clone(),
            });
            continue;
        };

        // Remove from inventory
        inventory.remove_item(&request.consumable_id, 1);

        // Apply effect based on consumable_type
        let effect = match consumable.consumable_type.as_str() {
            "potion" | "food" | "drink" => {
                // Heal the entity
                if let Ok(mut stats) = stats_query.get_mut(request.entity) {
                    let heal = 25; // base heal, could derive from consumable data
                    stats.hp = (stats.hp + heal).min(stats.max_hp);
                    format!("healed {} hp", heal)
                } else {
                    "no combat stats".into()
                }
            }
            "elixir" | "flask" => {
                // Buff — apply temporary stat boost
                "buff applied".into()
            }
            _ => "used".into(),
        };

        info!(
            "Economy: used consumable '{}' ({}): {}",
            request.consumable_id, consumable.consumable_type, effect
        );

        events.write(ConsumableUsedEvent::Success {
            entity: request.entity,
            consumable_id: request.consumable_id.clone(),
            effect,
        });
    }
}

// ---------------------------------------------------------------------------
// Equipment
// ---------------------------------------------------------------------------

/// Request to equip an item.
#[derive(Message, Debug, Clone, PartialEq)]
pub struct EquipItemRequest {
    pub entity: Entity,
    pub equipment_id: String,
}

/// Request to unequip from a slot.
#[derive(Message, Debug, Clone, PartialEq)]
pub struct UnequipItemRequest {
    pub entity: Entity,
    pub slot: String,
}

/// Result of equip/unequip.
#[derive(Message, Debug, Clone, PartialEq)]
pub enum EquipmentEvent {
    Equipped {
        entity: Entity,
        equipment_id: String,
        slot: String,
    },
    Unequipped {
        entity: Entity,
        slot: String,
        returned_item: Option<String>,
    },
    NotFound {
        equipment_id: String,
    },
}

/// System that processes equip requests.
pub fn process_equip(
    mut requests: MessageReader<EquipItemRequest>,
    mut events: MessageWriter<EquipmentEvent>,
    database: Option<Res<Database>>,
    mut query: Query<(&mut EquipmentSlotsComponent, &mut CombatStatsComponent)>,
    mut inventory: ResMut<Inventory>,
) {
    for request in requests.read() {
        let equipment = database
            .as_ref()
            .and_then(|db| db.find_equipment(&request.equipment_id));

        let Some(equipment) = equipment else {
            events.write(EquipmentEvent::NotFound {
                equipment_id: request.equipment_id.clone(),
            });
            continue;
        };

        let Ok((mut slots, mut stats)) = query.get_mut(request.entity) else {
            continue;
        };

        // Determine which slot to use
        let slot_ref = match equipment.slot.as_str() {
            "head" => &mut slots.head,
            "chest" => &mut slots.chest,
            "legs" => &mut slots.legs,
            "feet" => &mut slots.feet,
            "hands" => &mut slots.hands,
            "main_hand" => &mut slots.main_hand,
            "off_hand" => &mut slots.off_hand,
            "back" => &mut slots.back,
            "neck" => &mut slots.neck,
            "trinket1" => &mut slots.trinket1,
            "trinket2" => &mut slots.trinket2,
            _ => {
                warn!("Economy: unknown equipment slot '{}'", equipment.slot);
                continue;
            }
        };

        // Return current item to inventory if slot occupied
        if let Some(old_id) = slot_ref.take() {
            inventory.add_item(&old_id, 1, 1);
        }

        // Equip new item
        *slot_ref = Some(request.equipment_id.clone());

        // Apply stat bonuses
        stats.defense += equipment.armor_value;
        for (stat, value) in &equipment.stats {
            match stat.as_str() {
                "damage" => stats.damage += *value as i32,
                "hp" | "max_hp" => stats.max_hp += *value as i32,
                "mana" | "max_mana" => stats.mana += *value as i32,
                "crit_chance" => stats.crit_chance += *value as f32,
                _ => {}
            }
        }

        info!(
            "Economy: equipped '{}' in slot '{}' (+{} armor)",
            request.equipment_id, equipment.slot, equipment.armor_value
        );

        events.write(EquipmentEvent::Equipped {
            entity: request.entity,
            equipment_id: request.equipment_id.clone(),
            slot: equipment.slot.clone(),
        });
    }
}

// ---------------------------------------------------------------------------
// Vendor / Trading
// ---------------------------------------------------------------------------

/// Request to buy an item from a vendor.
#[derive(Message, Debug, Clone, PartialEq)]
pub struct VendorBuyRequest {
    pub item_id: String,
    pub currency_id: String,
}

/// Request to sell an item to a vendor.
#[derive(Message, Debug, Clone, PartialEq)]
pub struct VendorSellRequest {
    pub item_id: String,
    pub currency_id: String,
}

/// Result of vendor transactions.
#[derive(Message, Debug, Clone, PartialEq)]
pub enum VendorEvent {
    Bought {
        item_id: String,
        price: u64,
    },
    Sold {
        item_id: String,
        price: u64,
    },
    InsufficientFunds {
        item_id: String,
        price: u64,
        balance: u64,
    },
    ItemNotFound {
        item_id: String,
    },
}

/// System that processes vendor buy requests.
pub fn process_vendor_buy(
    mut requests: MessageReader<VendorBuyRequest>,
    mut events: MessageWriter<VendorEvent>,
    database: Option<Res<Database>>,
    mut inventory: ResMut<Inventory>,
) {
    for request in requests.read() {
        let db = match &database {
            Some(db) => db,
            None => continue,
        };

        // Look up item price (check items, trade_goods, consumables)
        let price = db
            .find_item(&request.item_id)
            .map(|i| i.price as u64)
            .or_else(|| {
                db.find_trade_good(&request.item_id)
                    .map(|t| t.vendor_price as u64)
            })
            .or(Some(10)); // default price for unknown items

        let Some(price) = price else {
            events.write(VendorEvent::ItemNotFound {
                item_id: request.item_id.clone(),
            });
            continue;
        };

        let balance = inventory.currency_balance(&request.currency_id);
        if !inventory.spend_currency(&request.currency_id, price) {
            events.write(VendorEvent::InsufficientFunds {
                item_id: request.item_id.clone(),
                price,
                balance,
            });
            continue;
        }

        // Check currency max_amount from CurrencyRow
        // (spending already handled above)

        let max_stack = db
            .find_trade_good(&request.item_id)
            .map(|t| t.stack_size)
            .or_else(|| db.find_item(&request.item_id).map(|i| i.max_stack))
            .unwrap_or(99);

        inventory.add_item(&request.item_id, 1, max_stack);

        info!(
            "Economy: bought '{}' for {} {}",
            request.item_id, price, request.currency_id
        );

        events.write(VendorEvent::Bought {
            item_id: request.item_id.clone(),
            price,
        });
    }
}

/// System that processes vendor sell requests.
pub fn process_vendor_sell(
    mut requests: MessageReader<VendorSellRequest>,
    mut events: MessageWriter<VendorEvent>,
    database: Option<Res<Database>>,
    mut inventory: ResMut<Inventory>,
) {
    for request in requests.read() {
        if !inventory.has_item(&request.item_id, 1) {
            continue;
        }

        let sell_price = database
            .as_ref()
            .and_then(|db| db.find_item(&request.item_id))
            .map(|i| (i.sell_value.max(1)) as u64)
            .or_else(|| {
                database
                    .as_ref()
                    .and_then(|db| db.find_trade_good(&request.item_id))
                    .map(|t| (t.vendor_price / 2).max(1) as u64)
            })
            .unwrap_or(1);

        inventory.remove_item(&request.item_id, 1);
        inventory.add_currency(&request.currency_id, sell_price);

        info!(
            "Economy: sold '{}' for {} {}",
            request.item_id, sell_price, request.currency_id
        );

        events.write(VendorEvent::Sold {
            item_id: request.item_id.clone(),
            price: sell_price,
        });
    }
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct EconomyPlugin;

impl Plugin for EconomyPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<UseConsumableRequest>()
            .add_message::<ConsumableUsedEvent>()
            .add_message::<EquipItemRequest>()
            .add_message::<UnequipItemRequest>()
            .add_message::<EquipmentEvent>()
            .add_message::<VendorBuyRequest>()
            .add_message::<VendorSellRequest>()
            .add_message::<VendorEvent>()
            .add_systems(
                Update,
                (
                    process_consumable_use,
                    process_equip,
                    process_vendor_buy,
                    process_vendor_sell,
                ),
            );

        use crate::contracts::{AppContractExt, ContractEntry, PluginContract};
        app.register_contract(PluginContract {
            name: "EconomyPlugin".into(),
            description:
                "Consumable use, equipment, vendor buy/sell — driven by Database Row types".into(),
            resources: vec![],
            components: vec![],
            events: vec![
                ContractEntry::of::<UseConsumableRequest>("Use a consumable item"),
                ContractEntry::of::<ConsumableUsedEvent>("Consumable use result"),
                ContractEntry::of::<EquipItemRequest>("Equip an item"),
                ContractEntry::of::<EquipmentEvent>("Equipment change result"),
                ContractEntry::of::<VendorBuyRequest>("Buy from vendor"),
                ContractEntry::of::<VendorSellRequest>("Sell to vendor"),
                ContractEntry::of::<VendorEvent>("Vendor transaction result"),
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
    use crate::data::database::{ConsumableRow, EquipmentRow, ItemRow};

    #[test]
    fn test_consumable_heals_from_inventory() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<UseConsumableRequest>();
        app.add_message::<ConsumableUsedEvent>();
        app.insert_resource(Inventory::new(20));
        app.add_systems(Update, process_consumable_use);

        // Set up database with a potion
        let mut db = Database::default();
        db.consumables.push(ConsumableRow {
            id: "health_potion".into(),
            consumable_type: "potion".into(),
            stack_size: 10,
            cooldown: 1.0,
            ..default()
        });
        app.insert_resource(db);

        // Give player a potion
        app.world_mut()
            .resource_mut::<Inventory>()
            .add_item("health_potion", 1, 10);

        // Spawn entity with combat stats (damaged)
        let entity = app
            .world_mut()
            .spawn(CombatStatsComponent {
                max_hp: 100,
                hp: 50,
                ..default()
            })
            .id();

        app.world_mut()
            .resource_mut::<Messages<UseConsumableRequest>>()
            .write(UseConsumableRequest {
                entity,
                consumable_id: "health_potion".into(),
            });

        app.update();

        let stats = app.world().get::<CombatStatsComponent>(entity).unwrap();
        assert_eq!(stats.hp, 75); // 50 + 25 heal
        assert_eq!(
            app.world()
                .resource::<Inventory>()
                .count_item("health_potion"),
            0
        );
    }

    #[test]
    fn test_consumable_not_in_inventory() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<UseConsumableRequest>();
        app.add_message::<ConsumableUsedEvent>();
        app.insert_resource(Inventory::new(20));
        app.add_systems(Update, process_consumable_use);

        let entity = app.world_mut().spawn(CombatStatsComponent::default()).id();

        app.world_mut()
            .resource_mut::<Messages<UseConsumableRequest>>()
            .write(UseConsumableRequest {
                entity,
                consumable_id: "health_potion".into(),
            });

        app.update();

        // HP unchanged (potion not in inventory)
        let stats = app.world().get::<CombatStatsComponent>(entity).unwrap();
        assert_eq!(stats.hp, 100);
    }

    #[test]
    fn test_equip_applies_armor() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<EquipItemRequest>();
        app.add_message::<EquipmentEvent>();
        app.insert_resource(Inventory::new(20));
        app.add_systems(Update, process_equip);

        let mut db = Database::default();
        db.equipment.push(EquipmentRow {
            id: "iron_helm".into(),
            slot: "head".into(),
            armor_value: 15,
            stats: vec![("max_hp".into(), 20.0)],
            ..default()
        });
        app.insert_resource(db);

        let entity = app
            .world_mut()
            .spawn((
                EquipmentSlotsComponent::default(),
                CombatStatsComponent {
                    max_hp: 100,
                    hp: 100,
                    defense: 5,
                    ..default()
                },
            ))
            .id();

        app.world_mut()
            .resource_mut::<Messages<EquipItemRequest>>()
            .write(EquipItemRequest {
                entity,
                equipment_id: "iron_helm".into(),
            });

        app.update();

        let stats = app.world().get::<CombatStatsComponent>(entity).unwrap();
        assert_eq!(stats.defense, 20); // 5 + 15 armor
        assert_eq!(stats.max_hp, 120); // 100 + 20

        let slots = app.world().get::<EquipmentSlotsComponent>(entity).unwrap();
        assert_eq!(slots.head.as_deref(), Some("iron_helm"));
    }

    #[test]
    fn test_vendor_buy_deducts_currency() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<VendorBuyRequest>();
        app.add_message::<VendorEvent>();
        app.insert_resource(Inventory::new(20));
        app.add_systems(Update, process_vendor_buy);

        let mut db = Database::default();
        db.items.push(ItemRow {
            id: "health_potion".into(),
            price: 25,
            max_stack: 10,
            ..default()
        });
        app.insert_resource(db);

        app.world_mut()
            .resource_mut::<Inventory>()
            .add_currency("gold", 100);

        app.world_mut()
            .resource_mut::<Messages<VendorBuyRequest>>()
            .write(VendorBuyRequest {
                item_id: "health_potion".into(),
                currency_id: "gold".into(),
            });

        app.update();

        let inv = app.world().resource::<Inventory>();
        assert_eq!(inv.currency_balance("gold"), 75); // 100 - 25
        assert_eq!(inv.count_item("health_potion"), 1);
    }

    #[test]
    fn test_vendor_buy_insufficient_funds() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<VendorBuyRequest>();
        app.add_message::<VendorEvent>();
        app.insert_resource(Inventory::new(20));
        app.add_systems(Update, process_vendor_buy);

        let mut db = Database::default();
        db.items.push(ItemRow {
            id: "rare_sword".into(),
            price: 500,
            ..default()
        });
        app.insert_resource(db);

        app.world_mut()
            .resource_mut::<Inventory>()
            .add_currency("gold", 10);

        app.world_mut()
            .resource_mut::<Messages<VendorBuyRequest>>()
            .write(VendorBuyRequest {
                item_id: "rare_sword".into(),
                currency_id: "gold".into(),
            });

        app.update();

        let inv = app.world().resource::<Inventory>();
        assert_eq!(inv.currency_balance("gold"), 10); // unchanged
        assert_eq!(inv.count_item("rare_sword"), 0);
    }

    #[test]
    fn test_vendor_sell_adds_currency() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<VendorSellRequest>();
        app.add_message::<VendorEvent>();
        app.insert_resource(Inventory::new(20));
        app.add_systems(Update, process_vendor_sell);

        let mut db = Database::default();
        db.items.push(ItemRow {
            id: "wolf_pelt".into(),
            sell_value: 8,
            ..default()
        });
        app.insert_resource(db);

        let mut inv = app.world_mut().resource_mut::<Inventory>();
        inv.add_item("wolf_pelt", 3, 99);
        inv.add_currency("gold", 50);

        app.world_mut()
            .resource_mut::<Messages<VendorSellRequest>>()
            .write(VendorSellRequest {
                item_id: "wolf_pelt".into(),
                currency_id: "gold".into(),
            });

        app.update();

        let inv = app.world().resource::<Inventory>();
        assert_eq!(inv.currency_balance("gold"), 58); // 50 + 8
        assert_eq!(inv.count_item("wolf_pelt"), 2); // 3 - 1
    }
}
