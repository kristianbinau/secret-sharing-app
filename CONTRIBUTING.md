# Contributing to Secret Sharing

Thank you for your interest in contributing! This guide covers development setup, coding conventions, pull request workflow, and the release process.

## License

By contributing, you agree that your contributions will be licensed under the [MIT License](./LICENSE).

## Development Setup

### Prerequisites

- [Node.js 26](https://nodejs.org/) (pinned via [mise](https://mise.jdx.dev/))
- [Rust 1.95.0](https://rustup.rs/) (pinned via mise)
- [mise](https://mise.jdx.dev/) for tool version management

If you have mise installed, it will automatically use the correct versions from `mise.toml` when you enter the project directory.

### Getting Started

```bash
# Install dependencies (also runs nuxt prepare to generate .nuxt/ types)
npm install

# Run the full app (Nuxt dev server + Tauri window)
npm run tauri dev
```

> `npm run dev` alone only starts the web frontend — Tauri commands (`invoke`) will fail without the Rust backend. Always use `npm run tauri dev` for the full app.

### Useful Commands

| Command | Description |
|---------|-------------|
| `npm run tauri dev` | Full app dev (Nuxt + Tauri) |
| `npm run dev` | Web frontend only |
| `npm run generate` | Static build to `dist/` |
| `npm run typecheck` | TypeScript type checking (`vue-tsc --noEmit`) |
| `cargo test` | Run Rust unit tests (from `src-tauri/`) |
| `cargo fmt` | Format Rust code |
| `cargo clippy` | Lint Rust code |
| `npm run tauri build` | Production build |

### Project Architecture

- **Frontend** (`app/`): Nuxt 4 SPA (`ssr: false`). Uses `@nuxt/ui`, Tailwind CSS, Zod. Single page (`app/pages/index.vue`) with Encrypt/Decrypt tabs. Calls Tauri commands via `invoke()` from `@tauri-apps/api/core`.
- **Backend** (`src-tauri/src/`): Rust. All crypto logic lives here, not in TS. Uses the `blahaj` crate (Shamir's Secret Sharing) and `base64` (URL-safe) for share encoding. Two Tauri commands: `simple_split` and `simple_combine`, defined in `src-tauri/src/lib.rs`.

See [`AGENTS.md`](./AGENTS.md) for the full architecture reference, and [`docs/share-format.md`](./docs/share-format.md) for the binary share encoding specification.

## Coding Conventions

- **Rust**: Follow `cargo fmt` formatting. `cargo clippy` must pass with `-D warnings` (enforced in CI).
- **TypeScript/Vue**: Follow existing code style. `npm run typecheck` must pass.
- **No comments** unless the code is genuinely non-obvious.
- **All crypto logic must live in Rust** (`src-tauri/src/`), never in the TypeScript frontend.

## Pull Request Workflow

### Conventional Commits

All PR titles must follow the [Conventional Commits](https://www.conventionalcommits.org/) specification. This is enforced by a CI check (`.github/workflows/pr-title.yml`).

**Format:** `type(scope): description`

**Allowed types:**

| Type | Use for |
|------|---------|
| `feat` | New features |
| `fix` | Bug fixes |
| `perf` | Performance improvements |
| `refactor` | Code refactoring |
| `docs` | Documentation changes |
| `test` | Test changes |
| `build` | Build system / dependencies |
| `ci` | CI configuration changes |
| `chore` | Maintenance tasks |
| `revert` | Reverting changes |

Scopes are optional. Examples:
- `feat: add nested share support`
- `fix(encrypt): handle empty secret input`
- `ci: add CodeQL workflow`
- `docs: update README setup instructions`

### Before Submitting a PR

1. Ensure `cargo fmt`, `cargo clippy`, and `cargo test` pass (from `src-tauri/`).
2. Ensure `npm run typecheck` passes.
3. Ensure `npm run generate` (frontend build) succeeds.
4. Write a clear PR title in conventional commit format.
5. Squash commits if needed — the merge will squash automatically.

### CI Checks

Every PR runs:
- **CI** (`.github/workflows/ci.yml`): `cargo fmt --check`, `cargo clippy -D warnings`, `cargo test`, `cargo audit`, `cargo deny`, `npm ci`, `npm run typecheck`, `npm run generate`, `npm audit`
- **CodeQL** (`.github/workflows/codeql.yml`): Static analysis for TypeScript and Rust
- **PR Title** (`.github/workflows/pr-title.yml`): Conventional commit format validation

The full Tauri build (cross-platform) only runs on push to `master`, not on PRs.

## Release Process

Releases are fully automated via [release-please](https://github.com/googleapis/release-please) and the Tauri GitHub Action. There is no manual release step.

### How It Works

1. **Conventional commits accumulate.** Every commit to `master` with a conventional commit type (`feat`, `fix`, `perf`, `refactor`, etc.) is tracked by release-please.

2. **Release-please maintains a release PR.** It automatically opens and updates a PR titled `chore(master): release X.Y.Z` that:
   - Bumps the version in `package.json`, `src-tauri/Cargo.toml`, and `src-tauri/tauri.conf.json`
   - Updates `CHANGELOG.md` with the accumulated changes
   - Updates `.release-please-manifest.json` with the new version

3. **Merge the release PR to publish.** When you're ready to release, review and merge the release-please PR. This triggers the **Release workflow** (`.github/workflows/release.yml`):

   - **release-please** creates a GitHub Release with a git tag (`vX.Y.Z`) and auto-generated release notes.
   - **publish-tauri** builds the app on 4 targets:
     - macOS (aarch64 Apple Silicon)
     - macOS (x86_64 Intel)
     - Linux (x86_64)
     - Windows (x86_64)
   - **finalize-release** generates `SHA256SUMS.txt` and uploads it to the release.

4. **Verify the release.** Check the [Releases page](https://github.com/kristianbinau/secret-sharing-app/releases) — all platform binaries and checksums should be attached.

### Version Numbering

The project uses [semantic versioning](https://semver.org/) with a `v` prefix (e.g., `v0.0.2`). While in `0.0.x`, breaking changes increment the minor version.

### What Triggers a Release

| Commit type | Appears in changelog? | Bumps version? |
|-------------|----------------------|----------------|
| `feat` | Yes (Features) | Minor |
| `fix` | Yes (Bug Fixes) | Patch |
| `perf` | Yes (Performance Improvements) | Patch |
| `refactor` | Yes (Code Refactoring) | Patch |
| `docs` | No | No |
| `test` | No | No |
| `build` | No | No |
| `ci` | No | No |
| `chore` | No | No |

> `feat` and `fix` commits are the primary triggers for new releases. Other types are tracked but don't force a version bump on their own.

### Release Configuration

- **Config:** [`release-please-config.json`](./release-please-config.json) — defines the release type, extra files to version-bump, and component name.
- **Manifest:** [`.release-please-manifest.json`](./.release-please-manifest.json) — tracks the current released version.
- **Workflow:** [`.github/workflows/release.yml`](./.github/workflows/release.yml) — the full release pipeline.

## Dependency Management

Dependencies are monitored by [Dependabot](https://docs.github.com/en/code-security/dependabot) with weekly checks (Mondays) across three ecosystems:

- **npm** (root `package.json`)
- **cargo** (`src-tauri/Cargo.toml`)
- **github-actions** (workflow files)

Minor and patch updates are grouped into a single PR per ecosystem to reduce noise. Major version bumps get individual PRs. A maximum of 5 PRs per ecosystem (15 total) are open at any time.

Dependabot PRs follow the same conventional commit format (`build(deps): ...`) and must pass all CI checks before merging.

## Reporting Security Issues

See [`SECURITY.md`](./.github/SECURITY.md). **Do not open a public issue** for security vulnerabilities — email security@binau.dev instead.
