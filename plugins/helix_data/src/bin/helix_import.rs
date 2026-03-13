use dj_engine_helix::{import_helix_project, parse_helix_import_cli_args, HelixImportError};

fn main() {
    if std::env::args().any(|arg| arg == "--help" || arg == "-h") {
        println!(
            "Usage: cargo run -p dj_engine_helix --bin helix_import -- --helix-dist <dir> --project <dir|project.json>"
        );
        return;
    }

    match run() {
        Ok(()) => {}
        Err(error) => {
            eprintln!("Helix import failed: {error}");
            std::process::exit(1);
        }
    }
}

fn run() -> Result<(), HelixImportError> {
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
