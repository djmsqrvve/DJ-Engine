//! FFI bindings between Rust and Lua.
//!
//! Provides a generic scripting bridge that games can extend with their own APIs.

use bevy::prelude::*;
use mlua::prelude::*;
use std::sync::{Arc, RwLock};

/// Registers core FFI functions into the Lua global table.
/// Games should call this first, then register their own APIs.
pub fn register_core_api(lua: &Lua) -> LuaResult<()> {
    let globals = lua.globals();

    // log(string)
    let log_fn = lua.create_function(|_, message: String| {
        info!("[Lua] {}", message);
        Ok(())
    })?;
    globals.set("log", log_fn)?;

    // warn(string)
    let warn_fn = lua.create_function(|_, message: String| {
        warn!("[Lua] {}", message);
        Ok(())
    })?;
    globals.set("warn", warn_fn)?;

    // error(string)
    let error_fn = lua.create_function(|_, message: String| {
        error!("[Lua] {}", message);
        Ok(())
    })?;
    globals.set("error", error_fn)?;

    Ok(())
}

/// Generic shared state buffer for bridging Lua (non-ECS) and Bevy (ECS).
/// Games should create their own state buffer types.
#[derive(Default, Debug)]
pub struct GenericStateBuffer {
    /// Key-value store for simple state
    pub floats: std::collections::HashMap<String, f32>,
    pub strings: std::collections::HashMap<String, String>,
    pub bools: std::collections::HashMap<String, bool>,
}

pub type SharedGenericState = Arc<RwLock<GenericStateBuffer>>;

/// Helper to create a shared generic state buffer.
pub fn create_shared_state() -> SharedGenericState {
    Arc::new(RwLock::new(GenericStateBuffer::default()))
}

/// Register generic state access functions.
pub fn register_generic_state_api(lua: &Lua, state: SharedGenericState) -> LuaResult<()> {
    let globals = lua.globals();

    // set_float(key, value)
    let s = state.clone();
    let set_float = lua.create_function(move |_, (key, value): (String, f32)| {
        let mut data = s.write().unwrap();
        data.floats.insert(key, value);
        Ok(())
    })?;
    globals.set("set_float", set_float)?;

    // get_float(key) -> f32
    let s = state.clone();
    let get_float = lua.create_function(move |_, key: String| {
        let data = s.read().unwrap();
        Ok(data.floats.get(&key).copied().unwrap_or(0.0))
    })?;
    globals.set("get_float", get_float)?;

    // set_string(key, value)
    let s = state.clone();
    let set_string = lua.create_function(move |_, (key, value): (String, String)| {
        let mut data = s.write().unwrap();
        data.strings.insert(key, value);
        Ok(())
    })?;
    globals.set("set_string", set_string)?;

    // get_string(key) -> string
    let s = state.clone();
    let get_string = lua.create_function(move |_, key: String| {
        let data = s.read().unwrap();
        Ok(data.strings.get(&key).cloned().unwrap_or_default())
    })?;
    globals.set("get_string", get_string)?;

    // set_bool(key, value)
    let s = state.clone();
    let set_bool = lua.create_function(move |_, (key, value): (String, bool)| {
        let mut data = s.write().unwrap();
        data.bools.insert(key, value);
        Ok(())
    })?;
    globals.set("set_bool", set_bool)?;

    // get_bool(key) -> bool
    let s = state.clone();
    let get_bool = lua.create_function(move |_, key: String| {
        let data = s.read().unwrap();
        Ok(data.bools.get(&key).copied().unwrap_or(false))
    })?;
    globals.set("get_bool", get_bool)?;

    Ok(())
}
