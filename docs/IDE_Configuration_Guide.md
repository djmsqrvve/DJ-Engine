# IDE Configuration & Extensions Recommendations (2026)

## 1. RECOMMENDED IDE SETUP FOR dj_engine

### Primary IDE: VS Code + Rust-Analyzer (Cursor is an excellent alternative)

**Why VS Code?**
- Best Rust extension ecosystem (rust-analyzer, better_TOML, etc.)
- Egui + Bevy debugging support mature
- Remote Protocol integration (Bevy 0.18 inspector over network)
- Fast startup, lightweight

**Why Cursor if you have budget ($20/month)?**
- VS Code fork with AI embedded into editor
- Codebase-aware refactoring (critical for ECS architecture changes)
- Multi-file edits with AI understanding of entity relationships
- Worth the cost for architecture-heavy projects like dj_engine

---

## 2. ESSENTIAL VS CODE EXTENSIONS

### Code Quality & Navigation

| Extension | Purpose | Version | Config |
|-----------|---------|---------|--------|
| **rust-analyzer** | Rust intelligence, debugging | Latest | Enable proc macros: `"rust-analyzer.procMacroServer"` |
| **Better TOML** | Cargo.toml syntax highlighting | 0.3.3+ | Auto-format on save |
| **Even Better TOML** | (Optional) Alternative TOML support | Latest | - |
| **Bevy Diagnostics** (community) | Bevy-specific warnings/hints | Latest | Highlights ECS antipatterns |
| **CodeLLDB** | Debugger for Rust | 1.10.0+ | Configure for Bevy hot-reload |

### Editor Enhancements

| Extension | Purpose | Config |
|-----------|---------|--------|
| **EditorConfig** | Consistent formatting across team | Use `.editorconfig` at project root |
| **Trailing Comma** | Auto-add commas in JSON | Useful for .ron files |
| **Bracket Pair Colorizer 2** | Color-code nested brackets | Help with complex Bevy queries |
| **Error Lens** | Inline error display | Show warnings from rust-analyzer |

### Bevy-Specific

| Extension | Purpose | Install Command |
|-----------|---------|-----------------|
| **Bevy Cheat Book** | In-editor Bevy 0.18 API docs | `bevy_cheatbook` snippet pack |
| **WGSL** | WebGPU shader support (Bevy uses WGSL) | For graphics debugging |
| **Biome** or **Prettier** | Format Rust/JSON/TOML consistently | Use Biome for Rust formatting |

### Optional but Recommended

| Extension | Purpose | Why |
|-----------|---------|-----|
| **Copilot** or **Claude** | AI code completion | Quick prototyping, ECS boilerplate |
| **GitLens** | Git blame + history | Track ECS component changes |
| **Thunder Client** / **REST Client** | API testing | Test Bevy Remote Protocol |
| **Markdown All in One** | Documentation writing | For roadmap/architecture docs |

---

## 3. VS CODE SETTINGS (`.vscode/settings.json`)

```json
{
  // Rust-Analyzer Configuration
  "[rust]": {
    "editor.defaultFormatter": "rust-lang.rust-analyzer",
    "editor.formatOnSave": true,
    "editor.codeActionsOnSave": {
      "source.fixAll.clippy": "explicit"
    }
  },
  
  "rust-analyzer.checkOnSave.command": "clippy",
  "rust-analyzer.checkOnSave.extraArgs": [
    "--all-targets",
    "--",
    "-W",
    "clippy::all"
  ],
  
  // Enable proc macro debugging (critical for Bevy #[derive()])
  "rust-analyzer.procMacroServer": true,
  "rust-analyzer.linkedProjects": [
    "./Cargo.toml"
  ],
  
  // ECS query optimization hints
  "rust-analyzer.inlayHints.enabled": true,
  "rust-analyzer.inlayHints.typeHints.enabled": true,
  "rust-analyzer.inlayHints.parameterHints.enabled": true,
  
  // Performance
  "rust-analyzer.lens.enable": true,
  "rust-analyzer.hover.documentation.enable": true,
  
  // Formatting
  "[json]": {
    "editor.defaultFormatter": "esbenp.prettier-vscode",
    "editor.formatOnSave": true
  },
  
  "[toml]": {
    "editor.defaultFormatter": "Even Better TOML",
    "editor.formatOnSave": true
  },
  
  // General Editor Config
  "editor.rulers": [100, 120],  // Column guides for ECS component width
  "editor.insertSpaces": true,
  "editor.tabSize": 4,
  "editor.trimAutoWhitespace": true,
  "files.trimTrailingWhitespace": true,
  "files.insertFinalNewline": true,
  
  // Search Settings
  "search.exclude": {
    "**/target": true,
    "**/.cargo": true,
    "**/node_modules": true
  },
  
  // Bevy-Specific Exclusions
  "files.watcherExclude": {
    "**/.git/objects/**": true,
    "**/target/**": true,
    "**/.cargo/**": true
  },
  
  // Git Configuration
  "git.ignoreLimitWarning": true,
  "git.autofetch": true,
  
  // Terminal
  "terminal.integrated.defaultProfile.linux": "bash",
  "terminal.integrated.defaultProfile.windows": "PowerShell",
  "terminal.integrated.defaultProfile.osx": "zsh"
}
```

---

## 4. LAUNCH CONFIGURATION (`.vscode/launch.json`)

```json
{
  "version": "0.2.0",
  "configurations": [
    {
      "name": "dj_engine (debug)",
      "type": "lldb",
      "request": "launch",
      "cargo": {
        "args": [
          "build",
          "--bin=dj_engine",
          "--message-format=json"
        ],
        "filter": {
          "name": "dj_engine",
          "kind": "bin"
        }
      },
      "args": [],
      "cwd": "${workspaceFolder}",
      "stopOnEntry": false,
      "console": "integratedTerminal",
      "sourceLanguages": ["rust"],
      "preLaunchTask": "cargo: build",
      "env": {
        "RUST_LOG": "debug,bevy_render::renderer=info"
      }
    },
    {
      "name": "DoomExe (JRPG)",
      "type": "lldb",
      "request": "launch",
      "cargo": {
        "args": [
          "build",
          "--release",
          "--bin=doomexe",
          "--message-format=json"
        ]
      },
      "preLaunchTask": "cargo: build release"
    },
    {
      "name": "RTS-TBD (Single Player)",
      "type": "lldb",
      "request": "launch",
      "cargo": {
        "args": [
          "build",
          "--release",
          "--bin=rts_tbd"
        ]
      }
    },
    {
      "name": "Bevy Hot Reload",
      "type": "lldb",
      "request": "launch",
      "cargo": {
        "args": [
          "build",
          "--bin=dj_engine",
          "--features=bevy/dynamic_linking"
        ]
      },
      "env": {
        "RUST_BACKTRACE": "1"
      }
    }
  ]
}
```

---

## 5. TASKS CONFIGURATION (`.vscode/tasks.json`)

```json
{
  "version": "2.0.0",
  "tasks": [
    {
      "label": "cargo: build",
      "type": "shell",
      "command": "cargo",
      "args": ["build"],
      "problemMatcher": ["$rustc"],
      "presentation": {
        "echo": true,
        "reveal": "always",
        "panel": "shared",
        "clear": false
      }
    },
    {
      "label": "cargo: build release",
      "type": "shell",
      "command": "cargo",
      "args": ["build", "--release"],
      "problemMatcher": ["$rustc"],
      "presentation": { "reveal": "always" }
    },
    {
      "label": "cargo: check",
      "type": "shell",
      "command": "cargo",
      "args": ["check", "--all-targets"],
      "problemMatcher": ["$rustc"],
      "runOptions": { "runOn": "folderOpen" }
    },
    {
      "label": "cargo: clippy",
      "type": "shell",
      "command": "cargo",
      "args": ["clippy", "--all-targets", "--", "-W", "clippy::all"],
      "problemMatcher": ["$rustc"]
    },
    {
      "label": "cargo: test",
      "type": "shell",
      "command": "cargo",
      "args": ["test", "--lib"],
      "problemMatcher": ["$rustc"]
    },
    {
      "label": "cargo: fmt check",
      "type": "shell",
      "command": "cargo",
      "args": ["fmt", "--all", "--check"],
      "problemMatcher": []
    },
    {
      "label": "cargo: fmt fix",
      "type": "shell",
      "command": "cargo",
      "args": ["fmt", "--all"],
      "problemMatcher": []
    },
    {
      "label": "Hot Reload Watch",
      "type": "shell",
      "command": "cargo",
      "args": ["watch", "-x", "check", "-x", "clippy"],
      "isBackground": true,
      "problemMatcher": ["$rustc"]
    },
    {
      "label": "Documentation (serve)",
      "type": "shell",
      "command": "cargo",
      "args": ["doc", "--no-deps", "--open"],
      "presentation": { "reveal": "always" }
    }
  ]
}
```

---

## 6. WORKSPACE RECOMMENDATIONS (`.vscode/extensions.json`)

```json
{
  "recommendations": [
    "rust-lang.rust-analyzer",
    "tamasfe.even-better-toml",
    "vadimcn.vscode-lldb",
    "GitHub.Copilot",
    "eamodio.gitlens",
    "ms-vscode.makefile-tools",
    "charliermarsh.ruff",
    "esbenp.prettier-vscode",
    "mhutchie.git-graph",
    "ms-vscode-remote.remote-containers",
    "sonarsource.sonarlint-vscode"
  ]
}
```

---

## 7. ALTERNATIVE IDE: JetBrains CLion + IntelliJ IDEA

**Pros:**
- Superior Rust debugging
- Integrated profiler
- Better refactoring tools

**Cons:**
- $15/month (CLion)
- Heavier resource consumption
- Slower startup than VS Code

**If using CLion, configure:**

```xml
<!-- .idea/runConfigurations/Bevy_Debug.xml -->
<component name="ProjectRunConfigurationManager">
  <configuration default="false" name="Bevy Debug" type="CargoCommandRunConfiguration" factoryName="Cargo">
    <option name="command" value="build" />
    <option name="workingDirectory" value="$PROJECT_DIR$" />
    <envs>
      <env name="RUST_BACKTRACE" value="1" />
      <env name="RUST_LOG" value="debug" />
    </envs>
  </configuration>
</component>
```

---

## 8. REMOTE DEVELOPMENT (Bevy Remote Protocol)

### Connecting External Editor to Running Bevy App

Bevy 0.18 introduced **Bevy Remote Protocol (BRP)** allowing editors to inspect/modify ECS at runtime.

**Setup (VS Code):**

1. Install **Thunder Client** or **REST Client** extension
2. Create `.vscode/requests.rest`:

```http
### Connect to Bevy Remote Protocol
POST http://localhost:9000/connect
Content-Type: application/json

{
  "protocol_version": "1.0"
}

### Query all entities
GET http://localhost:9000/entities

### Inspect specific entity
GET http://localhost:9000/entity/{entity_id}

### List components on entity
GET http://localhost:9000/entity/{entity_id}/components

### Modify component value
PATCH http://localhost:9000/entity/{entity_id}/component/Transform
Content-Type: application/json

{
  "translation": [10.0, 5.0, 0.0],
  "rotation": [0.0, 0.0, 0.0, 1.0]
}
```

**In your `main.rs`:**

```rust
use bevy::prelude::*;
use bevy::remote::{RemotePlugin};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(RemotePlugin::default()) // Enables BRP on :9000
        .run();
}
```

Now you can **inspect and modify entities while the game runs**—no recompile!

---

## 9. ADDITIONAL TOOLS FOR GAME DEVELOPMENT

### Asset Management & Debugging

- **Blender** (3D/2D art): Use GLTF export → Bevy loads directly
- **Aseprite** (sprite animation): Export as PNG sprite sheets → Bevy sprite loader
- **Audacity** (audio): Export as WAV → Bevy MIDI support
- **Visual Studio Code Extension: Bevy Game Development Kit** (in development)

### Version Control & Collaboration

```bash
# .gitignore for dj_engine projects
target/
*.exe
*.o
*.so
*.dylib
*.swp
*.swo
*~
.DS_Store
.vscode/
.idea/
Cargo.lock  # For libraries; include for binary projects

# Hot reload artifacts
*.o.bevy_hot
```

### Performance Profiling

**Bevy Built-in Profiler:**

```rust
pub fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(FrameTimeDiagnosticsPlugin)
        .add_systems(Update, print_diagnostics)
        .run();
}

fn print_diagnostics(diagnostics: Res<Diagnostics>) {
    if let Some(fps_diagnostic) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
        if let Some(fps) = fps_diagnostic.smoothed() {
            println!("FPS: {:.2}", fps);
        }
    }
}
```

---

## 10. TEAM SETUP CHECKLIST

```markdown
# Project Setup for New Team Members

- [ ] Clone repository
- [ ] Install Rust: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- [ ] Install Bevy dependencies:
  - Linux: `sudo apt install -y gcc g++ pkg-config libx11-dev libxrandr-dev libxinerama-dev libxcursor-dev libxi-dev libasound2-dev`
  - macOS: No extra deps (just Xcode CLI tools)
  - Windows: Visual Studio Build Tools (C++ workload)
- [ ] Open in VS Code
- [ ] Install extensions from `.vscode/extensions.json`
- [ ] Run `cargo build` to verify setup
- [ ] Run `cargo check --all-targets` (should pass)
- [ ] Launch Bevy editor: `cargo run`
- [ ] Verify story graph loads correctly
```

---

**Configuration Version:** 2026-01-21  
**IDE Support:** VS Code (primary), Cursor (if budget allows), CLion (if you prefer JetBrains)
