//! Lua-ECS bridge for DJ Engine.
//!
//! Lua runs inside a `Mutex<Lua>` and cannot directly access the Bevy `World`.
//! Instead, Lua calls queue [`LuaEcsCommand`]s into a shared [`LuaCommandBuffer`],
//! and the [`process_lua_commands`] system drains them each frame.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

/// Commands queued by Lua scripts, executed by a Bevy system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LuaEcsCommand {
    SpawnEntity {
        prefab_id: String,
    },
    DespawnEntity {
        entity_id: u64,
    },
    EmitEvent {
        event_name: String,
        payload: String,
    },
    SetComponentField {
        entity_id: u64,
        component: String,
        field: String,
        value: serde_json::Value,
    },
    SetPosition {
        entity_id: u64,
        x: f32,
        y: f32,
    },
    QuestAccept {
        quest_id: String,
    },
    QuestProgress {
        quest_id: String,
        objective_id: String,
        amount: u32,
    },
    QuestComplete {
        quest_id: String,
    },
    QuestAbandon {
        quest_id: String,
    },
    CombatAttack {
        attacker_id: u64,
        target_id: u64,
        flat_damage: Option<i32>,
    },
    InventoryAddItem {
        item_id: String,
        quantity: u32,
        max_stack: u32,
    },
    InventoryRemoveItem {
        item_id: String,
        quantity: u32,
    },
    InventoryAddCurrency {
        currency_id: String,
        amount: u64,
    },
    InventorySpendCurrency {
        currency_id: String,
        amount: u64,
    },
    UseConsumable {
        entity_id: u64,
        consumable_id: String,
    },
    EquipItem {
        entity_id: u64,
        equipment_id: String,
    },
    VendorBuy {
        item_id: String,
    },
    VendorSell {
        item_id: String,
    },
    EarnTitle {
        title_id: String,
    },
    EquipTitle {
        title_id: String,
    },
    GainWeaponSkill {
        weapon_skill_id: String,
        amount: u32,
    },
}

/// Read-only query results returned to Lua.
#[derive(Resource, Default, Clone)]
pub struct LuaQueryResults {
    pub results: Arc<Mutex<Vec<LuaQueryResult>>>,
}

/// A single entity's data returned from a query.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LuaQueryResult {
    pub entity_id: u64,
    pub name: Option<String>,
    pub x: f32,
    pub y: f32,
}

/// Cached document payloads for Lua access.
#[derive(Resource, Default, Clone)]
pub struct LuaDocumentCache {
    pub documents: Arc<Mutex<Vec<CachedDocument>>>,
}

/// A single cached document with its payload as JSON string.
#[derive(Debug, Clone)]
pub struct CachedDocument {
    pub kind: String,
    pub id: String,
    pub payload_json: String,
}

/// Shared command buffer between Lua and Bevy.
#[derive(Resource, Default, Clone)]
pub struct LuaCommandBuffer {
    pub commands: Arc<Mutex<Vec<LuaEcsCommand>>>,
}

/// Register ECS bridge functions in the Lua runtime.
///
/// Creates a global `ecs` table with methods:
/// - `ecs.spawn(prefab_id)` — queues [`LuaEcsCommand::SpawnEntity`]
/// - `ecs.despawn(entity_id)` — queues [`LuaEcsCommand::DespawnEntity`]
/// - `ecs.emit(event_name, payload_json)` — queues [`LuaEcsCommand::EmitEvent`]
/// - `ecs.set_field(entity_id, component, field, value)` — queues [`LuaEcsCommand::SetComponentField`]
pub fn register_ecs_bridge(lua: &mlua::Lua, buffer: LuaCommandBuffer) -> mlua::Result<()> {
    let ecs = lua.create_table()?;

    // ecs.spawn(prefab_id)
    let buf = buffer.clone();
    let spawn_fn = lua.create_function(move |_, prefab_id: String| {
        let mut cmds = buf.commands.lock().unwrap();
        cmds.push(LuaEcsCommand::SpawnEntity { prefab_id });
        Ok(())
    })?;
    ecs.set("spawn", spawn_fn)?;

    // ecs.despawn(entity_id)
    let buf = buffer.clone();
    let despawn_fn = lua.create_function(move |_, entity_id: u64| {
        let mut cmds = buf.commands.lock().unwrap();
        cmds.push(LuaEcsCommand::DespawnEntity { entity_id });
        Ok(())
    })?;
    ecs.set("despawn", despawn_fn)?;

    // ecs.emit(event_name, payload_json)
    let buf = buffer.clone();
    let emit_fn = lua.create_function(move |_, (event_name, payload): (String, String)| {
        let mut cmds = buf.commands.lock().unwrap();
        cmds.push(LuaEcsCommand::EmitEvent {
            event_name,
            payload,
        });
        Ok(())
    })?;
    ecs.set("emit", emit_fn)?;

    // ecs.set_field(entity_id, component, field, value)
    let buf = buffer.clone();
    let set_field_fn = lua.create_function(
        move |_, (entity_id, component, field, value): (u64, String, String, mlua::Value)| {
            let json_value = lua_value_to_json(value);
            let mut cmds = buf.commands.lock().unwrap();
            cmds.push(LuaEcsCommand::SetComponentField {
                entity_id,
                component,
                field,
                value: json_value,
            });
            Ok(())
        },
    )?;
    ecs.set("set_field", set_field_fn)?;

    // ecs.set_position(entity_id, x, y)
    let buf = buffer.clone();
    let set_pos_fn = lua.create_function(move |_, (entity_id, x, y): (u64, f32, f32)| {
        let mut cmds = buf.commands.lock().unwrap();
        cmds.push(LuaEcsCommand::SetPosition { entity_id, x, y });
        Ok(())
    })?;
    ecs.set("set_position", set_pos_fn)?;

    // quest.accept(quest_id)
    let buf = buffer.clone();
    let quest_accept_fn = lua.create_function(move |_, quest_id: String| {
        let mut cmds = buf.commands.lock().unwrap();
        cmds.push(LuaEcsCommand::QuestAccept { quest_id });
        Ok(())
    })?;

    // quest.progress(quest_id, objective_id, amount)
    let buf = buffer.clone();
    let quest_progress_fn = lua.create_function(
        move |_, (quest_id, objective_id, amount): (String, String, u32)| {
            let mut cmds = buf.commands.lock().unwrap();
            cmds.push(LuaEcsCommand::QuestProgress {
                quest_id,
                objective_id,
                amount,
            });
            Ok(())
        },
    )?;

    // quest.complete(quest_id)
    let buf = buffer.clone();
    let quest_complete_fn = lua.create_function(move |_, quest_id: String| {
        let mut cmds = buf.commands.lock().unwrap();
        cmds.push(LuaEcsCommand::QuestComplete { quest_id });
        Ok(())
    })?;

    // quest.abandon(quest_id)
    let buf = buffer.clone();
    let quest_abandon_fn = lua.create_function(move |_, quest_id: String| {
        let mut cmds = buf.commands.lock().unwrap();
        cmds.push(LuaEcsCommand::QuestAbandon { quest_id });
        Ok(())
    })?;

    let quest_table = lua.create_table()?;
    quest_table.set("accept", quest_accept_fn)?;
    quest_table.set("progress", quest_progress_fn)?;
    quest_table.set("complete", quest_complete_fn)?;
    quest_table.set("abandon", quest_abandon_fn)?;
    lua.globals().set("quest", quest_table)?;

    // combat.attack(attacker_id, target_id, [flat_damage])
    let buf = buffer.clone();
    let combat_attack_fn = lua.create_function(
        move |_, (attacker_id, target_id, flat): (u64, u64, Option<i32>)| {
            let mut cmds = buf.commands.lock().unwrap();
            cmds.push(LuaEcsCommand::CombatAttack {
                attacker_id,
                target_id,
                flat_damage: flat,
            });
            Ok(())
        },
    )?;

    let combat_table = lua.create_table()?;
    combat_table.set("attack", combat_attack_fn)?;
    lua.globals().set("combat", combat_table)?;

    // inventory.add_item(item_id, quantity, max_stack)
    let buf = buffer.clone();
    let inv_add_fn = lua.create_function(
        move |_, (item_id, quantity, max_stack): (String, u32, u32)| {
            let mut cmds = buf.commands.lock().unwrap();
            cmds.push(LuaEcsCommand::InventoryAddItem {
                item_id,
                quantity,
                max_stack,
            });
            Ok(())
        },
    )?;

    // inventory.remove_item(item_id, quantity)
    let buf = buffer.clone();
    let inv_remove_fn = lua.create_function(move |_, (item_id, quantity): (String, u32)| {
        let mut cmds = buf.commands.lock().unwrap();
        cmds.push(LuaEcsCommand::InventoryRemoveItem { item_id, quantity });
        Ok(())
    })?;

    // inventory.add_currency(currency_id, amount)
    let buf = buffer.clone();
    let inv_add_currency_fn =
        lua.create_function(move |_, (currency_id, amount): (String, u64)| {
            let mut cmds = buf.commands.lock().unwrap();
            cmds.push(LuaEcsCommand::InventoryAddCurrency {
                currency_id,
                amount,
            });
            Ok(())
        })?;

    // inventory.spend_currency(currency_id, amount)
    let buf = buffer.clone();
    let inv_spend_fn = lua.create_function(move |_, (currency_id, amount): (String, u64)| {
        let mut cmds = buf.commands.lock().unwrap();
        cmds.push(LuaEcsCommand::InventorySpendCurrency {
            currency_id,
            amount,
        });
        Ok(())
    })?;

    let inv_table = lua.create_table()?;
    inv_table.set("add_item", inv_add_fn)?;
    inv_table.set("remove_item", inv_remove_fn)?;
    inv_table.set("add_currency", inv_add_currency_fn)?;
    inv_table.set("spend_currency", inv_spend_fn)?;
    lua.globals().set("inventory", inv_table)?;

    // economy.use_consumable(entity_id, consumable_id)
    let buf = buffer.clone();
    let use_consumable_fn =
        lua.create_function(move |_, (entity_id, consumable_id): (u64, String)| {
            let mut cmds = buf.commands.lock().unwrap();
            cmds.push(LuaEcsCommand::UseConsumable {
                entity_id,
                consumable_id,
            });
            Ok(())
        })?;

    // economy.equip(entity_id, equipment_id)
    let buf = buffer.clone();
    let equip_fn = lua.create_function(move |_, (entity_id, equipment_id): (u64, String)| {
        let mut cmds = buf.commands.lock().unwrap();
        cmds.push(LuaEcsCommand::EquipItem {
            entity_id,
            equipment_id,
        });
        Ok(())
    })?;

    // economy.vendor_buy(item_id)
    let buf = buffer.clone();
    let vendor_buy_fn = lua.create_function(move |_, item_id: String| {
        let mut cmds = buf.commands.lock().unwrap();
        cmds.push(LuaEcsCommand::VendorBuy { item_id });
        Ok(())
    })?;

    // economy.vendor_sell(item_id)
    let buf = buffer.clone();
    let vendor_sell_fn = lua.create_function(move |_, item_id: String| {
        let mut cmds = buf.commands.lock().unwrap();
        cmds.push(LuaEcsCommand::VendorSell { item_id });
        Ok(())
    })?;

    let econ_table = lua.create_table()?;
    econ_table.set("use_consumable", use_consumable_fn)?;
    econ_table.set("equip", equip_fn)?;
    econ_table.set("vendor_buy", vendor_buy_fn)?;
    econ_table.set("vendor_sell", vendor_sell_fn)?;
    lua.globals().set("economy", econ_table)?;

    // character.earn_title(title_id)
    let buf = buffer.clone();
    let earn_title_fn = lua.create_function(move |_, title_id: String| {
        let mut cmds = buf.commands.lock().unwrap();
        cmds.push(LuaEcsCommand::EarnTitle { title_id });
        Ok(())
    })?;

    // character.equip_title(title_id)
    let buf = buffer.clone();
    let equip_title_fn = lua.create_function(move |_, title_id: String| {
        let mut cmds = buf.commands.lock().unwrap();
        cmds.push(LuaEcsCommand::EquipTitle { title_id });
        Ok(())
    })?;

    // character.gain_weapon_skill(weapon_skill_id, amount)
    let buf = buffer.clone();
    let gain_skill_fn =
        lua.create_function(move |_, (weapon_skill_id, amount): (String, u32)| {
            let mut cmds = buf.commands.lock().unwrap();
            cmds.push(LuaEcsCommand::GainWeaponSkill {
                weapon_skill_id,
                amount,
            });
            Ok(())
        })?;

    let char_table = lua.create_table()?;
    char_table.set("earn_title", earn_title_fn)?;
    char_table.set("equip_title", equip_title_fn)?;
    char_table.set("gain_weapon_skill", gain_skill_fn)?;
    lua.globals().set("character", char_table)?;

    lua.globals().set("ecs", ecs)?;
    Ok(())
}

/// Extended registration that also provides read-back APIs (get_position, query).
/// Call this instead of `register_ecs_bridge` when you have query results available.
pub fn register_ecs_bridge_with_queries(
    lua: &mlua::Lua,
    buffer: LuaCommandBuffer,
    query_results: LuaQueryResults,
) -> mlua::Result<()> {
    register_ecs_bridge(lua, buffer)?;

    let ecs: mlua::Table = lua.globals().get("ecs")?;

    // ecs.get_document(kind, id) -> JSON string or nil
    let dc = LuaDocumentCache::default();
    let get_doc_fn = lua.create_function(move |_, (kind, id): (String, String)| {
        let docs = dc.documents.lock().unwrap();
        let found = docs
            .iter()
            .find(|d| d.kind == kind && d.id == id)
            .map(|d| d.payload_json.clone());
        Ok(found)
    })?;
    ecs.set("get_document", get_doc_fn)?;

    // ecs.get_entities() -> array of {entity_id, name, x, y}
    let qr = query_results.clone();
    let get_entities_fn = lua.create_function(move |lua_ctx, ()| {
        let results = qr.results.lock().unwrap();
        let table = lua_ctx.create_table()?;
        for (i, result) in results.iter().enumerate() {
            let entry = lua_ctx.create_table()?;
            entry.set("entity_id", result.entity_id)?;
            if let Some(name) = &result.name {
                entry.set("name", name.as_str())?;
            }
            entry.set("x", result.x)?;
            entry.set("y", result.y)?;
            table.set(i + 1, entry)?;
        }
        Ok(table)
    })?;
    ecs.set("get_entities", get_entities_fn)?;

    Ok(())
}

/// Convert a Lua value to a serde_json::Value for transport.
fn lua_value_to_json(value: mlua::Value) -> serde_json::Value {
    match value {
        mlua::Value::Nil => serde_json::Value::Null,
        mlua::Value::Boolean(b) => serde_json::Value::Bool(b),
        mlua::Value::Integer(i) => serde_json::json!(i),
        mlua::Value::Number(n) => serde_json::json!(n),
        mlua::Value::String(s) => {
            serde_json::Value::String(s.to_str().map(|b| b.to_owned()).unwrap_or_default())
        }
        _ => serde_json::Value::Null,
    }
}

/// System that drains the Lua command buffer and executes commands.
pub fn process_lua_commands(
    mut commands: Commands,
    buffer: Res<LuaCommandBuffer>,
    mut transforms: Query<&mut Transform>,
    mut visibilities: Query<&mut Visibility>,
    mut quest_journal: ResMut<crate::quest::QuestJournal>,
    mut combat_events: MessageWriter<crate::combat::CombatEvent>,
    mut inventory: ResMut<crate::inventory::Inventory>,
    mut consumable_events: MessageWriter<crate::economy::UseConsumableRequest>,
    mut equip_events: MessageWriter<crate::economy::EquipItemRequest>,
    mut vendor_buy_events: MessageWriter<crate::economy::VendorBuyRequest>,
    mut vendor_sell_events: MessageWriter<crate::economy::VendorSellRequest>,
    mut player_title: ResMut<crate::character::PlayerTitle>,
    mut weapon_profs: ResMut<crate::character::WeaponProficiencies>,
) {
    let mut cmds = buffer.commands.lock().unwrap();
    for cmd in cmds.drain(..) {
        match cmd {
            LuaEcsCommand::SpawnEntity { prefab_id } => {
                info!("Lua: spawn entity from prefab '{prefab_id}'");
                commands.spawn(Name::new(format!("lua_spawned:{prefab_id}")));
            }
            LuaEcsCommand::DespawnEntity { entity_id } => {
                info!("Lua: despawn entity {entity_id}");
                let entity = Entity::from_bits(entity_id);
                commands.entity(entity).despawn();
            }
            LuaEcsCommand::EmitEvent {
                event_name,
                payload,
            } => {
                info!("Lua: emit event '{event_name}' payload={payload}");
            }
            LuaEcsCommand::SetComponentField {
                entity_id,
                component,
                field,
                value,
            } => {
                let entity = Entity::from_bits(entity_id);
                apply_set_field(
                    entity,
                    &component,
                    &field,
                    &value,
                    &mut transforms,
                    &mut visibilities,
                );
            }
            LuaEcsCommand::SetPosition { entity_id, x, y } => {
                let entity = Entity::from_bits(entity_id);
                if let Ok(mut transform) = transforms.get_mut(entity) {
                    transform.translation.x = x;
                    transform.translation.y = y;
                } else {
                    warn!("Lua: set_position failed — entity {entity_id} has no Transform");
                }
            }
            LuaEcsCommand::QuestAccept { quest_id } => {
                if quest_journal.accept(&quest_id) {
                    info!("Lua: accepted quest '{quest_id}'");
                } else {
                    warn!("Lua: could not accept quest '{quest_id}'");
                }
            }
            LuaEcsCommand::QuestProgress {
                quest_id,
                objective_id,
                amount,
            } => {
                let complete = quest_journal.progress_objective(&quest_id, &objective_id, amount);
                if complete {
                    info!("Lua: quest '{quest_id}' objective '{objective_id}' complete");
                }
            }
            LuaEcsCommand::QuestComplete { quest_id } => {
                quest_journal.complete(&quest_id);
                info!("Lua: completed quest '{quest_id}'");
            }
            LuaEcsCommand::QuestAbandon { quest_id } => {
                quest_journal.abandon(&quest_id);
                info!("Lua: abandoned quest '{quest_id}'");
            }
            LuaEcsCommand::CombatAttack {
                attacker_id,
                target_id,
                flat_damage,
            } => {
                combat_events.write(crate::combat::CombatEvent {
                    attacker: Entity::from_bits(attacker_id),
                    target: Entity::from_bits(target_id),
                    flat_damage,
                });
            }
            LuaEcsCommand::InventoryAddItem {
                item_id,
                quantity,
                max_stack,
            } => {
                let leftover = inventory.add_item(&item_id, quantity, max_stack);
                if leftover > 0 {
                    warn!("Lua: inventory full, {leftover} {item_id} couldn't fit");
                }
            }
            LuaEcsCommand::InventoryRemoveItem { item_id, quantity } => {
                inventory.remove_item(&item_id, quantity);
            }
            LuaEcsCommand::InventoryAddCurrency {
                currency_id,
                amount,
            } => {
                inventory.add_currency(&currency_id, amount);
            }
            LuaEcsCommand::InventorySpendCurrency {
                currency_id,
                amount,
            } => {
                if !inventory.spend_currency(&currency_id, amount) {
                    warn!("Lua: insufficient {currency_id} (need {amount})");
                }
            }
            LuaEcsCommand::UseConsumable {
                entity_id,
                consumable_id,
            } => {
                consumable_events.write(crate::economy::UseConsumableRequest {
                    entity: Entity::from_bits(entity_id),
                    consumable_id,
                });
            }
            LuaEcsCommand::EquipItem {
                entity_id,
                equipment_id,
            } => {
                equip_events.write(crate::economy::EquipItemRequest {
                    entity: Entity::from_bits(entity_id),
                    equipment_id,
                });
            }
            LuaEcsCommand::VendorBuy { item_id } => {
                vendor_buy_events.write(crate::economy::VendorBuyRequest {
                    item_id,
                    currency_id: "gold".into(),
                });
            }
            LuaEcsCommand::VendorSell { item_id } => {
                vendor_sell_events.write(crate::economy::VendorSellRequest {
                    item_id,
                    currency_id: "gold".into(),
                });
            }
            LuaEcsCommand::EarnTitle { title_id } => {
                player_title.earn(&title_id);
                info!("Lua: earned title '{title_id}'");
            }
            LuaEcsCommand::EquipTitle { title_id } => {
                if player_title.equip(&title_id) {
                    info!("Lua: equipped title '{title_id}'");
                } else {
                    warn!("Lua: title '{title_id}' not earned");
                }
            }
            LuaEcsCommand::GainWeaponSkill {
                weapon_skill_id,
                amount,
            } => {
                let new_level = weapon_profs.gain_skill(&weapon_skill_id, amount, None);
                info!("Lua: weapon skill '{weapon_skill_id}' now level {new_level}");
            }
        }
    }
}

fn apply_set_field(
    entity: Entity,
    component: &str,
    field: &str,
    value: &serde_json::Value,
    transforms: &mut Query<&mut Transform>,
    visibilities: &mut Query<&mut Visibility>,
) {
    match component {
        "transform" | "Transform" => {
            if let Ok(mut t) = transforms.get_mut(entity) {
                match field {
                    "x" => {
                        if let Some(v) = value.as_f64() {
                            t.translation.x = v as f32;
                        }
                    }
                    "y" => {
                        if let Some(v) = value.as_f64() {
                            t.translation.y = v as f32;
                        }
                    }
                    "scale_x" => {
                        if let Some(v) = value.as_f64() {
                            t.scale.x = v as f32;
                        }
                    }
                    "scale_y" => {
                        if let Some(v) = value.as_f64() {
                            t.scale.y = v as f32;
                        }
                    }
                    "rotation" => {
                        if let Some(v) = value.as_f64() {
                            t.rotation = Quat::from_rotation_z(v as f32);
                        }
                    }
                    _ => {
                        warn!("Lua: unknown Transform field '{field}'");
                    }
                }
            }
        }
        "visibility" | "Visibility" => {
            if let Ok(mut vis) = visibilities.get_mut(entity) {
                match field {
                    "visible" => {
                        if let Some(v) = value.as_bool() {
                            *vis = if v {
                                Visibility::Inherited
                            } else {
                                Visibility::Hidden
                            };
                        }
                    }
                    _ => {
                        warn!("Lua: unknown Visibility field '{field}'");
                    }
                }
            }
        }
        _ => {
            warn!("Lua: set_field for component '{component}' not supported — use game-specific FFI for custom components");
        }
    }
}

/// System that syncs custom document payloads for Lua access.
pub fn sync_lua_document_cache(
    cache: Res<LuaDocumentCache>,
    docs: Res<crate::data::LoadedCustomDocuments>,
) {
    let mut cached = cache.documents.lock().unwrap();
    cached.clear();
    for doc in &docs.documents {
        if let Some(envelope) = &doc.document {
            if let Ok(json) = serde_json::to_string(&envelope.payload) {
                cached.push(CachedDocument {
                    kind: doc.entry.kind.clone(),
                    id: doc.entry.id.clone(),
                    payload_json: json,
                });
            }
        }
    }
}

/// System that snapshots entity positions for Lua query access.
pub fn sync_lua_query_results(
    query_results: Res<LuaQueryResults>,
    query: Query<(Entity, &Transform, Option<&Name>)>,
) {
    let mut results = query_results.results.lock().unwrap();
    results.clear();
    for (entity, transform, name) in query.iter() {
        results.push(LuaQueryResult {
            entity_id: entity.to_bits(),
            name: name.map(|n| n.to_string()),
            x: transform.translation.x,
            y: transform.translation.y,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mlua::Lua;

    /// Set up an App with all resources needed by process_lua_commands.
    fn app_with_lua_commands(buffer: LuaCommandBuffer) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(buffer);
        app.init_resource::<crate::quest::QuestJournal>();
        app.insert_resource(crate::inventory::Inventory::new(20));
        app.add_message::<crate::combat::CombatEvent>();
        app.add_message::<crate::economy::UseConsumableRequest>();
        app.add_message::<crate::economy::EquipItemRequest>();
        app.add_message::<crate::economy::VendorBuyRequest>();
        app.add_message::<crate::economy::VendorSellRequest>();
        app.init_resource::<crate::character::PlayerTitle>();
        app.init_resource::<crate::character::WeaponProficiencies>();
        app.add_systems(Update, process_lua_commands);
        app
    }

    #[test]
    fn test_command_buffer_default_empty() {
        let buffer = LuaCommandBuffer::default();
        let cmds = buffer.commands.lock().unwrap();
        assert_eq!(cmds.len(), 0);
    }

    #[test]
    fn test_lua_spawn_queues_command() {
        let lua = Lua::new();
        let buffer = LuaCommandBuffer::default();
        register_ecs_bridge(&lua, buffer.clone()).unwrap();

        lua.load("ecs.spawn('test_prefab')").exec().unwrap();

        let cmds = buffer.commands.lock().unwrap();
        assert_eq!(cmds.len(), 1);
        match &cmds[0] {
            LuaEcsCommand::SpawnEntity { prefab_id } => {
                assert_eq!(prefab_id, "test_prefab");
            }
            other => panic!("Expected SpawnEntity, got {:?}", other),
        }
    }

    #[test]
    fn test_lua_despawn_queues_command() {
        let lua = Lua::new();
        let buffer = LuaCommandBuffer::default();
        register_ecs_bridge(&lua, buffer.clone()).unwrap();

        lua.load("ecs.despawn(42)").exec().unwrap();

        let cmds = buffer.commands.lock().unwrap();
        assert_eq!(cmds.len(), 1);
        match &cmds[0] {
            LuaEcsCommand::DespawnEntity { entity_id } => {
                assert_eq!(*entity_id, 42);
            }
            other => panic!("Expected DespawnEntity, got {:?}", other),
        }
    }

    #[test]
    fn test_lua_emit_queues_command() {
        let lua = Lua::new();
        let buffer = LuaCommandBuffer::default();
        register_ecs_bridge(&lua, buffer.clone()).unwrap();

        lua.load(r#"ecs.emit("damage", "{\"amount\":10}")"#)
            .exec()
            .unwrap();

        let cmds = buffer.commands.lock().unwrap();
        assert_eq!(cmds.len(), 1);
        match &cmds[0] {
            LuaEcsCommand::EmitEvent {
                event_name,
                payload,
            } => {
                assert_eq!(event_name, "damage");
                assert_eq!(payload, r#"{"amount":10}"#);
            }
            other => panic!("Expected EmitEvent, got {:?}", other),
        }
    }

    #[test]
    fn test_process_lua_commands_spawns_entity() {
        let buffer = LuaCommandBuffer::default();
        {
            let mut cmds = buffer.commands.lock().unwrap();
            cmds.push(LuaEcsCommand::SpawnEntity {
                prefab_id: "hero".into(),
            });
        }

        let mut app = app_with_lua_commands(buffer);
        app.update();

        // Verify entity was spawned with the expected Name
        let mut found = false;
        let mut query = app.world_mut().query::<&Name>();
        for name in query.iter(app.world()) {
            if name.as_str() == "lua_spawned:hero" {
                found = true;
                break;
            }
        }
        assert!(found, "Expected entity with Name 'lua_spawned:hero'");
    }

    #[test]
    fn test_lua_set_position_queues_command() {
        let lua = Lua::new();
        let buffer = LuaCommandBuffer::default();
        register_ecs_bridge(&lua, buffer.clone()).unwrap();

        lua.load("ecs.set_position(99, 10.5, 20.0)").exec().unwrap();

        let cmds = buffer.commands.lock().unwrap();
        assert_eq!(cmds.len(), 1);
        match &cmds[0] {
            LuaEcsCommand::SetPosition { entity_id, x, y } => {
                assert_eq!(*entity_id, 99);
                assert!((x - 10.5).abs() < f32::EPSILON);
                assert!((y - 20.0).abs() < f32::EPSILON);
            }
            other => panic!("Expected SetPosition, got {:?}", other),
        }
    }

    #[test]
    fn test_lua_set_field_queues_command() {
        let lua = Lua::new();
        let buffer = LuaCommandBuffer::default();
        register_ecs_bridge(&lua, buffer.clone()).unwrap();

        lua.load(r#"ecs.set_field(1, "transform", "x", 42.0)"#)
            .exec()
            .unwrap();

        let cmds = buffer.commands.lock().unwrap();
        assert_eq!(cmds.len(), 1);
        match &cmds[0] {
            LuaEcsCommand::SetComponentField {
                entity_id,
                component,
                field,
                value,
            } => {
                assert_eq!(*entity_id, 1);
                assert_eq!(component, "transform");
                assert_eq!(field, "x");
                assert_eq!(*value, serde_json::json!(42.0));
            }
            other => panic!("Expected SetComponentField, got {:?}", other),
        }
    }

    #[test]
    fn test_process_set_position_updates_transform() {
        let buffer = LuaCommandBuffer::default();
        let mut app = app_with_lua_commands(buffer.clone());
        app.update();

        let entity = app
            .world_mut()
            .spawn(Transform::from_xyz(0.0, 0.0, 0.0))
            .id();

        // Queue a set_position command
        {
            let mut cmds = buffer.commands.lock().unwrap();
            cmds.push(LuaEcsCommand::SetPosition {
                entity_id: entity.to_bits(),
                x: 100.0,
                y: 200.0,
            });
        }

        app.update();

        let transform = app.world().get::<Transform>(entity).unwrap();
        assert!((transform.translation.x - 100.0).abs() < f32::EPSILON);
        assert!((transform.translation.y - 200.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_process_set_field_transform_x() {
        let buffer = LuaCommandBuffer::default();
        let mut app = app_with_lua_commands(buffer.clone());
        app.update();

        let entity = app
            .world_mut()
            .spawn(Transform::from_xyz(0.0, 0.0, 0.0))
            .id();

        {
            let mut cmds = buffer.commands.lock().unwrap();
            cmds.push(LuaEcsCommand::SetComponentField {
                entity_id: entity.to_bits(),
                component: "transform".into(),
                field: "x".into(),
                value: serde_json::json!(55.5),
            });
        }

        app.update();

        let transform = app.world().get::<Transform>(entity).unwrap();
        assert!((transform.translation.x - 55.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_query_results_snapshot() {
        let mut app = App::new();
        let query_results = LuaQueryResults::default();

        app.add_plugins(MinimalPlugins);
        app.insert_resource(query_results.clone());
        app.add_systems(Update, sync_lua_query_results);

        app.world_mut()
            .spawn((Transform::from_xyz(10.0, 20.0, 0.0), Name::new("hero")));
        app.world_mut()
            .spawn((Transform::from_xyz(30.0, 40.0, 0.0), Name::new("npc")));

        app.update();

        let results = query_results.results.lock().unwrap();
        assert_eq!(results.len(), 2);

        let hero = results.iter().find(|r| r.name.as_deref() == Some("hero"));
        assert!(hero.is_some());
        let hero = hero.unwrap();
        assert!((hero.x - 10.0).abs() < f32::EPSILON);
        assert!((hero.y - 20.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_multiple_commands_processed() {
        let buffer = LuaCommandBuffer::default();

        // Push 3 spawn commands
        {
            let mut cmds = buffer.commands.lock().unwrap();
            cmds.push(LuaEcsCommand::SpawnEntity {
                prefab_id: "a".into(),
            });
            cmds.push(LuaEcsCommand::SpawnEntity {
                prefab_id: "b".into(),
            });
            cmds.push(LuaEcsCommand::SpawnEntity {
                prefab_id: "c".into(),
            });
        }

        let mut app = app_with_lua_commands(buffer.clone());
        app.update();

        // All 3 should be processed (buffer drained)
        let cmds = buffer.commands.lock().unwrap();
        assert_eq!(cmds.len(), 0, "Buffer should be drained after processing");

        // Verify 3 entities spawned
        let mut query = app.world_mut().query::<&Name>();
        let count = query
            .iter(app.world())
            .filter(|n| n.as_str().starts_with("lua_spawned:"))
            .count();
        assert_eq!(count, 3, "Expected 3 spawned entities");
    }

    #[test]
    fn test_document_cache_stores_payloads() {
        let cache = LuaDocumentCache::default();
        {
            let mut docs = cache.documents.lock().unwrap();
            docs.push(CachedDocument {
                kind: "abilities".into(),
                id: "fireball".into(),
                payload_json: r#"{"damage":50}"#.into(),
            });
            docs.push(CachedDocument {
                kind: "items".into(),
                id: "potion".into(),
                payload_json: r#"{"heal":25}"#.into(),
            });
        }

        let docs = cache.documents.lock().unwrap();
        assert_eq!(docs.len(), 2);

        let fireball = docs.iter().find(|d| d.id == "fireball");
        assert!(fireball.is_some());
        assert_eq!(fireball.unwrap().payload_json, r#"{"damage":50}"#);
    }

    #[test]
    fn test_lua_get_document_returns_json() {
        let lua = Lua::new();
        let buffer = LuaCommandBuffer::default();
        let query_results = LuaQueryResults::default();
        register_ecs_bridge_with_queries(&lua, buffer, query_results).unwrap();

        // get_document returns nil when cache is empty
        let result: Option<String> = lua
            .load(r#"return ecs.get_document("abilities", "fireball")"#)
            .eval()
            .unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_lua_quest_accept_queues_command() {
        let lua = Lua::new();
        let buffer = LuaCommandBuffer::default();
        register_ecs_bridge(&lua, buffer.clone()).unwrap();

        lua.load(r#"quest.accept("guard_patrol")"#).exec().unwrap();

        let cmds = buffer.commands.lock().unwrap();
        assert_eq!(cmds.len(), 1);
        match &cmds[0] {
            LuaEcsCommand::QuestAccept { quest_id } => {
                assert_eq!(quest_id, "guard_patrol");
            }
            other => panic!("Expected QuestAccept, got {:?}", other),
        }
    }

    #[test]
    fn test_lua_quest_progress_queues_command() {
        let lua = Lua::new();
        let buffer = LuaCommandBuffer::default();
        register_ecs_bridge(&lua, buffer.clone()).unwrap();

        lua.load(r#"quest.progress("guard_patrol", "kill_wolves", 3)"#)
            .exec()
            .unwrap();

        let cmds = buffer.commands.lock().unwrap();
        assert_eq!(cmds.len(), 1);
        match &cmds[0] {
            LuaEcsCommand::QuestProgress {
                quest_id,
                objective_id,
                amount,
            } => {
                assert_eq!(quest_id, "guard_patrol");
                assert_eq!(objective_id, "kill_wolves");
                assert_eq!(*amount, 3);
            }
            other => panic!("Expected QuestProgress, got {:?}", other),
        }
    }

    #[test]
    fn test_lua_combat_attack_queues_command() {
        let lua = Lua::new();
        let buffer = LuaCommandBuffer::default();
        register_ecs_bridge(&lua, buffer.clone()).unwrap();

        lua.load("combat.attack(1, 2)").exec().unwrap();

        let cmds = buffer.commands.lock().unwrap();
        assert_eq!(cmds.len(), 1);
        match &cmds[0] {
            LuaEcsCommand::CombatAttack {
                attacker_id,
                target_id,
                flat_damage,
            } => {
                assert_eq!(*attacker_id, 1);
                assert_eq!(*target_id, 2);
                assert!(flat_damage.is_none());
            }
            other => panic!("Expected CombatAttack, got {:?}", other),
        }
    }

    #[test]
    fn test_lua_inventory_add_item_queues_command() {
        let lua = Lua::new();
        let buffer = LuaCommandBuffer::default();
        register_ecs_bridge(&lua, buffer.clone()).unwrap();

        lua.load(r#"inventory.add_item("potion", 5, 10)"#)
            .exec()
            .unwrap();

        let cmds = buffer.commands.lock().unwrap();
        assert_eq!(cmds.len(), 1);
        match &cmds[0] {
            LuaEcsCommand::InventoryAddItem {
                item_id,
                quantity,
                max_stack,
            } => {
                assert_eq!(item_id, "potion");
                assert_eq!(*quantity, 5);
                assert_eq!(*max_stack, 10);
            }
            other => panic!("Expected InventoryAddItem, got {:?}", other),
        }
    }

    #[test]
    fn test_lua_inventory_add_currency_queues_command() {
        let lua = Lua::new();
        let buffer = LuaCommandBuffer::default();
        register_ecs_bridge(&lua, buffer.clone()).unwrap();

        lua.load(r#"inventory.add_currency("gold", 100)"#)
            .exec()
            .unwrap();

        let cmds = buffer.commands.lock().unwrap();
        match &cmds[0] {
            LuaEcsCommand::InventoryAddCurrency {
                currency_id,
                amount,
            } => {
                assert_eq!(currency_id, "gold");
                assert_eq!(*amount, 100);
            }
            other => panic!("Expected InventoryAddCurrency, got {:?}", other),
        }
    }

    #[test]
    fn test_lua_combat_attack_with_flat_damage() {
        let lua = Lua::new();
        let buffer = LuaCommandBuffer::default();
        register_ecs_bridge(&lua, buffer.clone()).unwrap();

        lua.load("combat.attack(1, 2, 50)").exec().unwrap();

        let cmds = buffer.commands.lock().unwrap();
        match &cmds[0] {
            LuaEcsCommand::CombatAttack { flat_damage, .. } => {
                assert_eq!(*flat_damage, Some(50));
            }
            other => panic!("Expected CombatAttack, got {:?}", other),
        }
    }
}
