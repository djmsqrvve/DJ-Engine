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

    lua.globals().set("ecs", ecs)?;
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
pub fn process_lua_commands(mut commands: Commands, buffer: Res<LuaCommandBuffer>) {
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
                // Events will be handled by game-specific systems
            }
            LuaEcsCommand::SetComponentField { .. } => {
                warn!("Lua: set_field not yet implemented for runtime components");
            }
        }
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
}
