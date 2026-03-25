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
        let mut app = App::new();
        let buffer = LuaCommandBuffer::default();

        // Push a spawn command into the buffer
        {
            let mut cmds = buffer.commands.lock().unwrap();
            cmds.push(LuaEcsCommand::SpawnEntity {
                prefab_id: "hero".into(),
            });
        }

        app.insert_resource(buffer);
        app.add_systems(Update, process_lua_commands);
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
        let mut app = App::new();
        let buffer = LuaCommandBuffer::default();

        // Spawn an entity with a Transform
        app.add_plugins(MinimalPlugins);
        app.insert_resource(buffer.clone());
        app.add_systems(Update, process_lua_commands);
        app.update(); // initialize

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
        let mut app = App::new();
        let buffer = LuaCommandBuffer::default();

        app.add_plugins(MinimalPlugins);
        app.insert_resource(buffer.clone());
        app.add_systems(Update, process_lua_commands);
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
        let mut app = App::new();
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

        app.insert_resource(buffer.clone());
        app.add_systems(Update, process_lua_commands);
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
}
