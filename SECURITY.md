# Security Policy

## Reporting a Vulnerability

The rust-poker team takes security vulnerabilities seriously. We appreciate your efforts to responsibly disclose your findings.

### How to Report

**Please do NOT report security vulnerabilities through public GitHub issues.**

Instead, please report security vulnerabilities by emailing:

**security@rumenx.com**

### What to Include

Please include the following information in your report:

- **Description** of the vulnerability
- **Steps to reproduce** the issue
- **Potential impact** of the vulnerability
- **Suggested fix** (if you have one)
- **Your contact information** for follow-up questions

### Response Timeline

- **Initial response**: Within 48 hours
- **Status update**: Within 7 days
- **Fix timeline**: Depends on severity

| Severity | Target Fix Time |
|----------|----------------|
| Critical | 24 hours |
| High | 7 days |
| Medium | 30 days |
| Low | Next release |

## Supported Versions

| Version | Supported |
|---------|-----------|
| Latest | ✅ |
| Older | ❌ |

## Security Best Practices

This project follows these security practices:

- **Dependency auditing** via `cargo audit` in CI
- **Automated dependency updates** via Dependabot
- **No `unsafe` code** in the codebase
- **Minimal dependencies** to reduce attack surface
