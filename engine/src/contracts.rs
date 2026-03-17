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

/// Format a plain-text summary of all registered contracts.
pub fn format_contracts_text(registry: &ContractRegistry) -> String {
    let mut out = String::new();
    out.push_str("DJ Engine Contracts\n");
    out.push_str("===================\n\n");

    for contract in &registry.contracts {
        let r = contract.resources.len();
        let c = contract.components.len();
        let e = contract.events.len();
        let s = contract.system_sets.len();
        out.push_str(&format!(
            "  {:<28} {:>2} resources  {:>2} components  {:>2} events  {:>2} sets\n",
            contract.name, r, c, e, s
        ));
    }

    out.push('\n');
    out.push_str(&format!(
        "  Total: {} plugins, {} resources, {} components, {} events, {} system sets\n",
        registry.contracts.len(),
        registry.total_resources(),
        registry.total_components(),
        registry.total_events(),
        registry.total_system_sets(),
    ));

    // Detailed breakdown
    out.push_str("\n--- Detailed ---\n\n");
    for contract in &registry.contracts {
        out.push_str(&format!("{}\n", contract.name));
        out.push_str(&format!("  {}\n", contract.description));
        for entry in &contract.resources {
            out.push_str(&format!(
                "  [Resource]  {} — {}\n",
                entry.name, entry.description
            ));
        }
        for entry in &contract.components {
            out.push_str(&format!(
                "  [Component] {} — {}\n",
                entry.name, entry.description
            ));
        }
        for entry in &contract.events {
            out.push_str(&format!(
                "  [Event]     {} — {}\n",
                entry.name, entry.description
            ));
        }
        for set in &contract.system_sets {
            out.push_str(&format!("  [Set]       {} ({})\n", set.name, set.schedule));
        }
        out.push('\n');
    }

    out
}

/// Print a formatted dashboard of all registered contracts.
pub fn print_contracts_summary(registry: &ContractRegistry) {
    print!("{}", format_contracts_text(registry));
}

// ---------------------------------------------------------------------------
// Validation
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContractIssueLevel {
    Warning,
    Info,
}

#[derive(Debug, Clone)]
pub struct ContractIssue {
    pub level: ContractIssueLevel,
    pub message: String,
}

/// Validate the contract registry for common problems.
pub fn validate_contracts(registry: &ContractRegistry) -> Vec<ContractIssue> {
    let mut issues = Vec::new();

    // Check for empty contracts
    for contract in &registry.contracts {
        let total = contract.resources.len()
            + contract.components.len()
            + contract.events.len()
            + contract.system_sets.len();
        if total == 0 {
            issues.push(ContractIssue {
                level: ContractIssueLevel::Warning,
                message: format!(
                    "{}: empty contract (no resources, components, events, or sets)",
                    contract.name
                ),
            });
        }
    }

    // Check for duplicate type names across plugins
    let mut seen: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    for contract in &registry.contracts {
        for entry in contract
            .resources
            .iter()
            .chain(&contract.components)
            .chain(&contract.events)
        {
            if let Some(prev_plugin) = seen.get(&entry.type_name) {
                issues.push(ContractIssue {
                    level: ContractIssueLevel::Warning,
                    message: format!(
                        "duplicate type '{}' registered by both {} and {}",
                        entry.name, prev_plugin, contract.name
                    ),
                });
            } else {
                seen.insert(entry.type_name.clone(), contract.name.clone());
            }
        }
    }

    // Summary info
    issues.push(ContractIssue {
        level: ContractIssueLevel::Info,
        message: format!(
            "{} plugins, {} resources, {} components, {} events, {} system sets",
            registry.contracts.len(),
            registry.total_resources(),
            registry.total_components(),
            registry.total_events(),
            registry.total_system_sets(),
        ),
    });

    issues
}

/// Print validation issues and return warning count.
pub fn print_validation_issues(issues: &[ContractIssue]) -> usize {
    let warnings: Vec<_> = issues
        .iter()
        .filter(|i| i.level == ContractIssueLevel::Warning)
        .collect();
    let infos: Vec<_> = issues
        .iter()
        .filter(|i| i.level == ContractIssueLevel::Info)
        .collect();

    if !warnings.is_empty() {
        println!("Validation:");
        for issue in &warnings {
            println!("  [WARN] {}", issue.message);
        }
    }
    for issue in &infos {
        println!("  [INFO] {}", issue.message);
    }

    if warnings.is_empty() {
        println!("  All checks passed.");
    }
    println!();

    warnings.len()
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
    fn validate_warns_on_empty_contract() {
        let reg = ContractRegistry {
            contracts: vec![PluginContract {
                name: "EmptyPlugin".into(),
                description: "".into(),
                resources: vec![],
                components: vec![],
                events: vec![],
                system_sets: vec![],
            }],
        };
        let issues = validate_contracts(&reg);
        assert!(issues.iter().any(
            |i| i.level == ContractIssueLevel::Warning && i.message.contains("empty contract")
        ));
    }

    #[test]
    fn validate_warns_on_duplicate_types() {
        let reg = ContractRegistry {
            contracts: vec![
                PluginContract {
                    name: "PluginA".into(),
                    description: "".into(),
                    resources: vec![ContractEntry::of::<bool>("shared type")],
                    components: vec![],
                    events: vec![],
                    system_sets: vec![],
                },
                PluginContract {
                    name: "PluginB".into(),
                    description: "".into(),
                    resources: vec![ContractEntry::of::<bool>("also uses bool")],
                    components: vec![],
                    events: vec![],
                    system_sets: vec![],
                },
            ],
        };
        let issues = validate_contracts(&reg);
        assert!(issues.iter().any(
            |i| i.level == ContractIssueLevel::Warning && i.message.contains("duplicate type")
        ));
    }

    #[test]
    fn validate_clean_registry_has_no_warnings() {
        let reg = ContractRegistry {
            contracts: vec![PluginContract {
                name: "CleanPlugin".into(),
                description: "".into(),
                resources: vec![ContractEntry::of::<bool>("a resource")],
                components: vec![],
                events: vec![],
                system_sets: vec![],
            }],
        };
        let issues = validate_contracts(&reg);
        let warnings = issues
            .iter()
            .filter(|i| i.level == ContractIssueLevel::Warning)
            .count();
        assert_eq!(warnings, 0);
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
