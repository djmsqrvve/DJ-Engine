# Project Maintainer Guide

Welcome! This guide covers everything you need to know about maintaining DJ Engine as a GitHub project owner.

---

## 📋 Quick Reference

| Task | Command |
|------|---------|
| Run tests | `make test` |
| Run editor | `make editor` |
| Build release | `cargo build --release` |
| Check code | `cargo check --workspace` |
| Format code | `cargo fmt` |
| Lint | `cargo clippy` |

---

## 🔧 Day-to-Day Maintenance

### Reviewing Pull Requests

1. **Check CI passes** - All tests must pass
2. **Review code** - Look for:
   - Clear commit messages
   - No new warnings (`cargo clippy`)
   - Tests for new features
   - Documentation updates
3. **Merge strategy** - Use "Squash and merge" for clean history

### Handling Issues

- **Triage labels**: `bug`, `enhancement`, `question`, `good first issue`
- **Respond within 48 hours** - Even just an acknowledgment helps
- **Close stale issues** - After 30 days without response

### Releases

```bash
# Update version in Cargo.toml
# Create a tag
git tag -a v0.1.0 -m "Release v0.1.0"
git push origin v0.1.0
```

---

## 🛡️ Repository Settings (GitHub Web)

### Recommended Settings

1. **Settings > General**
   - Enable "Always suggest updating PR branches"
   - Disable "Allow merge commits" (use squash only)

2. **Settings > Branches**
   - Add branch protection for `main`:
     - ✅ Require pull request before merging
     - ✅ Require status checks to pass
     - ✅ Require conversation resolution

3. **Settings > Actions > General**
   - Allow actions from this repository only

### Labels to Create

| Label | Color | Description |
|-------|-------|-------------|
| `bug` | #d73a4a | Something isn't working |
| `enhancement` | #a2eeef | New feature request |
| `documentation` | #0075ca | Documentation improvements |
| `good first issue` | #7057ff | Good for newcomers |
| `help wanted` | #008672 | Extra attention needed |
| `wontfix` | #ffffff | Won't be fixed |

---

## 📊 Growing Your Community

### Visibility
- Add topics: `game-engine`, `rust`, `bevy`, `visual-novel`, `jrpg`
- Write a good description
- Pin important issues

### Encouraging Contributors
- Mark issues as `good first issue`
- Write clear CONTRIBUTING.md (✅ done)
- Be welcoming and responsive
- Credit contributors in release notes

### Documentation
- Keep README updated
- Add examples and tutorials
- Document breaking changes

---

## 🔐 Security

- **Never commit secrets** - Use environment variables
- **Review dependencies** - Run `cargo audit` periodically
- **Enable Dependabot** - Settings > Security > Code security and analysis

---

## 📁 Repository Structure

```
.github/
├── ISSUE_TEMPLATE/      # Issue templates
│   ├── bug_report.md
│   └── feature_request.md
├── PULL_REQUEST_TEMPLATE.md
└── workflows/           # CI/CD (future)

Root files:
├── README.md            # Project overview
├── CONTRIBUTING.md      # Contribution guide
├── MAINTAINERS.md       # This file
├── CODE_OF_CONDUCT.md   # Contributor Covenant
├── SECURITY.md          # Vulnerability reporting
└── LICENSE              # MIT license
```

---

## 🆘 Getting Help

- **Bevy Discord** - Great for engine questions
- **Rust Users Forum** - For Rust-specific issues
- **GitHub Discussions** - Enable for community Q&A

---

*Last updated: March 2026*
