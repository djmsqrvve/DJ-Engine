# DJ Engine - Unified Command Interface

.PHONY: help check build test lint fmt format-fix clean dev editor game minimal quality-check guardrail

# Ensure rustup toolchain takes precedence over system cargo/rustc
export PATH := $(HOME)/.cargo/bin:$(PATH)

CARGO_TARGET_DIR ?= $(HOME)/.cargo-targets/dj_engine_bevy18
export CARGO_TARGET_DIR

# Default target
help:
	@echo "DJ Engine - Command Interface"
	@echo ""
	@echo "Quick Start:"
	@echo "  make dev          Alias for 'make game'"
	@echo "  make editor       Launch the editor"
	@echo "  make game         Run DoomExe game"
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

dev: game

editor:
	@cargo run --bin dj_editor

game:
	@cargo run --bin doomexe

minimal:
	@cargo run --bin minimal

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
