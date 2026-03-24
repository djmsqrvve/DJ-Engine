# dj_engine Complete Package - INDEX & NAVIGATION GUIDE

> **Staleness Warning (2026-03-10):** This guide was written January 2026 before the Bevy 0.18 migration and Makefile infrastructure standardization. The 20-week timeline and some file references are outdated. For current onboarding, use the [AI Handoff Suite](AI_HANDOFF_SUITE/README.md) and [ROADMAP.md](ROADMAP.md).

**Version:** 1.0 (2026-01-21)
**Status:** Production Ready - All Documents Ready for Implementation
**Total Content:** ~9,500 lines of documentation + architecture specs

---

## 📑 COMPLETE FILE LISTING

### Primary Deliverables (Read in This Order)

1. **Implementation_Summary.md** ← **START HERE** (5-10 min read)
   - Quick overview of what you have
   - Quick start guide (first week actions)
   - Success criteria checklist
   - FAQ section

2. **Game_Engine_Technical_Roadmap.md** ← **TECHNICAL DEEP DIVE** (30-45 min read)
   - Complete architecture for Story Graph system
   - Universal Unit component design
   - Director event sequencing system
   - Lua API specification (copy-paste ready)
   - 20-week phased implementation plan
   - Critical architectural decisions with rationale
   - **Action Items:** Copy Rust code snippets directly to project

3. **IDE_Configuration_Guide.md** ← **ENVIRONMENT SETUP** (15-20 min read)
   - VS Code configuration (recommended)
   - 15+ extensions with explanations
   - Pre-built `.vscode/` config files (copy directly)
   - Bevy Remote Protocol for runtime editing
   - CLion/JetBrains alternative setup
   - Team onboarding checklist
   - **Action Items:** `cp config files && code --install-extension ...`

4. **AI_Coding_Assistant_Config.md** ← **PRODUCTIVITY BOOST** (20-30 min read)
   - AI tool comparison (Cursor vs Copilot vs Claude vs ChatGPT)
   - Cursor setup (recommended: $20/mo)
   - 5 copy-paste prompts for architecture decisions
   - Continue.dev integration for self-hosted LLM
   - Real-world debugging workflows
   - Team guidelines for responsible AI use
   - **Action Items:** Install Cursor, save prompts locally

5. **Architecture_Specification.json** ← **REFERENCE** (scan as needed)
   - Machine-readable architecture spec
   - Component inventory (JSON)
   - Phase breakdown (JSON)
   - File structure template (JSON)
   - Success criteria (JSON)
   - **Use:** Import into your project management tool, or reference for specifics

---

## 🎯 BY ROLE: WHICH FILE TO READ?

### Solo Developer
```
Week 1:     Implementation_Summary.md + IDE_Configuration_Guide.md
Week 2-3:   Game_Engine_Technical_Roadmap.md (Sections 1-3)
Week 4+:    Start implementing Phase 1, use AI_Coding_Assistant_Config.md
Reference:  Architecture_Specification.json as checklist
```

### Tech Lead / Architect
```
Priority:   Game_Engine_Technical_Roadmap.md (Sections 5 - Critical Decisions)
Then:       Architecture_Specification.json (JSON structure)
Discussion: Share "5 Critical Architectural Decisions" with team (1-2 hours)
Share:      Implementation_Summary.md decision tree with team
Reference:  All files for comprehensive knowledge transfer
```

### Engine Programmer
```
Start:      Game_Engine_Technical_Roadmap.md Sections 1-3 (code architecture)
Reference:  Copy code snippets into your project
Optimize:   Use AI_Coding_Assistant_Config.md (Prompt 1: StoryGraph Execution)
Debug:      Use IDE_Configuration_Guide.md Section 8 (Bevy Remote Protocol)
```

### Game Designer / Content Creator
```
Read:       Implementation_Summary.md (high-level overview)
Then:       Game_Engine_Technical_Roadmap.md Section 1 (Story Graph basics)
Learn:      AI_Coding_Assistant_Config.md Section 5 (Lua API)
Reference:  Will use Story Graph visual editor (coming Phase 5)
```

### Technical Artist / Tools Programmer
```
Focus:      IDE_Configuration_Guide.md (editor setup)
Then:       Game_Engine_Technical_Roadmap.md Section 4 (Editor Enhancements)
Reference:  Architecture_Specification.json (file structure)
Tools:      Bevy Remote Protocol (IDE_Configuration_Guide.md Section 8)
```

### DevOps / Build Engineer
```
Read:       IDE_Configuration_Guide.md (sections 3, 5 - build tasks)
Reference:  Architecture_Specification.json (dependencies section)
Automate:   CI/CD configuration for cargo build/test/doc
Monitor:    Performance profiling setup (IDE Guide section 9)
```

---

## 🔍 BY QUESTION: FIND YOUR ANSWER

### "How do I get started TODAY?"
→ Implementation_Summary.md → "Quick Start: Implementing dj_engine"

### "What's the architecture for Story Graphs?"
→ Game_Engine_Technical_Roadmap.md → "Section 1: Visual Novel System"

### "How do I design Universal Units?"
→ Game_Engine_Technical_Roadmap.md → "Section 2: Universal Unit Data Structure"

### "What's the Director system?"
→ Game_Engine_Technical_Roadmap.md → "Section 3: Director / Event Sequencing"

### "How do I set up my IDE?"
→ IDE_Configuration_Guide.md → "Section 2-6: Extensions & Settings"

### "Should I use Cursor or Copilot?"
→ AI_Coding_Assistant_Config.md → "Section 1: Tool Comparison"

### "What are the 5 AI prompts I should use?"
→ AI_Coding_Assistant_Config.md → "Section 5: Best Prompts for dj_engine"

### "How long will this take to implement?"
→ Implementation_Summary.md → "Architecture at a Glance" → 20 weeks solo

### "What about time management during dialogue?"
→ Game_Engine_Technical_Roadmap.md → "Section 5.1: Time Management"

### "How do I integrate Lua with Bevy?"
→ Game_Engine_Technical_Roadmap.md → "Section 5.2: Lua Unit API Proposal"

### "What's in the Cargo.toml?"
→ Game_Engine_Technical_Roadmap.md → "Section 5: Recommended Rust Crates"

### "How do I debug at runtime?"
→ IDE_Configuration_Guide.md → "Section 8: Bevy Remote Protocol"

### "What's the file structure?"
→ Architecture_Specification.json → `file_structure` section

### "How do I test my implementation?"
→ Architecture_Specification.json → `testing_strategy` section

### "What are the success criteria?"
→ Architecture_Specification.json → `success_criteria` section  
→ Implementation_Summary.md → "Success Looks Like"

---

## 📊 DOCUMENT STATISTICS

| Document | Lines | Read Time | Code Snippets | Diagrams | JSON |
|----------|-------|-----------|---------------|----------|------|
| Implementation_Summary.md | 350 | 5-10 min | 3 | 1 | - |
| Game_Engine_Technical_Roadmap.md | 2,500 | 30-45 min | 25+ | 2 | 2 |
| IDE_Configuration_Guide.md | 1,500 | 15-20 min | 15 | - | 1 |
| AI_Coding_Assistant_Config.md | 2,000 | 20-30 min | 10 | - | 1 |
| Architecture_Specification.json | 400 | scan | - | - | 1 |
| **TOTAL** | **~6,750** | **~70-105 min** | **~53** | **3** | **5** |

---

## 🚀 IMPLEMENTATION TIMELINE

```
NOW (Today)
│
├─ Read: Implementation_Summary.md (10 min)
├─ Read: IDE_Configuration_Guide.md (15 min)
└─ Action: Set up VS Code + install extensions
        │
        ├─ Weeks 1-4: Phase 1 (Story Graph)
        │ ├─ Reference: Roadmap Section 1.2-1.4
        │ ├─ Copy: StoryNode enum + storyadvancement_system
        │ └─ Test: Load JSON story graph
        │
        ├─ Weeks 5-8: Phase 2 (Director)
        │ ├─ Reference: Roadmap Section 3.2-3.3
        │ ├─ Copy: DirectorCommand enum + director_system
        │ └─ Test: 5-minute cutscene sequence
        │
        ├─ Weeks 9-12: Phase 3 (Universal Unit)
        │ ├─ Reference: Roadmap Section 2.2-2.5
        │ ├─ Copy: Actor, Stats, JRPG/RTS components
        │ └─ Test: Spawn same unit in both games
        │
        ├─ Weeks 13-16: Phase 4 (Lua Integration)
        │ ├─ Reference: Roadmap Section 5.2
        │ ├─ Copy: mlua bindings + unit API
        │ └─ Test: Same Lua script in both games
        │
        └─ Weeks 17-20: Phase 5 (Editor Enhancements)
          ├─ Reference: Roadmap Section 1.6
          ├─ Tool 1: Story Graph visual editor
          ├─ Tool 2: Story variable inspector
          └─ Tool 3: Story asset manager
```

---

## 💾 HOW TO USE THESE DOCUMENTS

### As a Solo Developer

```bash
# Clone/download all files
git clone <dj_engine repo>
cd dj_engine

# Week 1: Setup
cat Implementation_Summary.md
cat IDE_Configuration_Guide.md
cp -r IDE_Configuration_Guide.md/.vscode .vscode/

# Week 2-3: Learn Architecture
less Game_Engine_Technical_Roadmap.md
# (Read sections 1-3, take notes)

# Week 4: Start Coding
cargo init
# Copy code from Roadmap section 1.2 (StoryNode enum)
# Copy code from Roadmap section 1.4 (story_advancement_system)
cargo build

# Weeks 5-20: Implement phases
# Reference Roadmap for each phase
# Use AI_Coding_Assistant_Config.md for debugging
```

### As a Team Lead

```bash
# Day 1: Distribute documents
share Implementation_Summary.md with whole team
share IDE_Configuration_Guide.md with all developers

# Day 2: Architecture review (1-2 hours)
team meeting: review Roadmap "5 Critical Decisions" (Section 5)
discussion: confirm approach aligns with your goals

# Day 3: Assign roles
solo dev → phases 1-2, then 3-4
tools dev → phases 1-5 (editor tools)

# Weekly: Progress tracking
use Architecture_Specification.json deliverables_checklist
report: which phase is complete, what's blockers
```

### As a Content Creator (Later Phase 5)

```bash
# When Story Graph visual editor is ready:
open editor
read Roadmap Section 1.2 (StoryNodeType enum) for features
create story graphs by dragging nodes
export to JSON
commit to git

# When Lua API is ready:
read AI_Coding_Assistant_Config.md (Lua bindings section)
write Lua scripts using unit:*, party:*, trigger_* functions
test in both DoomExe and RTS
iterate
```

---

## 🔐 STORAGE & VERSION CONTROL

### What to Commit to Git

```bash
# ✅ Commit these documents
git add Game_Engine_Technical_Roadmap.md
git add IDE_Configuration_Guide.md
git add AI_Coding_Assistant_Config.md
git add Architecture_Specification.json
git add Implementation_Summary.md
git commit -m "docs: complete dj_engine architecture & implementation guide"

# ✅ Commit these config files
git add .vscode/settings.json
git add .vscode/launch.json
git add .vscode/tasks.json
git add .vscode/extensions.json

# ✅ Commit story graphs (JSON)
git add assets/story_graphs/*.json

# ✅ Commit Lua scripts
git add assets/lua/*.lua

# ❌ Don't commit
# .vscode/settings (user-specific)
# target/ (build artifacts)
# Cargo.lock (if library)
```

### Staying Updated

```
Check quarterly for updates:
- Bevy release → review breaking changes vs Roadmap
- Rust language updates → verify code still compiles
- New LLM capabilities → update AI prompts in AI_Coding_Assistant_Config.md
- IDE tool updates → refresh IDE_Configuration_Guide.md
```

---

## 🎓 LEARNING RESOURCES REFERENCED

### Bevy 0.18 Documentation
- Official: https://bevyengine.org/learn/quick-start/
- Docs.rs: https://docs.rs/bevy/0.18/bevy/

### Rust + ECS Resources
- FOSDEM 2026: "Practical ECS for Game Development in Rust with Bevy"
- Bevy Cheat Book: https://bevy-cheatbook.dev/

### Lua Integration
- mlua crate: https://docs.rs/mlua/
- Spring Engine (open-source RTS + Lua): https://springrts.com/

### Narrative Design
- Game Developer Magazine: "Branching Narrative Techniques"
- Baldur's Gate 3 case study: Critical path design

### AI Coding Tools (2026 Update)
- Cursor: https://cursor.sh/
- Continue.dev: https://continue.dev/
- Claude 3.5 Sonnet: https://www.anthropic.com/
- GitHub Copilot: https://github.com/features/copilot/

---

## ✅ PRE-IMPLEMENTATION CHECKLIST

Before you start coding, make sure you have:

- [ ] Read Implementation_Summary.md (5-10 min)
- [ ] Read IDE_Configuration_Guide.md (15 min)
- [ ] VS Code installed + Rust extensions
- [ ] Rust toolchain installed (`rustup`)
- [ ] Bevy dependencies installed (OS-specific)
- [ ] Decision: Cursor or Copilot for AI assistance
- [ ] Decision: Solo or team development
- [ ] Cargo project initialized (`cargo init dj_engine`)
- [ ] `.vscode/` config files in place
- [ ] First build successful (`cargo build`)
- [ ] Created `src/components/` and `src/systems/` directories
- [ ] Read Game_Engine_Technical_Roadmap.md Sections 1-3
- [ ] Ready to start Phase 1 (Story Graph)

---

## 📞 GETTING HELP

### If you're stuck on...

**Rust syntax errors?**
→ Use Cursor with "Show compilation error explanation"  
→ Or: Paste error in ChatGPT/Claude with code context

**Bevy ECS concepts?**
→ Read Bevy official docs (links in IDE Guide)  
→ Watch FOSDEM 2026 presentation (linked above)

**Story Graph architecture?**
→ Re-read Roadmap Section 1.2-1.4 (carefully)  
→ Or: Prompt Claude: "Why is StoryNodeType an enum instead of a trait?"

**Lua integration issues?**
→ Check mlua docs: https://docs.rs/mlua/  
→ Use AI prompt from AI_Coding_Assistant_Config.md Section 5 (Prompt 3)

**Performance bottlenecks?**
→ Follow profiling steps in IDE_Configuration_Guide.md Section 9  
→ Use Bevy's built-in diagnostics system

**Team coordination?**
→ Use Architecture_Specification.json as reference  
→ Hold weekly standup on Implementation_Summary.md phases

---

## 📝 DOCUMENT METADATA

| Property | Value |
|----------|-------|
| **Total Package Size** | ~6,750 lines of documentation |
| **Estimated Read Time** | 70-105 minutes (all documents) |
| **Estimated Implementation Time** | 20 weeks (1 dev), 10 weeks (3 devs) |
| **Target Framework** | Bevy 0.18 |
| **Language** | Rust |
| **Scripting** | Lua 5.4 |
| **Created** | 2026-01-21 |
| **Status** | Production Ready |
| **Next Update** | Q2 2026 |

---

## 🎉 YOU ARE NOW READY TO BUILD

You have:
✅ Complete technical architecture  
✅ Copy-paste ready code snippets  
✅ IDE configuration (ready to use)  
✅ AI coding prompts (ready to paste)  
✅ 20-week implementation plan  
✅ Success metrics and checklist  

**Next step:** Open Implementation_Summary.md, follow "Quick Start" section, and start coding today!

---

**Happy building! Questions? Read the appropriate section above, then ask Claude/Cursor using prompts from AI_Coding_Assistant_Config.md.**
