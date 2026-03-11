# DJ Engine AI Handoff Suite

Last updated: 2026-03-10
Canonical repo: `/home/dj/dev/engines/DJ-Engine`
Branch at audit time: `main`
HEAD at audit time: `5e36a1d`
Remote state at audit time: local `main` and `origin/main` only

This folder is the current handoff set for future AI agents working in this
repository. It is intentionally based on the checked-in source tree, current
tooling, and current remote-dev setup rather than the broader historical design
docs.

Use this suite in the following order:

1. `00_ACCURACY_AUDIT.md`
2. `01_CURRENT_STATE.md`
3. `02_WORKSPACE_MAP.md`
4. `03_ENGINE_GUIDE.md`
5. `04_GAME_GUIDE.md`
6. `05_DATA_SCRIPTING_AND_STORY.md`
7. `06_REMOTE_DEV_AND_CI.md`
8. `07_AGENT_WORKFLOW.md`
9. `08_SESSION_HANDOFF_2026_03_09.md`
10. `09_SESSION_HANDOFF_2026_03_10.md`  ← **START HERE if resuming from 2026-03-10**
11. `10_BRANCH_LOG.md`
12. `PLAN.md`
13. `PROMPT.md`
14. `AUDIT_REQUEST.md`

Source-of-truth rules:

- Cargo manifests, Rust source, `.devcontainer/`, and `.github/workflows/`
  beat older prose docs when they disagree.
- A number of older docs in `docs/` and the project-level `AGENTS.md` still
  describe Bevy `0.15` or planned systems that no longer match the code.
- Treat this suite as the fastest path to the repo's current working shape, but
  still verify anything time-sensitive such as GitHub-hosted Codespaces or CI
  behavior.
- The earlier branch-cleanup conversation aligns with current refs, but current
  repo truth still beats historical prose if any old docs disagree.

What this suite is for:

- Getting a new coding agent oriented quickly.
- Showing which parts of the engine are implemented versus still prototype or
  TODO-backed.
- Explaining how the editor, game, data layer, scripting layer, Codespaces, and
  CI fit together today.
- Tracking active branches, their purpose, and the next planned work.
- Supporting one last external AI audit before context reset via
  `AUDIT_REQUEST.md`.
