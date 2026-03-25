//! Scripting system for DJ Engine.
//!
//! Provides Lua integration via mlua. Games extend this with their own APIs.

use bevy::prelude::*;

pub mod context;
pub mod ecs_bridge;
pub mod ffi;

pub use context::LuaContext;
pub use ecs_bridge::{LuaCommandBuffer, LuaEcsCommand, LuaQueryResults};
pub use ffi::{
    create_shared_state, register_core_api, register_generic_state_api, GenericStateBuffer,
    SharedGenericState,
};

/// Events for script control.
#[derive(Message, Debug, Clone, Reflect)]
pub enum ScriptCommand {
    /// Load and execute a Lua script from file
    Load { path: String },
}

/// Scripting plugin that provides the Lua runtime.
pub struct DJScriptingPlugin;

impl Plugin for DJScriptingPlugin {
    fn build(&self, app: &mut App) {
        let lua_ctx = LuaContext::new();
        let lua_cmd_buffer = LuaCommandBuffer::default();

        // Register core APIs (log, warn, error) and ECS bridge
        {
            let lua = lua_ctx.lua.lock().unwrap();
            if let Err(e) = ffi::register_core_api(&lua) {
                error!("Failed to register core Lua API: {}", e);
            }
            if let Err(e) = ecs_bridge::register_ecs_bridge(&lua, lua_cmd_buffer.clone()) {
                error!("Failed to register Lua ECS bridge: {}", e);
            }
        }

        let lua_query_results = ecs_bridge::LuaQueryResults::default();

        app.insert_resource(lua_ctx)
            .insert_resource(lua_cmd_buffer)
            .insert_resource(lua_query_results)
            .register_type::<ScriptCommand>()
            .add_message::<ScriptCommand>()
            .add_systems(
                Update,
                (
                    handle_script_commands,
                    ecs_bridge::process_lua_commands,
                    ecs_bridge::sync_lua_query_results,
                ),
            );

        use crate::contracts::{AppContractExt, ContractEntry, PluginContract};
        app.register_contract(PluginContract {
            name: "DJScriptingPlugin".into(),
            description: "Lua 5.4 runtime via mlua with FFI bridge".into(),
            resources: vec![
                ContractEntry::of::<LuaContext>("Thread-safe Lua context"),
                ContractEntry::of::<LuaCommandBuffer>("Lua-ECS command buffer"),
            ],
            components: vec![],
            events: vec![ContractEntry::of::<ScriptCommand>(
                "Script load/execute commands",
            )],
            system_sets: vec![],
        });

        info!("DJ Scripting Plugin initialized");
    }
}

/// System that processes script commands.
fn handle_script_commands(lua_ctx: Res<LuaContext>, mut events: MessageReader<ScriptCommand>) {
    for event in events.read() {
        match event {
            ScriptCommand::Load { path } => {
                info!("Scripting: Loading script from {}", path);
                let lua = lua_ctx.lua.lock().unwrap();

                let result: mlua::Result<()> = (|| {
                    let script = std::fs::read_to_string(path)?;
                    lua.load(&script).exec()
                })();

                if let Err(e) = result {
                    error!("Failed to execute script {}: {}", path, e);
                }
            }
        }
    }
}
