use bevy::prelude::*;
use std::fmt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use super::types::{RuntimePreviewLaunchPhase, RuntimePreviewLaunchState};
use crate::diagnostics::console::ConsoleLogStore;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedRuntimePreviewCommand {
    pub program: PathBuf,
    pub args: Vec<String>,
    pub current_dir: Option<PathBuf>,
}

impl ResolvedRuntimePreviewCommand {
    pub fn spawn(&self) -> std::io::Result<Child> {
        let mut command = Command::new(&self.program);
        command.args(&self.args);
        if let Some(current_dir) = &self.current_dir {
            command.current_dir(current_dir);
        }
        // Capture stderr so we can display error diagnostics in the editor.
        command.stderr(Stdio::piped());
        command.spawn()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimePreviewLaunchError {
    CurrentExecutableUnavailable(String),
    RuntimePreviewExecutableNotFound(PathBuf),
}

impl fmt::Display for RuntimePreviewLaunchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CurrentExecutableUnavailable(message) => write!(f, "{message}"),
            Self::RuntimePreviewExecutableNotFound(path) => write!(
                f,
                "Runtime preview executable not found at {:?}, and dev fallback is unavailable.",
                path
            ),
        }
    }
}

pub fn runtime_preview_sibling_path(current_exe: &Path) -> PathBuf {
    current_exe.with_file_name(format!("runtime_preview{}", std::env::consts::EXE_SUFFIX))
}

pub fn workspace_root() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir.parent().unwrap_or(&manifest_dir).to_path_buf()
}

pub fn resolve_runtime_preview_command(
    manifest_path: &Path,
    preview_profile: Option<&str>,
) -> Result<ResolvedRuntimePreviewCommand, RuntimePreviewLaunchError> {
    resolve_runtime_preview_command_from_mode(
        manifest_path,
        preview_profile,
        std::env::current_exe().ok(),
        cfg!(debug_assertions),
    )
}

pub fn resolve_runtime_preview_command_from_mode(
    manifest_path: &Path,
    preview_profile: Option<&str>,
    current_exe: Option<PathBuf>,
    is_dev: bool,
) -> Result<ResolvedRuntimePreviewCommand, RuntimePreviewLaunchError> {
    let mut extra_args = vec![];
    if let Some(profile_id) = preview_profile {
        extra_args.push("--preview-profile".into());
        extra_args.push(profile_id.to_string());
    }

    if let Some(current_exe) = current_exe {
        let sibling_path = runtime_preview_sibling_path(&current_exe);
        if sibling_path.is_file() {
            return Ok(ResolvedRuntimePreviewCommand {
                program: sibling_path,
                args: [
                    vec![manifest_path.to_string_lossy().into_owned()],
                    extra_args,
                ]
                .concat(),
                current_dir: None,
            });
        }

        if is_dev {
            return Ok(ResolvedRuntimePreviewCommand {
                program: PathBuf::from("cargo"),
                args: [
                    vec![
                        "run".into(),
                        "--bin".into(),
                        "runtime_preview".into(),
                        "--".into(),
                        manifest_path.to_string_lossy().into_owned(),
                    ],
                    extra_args,
                ]
                .concat(),
                current_dir: Some(workspace_root()),
            });
        }

        return Err(RuntimePreviewLaunchError::RuntimePreviewExecutableNotFound(
            sibling_path,
        ));
    }

    Err(RuntimePreviewLaunchError::CurrentExecutableUnavailable(
        "Could not determine current executable path.".into(),
    ))
}

pub fn log_preview_message(console: Option<&mut ConsoleLogStore>, message: &str) {
    if let Some(console) = console {
        console.log(format!("Preview: {}", message));
    }
}

pub fn set_launch_state_message(
    state: &mut RuntimePreviewLaunchState,
    console: Option<&mut ConsoleLogStore>,
    message: &str,
) {
    state.status_message = Some(message.to_string());
    log_preview_message(console, message);
}

pub fn format_exit_status(exit_status: std::process::ExitStatus) -> String {
    if exit_status.success() {
        "success".into()
    } else if let Some(code) = exit_status.code() {
        format!("code {}", code)
    } else {
        "terminated by signal".into()
    }
}

pub fn poll_runtime_preview_process_system(
    mut launch_state: ResMut<RuntimePreviewLaunchState>,
    mut console: Option<ResMut<ConsoleLogStore>>,
) {
    if !matches!(launch_state.phase, RuntimePreviewLaunchPhase::Running) {
        return;
    }

    let process_opt = launch_state.process.clone();
    let Some(process_arc) = process_opt else {
        return;
    };

    let mut process_guard = process_arc.lock().unwrap();

    match process_guard.try_wait() {
        Ok(Some(status)) => {
            let status_str = format_exit_status(status);

            // Read stderr for error diagnostics when the process exits non-zero.
            if !status.success() {
                if let Some(stderr) = process_guard.stderr.take() {
                    use std::io::Read;
                    let mut error_output = String::new();
                    let mut reader = std::io::BufReader::new(stderr);
                    if reader.read_to_string(&mut error_output).is_ok()
                        && !error_output.is_empty()
                    {
                        // Truncate very long error output for display.
                        let truncated = if error_output.len() > 2000 {
                            format!("{}... (truncated)", &error_output[..2000])
                        } else {
                            error_output.clone()
                        };
                        launch_state.last_error = Some(truncated.clone());
                        log_preview_message(
                            console.as_deref_mut(),
                            &format!("Preview stderr:\n{}", truncated),
                        );
                    }
                }
            } else {
                launch_state.last_error = None;
            }

            set_launch_state_message(
                &mut launch_state,
                console.as_deref_mut(),
                &format!("Terminated ({})", status_str),
            );
            info!("Runtime preview terminated with status: {}", status);
            launch_state.phase = RuntimePreviewLaunchPhase::Idle;
        }
        Ok(None) => {}
        Err(e) => {
            set_launch_state_message(
                &mut launch_state,
                console.as_deref_mut(),
                &format!("Error polling: {}", e),
            );
            error!("Error polling runtime preview process: {}", e);
            launch_state.phase = RuntimePreviewLaunchPhase::Idle;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_runtime_preview_command_prefers_sibling_binary() {
        let temp_dir = tempfile::tempdir().unwrap();
        let sibling_bin = runtime_preview_sibling_path(&temp_dir.path().join("dj_engine"));
        std::fs::write(&sibling_bin, "").unwrap();

        let cmd = resolve_runtime_preview_command_from_mode(
            Path::new("dummy.json"),
            None,
            Some(temp_dir.path().join("dj_engine")),
            true, // Dev mode shouldn't matter if sibling exists
        )
        .unwrap();

        assert_eq!(cmd.program, sibling_bin);
        assert_eq!(cmd.args, vec!["dummy.json"]);
        assert_eq!(cmd.current_dir, None);
    }

    #[test]
    fn test_resolve_runtime_preview_command_uses_dev_cargo_fallback() {
        let cmd = resolve_runtime_preview_command_from_mode(
            Path::new("dummy.json"),
            None,
            Some(PathBuf::from("/nonexistent/dj_engine")),
            true,
        )
        .unwrap();

        assert_eq!(cmd.program, PathBuf::from("cargo"));
        assert_eq!(
            cmd.args,
            vec!["run", "--bin", "runtime_preview", "--", "dummy.json"]
        );
        assert!(cmd.current_dir.is_some());
    }

    #[test]
    fn test_resolve_runtime_preview_command_returns_structured_error_without_fallback() {
        let current_exe = PathBuf::from("/nonexistent/dj_engine");
        let result = resolve_runtime_preview_command_from_mode(
            Path::new("dummy.json"),
            None,
            Some(current_exe.clone()),
            false,
        );

        assert!(matches!(
            result,
            Err(RuntimePreviewLaunchError::RuntimePreviewExecutableNotFound(path))
                if path == runtime_preview_sibling_path(&current_exe)
        ));
    }

    #[test]
    fn test_resolve_runtime_preview_command_passes_preview_profile() {
        let cmd = resolve_runtime_preview_command_from_mode(
            Path::new("dummy.json"),
            Some("quick_test"),
            Some(PathBuf::from("/nonexistent/dj_engine")),
            true,
        )
        .unwrap();

        assert_eq!(
            cmd.args,
            vec![
                "run",
                "--bin",
                "runtime_preview",
                "--",
                "dummy.json",
                "--preview-profile",
                "quick_test"
            ]
        );
    }
}