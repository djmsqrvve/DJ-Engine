# Next-Agent Prompt

You are taking over DJ Engine from a prior agent. Use current repo truth, not
historical roadmap assumptions.

Read these files first:

1. `/home/dj/dev/engines/DJ-Engine/docs/AI_HANDOFF_SUITE/00_ACCURACY_AUDIT.md`
2. `/home/dj/dev/engines/DJ-Engine/docs/AI_HANDOFF_SUITE/01_CURRENT_STATE.md`
3. `/home/dj/dev/engines/DJ-Engine/docs/AI_HANDOFF_SUITE/07_AGENT_WORKFLOW.md`
4. `/home/dj/dev/engines/DJ-Engine/Cargo.toml`
5. `/home/dj/dev/engines/DJ-Engine/engine/src/lib.rs`
6. `/home/dj/dev/engines/DJ-Engine/games/dev/doomexe/src/main.rs`

Ground truth:

- repo: `/home/dj/dev/engines/DJ-Engine`
- branch: `main`
- HEAD: `5f0107d`
- local and remote branch picture: `main` and `origin/main`
- worktree: clean

Critical rule:

- if older docs disagree with source or current refs, trust source and refs

Your first response should restate the current workspace shape, identify any
stale docs that could mislead a fresh agent, and propose the smallest safe next
engineering task.
