# DJ Engine AI Handoff Suite

Last updated: 2026-03-09

This folder is the current handoff set for future AI agents working in this
repository. It is intentionally based on the checked-in source tree, current
tooling, and current remote-dev setup rather than the broader historical design
docs.

Use this suite in the following order:

1. `01_CURRENT_STATE.md`
2. `02_WORKSPACE_MAP.md`
3. `03_ENGINE_GUIDE.md`
4. `04_GAME_GUIDE.md`
5. `05_DATA_SCRIPTING_AND_STORY.md`
6. `06_REMOTE_DEV_AND_CI.md`
7. `07_AGENT_WORKFLOW.md`

Source-of-truth rules:

- Cargo manifests, Rust source, `.devcontainer/`, and `.github/workflows/`
  beat older prose docs when they disagree.
- A number of older docs in `docs/` and the project-level `AGENTS.md` still
  describe Bevy `0.15` or planned systems that no longer match the code.
- Treat this suite as the fastest path to the repo's current working shape, but
  still verify anything time-sensitive such as GitHub-hosted Codespaces or CI
  behavior.

What this suite is for:

- Getting a new coding agent oriented quickly.
- Showing which parts of the engine are implemented versus still prototype or
  TODO-backed.
- Explaining how the editor, game, data layer, scripting layer, Codespaces, and
  CI fit together today.

