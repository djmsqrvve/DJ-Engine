//! CLI dashboard for DJ Engine contracts.
//!
//! Builds a headless Bevy app, registers all plugins to populate the
//! ContractRegistry, then prints a summary of all engine APIs.
//!
//! Usage: `cargo run -p dj_engine --bin contracts` or `make contracts`

use bevy::prelude::*;
use dj_engine::contracts::{print_contracts_summary, ContractRegistry};
use dj_engine::core::DJEnginePlugin;

fn main() {
    // Build a headless app — we only need plugin build() to run,
    // not any systems, so we never call app.update().
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(AssetPlugin::default());
    app.add_plugins(DJEnginePlugin::default());
    app.finish();
    app.cleanup();

    let registry = app.world().resource::<ContractRegistry>();
    print_contracts_summary(registry);

    if registry.contracts.is_empty() {
        std::process::exit(1);
    }
}
