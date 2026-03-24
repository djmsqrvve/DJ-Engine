# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| main    | :white_check_mark: |

## Reporting a Vulnerability

If you discover a security vulnerability, please report it responsibly:

1. **Do NOT** create a public GitHub issue
2. Use [GitHub's private vulnerability reporting](https://github.com/djmsqrvve/DJ-Engine/security/advisories/new) on this repository
3. Include:
   - Description of the vulnerability
   - Steps to reproduce
   - Potential impact
   - Suggested fix (if any)

## Response Timeline

- **Acknowledgment**: Within 48 hours
- **Initial Assessment**: Within 1 week
- **Fix/Disclosure**: Coordinated with reporter

## Security Best Practices for Contributors

- Never commit secrets, API keys, or credentials
- Use environment variables for sensitive configuration
- Run `cargo audit` periodically to check dependencies
- Keep dependencies updated

## Dependency Security

We use standard Rust tooling for security:

```bash
# Check for known vulnerabilities
cargo install cargo-audit
cargo audit

# Update dependencies
cargo update
```
