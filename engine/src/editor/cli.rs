use super::types::EditorView;
use bevy::prelude::*;
use std::path::PathBuf;

#[derive(Debug, Default, PartialEq, Eq)]
pub struct EditorCliOptions {
    pub project_path: Option<PathBuf>,
    pub initial_view: EditorView,
    pub test_mode: bool,
    pub start_tutorial: bool,
}

pub fn parse_editor_cli_args(args: impl IntoIterator<Item = String>) -> EditorCliOptions {
    let args: Vec<String> = args.into_iter().collect();
    let mut options = EditorCliOptions::default();
    let mut positional_project = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--project" => {
                if i + 1 < args.len() {
                    options.project_path = Some(PathBuf::from(&args[i + 1]));
                    info!("CLI: Pre-loading project from {}", args[i + 1]);
                    i += 1;
                }
            }
            "--view" => {
                if i + 1 < args.len() {
                    options.initial_view = match args[i + 1].as_str() {
                        "story" => EditorView::StoryGraph,
                        _ => EditorView::Level,
                    };
                    info!("CLI: Setting initial view to {:?}", options.initial_view);
                    i += 1;
                }
            }
            "--test-mode" => {
                options.test_mode = true;
                info!("CLI: Automated Test Mode Enabled");
            }
            "--tutorial" => {
                options.start_tutorial = true;
                info!("CLI: Starting tutorial automatically via flag");
            }
            arg if !arg.starts_with("--") && positional_project.is_none() => {
                positional_project = Some(PathBuf::from(arg));
            }
            _ => {}
        }
        i += 1;
    }

    if options.project_path.is_none() {
        options.project_path = positional_project;
    }

    options
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_editor_cli_args_supports_positional_project_path() {
        let cli = parse_editor_cli_args([
            "dj_engine".into(),
            "projects/sample".into(),
            "--view".into(),
            "story".into(),
            "--test-mode".into(),
            "--tutorial".into(),
        ]);

        assert_eq!(cli.project_path, Some(PathBuf::from("projects/sample")));
        assert_eq!(cli.initial_view, EditorView::StoryGraph);
        assert!(cli.test_mode);
        assert!(cli.start_tutorial);
    }

    #[test]
    fn test_parse_editor_cli_args_prefers_explicit_project_flag() {
        let cli = parse_editor_cli_args([
            "dj_engine".into(),
            "projects/ignored".into(),
            "--project".into(),
            "projects/explicit".into(),
        ]);

        assert_eq!(cli.project_path, Some(PathBuf::from("projects/explicit")));
    }
}
