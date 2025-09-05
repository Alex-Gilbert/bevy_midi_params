# Publishing Guide

This crate is ready for publishing to crates.io. Here's what has been set up:

## Pre-flight Checklist ✅

- [x] Code formatting (cargo fmt)
- [x] Linting (cargo clippy)
- [x] Tests passing
- [x] Documentation builds without warnings
- [x] Cargo.toml metadata complete
- [x] README.md present
- [x] LICENSE files present (MIT + Apache 2.0)
- [x] CHANGELOG.md created
- [x] CI/CD workflows configured

## CI/CD Setup

### GitHub Actions Workflows:
- `.github/workflows/ci.yml` - Runs on every push/PR:
  - Formatting check
  - Clippy linting
  - Tests on multiple platforms (Linux, Windows, macOS)
  - Documentation build
  - Security audit
  - MSRV check (1.80.0)
  - Publish dry-run

- `.github/workflows/release.yml` - Runs on version tags:
  - Full CI suite
  - Publishes to crates.io
  - Creates GitHub release

- `.github/dependabot.yml` - Automated dependency updates

## Publishing Steps

### First-time publishing:

1. **Publish the derive crate first** (dependency):
   ```bash
   cd bevy_midi_params_derive
   cargo publish
   ```

2. **Wait for crates.io to index** (30-60 seconds)

3. **Publish the main crate**:
   ```bash
   cd ../bevy_midi_params
   cargo publish
   ```

### Subsequent releases:

1. Update version in workspace `Cargo.toml`
2. Update CHANGELOG.md
3. Commit changes
4. Create and push a tag:
   ```bash
   git tag v0.1.0
   git push origin v0.1.0
   ```
5. GitHub Actions will handle the rest

## Required Secrets

Add these to your GitHub repository secrets:
- `CARGO_REGISTRY_TOKEN` - Your crates.io API token

## Version Requirements

- MSRV: Rust 1.80.0 (for Bevy 0.16 compatibility)
- Edition: 2021

## Notes

- Both crates share workspace metadata for consistency
- README.md is duplicated in each crate directory for crates.io
- The derive crate must be published before the main crate