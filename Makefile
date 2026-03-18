# DJ Engine - Unified Command Interface

.PHONY: help check build test lint fmt format-fix clean dev engine editor preview new-game game doom stratego iso minimal quality-check guardrail contracts validate helix-import helix-import-toml helix-export helix-dashboard helix-editor helix-preview dev-exe

# Ensure rustup toolchain takes precedence over system cargo/rustc
export PATH := $(HOME)/.cargo/bin:$(PATH)

CARGO_TARGET_DIR ?= $(HOME)/.cargo-targets/dj_engine_bevy18
export CARGO_TARGET_DIR

# Default target
help:
	@echo "DJ Engine - Command Interface"
	@echo ""
	@echo "Quick Start:"
	@echo "  make dev          Launch the engine editor"
	@echo "  make engine       Alias for 'make editor'"
	@echo "  make editor       Launch the engine editor"
	@echo "  make new-game     Create a new game project (NAME=<name> DIR=<path> optional)"
	@echo "  make preview      Launch runtime preview (PROJECT=<dir|project.json>)"
	@echo "  make helix-import Import Helix dist into a mounted project (HELIX_DIST=<dir> PROJECT=<dir|project.json>)"
	@echo "  make helix-import-toml Load typed TOML registries (HELIX3D=<dir>)"
	@echo "  make helix-export Export LoadedCustomDocuments back to helix3d TOML (HELIX3D=<input> OUTPUT=<dir>)"
	@echo "  make helix-dashboard Run Helix data contract validation (HELIX3D=<dir>)"
	@echo "  make helix-editor Launch the Helix editor wrapper (PROJECT=<dir|project.json> optional)"
	@echo "  make helix-preview Launch the Helix runtime preview wrapper (PROJECT=<dir|project.json>)"
	@echo "  make game         Run the sample DoomExe game"
	@echo "  make doom         Alias for 'make game'"
	@echo "  make minimal      Run minimal rendering binary"
	@echo ""
	@echo "Distribution:"
	@echo "  make dev-exe      Build standalone editor exe (release, static, stripped)"
	@echo ""
	@echo "Quality:"
	@echo "  make check        cargo check --workspace"
	@echo "  make build        cargo build --workspace"
	@echo "  make test         Run all tests"
	@echo "  make lint         Run clippy"
	@echo "  make fmt          Check formatting"
	@echo "  make format-fix   Fix formatting"
	@echo "  make quality-check Full pipeline (fmt + clippy + test)"
	@echo "  make guardrail    Quick safety checks"
	@echo "  make contracts    Print engine API contracts dashboard"
	@echo "  make validate     Full QA pipeline (fmt + clippy + test + contracts)"
	@echo ""
	@echo "Utility:"
	@echo "  make clean        Clean build artifacts"

# Quick Start

dev: editor

engine: editor

editor:
	@cargo run -p dj_engine --bin dj_engine

new-game:
	@test -n "$(NAME)" || (echo "NAME is required: make new-game NAME=\"My Game\" [DIR=<path>]"; exit 1)
	@if [ -n "$(DIR)" ]; then \
		cargo run -p dj_engine --bin project_init -- "$(NAME)" --dir "$(DIR)"; \
	else \
		cargo run -p dj_engine --bin project_init -- "$(NAME)"; \
	fi

preview:
	@test -n "$(PROJECT)" || (echo "PROJECT is required: make preview PROJECT=<dir|project.json>"; exit 1)
	@cargo run -p dj_engine --bin runtime_preview -- --project "$(PROJECT)"

helix-import:
	@test -n "$(HELIX_DIST)" || (echo "HELIX_DIST is required: make helix-import HELIX_DIST=<dir> PROJECT=<dir|project.json>"; exit 1)
	@test -n "$(PROJECT)" || (echo "PROJECT is required: make helix-import HELIX_DIST=<dir> PROJECT=<dir|project.json>"; exit 1)
	@cargo run -p dj_engine_helix --bin helix_import -- --helix-dist "$(HELIX_DIST)" --project "$(PROJECT)"

helix-import-toml:
	@test -n "$(HELIX3D)" || (echo "HELIX3D is required: make helix-import-toml HELIX3D=<dir>"; exit 1)
	@cargo run -p dj_engine_helix --bin helix_import -- --helix3d "$(HELIX3D)"

helix-export:
	@test -n "$(HELIX3D)" || (echo "HELIX3D is required: make helix-export HELIX3D=<input_dir> OUTPUT=<output_dir>"; exit 1)
	@test -n "$(OUTPUT)" || (echo "OUTPUT is required: make helix-export HELIX3D=<input_dir> OUTPUT=<output_dir>"; exit 1)
	@cargo run -p dj_engine_helix --bin helix_export -- --helix3d "$(HELIX3D)" --output "$(OUTPUT)"

helix-dashboard:
	@test -n "$(HELIX3D)" || (echo "HELIX3D is required: make helix-dashboard HELIX3D=<dir>"; exit 1)
	@cargo run -p dj_engine_helix --bin helix_dashboard -- --helix3d "$(HELIX3D)"

helix-editor:
	@HELIX_ARGS=""; \
	if [ -n "$(PROJECT)" ]; then HELIX_ARGS="$$HELIX_ARGS --project $(PROJECT)"; fi; \
	if [ -n "$(HELIX_DIST)" ]; then HELIX_ARGS="$$HELIX_ARGS --helix-dist $(HELIX_DIST)"; fi; \
	if [ -n "$$HELIX_ARGS" ]; then \
		cargo run -p dj_engine_helix --bin helix_editor -- $$HELIX_ARGS; \
	else \
		cargo run -p dj_engine_helix --bin helix_editor; \
	fi

helix-preview:
	@test -n "$(PROJECT)" || (echo "PROJECT is required: make helix-preview PROJECT=<dir|project.json>"; exit 1)
	@cargo run -p dj_engine_helix --bin helix_runtime_preview -- --project "$(PROJECT)"

game:
	@cargo run -p doomexe --bin doomexe

doom: game

stratego:
	@cargo run -p stratego --bin stratego

iso:
	@cargo run -p iso_sandbox

minimal:
	@cargo run -p dj_engine --bin minimal

# Distribution

DIST_DIR := dist
BUILD_LATEST := /tmp/dj-engine-builds/latest
VERSION := $(shell date +%Y%m%d-%H%M%S)

dev-exe:
	@echo "Building DJ Engine editor (release, static linking, stripped)..."
	@cargo build -p dj_engine --bin dj_engine --release --no-default-features
	@mkdir -p $(DIST_DIR)
	@cp $(CARGO_TARGET_DIR)/release/dj_engine $(DIST_DIR)/dj_engine
	@mkdir -p $(BUILD_LATEST)
	@cp $(CARGO_TARGET_DIR)/release/dj_engine $(BUILD_LATEST)/dj_engine
	@cp $(CARGO_TARGET_DIR)/release/dj_engine $(BUILD_LATEST)/dj_engine-$(VERSION)
	@ls -lh $(BUILD_LATEST)/dj_engine
	@echo ""
	@echo "Build complete:"
	@echo "  Local:   $(DIST_DIR)/dj_engine"
	@echo "  Latest:  $(BUILD_LATEST)/dj_engine"
	@echo "  Tagged:  $(BUILD_LATEST)/dj_engine-$(VERSION)"

# Quality

check:
	@cargo check --workspace

build:
	@cargo build --workspace

test:
	@cargo test --workspace

lint:
	@cargo clippy --workspace --all-targets -- -W clippy::all

fmt:
	@cargo fmt --all --check

format-fix:
	@cargo fmt --all

quality-check:
	@echo "Checking format..."
	@cargo fmt --all --check
	@echo "Running clippy..."
	@cargo clippy --workspace --all-targets -- -W clippy::all
	@echo "Running tests..."
	@cargo test --workspace
	@echo "All quality checks passed."

contracts:
	@cargo run -p dj_engine --bin contracts

validate:
	@echo "=== DJ Engine Validate ==="
	@echo ""
	@echo "[1/5] Checking format..."
	@cargo fmt --all --check || (echo "FAILED: Formatting issues"; exit 1)
	@echo "[2/5] Running clippy..."
	@cargo clippy --workspace --all-targets -- -W clippy::all || (echo "FAILED: Clippy warnings"; exit 1)
	@echo "[3/5] Running tests..."
	@cargo test --workspace || (echo "FAILED: Tests broken"; exit 1)
	@echo "[4/5] Checking contracts..."
	@cargo run -p dj_engine --bin contracts
	@echo "[5/5] Checking test count..."
	@test_count=$$(cargo test --workspace -- --list 2>/dev/null | grep -c ': test$$'); \
	if [ "$$test_count" -lt 300 ]; then \
		echo "FAILED: Test count regression ($$test_count < 300)"; exit 1; \
	else \
		echo "Test count: $$test_count (>= 300 minimum)"; \
	fi
	@echo ""
	@echo "=== All validation passed ==="

guardrail:
	@echo "Running guardrail checks..."
	@cargo check --workspace || (echo "FAILED: Build broken"; exit 1)
	@cargo test --workspace || (echo "FAILED: Tests broken"; exit 1)
	@cargo fmt --all --check || (echo "FAILED: Formatting issues"; exit 1)
	@echo "All guardrails passed."

# Utility

clean:
	@cargo clean
