# Game Developer's Guide

This guide explains how to build a game on DJ Engine. It covers every major runtime system with usage patterns and code examples. For a working reference, see `games/dev/rpg_demo/`.

## Quick Start

```bash
make new-game NAME="my_game"
```

This creates a project directory with `project.json`, `scenes/`, `story_graphs/`, and `data/registry.json`. Add DJEnginePlugin to your app:

```rust
use bevy::prelude::*;
use dj_engine::core::DJEnginePlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(DJEnginePlugin::default())
        .add_systems(Startup, setup)
        .run();
}
```

DJEnginePlugin bundles all engine systems. For fine-grained control, add individual plugins instead.

## Combat

Fire a `CombatEvent` to trigger damage calculation. The engine resolves damage using `CombatConfig` (configurable defense factor, crit multiplier, variance) and emits a `DamageEvent`.

```rust
use dj_engine::combat::{CombatEvent, DamageEvent, CombatConfig};

// Trigger an attack
fn attack_system(
    mut combat_events: MessageWriter<CombatEvent>,
    player: Query<Entity, With<Player>>,
    enemy: Query<Entity, With<Enemy>>,
) {
    let player = player.single().unwrap();
    let enemy = enemy.single().unwrap();
    combat_events.write(CombatEvent {
        attacker: player,
        target: enemy,
        flat_damage: None, // uses attacker's CombatStatsComponent
    });
}

// React to damage
fn on_damage(mut events: MessageReader<DamageEvent>) {
    for event in events.read() {
        println!("{} damage (crit={})", event.final_damage, event.is_critical);
        if event.target_defeated {
            println!("Enemy defeated!");
        }
    }
}
```

Entities need `CombatStatsComponent` (hp, damage, defense, crit_chance) for combat to resolve.

## Quests

`QuestJournal` is a Bevy Resource tracking quest lifecycle:

```rust
use dj_engine::quest::{QuestJournal, QuestStatus};

fn setup_quest(mut journal: ResMut<QuestJournal>) {
    journal.accept("slay_slimes");
    journal.add_objective("slay_slimes", "kill_slime", 3);
}

fn on_enemy_kill(mut journal: ResMut<QuestJournal>) {
    let complete = journal.progress_objective("slay_slimes", "kill_slime", 1);
    if complete {
        journal.complete("slay_slimes");
    }
}

fn check_status(journal: Res<QuestJournal>) {
    match journal.status("slay_slimes") {
        Some(QuestStatus::Completed) => { /* ready to turn in */ }
        Some(QuestStatus::Accepted) => { /* still in progress */ }
        _ => {}
    }
}
```

Lifecycle: Available -> Accepted -> InProgress -> Completed -> TurnedIn.

## Inventory

`Inventory` manages item stacks with configurable capacity and currency:

```rust
use dj_engine::inventory::Inventory;

fn loot_item(mut inventory: ResMut<Inventory>) {
    let leftover = inventory.add_item("health_potion", 3, 10); // id, qty, max_stack
    if leftover > 0 {
        println!("Inventory full! {} couldn't fit", leftover);
    }
}

fn buy_item(mut inventory: ResMut<Inventory>) {
    if inventory.spend_currency("gold", 25) {
        inventory.add_item("iron_sword", 1, 1);
    }
}

fn check_items(inventory: Res<Inventory>) {
    let potions = inventory.count_item("health_potion");
    let gold = inventory.currency_balance("gold");
    println!("{} potions, {} gold", potions, gold);
}
```

## NPC Interaction

Add `InteractionSource` to the player and `InteractivityComponent` to NPCs. The engine fires `InteractionEvent` when the player presses Confirm near an interactable entity.

```rust
use dj_engine::interaction::{InteractionEvent, InteractionSource};
use dj_engine::data::components::{InteractivityComponent, NpcComponent, TriggerType};

// Player setup
commands.spawn((Player, InteractionSource, Transform::default()));

// NPC setup
commands.spawn((
    InteractivityComponent {
        trigger_type: TriggerType::Npc,
        trigger_id: "shopkeeper".into(),
        ..default()
    },
    NpcComponent {
        npc_id: "shop_01".into(),
        dialogue_set_id: "shop_greeting".into(),
        ..default()
    },
    Transform::from_xyz(100.0, 0.0, 0.0),
));

// React to interaction
fn on_interact(mut events: MessageReader<InteractionEvent>) {
    for event in events.read() {
        match event.trigger_type {
            TriggerType::Npc => println!("Talk to NPC: {}", event.trigger_id),
            TriggerType::Door => println!("Open door: {}", event.trigger_id),
            TriggerType::Chest => println!("Open chest: {}", event.trigger_id),
            _ => {}
        }
    }
}
```

## Abilities

`UseAbilityRequest` validates mana cost and cooldown, then fires `CombatEvent` (damage) or applies healing:

```rust
use dj_engine::ability::UseAbilityRequest;

fn use_fireball(
    mut requests: MessageWriter<UseAbilityRequest>,
    player: Query<Entity, With<Player>>,
    target: Query<Entity, With<Enemy>>,
) {
    let player = player.single().unwrap();
    let target = target.single().unwrap();
    requests.write(UseAbilityRequest {
        caster: player,
        target: Some(target),
        ability_id: "fireball".into(),
        mana_cost: 15,
        cooldown: 3.0,
        damage: Some(40),
        heal: None,
        effect_id: None,
        effect_duration: None,
    });
}
```

The system checks mana, checks cooldown, deducts mana, starts cooldown, and fires the combat or heal effect. React to `AbilityUsedEvent` for feedback (Success, InsufficientMana, OnCooldown).

## Loot

When an enemy is defeated (`DamageEvent.target_defeated`), the engine automatically looks up `CombatStatsComponent.loot_table_id`, rolls the loot table, and adds items to `Inventory`. React to `LootDropEvent` for UI:

```rust
use dj_engine::loot::LootDropEvent;

fn on_loot(mut events: MessageReader<LootDropEvent>) {
    for event in events.read() {
        println!("Dropped: {} x{}", event.item_id, event.quantity);
    }
}
```

Set up loot tables in the `Database` resource at startup.

## Consumables

Use consumable items from Database definitions. `UseConsumableRequest` checks inventory, looks up `ConsumableRow`, removes the item, and applies the effect:

```rust
use dj_engine::economy::UseConsumableRequest;

fn use_potion(
    mut requests: MessageWriter<UseConsumableRequest>,
    player: Query<Entity, With<Player>>,
) {
    let player = player.single().unwrap();
    requests.write(UseConsumableRequest {
        entity: player,
        consumable_id: "health_potion".into(),
    });
}
```

React to `ConsumableUsedEvent` for feedback (Success/NotFound/NotInInventory).

## Equipment

Equip items from Database. `EquipItemRequest` looks up `EquipmentRow`, assigns to the correct slot, and modifies `CombatStatsComponent` with armor and stat bonuses:

```rust
use dj_engine::economy::EquipItemRequest;

fn equip_helm(
    mut requests: MessageWriter<EquipItemRequest>,
    player: Query<Entity, With<Player>>,
) {
    let player = player.single().unwrap();
    requests.write(EquipItemRequest {
        entity: player,
        equipment_id: "iron_helm".into(),
    });
}
```

The player entity needs `EquipmentSlotsComponent` and `CombatStatsComponent`. Previously equipped items are returned to inventory automatically.

## Vendor Trading

Buy and sell items using Database prices. `VendorBuyRequest` looks up price from `ItemRow.price` or `TradeGoodRow.vendor_price`, deducts currency, adds item. `VendorSellRequest` sells at `ItemRow.sell_value`:

```rust
use dj_engine::economy::{VendorBuyRequest, VendorSellRequest};

fn buy_from_vendor(mut requests: MessageWriter<VendorBuyRequest>) {
    requests.write(VendorBuyRequest {
        item_id: "health_potion".into(),
        currency_id: "gold".into(),
    });
}
```

## Titles

Players earn and display titles from Database `TitleRow` definitions. Titles have a style ("prefix" or "suffix") that formats the player name:

```rust
use dj_engine::character::PlayerTitle;

fn grant_title(mut title: ResMut<PlayerTitle>) {
    title.earn("champion");
    title.equip("champion");
}

fn display_name(title: Res<PlayerTitle>, database: Option<Res<Database>>) {
    let name = title.format_name("PlayerOne", database.as_deref());
    // → "Champion PlayerOne" (prefix) or "PlayerOne, the Brave" (suffix)
}
```

## Weapon Skills

Track weapon proficiency per type. `WeaponSkillRow` defines max skill level and class restrictions:

```rust
use dj_engine::character::WeaponProficiencies;

fn on_attack(mut profs: ResMut<WeaponProficiencies>, database: Option<Res<Database>>) {
    let new_level = profs.gain_skill("swords", 1, database.as_deref());
    // Caps at max_skill from Database (e.g., 300)
}

fn can_use_weapon(profs: Res<WeaponProficiencies>, database: Option<Res<Database>>) {
    if profs.can_class_use_weapon("swords", "warrior", database.as_deref()) {
        // warrior can use swords
    }
}
```

## Status Effects

`apply_effect` adds buff/debuffs with duration and stacks. `tick_status_effects` runs automatically and fires `StatusEffectExpired` when effects run out:

```rust
use dj_engine::status::{apply_effect, start_cooldown, StatusEffectExpired};
use dj_engine::data::components::{StatusEffectsComponent, AbilityCooldownsComponent};

fn apply_poison(mut query: Query<&mut StatusEffectsComponent>) {
    for mut effects in &mut query {
        apply_effect(&mut effects, "poison", 5.0, 1);
    }
}

fn on_effect_expired(mut events: MessageReader<StatusEffectExpired>) {
    for event in events.read() {
        println!("Effect '{}' expired on {:?}", event.effect_id, event.entity);
    }
}
```

## Sprite Animation

Attach `SpriteAnimationPlayer` for frame cycling:

```rust
use dj_engine::animation::SpriteAnimationPlayer;

commands.spawn((
    Sprite { /* with texture_atlas */ ..default() },
    SpriteAnimationPlayer::new(8, 1.0, true), // 8 frames, 1s cycle, looping
));
```

The engine ticks frames automatically and updates `Sprite.texture_atlas.index`.

## Lua Scripting

Eight Lua tables are available for gameplay scripting:

```lua
-- Quest management
quest.accept("guard_patrol")
quest.progress("guard_patrol", "kill_wolves", 1)
quest.complete("guard_patrol")
quest.abandon("guard_patrol")

-- Combat
combat.attack(attacker_entity_id, target_entity_id)
combat.attack(attacker_id, target_id, 50) -- flat damage override

-- Inventory
inventory.add_item("health_potion", 5, 10) -- id, qty, max_stack
inventory.remove_item("health_potion", 1)
inventory.add_currency("gold", 100)
inventory.spend_currency("gold", 25)

-- Economy (Database-driven)
economy.use_consumable(entity_id, "health_potion") -- look up ConsumableRow, apply effect
economy.equip(entity_id, "iron_helm")              -- look up EquipmentRow, modify stats
economy.vendor_buy("health_potion")                -- deduct gold, add item
economy.vendor_sell("wolf_pelt")                   -- remove item, add gold

-- Character progression
character.earn_title("champion")                   -- unlock title
character.equip_title("champion")                  -- set active display title
character.gain_weapon_skill("swords", 5)           -- increase proficiency

-- ECS access
ecs.set_position(entity_id, x, y)
ecs.set_field(entity_id, "transform", "x", 100.0)
local entities = ecs.get_entities() -- returns {entity_id, name, x, y}
local doc = ecs.get_document("abilities", "fireball") -- returns JSON string
```

## Custom Documents

Register game-specific data types for the editor and runtime:

```rust
use dj_engine::data::{AppCustomDocumentExt, CustomDocumentRegistration};

app.register_custom_document_kind(CustomDocumentRegistration {
    kind: "abilities".into(),
    display_name: "Abilities".into(),
    ..default()
});
```

Documents load from `data/<kind>/` in the mounted project and are accessible via `LoadedCustomDocuments` resource or the Lua `ecs.get_document()` API.

## Extension Points

Games can extend the editor with:

```rust
use dj_engine::editor::extensions::AppEditorExtensionExt;

// Custom toolbar button
app.register_toolbar_action(RegisteredToolbarAction {
    action_id: "reload_data".into(),
    title: "Reload Game Data".into(),
    kind_filter: None,
});

// Custom editor panel
app.register_custom_editor_panel(RegisteredCustomEditorPanel {
    kind: "abilities".into(),
    panel_id: "ability_editor".into(),
    title: "Ability Editor".into(),
});

// Preview preset
app.register_preview_preset(RegisteredPreviewPreset {
    preset_id: "combat_test".into(),
    title: "Combat Test".into(),
    profile_id: Some("combat_profile".into()),
});
```

## Import Pattern

Use explicit imports to avoid name conflicts (especially `Entity` from both Bevy and scene data):

```rust
// Recommended: explicit imports
use dj_engine::combat::{CombatEvent, DamageEvent};
use dj_engine::quest::{QuestJournal, QuestStatus};
use dj_engine::inventory::Inventory;

// Avoid: wildcard prelude (causes Entity ambiguity)
// use dj_engine::prelude::*;
```

## Next Steps

- See `games/dev/rpg_demo/` for a working reference implementation
- See `docs/tutorials/01-build-a-board-game/` for a step-by-step tutorial
- See `plugins/helix_data/` for a real plugin consuming external MMORPG data
- Run `make contracts` to see the full engine API surface
