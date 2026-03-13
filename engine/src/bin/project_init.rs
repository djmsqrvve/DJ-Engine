use dj_engine::project_mount::{create_new_project, workspace_root};
use std::path::PathBuf;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut name: Option<String> = None;
    let mut dir: Option<PathBuf> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--name" | "-n" => {
                if i + 1 < args.len() {
                    name = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            "--dir" | "-d" => {
                if i + 1 < args.len() {
                    dir = Some(PathBuf::from(&args[i + 1]));
                    i += 1;
                }
            }
            "--help" | "-h" => {
                print_usage();
                return;
            }
            arg if !arg.starts_with('-') && name.is_none() => {
                name = Some(arg.to_string());
            }
            _ => {
                eprintln!("Unknown argument: {}", args[i]);
                print_usage();
                std::process::exit(1);
            }
        }
        i += 1;
    }

    let Some(name) = name else {
        eprintln!("Error: project name is required.\n");
        print_usage();
        std::process::exit(1);
    };

    let slug = name
        .to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect::<String>();

    let project_dir = dir.unwrap_or_else(|| workspace_root().join("projects").join(&slug));

    match create_new_project(&name, &project_dir) {
        Ok(mount) => {
            let manifest = mount
                .manifest_path
                .as_ref()
                .map(|p| p.display().to_string())
                .unwrap_or_default();
            println!("Created project '{}' at {}", name, manifest);
            println!();
            println!("Next steps:");
            println!("  make editor                          Open in the editor");
            println!(
                "  make preview PROJECT={}  Run the runtime preview",
                project_dir.display()
            );
        }
        Err(error) => {
            eprintln!("Error: {}", error);
            std::process::exit(1);
        }
    }
}

fn print_usage() {
    eprintln!("Usage: project_init <name> [--dir <path>]");
    eprintln!();
    eprintln!("Create a new DJ Engine game project with a starter scene and story graph.");
    eprintln!();
    eprintln!("Arguments:");
    eprintln!("  <name>          Project name (e.g. \"My RPG\")");
    eprintln!("  --dir, -d       Project directory (default: projects/<slug>/)");
    eprintln!("  --help, -h      Show this help");
}
