# DJ Engine - Unified Command Interface

.PHONY: help check build test lint fmt format-fix clean dev engine editor preview game doom minimal quality-check guardrail helix-import helix-editor helix-preview

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
	@echo "  make preview      Launch runtime preview (PROJECT=<dir|project.json>)"
	@echo "  make helix-import Import Helix dist into a mounted project (HELIX_DIST=<dir> PROJECT=<dir|project.json>)"
	@echo "  make helix-editor Launch the Helix editor wrapper (PROJECT=<dir|project.json> optional)"
	@echo "  make helix-preview Launch the Helix runtime preview wrapper (PROJECT=<dir|project.json>)"
	@echo "  make game         Run the sample DoomExe game"
	@echo "  make doom         Alias for 'make game'"
	@echo "  make minimal      Run minimal rendering binary"
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
	@echo ""
	@echo "Utility:"
	@echo "  make clean        Clean build artifacts"

# Quick Start

dev: editor

engine: editor

editor:
	@cargo run -p dj_engine --bin dj_engine

preview:
	@test -n "$(PROJECT)" || (echo "PROJECT is required: make preview PROJECT=<dir|project.json>"; exit 1)
	@cargo run -p dj_engine --bin runtime_preview -- --project "$(PROJECT)"

helix-import:
	@test -n "$(HELIX_DIST)" || (echo "HELIX_DIST is required: make helix-import HELIX_DIST=<dir> PROJECT=<dir|project.json>"; exit 1)
	@test -n "$(PROJECT)" || (echo "PROJECT is required: make helix-import HELIX_DIST=<dir> PROJECT=<dir|project.json>"; exit 1)
	@cargo run -p dj_engine_helix --bin helix_import -- --helix-dist "$(HELIX_DIST)" --project "$(PROJECT)"

helix-editor:
	@if [ -n "$(PROJECT)" ]; then \
		cargo run -p dj_engine_helix --bin helix_editor -- --project "$(PROJECT)"; \
	else \
		cargo run -p dj_engine_helix --bin helix_editor; \
	fi

helix-preview:
	@test -n "$(PROJECT)" || (echo "PROJECT is required: make helix-preview PROJECT=<dir|project.json>"; exit 1)
	@cargo run -p dj_engine_helix --bin helix_runtime_preview -- --project "$(PROJECT)"

game:
	@cargo run -p doomexe --bin doomexe

doom: game

minimal:
	@cargo run -p dj_engine --bin minimal

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

guardrail:
	@echo "Running guardrail checks..."
	@cargo check --workspace || (echo "FAILED: Build broken"; exit 1)
	@cargo test --workspace || (echo "FAILED: Tests broken"; exit 1)
	@cargo fmt --all --check || (echo "FAILED: Formatting issues"; exit 1)
	@echo "All guardrails passed."

# Utility

clean:
	@cargo clean
