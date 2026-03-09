# External Audit Request

You are the final external auditing AI for DJ Engine. Decide whether this
handoff suite is adequate for a new agent and final IDE closeout.

Read these files first:

1. `/home/dj/dev/engines/DJ-Engine/docs/AI_HANDOFF_SUITE/README.md`
2. `/home/dj/dev/engines/DJ-Engine/docs/AI_HANDOFF_SUITE/00_ACCURACY_AUDIT.md`
3. `/home/dj/dev/engines/DJ-Engine/docs/AI_HANDOFF_SUITE/01_CURRENT_STATE.md`
4. `/home/dj/dev/engines/DJ-Engine/docs/AI_HANDOFF_SUITE/07_AGENT_WORKFLOW.md`
5. `/home/dj/dev/engines/DJ-Engine/Cargo.toml`
6. `/home/dj/dev/engines/DJ-Engine/games/dev/doomexe/src/main.rs`

Current repo truth to audit against:

- repo: `/home/dj/dev/engines/DJ-Engine`
- branch: `main`
- HEAD: `5f0107d`
- local and remote branch picture: `main` and `origin/main`
- repo state: clean

Critical audit requirement:

- verify that stale Bevy-version or roadmap docs cannot easily mislead a new
  agent

Audit tasks:

1. Decide whether the suite is sufficient for a new agent to work safely in the
   current workspace.
2. Confirm whether the branch-cleanup and current-state claims are documented
   clearly enough.
3. Identify any missing warnings about stale broader docs versus current code.
4. If not ready, provide exact markdown edits.

Required output:

- verdict: `ready` or `not ready`
- confidence: `high`, `medium`, or `low`
- missing or weak areas
- exact suggested doc edits if needed
