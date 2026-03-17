//! Engine Contracts — API surface registry for all DJ Engine plugins.
//!
//! Each plugin registers a [`PluginContract`] describing the resources,
//! components, events, and system sets it provides.  The registry is
//! consumed by the CLI dashboard (`make contracts`) and the editor's
//! Contracts browser tab.

use bevy::prelude::*;

// ---------------------------------------------------------------------------
// Data types
// ---------------------------------------------------------------------------

/// A single type exposed by a plugin contract.
#[derive(Debug, Clone)]
pub struct ContractEntry {
    /// Short name (e.g. `"AudioState"`).
    pub name: String,
    /// Fully-qualified Rust type path.
    pub type_name: String,
    /// One-line description.
    pub description: String,
}

impl ContractEntry {
    /// Build a [`ContractEntry`] from a concrete Rust type.
    pub fn of<T: 'static>(description: &str) -> Self {
        let full = std::any::type_name::<T>();
        // Extract short name: strip crate path but keep generics intact.
        // e.g. "alloc::vec::Vec<alloc::string::String>" → "Vec<String>"
        let short = shorten_type_name(full);
        Self {
            name: short,
            type_name: full.to_string(),
            description: description.to_string(),
        }
    }
}

/// A system set declared by a plugin.
#[derive(Debug, Clone)]
pub struct ContractSystemSet {
    pub name: String,
    pub schedule: String,
}

/// The full contract for one plugin.
#[derive(Debug, Clone)]
pub struct PluginContract {
    pub name: String,
    pub description: String,
    pub resources: Vec<ContractEntry>,
    pub components: Vec<ContractEntry>,
    pub events: Vec<ContractEntry>,
    pub system_sets: Vec<ContractSystemSet>,
}

/// Accumulates contracts from every plugin at build time.
#[derive(Resource, Default, Debug, Clone)]
pub struct ContractRegistry {
    pub contracts: Vec<PluginContract>,
}

impl ContractRegistry {
    pub fn total_resources(&self) -> usize {
        self.contracts.iter().map(|c| c.resources.len()).sum()
    }
    pub fn total_components(&self) -> usize {
        self.contracts.iter().map(|c| c.components.len()).sum()
    }
    pub fn total_events(&self) -> usize {
        self.contracts.iter().map(|c| c.events.len()).sum()
    }
    pub fn total_system_sets(&self) -> usize {
        self.contracts.iter().map(|c| c.system_sets.len()).sum()
    }
}

// ---------------------------------------------------------------------------
// Registration trait
// ---------------------------------------------------------------------------

/// Extension trait for registering plugin contracts on [`App`].
pub trait AppContractExt {
    fn register_contract(&mut self, contract: PluginContract) -> &mut Self;
}

impl AppContractExt for App {
    fn register_contract(&mut self, contract: PluginContract) -> &mut Self {
        self.init_resource::<ContractRegistry>();
        self.world_mut()
            .resource_mut::<ContractRegistry>()
            .contracts
            .push(contract);
        self
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Shorten a fully-qualified Rust type name to its leaf segments.
///
/// `"dj_engine::audio::AudioState"` → `"AudioState"`
/// `"alloc::vec::Vec<alloc::string::String>"` → `"Vec<String>"`
fn shorten_type_name(full: &str) -> String {
    let mut result = String::new();
    for segment in full.split('<') {
        if !result.is_empty() {
            result.push('<');
        }
        // Each segment may contain comma-separated types (in generics)
        let parts: Vec<&str> = segment.split(',').collect();
        for (i, part) in parts.iter().enumerate() {
            if i > 0 {
                result.push_str(", ");
            }
            let trimmed = part.trim().trim_end_matches('>');
            let closing = part.len() - part.trim_end_matches('>').len();
            let short = trimmed.rsplit("::").next().unwrap_or(trimmed);
            result.push_str(short);
            for _ in 0..closing {
                result.push('>');
            }
        }
    }
    result
}

// ---------------------------------------------------------------------------
// CLI display
// ---------------------------------------------------------------------------

/// Print a formatted dashboard of all registered contracts.
pub fn print_contracts_summary(registry: &ContractRegistry) {
    println!();
    println!("DJ Engine Contracts");
    println!("===================");
    println!();

    for contract in &registry.contracts {
        let r = contract.resources.len();
        let c = contract.components.len();
        let e = contract.events.len();
        let s = contract.system_sets.len();
        println!(
            "  {:<28} {:>2} resources  {:>2} components  {:>2} events  {:>2} sets",
            contract.name, r, c, e, s
        );
    }

    println!();
    println!(
        "  Total: {} plugins, {} resources, {} components, {} events, {} system sets",
        registry.contracts.len(),
        registry.total_resources(),
        registry.total_components(),
        registry.total_events(),
        registry.total_system_sets(),
    );
    println!();
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_default_is_empty() {
        let reg = ContractRegistry::default();
        assert!(reg.contracts.is_empty());
        assert_eq!(reg.total_resources(), 0);
    }

    #[test]
    fn contract_entry_of_extracts_short_name() {
        let entry = ContractEntry::of::<Vec<String>>("test");
        assert_eq!(entry.name, "Vec<String>");
        assert!(entry.type_name.contains("Vec"));
    }

    #[test]
    fn register_contract_via_app() {
        let mut app = App::new();
        app.register_contract(PluginContract {
            name: "TestPlugin".into(),
            description: "test".into(),
            resources: vec![ContractEntry::of::<ContractRegistry>("the registry")],
            components: vec![],
            events: vec![],
            system_sets: vec![],
        });
        let reg = app.world().resource::<ContractRegistry>();
        assert_eq!(reg.contracts.len(), 1);
        assert_eq!(reg.contracts[0].name, "TestPlugin");
        assert_eq!(reg.total_resources(), 1);
    }

    #[test]
    fn totals_accumulate_across_plugins() {
        let mut app = App::new();
        app.register_contract(PluginContract {
            name: "A".into(),
            description: "".into(),
            resources: vec![ContractEntry::of::<bool>("r1")],
            components: vec![ContractEntry::of::<bool>("c1")],
            events: vec![],
            system_sets: vec![],
        });
        app.register_contract(PluginContract {
            name: "B".into(),
            description: "".into(),
            resources: vec![],
            components: vec![
                ContractEntry::of::<u32>("c2"),
                ContractEntry::of::<u64>("c3"),
            ],
            events: vec![ContractEntry::of::<bool>("e1")],
            system_sets: vec![],
        });
        let reg = app.world().resource::<ContractRegistry>();
        assert_eq!(reg.total_resources(), 1);
        assert_eq!(reg.total_components(), 3);
        assert_eq!(reg.total_events(), 1);
    }
}
