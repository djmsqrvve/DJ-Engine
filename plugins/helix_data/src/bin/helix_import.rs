use dj_engine_helix::registries::load_helix_registries_lenient;
use dj_engine_helix::{import_helix_project, parse_helix_import_cli_args, HelixImportError};
use std::path::PathBuf;

fn main() {
    if std::env::args().any(|arg| arg == "--help" || arg == "-h") {
        println!(
            "Usage:\n\
             Legacy JSON import:\n  \
               cargo run -p dj_engine_helix --bin helix_import -- --helix-dist <dir> --project <dir|project.json>\n\
             Typed TOML import:\n  \
               cargo run -p dj_engine_helix --bin helix_import -- --helix3d <dir>"
        );
        return;
    }

    // Check for --helix3d flag (new typed TOML pipeline)
    let helix3d_path = parse_helix3d_arg(std::env::args());
    if let Some(dir) = helix3d_path {
        match run_toml_import(&dir) {
            Ok(()) => {}
            Err(e) => {
                eprintln!("TOML import failed: {e}");
                std::process::exit(1);
            }
        }
        return;
    }

    // Legacy JSON bucket import
    match run_legacy_import() {
        Ok(()) => {}
        Err(error) => {
            eprintln!("Helix import failed: {error}");
            std::process::exit(1);
        }
    }
}

fn run_toml_import(helix3d_dir: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "Loading typed Helix registries from {}...",
        helix3d_dir.display()
    );

    let registries = load_helix_registries_lenient(helix3d_dir)?;

    println!(
        "Loaded {} entities across 22 registries:",
        registries.total_entities()
    );
    for (kind, count) in registries.summary() {
        if count > 0 {
            println!("  {:20} {} entities", kind, count);
        }
    }

    Ok(())
}

fn run_legacy_import() -> Result<(), HelixImportError> {
    let options = parse_helix_import_cli_args(std::env::args())?;
    let summary = import_helix_project(&options.helix_dist, &options.project_path)?;

    println!(
        "Imported Helix data into {}",
        summary.project_root.display()
    );
    println!(
        "  abilities: {}  items: {}  mobs: {}  skipped: {}",
        summary.abilities, summary.items, summary.mobs, summary.skipped_files
    );
    println!(
        "  preview profile: {}  registry: {}",
        summary.preview_profile_id,
        summary.registry_path.display()
    );

    Ok(())
}

fn parse_helix3d_arg(args: impl IntoIterator<Item = String>) -> Option<PathBuf> {
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
