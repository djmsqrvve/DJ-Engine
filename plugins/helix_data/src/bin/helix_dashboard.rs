//! Helix data contract validation dashboard.
//!
//! Runs all schema, cross-reference, and localization checks against
//! the helix3d TOML files and prints a summary.
//!
//! Usage:
//!   cargo run -p dj_engine_helix --bin helix_dashboard -- --helix3d <dir>

use dj_engine_helix::dashboard::{print_dashboard_summary, validate_helix_registries};
use dj_engine_helix::registries::load_helix_registries_lenient;
use std::path::PathBuf;
use std::process;

fn main() {
    let helix3d_path = parse_args(std::env::args());

    let Some(helix3d_dir) = helix3d_path else {
        eprintln!("Usage: helix_dashboard --helix3d <path/to/dist/helix3d>");
        process::exit(1);
    };

    println!("Source: {}", helix3d_dir.display());
    println!();

    // Load registries (lenient — reports schema mismatches as dashboard issues)
    let registries = match load_helix_registries_lenient(&helix3d_dir) {
        Ok(regs) => {
            println!("Loaded registries:");
            for (kind, count) in regs.summary() {
                let status = if count > 0 { "OK" } else { "--" };
                println!("  {:20} {:>4} entities  [{}]", kind, count, status);
            }
            println!("  {:20} {:>4} total", "", regs.total_entities());
            println!();
            regs
        }
        Err(e) => {
            eprintln!("Failed to load helix3d data: {e}");
            process::exit(1);
        }
    };

    // Run validation checks
    println!("Validation:");
    let mut issues = Vec::new();
    validate_helix_registries(&registries, Some(&helix3d_dir), &mut issues);
    print_dashboard_summary(&issues);

    // Exit code for CI
    let has_errors = issues
        .iter()
        .any(|i| matches!(i.severity, dj_engine::data::ValidationSeverity::Error));
    if has_errors {
        process::exit(1);
    }
}

fn parse_args(args: impl IntoIterator<Item = String>) -> Option<PathBuf> {
    let args: Vec<String> = args.into_iter().collect();
    let mut i = 1;
    while i < args.len() {
        if args[i] == "--helix3d" && i + 1 < args.len() {
            return Some(PathBuf::from(&args[i + 1]));
        }
        i += 1;
    }
    None
}
