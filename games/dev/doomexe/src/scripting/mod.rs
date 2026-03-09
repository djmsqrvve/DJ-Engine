//! Game-specific Lua scripting extensions for doomexe.
//!
//! Extends the engine's generic scripting with hamster-specific APIs.

use bevy::prelude::*;
use dj_engine::scripting::{create_shared_state, register_generic_state_api, LuaContext};
use mlua::prelude::*;
use std::sync::{Arc, RwLock};

use crate::types::{Expression, HamsterNarrator};

/// Game-specific scripting plugin that adds hamster APIs.
pub struct GameScriptingPlugin;

impl Plugin for GameScriptingPlugin {
    fn build(&self, app: &mut App) {
        // Create hamster-specific state buffer
        let buffer = Arc::new(RwLock::new(HamsterStateBuffer::default()));

        // Register hamster APIs with Lua
        if let Some(lua_ctx) = app.world().get_resource::<LuaContext>() {
            let lua = lua_ctx.lua.lock().unwrap();

            // Register generic state API
            let generic_state = create_shared_state();
            if let Err(e) = register_generic_state_api(&lua, generic_state) {
                error!("Failed to register generic state API: {}", e);
            }

            // Register hamster-specific API
            if let Err(e) = register_hamster_api(&lua, buffer.clone()) {
                error!("Failed to register hamster Lua API: {}", e);
            }
        }

        app.insert_resource(SharedHamsterStateResource(buffer));
        app.add_systems(Update, sync_hamster_state_system);
        app.add_systems(Startup, run_startup_scripts);
    }
}

/// Wrapper resource for the shared hamster state buffer.
#[derive(Resource)]
pub struct SharedHamsterStateResource(pub Arc<RwLock<HamsterStateBuffer>>);

/// Shared state buffer for hamster-specific Lua <-> ECS communication.
#[derive(Default, Debug)]
pub struct HamsterStateBuffer {
    pub current_corruption: f32,
    pub current_expression: Expression,
    pub pending_corruption: Option<f32>,
    pub pending_expression: Option<Expression>,
}

/// Register hamster-specific Lua APIs.
fn register_hamster_api(lua: &Lua, shared_state: Arc<RwLock<HamsterStateBuffer>>) -> LuaResult<()> {
    let globals = lua.globals();

    // set_corruption(f32)
    let state = shared_state.clone();
    let set_corruption = lua.create_function(move |_, value: f32| {
        let mut data = state.write().unwrap();
        data.pending_corruption = Some(value);
        Ok(())
    })?;
    globals.set("set_corruption", set_corruption)?;

    // get_corruption() -> f32
    let state = shared_state.clone();
    let get_corruption = lua.create_function(move |_, ()| {
        let data = state.read().unwrap();
        Ok(data.current_corruption)
    })?;
    globals.set("get_corruption", get_corruption)?;

    // set_expression(string) -> bool
    let state = shared_state.clone();
    let set_expression = lua.create_function(move |_, name: String| {
        if let Some(expr) = Expression::from_str(&name) {
            let mut data = state.write().unwrap();
            data.pending_expression = Some(expr);
            Ok(true)
        } else {
            Ok(false)
        }
    })?;
    globals.set("set_expression", set_expression)?;

    // get_expression() -> string
    let state = shared_state.clone();
    let get_expression = lua.create_function(move |_, ()| {
        let data = state.read().unwrap();
        Ok(data.current_expression.as_str().to_string())
    })?;
    globals.set("get_expression", get_expression)?;

    Ok(())
}

/// System to sync hamster state between Lua buffer and ECS.
fn sync_hamster_state_system(
    mut query: Query<&mut HamsterNarrator>,
    shared_resource: Res<SharedHamsterStateResource>,
) {
    if let Ok(mut narrator) = query.single_mut() {
        let mut buffer = shared_resource.0.write().unwrap();

        // Apply pending writes from Lua -> ECS
        if let Some(c) = buffer.pending_corruption {
            narrator.set_corruption(c);
            buffer.pending_corruption = None;
            debug!("Applied corruption from Lua: {}", c);
        }

        if let Some(e) = buffer.pending_expression {
            narrator.expression = e;
            buffer.pending_expression = None;
            debug!("Applied expression from Lua: {:?}", e);
        }

        // Sync ECS -> Buffer (for Lua reads)
        buffer.current_corruption = narrator.corruption_normalized() * 100.0;
        buffer.current_expression = narrator.expression;
    }
}

/// Run startup scripts.
fn run_startup_scripts(lua_ctx: Res<LuaContext>) {
    let lua = lua_ctx.lua.lock().unwrap();

    let possible_paths = [
        "games/dev/doomexe/assets/scripts/hamster_test.lua",
        "assets/scripts/hamster_test.lua",
    ];

    for path in possible_paths {
        if let Ok(content) = std::fs::read_to_string(path) {
            info!("Loading script: {}", path);
            if let Err(e) = lua.load(&content).exec() {
                error!("Error running script: {}", e);
            }
            return;
        }
    }

    warn!("No startup scripts found");
}
