# DJ Engine Codespaces Cleanup + Compile Handoff

Date: 2026-03-09
Branch: `main`
Commit: `955a2e4`

## Goal
Make this repository reliable to open, build, and validate inside GitHub Codespaces, while cleaning up the highest-value repo drift that blocks a smooth remote-dev experience.

## Current Baseline
- `main` is current and pushed to `origin/main` at `955a2e4`.
- Local validation already passes:
  - `RUSTC_WRAPPER= CARGO_TARGET_DIR=/tmp/dj_engine_bevy18 cargo check --workspace`
  - `RUSTC_WRAPPER= CARGO_TARGET_DIR=/tmp/dj_engine_bevy18 cargo test --workspace --no-run`
  - `RUSTC_WRAPPER= CARGO_TARGET_DIR=/tmp/dj_engine_bevy18 cargo clippy --workspace --all-targets -- -W clippy::all`
- Runtime smoke is stable enough for desktop Linux:
  - editor `--test-mode` launches and exits cleanly
  - `doomexe` launches cleanly under a timeout
- Known repo gaps for Codespaces:
  - no `.devcontainer/`
  - no GitHub Actions workflow under `.github/workflows/`
  - no `rust-toolchain.toml`
  - README/docs drift still exists (for example README still says Bevy 0.15, repo is now Bevy 0.18)
- Untracked local file exists in some worktrees: `docs/HANDOFF_BROAD_AUDIT.md`
  - Do not delete or overwrite it unless the user explicitly asks.

## High-Priority Deliverables
1. Add a Codespaces/devcontainer setup that can compile the whole workspace.
2. Pin the Rust toolchain used by Codespaces.
3. Add at least one CI workflow that verifies workspace compilation on GitHub.
4. Clean up the most misleading setup/docs drift so a fresh remote contributor can follow the repo successfully.

## Recommended Implementation Order

### 1. Add Codespaces / devcontainer support
- Create `.devcontainer/devcontainer.json`.
- Prefer a Debian/Ubuntu-based Rust image that works well with Bevy native dependencies.
- Install the Linux packages needed for Bevy + winit + audio compilation on Ubuntu.
  - Start with:
    - `pkg-config`
    - `libasound2-dev`
    - `libudev-dev`
    - `libwayland-dev`
    - `libxkbcommon-dev`
    - `libxcursor-dev`
    - `libxi-dev`
    - `libxrandr-dev`
    - `libxinerama-dev`
    - `libgl1-mesa-dev`
    - `libvulkan-dev`
    - `libxcb-render0-dev`
    - `libxcb-shape0-dev`
    - `libxcb-xfixes0-dev`
    - `clang`
    - `lld`
    - `cmake`
    - `git`
- Include Rust tooling/extensions that matter in Codespaces:
  - `rust-lang.rust-analyzer`
  - `tamasfe.even-better-toml`
  - `vadimcn.vscode-lldb`
- Set a default `postCreateCommand` or equivalent bootstrap that runs:
  - `rustup show`
  - `cargo fetch`
- Do not optimize for GUI runtime inside Codespaces first.
  - Primary success is compile/test/lint, not opening the Bevy window in browser-remote Linux.

### 2. Pin the Rust toolchain
- Add `rust-toolchain.toml`.
- Pin a stable toolchain version compatible with the current workspace.
- Include `rustfmt` and `clippy` components.
- If you choose a specific version instead of `stable`, document why in the commit message or handoff notes.

### 3. Add GitHub CI for compile validation
- Create `.github/workflows/ci.yml`.
- Minimum jobs:
  - `cargo check --workspace`
  - `cargo test --workspace --no-run`
  - `cargo clippy --workspace --all-targets -- -W clippy::all`
- Use the same environment assumptions as Codespaces where practical.
- If Linux system packages are needed in CI, install them explicitly in the workflow.
- Keep CI compile-focused.
  - Do not add GUI runtime jobs unless they are headless and deterministic.

### 4. Clean up setup/documentation drift
- Update README to reflect current truth:
  - Bevy `0.18`, not `0.15`
  - current helper commands
  - Codespaces/devcontainer availability once added
- Add a short `Codespaces` section to either:
  - `README.md`, or
  - `docs/GETTING_STARTED.md`
- Include the exact remote validation commands:
  - `RUSTC_WRAPPER= CARGO_TARGET_DIR=/tmp/dj_engine_bevy18 cargo check --workspace`
  - `RUSTC_WRAPPER= CARGO_TARGET_DIR=/tmp/dj_engine_bevy18 cargo test --workspace --no-run`
  - `RUSTC_WRAPPER= CARGO_TARGET_DIR=/tmp/dj_engine_bevy18 cargo clippy --workspace --all-targets -- -W clippy::all`
- Keep docs cleanup focused.
  - Do not turn this into a repo-wide documentation rewrite.

## Notes On Existing Runtime/Build Behavior
- Workspace Bevy dependency is currently configured with `dynamic_linking` and `wav`.
- Startup audio is muted by default via engine audio state.
- Editor `bevy_egui` integration now depends on `EguiPrimaryContextPass` and `PrimaryEguiContext`.
  - Do not regress this while doing Codespaces cleanup.
- Codespaces work should not remove diagnostics or editor startup behavior just to make compile jobs simpler.

## Validation Checklist
- Inside the devcontainer / Codespaces environment:
  - `cargo fmt --all --check`
  - `RUSTC_WRAPPER= CARGO_TARGET_DIR=/tmp/dj_engine_bevy18 cargo check --workspace`
  - `RUSTC_WRAPPER= CARGO_TARGET_DIR=/tmp/dj_engine_bevy18 cargo test --workspace --no-run`
  - `RUSTC_WRAPPER= CARGO_TARGET_DIR=/tmp/dj_engine_bevy18 cargo clippy --workspace --all-targets -- -W clippy::all`
- Verify a fresh Codespaces open does not require undocumented manual package installs.
- Verify CI passes on the same branch.

## Known Acceptable Remaining Warnings
These were intentionally deferred and should not block the Codespaces pass unless the user asks for broader cleanup:
- `clippy::too_many_arguments`
- `clippy::type_complexity`
- `clippy::upper_case_acronyms`
- `clippy::module_inception`
- one `field_reassign_with_default` warning in editor code
- one `collapsible_else_if` warning in story graph code

## Suggested Commit Breakdown
1. `chore: add codespaces devcontainer for workspace builds`
2. `chore: pin rust toolchain for remote dev`
3. `ci: add workspace compile validation workflow`
4. `docs: update setup docs for bevy 0.18 and codespaces`

## Exit Criteria
- Repo opens in GitHub Codespaces without extra undocumented setup.
- Workspace compile/test/clippy commands pass in Codespaces.
- CI reproduces the same compile validation on GitHub.
- README or getting-started docs no longer misstate the core version/setup story.
