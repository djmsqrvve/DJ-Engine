//! CLI dashboard for DJ Engine contracts.
//!
//! Builds a headless Bevy app, registers all plugins to populate the
//! ContractRegistry, then prints a summary of all engine APIs.
//!
//! Usage: `cargo run -p dj_engine --bin contracts` or `make contracts`
//! Flags: `--strict` exits non-zero on any validation warnings

use bevy::prelude::*;
use dj_engine::contracts::{
    print_contracts_summary, print_validation_issues, validate_contracts, ContractRegistry,
};
use dj_engine::core::DJEnginePlugin;

fn main() {
    let strict = std::env::args().any(|a| a == "--strict");

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

    let issues = validate_contracts(registry);
    let warning_count = print_validation_issues(&issues);

    if registry.contracts.is_empty() {
        std::process::exit(1);
    }
    if strict && warning_count > 0 {
        eprintln!(
            "--strict: {} warning(s) found, exiting with error",
            warning_count
        );
        std::process::exit(1);
    }
}
