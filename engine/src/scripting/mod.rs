//! Scripting system for DJ Engine.
//!
//! Provides Lua integration via mlua. Games extend this with their own APIs.

use bevy::prelude::*;

pub mod context;
pub mod ffi;

pub use context::LuaContext;
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

        // Register core APIs (log, warn, error)
        {
            let lua = lua_ctx.lua.lock().unwrap();
            if let Err(e) = ffi::register_core_api(&lua) {
                error!("Failed to register core Lua API: {}", e);
            }
        }

        app.insert_resource(lua_ctx)
            .register_type::<ScriptCommand>()
            .add_message::<ScriptCommand>()
            .add_systems(Update, handle_script_commands);

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
