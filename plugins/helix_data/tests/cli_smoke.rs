//! Smoke tests for helix CLI binaries.
//!
//! Verify each binary starts, prints usage on missing args, and exits non-zero.
//! This catches compilation issues and basic arg parsing regressions.

use std::process::Command;

fn cargo_target_dir() -> String {
    std::env::var("CARGO_TARGET_DIR")
        .unwrap_or_else(|_| format!("{}/.cargo-targets/dj_engine_bevy18", env!("HOME")))
}

fn run_binary(name: &str) -> std::process::Output {
    // First build the binary, then run it without args
    let status = Command::new("cargo")
        .args(["build", "-p", "dj_engine_helix", "--bin", name])
        .env("CARGO_TARGET_DIR", cargo_target_dir())
        .status()
        .expect("failed to build binary");
    assert!(status.success(), "failed to compile {name}");

    let bin_path = format!("{}/debug/{name}", cargo_target_dir());
    Command::new(&bin_path)
        .output()
        .unwrap_or_else(|_| panic!("failed to run {name}"))
}

#[test]
fn helix_dashboard_exits_nonzero_without_args() {
    let output = run_binary("helix_dashboard");
    assert!(
        !output.status.success(),
        "helix_dashboard should fail without --helix3d arg"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Usage") || stderr.contains("helix3d"),
        "expected usage message, got: {stderr}"
    );
}

#[test]
fn helix_import_exits_nonzero_without_args() {
    let output = run_binary("helix_import");
    assert!(
        !output.status.success(),
        "helix_import should fail without args"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Usage") || stderr.contains("helix"),
        "expected usage message, got: {stderr}"
    );
}

#[test]
fn contracts_binary_succeeds() {
    let output = Command::new("cargo")
        .args(["run", "-p", "dj_engine", "--bin", "contracts"])
        .env("CARGO_TARGET_DIR", cargo_target_dir())
        .output()
        .expect("failed to run contracts");
    assert!(
        output.status.success(),
        "contracts binary should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("DJ Engine Contracts"),
        "expected contracts output"
    );
}
