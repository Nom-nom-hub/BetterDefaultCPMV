# Publishing BetterDefaultCPMV

This guide explains how to publish BetterDefaultCPMV to crates.io and create releases.

## Setup (One-time)

### 1. Create crates.io Account

1. Go to https://crates.io
2. Sign up with GitHub
3. Copy your API token from https://crates.io/me

### 2. Add GitHub Secret

1. Go to GitHub repo Settings → Secrets and Variables → Actions
2. Create new secret: `CARGO_TOKEN`
3. Paste your crates.io API token

## Publishing Process

### Automatic (Recommended)

When you push a version tag, GitHub Actions automatically:

1. **Publishes to crates.io** (workflow: `publish-crates.yml`)
   - Triggers on any tag matching `v*` (e.g., `v0.2.0`)
   - Uses your `CARGO_TOKEN` secret
   - Publishes the package to crates.io

2. **Creates GitHub Release** (workflow: `release.yml`)
   - Builds binaries for Linux, Windows, macOS
   - Creates release page with binaries attached

3. **Deploys GitHub Pages** (workflow: `pages.yml`)
   - Updates website from docs/ folder

### Manual Publishing

If you need to publish manually:

```bash
# Test before publishing
cargo publish --dry-run

# Publish to crates.io
cargo publish --token <your-token>
```

## Release Checklist

- [ ] Update version in `Cargo.toml`
- [ ] Update `CHANGELOG.md` with new version
- [ ] Commit changes: `git commit -am "Release v0.X.X"`
- [ ] Create tag: `git tag -a vX.X.X -m "Release vX.X.X"`
- [ ] Push commits and tag:
  ```bash
  git push origin main
  git push origin vX.X.X
  ```
- [ ] GitHub Actions automatically:
  - [ ] Publishes to crates.io
  - [ ] Creates GitHub release with binaries
  - [ ] Deploys GitHub Pages

## Verifying Publication

### crates.io

After publishing, verify at: https://crates.io/crates/better-cp

### Using the Package

Users can now install directly:

```bash
cargo install better-cp
```

## Troubleshooting

### "crate 'better-cp' is already published"

This means the version already exists. Update the version in `Cargo.toml`:

```toml
[package]
version = "0.3.0"  # Increment to new version
```

### "authentication required"

Ensure your `CARGO_TOKEN` secret is set and valid:
1. Go to Settings → Secrets
2. Verify `CARGO_TOKEN` exists
3. Re-generate token from https://crates.io/me if needed

### Publishing Failed

Check workflow logs in GitHub Actions:
1. Go to Actions tab
2. Click "Publish to crates.io"
3. Review job logs for error messages

## Documentation

For more information:
- https://doc.rust-lang.org/cargo/reference/publishing.html
- https://crates.io/
