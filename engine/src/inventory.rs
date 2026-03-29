//! Inventory system for DJ Engine.
//!
//! Manages the player's item bag with stack limits, add/remove/transfer,
//! and equipment slot management. Games use [`Inventory`] as a Bevy Resource
//! and react to [`InventoryEvent`] for UI updates.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A single item stack in the inventory.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ItemStack {
    pub item_id: String,
    pub quantity: u32,
    pub max_stack: u32,
}

impl ItemStack {
    pub fn new(item_id: impl Into<String>, quantity: u32, max_stack: u32) -> Self {
        Self {
            item_id: item_id.into(),
            quantity: quantity.min(max_stack),
            max_stack,
        }
    }

    pub fn is_full(&self) -> bool {
        self.quantity >= self.max_stack
    }

    pub fn space_remaining(&self) -> u32 {
        self.max_stack.saturating_sub(self.quantity)
    }
}

/// Player inventory resource.
#[derive(Resource, Default, Debug, Clone, Serialize, Deserialize, Reflect)]
#[reflect(Resource)]
pub struct Inventory {
    /// Maximum number of slots in the inventory.
    pub capacity: usize,
    /// Item stacks (sparse — None = empty slot).
    #[reflect(ignore)]
    pub slots: Vec<Option<ItemStack>>,
    /// Currency balances (currency_id -> amount).
    #[reflect(ignore)]
    pub currencies: HashMap<String, u64>,
}

impl Inventory {
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            slots: vec![None; capacity],
            currencies: HashMap::new(),
        }
    }

    /// Add items to the inventory. Returns the number of items that couldn't fit.
    pub fn add_item(&mut self, item_id: &str, mut quantity: u32, max_stack: u32) -> u32 {
        // First try to stack onto existing stacks
        for slot in &mut self.slots {
            if quantity == 0 {
                break;
            }
            if let Some(stack) = slot {
                if stack.item_id == item_id && !stack.is_full() {
                    let can_add = stack.space_remaining().min(quantity);
                    stack.quantity += can_add;
                    quantity -= can_add;
                }
            }
        }

        // Then fill empty slots
        for slot in &mut self.slots {
            if quantity == 0 {
                break;
            }
            if slot.is_none() {
                let take = quantity.min(max_stack);
                *slot = Some(ItemStack::new(item_id, take, max_stack));
                quantity -= take;
            }
        }

        quantity // leftover that didn't fit
    }

    /// Remove items from the inventory. Returns the number actually removed.
    pub fn remove_item(&mut self, item_id: &str, mut quantity: u32) -> u32 {
        let mut removed = 0u32;
        for slot in &mut self.slots {
            if quantity == 0 {
                break;
            }
            if let Some(stack) = slot {
                if stack.item_id == item_id {
                    let take = stack.quantity.min(quantity);
                    stack.quantity -= take;
                    quantity -= take;
                    removed += take;
                    if stack.quantity == 0 {
                        *slot = None;
                    }
                }
            }
        }
        removed
    }

    /// Count total quantity of an item across all stacks.
    pub fn count_item(&self, item_id: &str) -> u32 {
        self.slots
            .iter()
            .filter_map(|s| s.as_ref())
            .filter(|s| s.item_id == item_id)
            .map(|s| s.quantity)
            .sum()
    }

    /// Check if the inventory has at least `quantity` of an item.
    pub fn has_item(&self, item_id: &str, quantity: u32) -> bool {
        self.count_item(item_id) >= quantity
    }

    /// Number of occupied slots.
    pub fn used_slots(&self) -> usize {
        self.slots.iter().filter(|s| s.is_some()).count()
    }

    /// Number of empty slots.
    pub fn free_slots(&self) -> usize {
        self.capacity - self.used_slots()
    }

    /// Add currency.
    pub fn add_currency(&mut self, currency_id: &str, amount: u64) {
        *self.currencies.entry(currency_id.to_string()).or_insert(0) += amount;
    }

    /// Spend currency. Returns false if insufficient.
    pub fn spend_currency(&mut self, currency_id: &str, amount: u64) -> bool {
        let balance = self.currencies.entry(currency_id.to_string()).or_insert(0);
        if *balance >= amount {
            *balance -= amount;
            true
        } else {
            false
        }
    }

    /// Get currency balance.
    pub fn currency_balance(&self, currency_id: &str) -> u64 {
        self.currencies.get(currency_id).copied().unwrap_or(0)
    }
}

/// Events for inventory state changes.
#[derive(Message, Debug, Clone, PartialEq)]
pub enum InventoryEvent {
    ItemAdded {
        item_id: String,
        quantity: u32,
    },
    ItemRemoved {
        item_id: String,
        quantity: u32,
    },
    CurrencyChanged {
        currency_id: String,
        new_balance: u64,
    },
    InventoryFull,
}

/// Plugin providing inventory management.
pub struct InventoryPlugin;

impl Plugin for InventoryPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Inventory::new(20))
            .register_type::<Inventory>()
            .add_message::<InventoryEvent>();

        use crate::contracts::{AppContractExt, ContractEntry, PluginContract};
        app.register_contract(PluginContract {
            name: "InventoryPlugin".into(),
            description: "Player inventory with item stacking and currency".into(),
            resources: vec![ContractEntry::of::<Inventory>("Player inventory bag")],
            components: vec![],
            events: vec![ContractEntry::of::<InventoryEvent>(
                "Inventory state change events",
            )],
            system_sets: vec![],
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_item_to_empty_inventory() {
        let mut inv = Inventory::new(10);
        let leftover = inv.add_item("potion", 5, 10);
        assert_eq!(leftover, 0);
        assert_eq!(inv.count_item("potion"), 5);
        assert_eq!(inv.used_slots(), 1);
    }

    #[test]
    fn test_add_item_stacks_on_existing() {
        let mut inv = Inventory::new(10);
        inv.add_item("potion", 5, 10);
        inv.add_item("potion", 3, 10);
        assert_eq!(inv.count_item("potion"), 8);
        assert_eq!(inv.used_slots(), 1); // same stack
    }

    #[test]
    fn test_add_item_overflows_to_new_stack() {
        let mut inv = Inventory::new(10);
        inv.add_item("potion", 8, 10);
        inv.add_item("potion", 5, 10);
        assert_eq!(inv.count_item("potion"), 13);
        assert_eq!(inv.used_slots(), 2); // 10 + 3
    }

    #[test]
    fn test_add_item_full_inventory() {
        let mut inv = Inventory::new(2);
        inv.add_item("potion", 10, 10);
        inv.add_item("sword", 1, 1);
        let leftover = inv.add_item("shield", 1, 1);
        assert_eq!(leftover, 1); // no room
        assert_eq!(inv.free_slots(), 0);
    }

    #[test]
    fn test_remove_item() {
        let mut inv = Inventory::new(10);
        inv.add_item("potion", 5, 10);
        let removed = inv.remove_item("potion", 3);
        assert_eq!(removed, 3);
        assert_eq!(inv.count_item("potion"), 2);
    }

    #[test]
    fn test_remove_item_clears_slot() {
        let mut inv = Inventory::new(10);
        inv.add_item("potion", 3, 10);
        inv.remove_item("potion", 3);
        assert_eq!(inv.count_item("potion"), 0);
        assert_eq!(inv.used_slots(), 0);
    }

    #[test]
    fn test_remove_more_than_available() {
        let mut inv = Inventory::new(10);
        inv.add_item("potion", 3, 10);
        let removed = inv.remove_item("potion", 10);
        assert_eq!(removed, 3);
        assert_eq!(inv.count_item("potion"), 0);
    }

    #[test]
    fn test_has_item() {
        let mut inv = Inventory::new(10);
        inv.add_item("potion", 5, 10);
        assert!(inv.has_item("potion", 5));
        assert!(inv.has_item("potion", 1));
        assert!(!inv.has_item("potion", 6));
        assert!(!inv.has_item("sword", 1));
    }

    #[test]
    fn test_currency_add_and_spend() {
        let mut inv = Inventory::new(10);
        inv.add_currency("gold", 100);
        assert_eq!(inv.currency_balance("gold"), 100);

        assert!(inv.spend_currency("gold", 40));
        assert_eq!(inv.currency_balance("gold"), 60);

        assert!(!inv.spend_currency("gold", 100)); // insufficient
        assert_eq!(inv.currency_balance("gold"), 60); // unchanged
    }

    #[test]
    fn test_currency_default_zero() {
        let inv = Inventory::new(10);
        assert_eq!(inv.currency_balance("gold"), 0);
    }

    #[test]
    fn test_remove_across_multiple_stacks() {
        let mut inv = Inventory::new(10);
        inv.add_item("potion", 5, 5); // fills one stack
        inv.add_item("potion", 3, 5); // second partial stack
        assert_eq!(inv.count_item("potion"), 8);

        let removed = inv.remove_item("potion", 7);
        assert_eq!(removed, 7);
        assert_eq!(inv.count_item("potion"), 1);
    }

    #[test]
    fn test_add_overflow_creates_new_stacks() {
        let mut inv = Inventory::new(10);
        let leftover = inv.add_item("arrow", 15, 10);
        assert_eq!(leftover, 0); // all 15 fit (10 + 5 across 2 stacks)
        assert_eq!(inv.count_item("arrow"), 15);
    }

    #[test]
    fn test_remove_more_than_have() {
        let mut inv = Inventory::new(10);
        inv.add_item("gem", 3, 99);
        let removed = inv.remove_item("gem", 10);
        assert_eq!(removed, 3); // only had 3
        assert_eq!(inv.count_item("gem"), 0);
    }

    #[test]
    fn test_inventory_capacity_full() {
        let mut inv = Inventory::new(2);
        inv.add_item("a", 1, 1);
        inv.add_item("b", 1, 1);
        let leftover = inv.add_item("c", 1, 1);
        assert_eq!(leftover, 1); // no room
    }

    #[test]
    fn test_multiple_currencies() {
        let mut inv = Inventory::new(10);
        inv.add_currency("gold", 100);
        inv.add_currency("silver", 500);
        inv.add_currency("honor", 25);

        assert_eq!(inv.currency_balance("gold"), 100);
        assert_eq!(inv.currency_balance("silver"), 500);
        assert_eq!(inv.currency_balance("honor"), 25);
    }
}
