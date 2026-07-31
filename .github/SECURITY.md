# Security Policy

## Reporting a Vulnerability

This application handles cryptographic operations (Shamir's Secret Sharing). Security vulnerabilities are taken seriously.

**Do NOT open a public GitHub issue for security vulnerabilities.**

To report a vulnerability:

1. Email **security@binau.dev** with a description of the issue and reproduction steps.
2. You will receive an acknowledgment within 48 hours.
3. A fix or mitigation will be prioritized based on severity.

## Build Verification

Releases are built via GitHub Actions and published to GitHub Releases. Each release includes a `SHA256SUMS.txt` file listing checksums for all platform binaries.

To verify a downloaded binary:

```bash
shasum -a 256 secret-sharing-app_<version>_<platform>.<ext>
# Compare the output against the corresponding line in SHA256SUMS.txt
```

## Supply Chain

- All third-party GitHub Actions are pinned to immutable SHAs (commit SHAs or annotated tag object SHAs, not mutable tags).
- Dependency advisories are scanned via `cargo audit`, `npm audit`, GitHub CodeQL, and `cargo deny`.
- Tool versions (Node, Rust) are pinned via `mise.toml` for reproducible builds.
