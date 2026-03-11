# Contributing to DJ Engine

Thank you for your interest in contributing! This guide will help you get started.

## Quick Start

```bash
# Clone the repository
git clone https://github.com/djmsqrvve/dj_engine.git
cd dj_engine

# Build and run
make dev

# Run tests
make test

# Run the editor
make editor
```

## Development Workflow

1. **Fork & Clone** - Fork the repo and clone your fork
2. **Branch** - Create a feature branch: `git checkout -b feature/my-feature`
3. **Code** - Make your changes following the coding standards
4. **Test** - Ensure all tests pass: `make test`
5. **Commit** - Write clear commit messages (see below)
6. **Push** - Push to your fork
7. **PR** - Open a Pull Request

## Commit Message Format

```
type: short description

[optional body]
```

Types: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`

Examples:
- `feat: add dialogue branching to story graph`
- `fix: resolve camera jitter in editor`
- `docs: update installation instructions`

## Code Style

- Run `make format-fix` before committing
- Run `make lint` and address warnings
- Run `make guardrail` before opening a PR
- Follow existing patterns in the codebase
- Add tests for new functionality

## Project Structure

```
dj_engine/
├── engine/          # Core engine library
│   ├── src/
│   │   ├── core/        # Core engine plugin
│   │   ├── data/        # Serializable data types
│   │   ├── editor/      # Egui-based editor
│   │   ├── story_graph/ # Narrative system
│   │   └── scripting/   # Lua integration
│   └── examples/    # Example JSON files
├── games/           # Game projects
│   └── dev/doomexe/ # Main development game
├── docs/            # Documentation
└── tools/           # CLI utilities
```

## Getting Help

- Open an issue for bugs or questions
- Check existing issues before creating new ones
- Join discussions on PRs

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
