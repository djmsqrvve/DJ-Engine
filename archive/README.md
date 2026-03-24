# Archive

Legacy documentation from early DJ Engine development (pre-Bevy 0.18, pre-Makefile
standardization), plus historical planning docs and AI agent session files. These
are kept for historical reference but should not be trusted over current source
code or the docs in `docs/`.

## Legacy Design Docs

| File | Description |
|------|-------------|
| AI_SUMMARY.md | Early AI-generated project summary |
| ANIMATION_GUIDE.md | Animation framework deep dive (pre-0.18) |
| ARCHITECTURE.md | Original engine architecture overview |
| ASSET_PIPELINE.md | Asset loading and pipeline design |
| CHANGELOG.md | Historical changelog |
| CODING_STANDARDS.md | Early coding standards (superseded by `docs/CODE_STYLE.md`) |
| dj-sprite-system-plan.md | Sprite system design plan |
| DOCUMENTATION_INDEX.md | Old documentation index |
| DOCUMENTATION_SUMMARY.md | Old documentation summary |
| enginefeaturedraft.md | Undated engine feature checklist draft |
| enginefeaturedraftjson.md | Undated engine feature spec (JSON schema) |
| hamster_milestone.md | Hamster prototype milestone notes |
| LUA_FFI.md | Lua FFI integration design |
| PROJECT_PLAN.md | Original project scaffolding plan |
| QUICKSTART.md | Old quickstart guide (references `./dj` script) |
| SPRITE_ARCHITECTURE.md | Sprite rendering pipeline spec |
| SPRITE_QUICKSTART.md | Sprite system quickstart |
| SPRITE_RESEARCH_SUMMARY.md | Sprite rendering research notes |
| SPRITE_SYSTEM.md | Complete sprite system specification |
| SPRITE_SYSTEM_V2.md | Sprite system v2 iteration |
| WORKFLOW.md | Old development workflow (references `./dj` script) |

## Docs Moved From docs/ (2026-03-24)

| File | Description |
|------|-------------|
| Architecture_Specification.json | Historical high-level architecture artifact |
| complete-detailed-docs.md | Historical implementation draft (64KB) |
| DELIVERABLES_Summary.md | Deliverables summary |
| DETAILED_TASK_DOCS.md | Historical task/spec draft (25KB) |
| docs-summary-reference.md | Old doc summary reference |
| EDITOR_Specification_Complete.md | Older editor specification (25KB) |
| Game_Engine_Technical_Roadmap.md | 20-week planning document (26KB) |
| HANDOFF_BEVY18_MIGRATION.md | Bevy 0.18 migration handoff notes |
| HANDOFF_BROAD_AUDIT.md | Broad audit handoff notes |
| HANDOFF_CODESPACES_COMPILE_CLEANUP.md | Codespaces compile cleanup handoff |
| IDE_Configuration_Guide.md | VS Code + Rust setup guide |
| Implementation_Summary.md | Implementation summary (25KB) |
| INDEX_Navigation_Guide.md | Old documentation navigation guide |
| TUTORIAL_PROJECT.md | Tutorial project planning notes |

## Other Archived Files

| File | Description |
|------|-------------|
| dj | Retired helper script (replaced by `make` targets) |

## AI Agent Files (`ai/`)

The `ai/` subdirectory contains AI agent configuration and session handoff files
that were used during development. These are internal tooling context, not
contributor-facing documentation.

| Path | Description |
|------|-------------|
| ai/AGENTS.md | AI agent configuration guide |
| ai/GEMINI.md | Gemini AI technical specification |
| ai/AI_Coding_Assistant_Config.md | LLM coding guidance |
| ai/enginebuildingprompt.txt | Engine building prompt |
| ai/AI_HANDOFF_SUITE/ | 18 session handoff files for inter-session continuity |
