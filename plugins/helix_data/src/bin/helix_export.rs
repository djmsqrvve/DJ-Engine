use dj_engine_helix::exporter::export_to_helix3d;
use dj_engine_helix::registries::load_helix_registries_lenient;
use std::path::PathBuf;

fn main() {
    if std::env::args().any(|arg| arg == "--help" || arg == "-h") {
        println!(
            "Usage:\n  \
             cargo run -p dj_engine_helix --bin helix_export -- --helix3d <input_dir> --output <output_dir>\n\n\
             Loads helix3d TOML data, passes it through the engine's LoadedCustomDocuments,\n\
             and exports back to TOML.  Use to round-trip edits made in the DJ-Engine editor."
        );
        return;
    }

    let (helix3d_dir, output_dir) = match parse_args(std::env::args()) {
        Ok(args) => args,
        Err(msg) => {
            eprintln!("Error: {msg}");
            std::process::exit(1);
        }
    };

    // Load typed registries from TOML source.
    println!("Loading registries from {}...", helix3d_dir.display());
    let registries = match load_helix_registries_lenient(&helix3d_dir) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Failed to load registries: {e}");
            std::process::exit(1);
        }
    };
    println!(
        "Loaded {} entities across 22 registries.",
        registries.total_entities()
    );

    // Synthesize LoadedCustomDocuments the same way the engine does.
    let mut loaded = dj_engine::data::LoadedCustomDocuments::default();
    registries.for_each_as_json(|kind, id, payload| {
        let envelope = dj_engine::data::CustomDocument {
            kind: kind.to_string(),
            id: id.to_string(),
            schema_version: 1,
            label: payload
                .get("name")
                .and_then(|n| n.get("en"))
                .and_then(serde_json::Value::as_str)
                .map(str::to_string),
            tags: vec!["source:toml_registry".to_string()],
            references: Vec::new(),
            payload: payload.clone(),
        };

        let raw_json = serde_json::to_string_pretty(&envelope).unwrap_or_default();

        loaded
            .documents
            .push(dj_engine::data::LoadedCustomDocument {
                entry: dj_engine::data::CustomDocumentEntry {
                    kind: kind.to_string(),
                    id: id.to_string(),
                    path: format!("{kind}/{id}.toml"),
                    schema_version: 1,
                    editor_route: dj_engine::data::EditorDocumentRoute::Table,
                    tags: vec!["source:toml_registry".to_string()],
                },
                raw_json,
                document: Some(envelope),
                parse_error: None,
                resolved_route: dj_engine::data::EditorDocumentRoute::Table,
            });
    });

    // Export back to TOML.
    println!("Exporting to {}...", output_dir.display());
    match export_to_helix3d(&loaded, &output_dir) {
        Ok(summary) => {
            println!(
                "Export complete: {} files written, {} entities exported.",
                summary.files_written, summary.entities_exported
            );
        }
        Err(e) => {
            eprintln!("Export failed: {e}");
            std::process::exit(1);
        }
    }
}

fn parse_args(args: impl IntoIterator<Item = String>) -> Result<(PathBuf, PathBuf), &'static str> {
    let args: Vec<String> = args.into_iter().collect();
    let mut helix3d = None;
    let mut output = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--helix3d" => {
                if i + 1 < args.len() {
                    helix3d = Some(PathBuf::from(&args[i + 1]));
                    i += 1;
                }
            }
            "--output" => {
                if i + 1 < args.len() {
                    output = Some(PathBuf::from(&args[i + 1]));
                    i += 1;
                }
            }
            _ => {}
        }
        i += 1;
    }

    let helix3d = helix3d.ok_or("--helix3d <dir> is required")?;
    let output = output.ok_or("--output <dir> is required")?;
    Ok((helix3d, output))
}
