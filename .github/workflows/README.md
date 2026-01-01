# GitHub Actions Workflows

This directory contains automated CI/CD workflows for BetterDefaultCPMV.

## Workflows

### `ci.yml` - Continuous Integration

**Triggers**: Push to `master` or `develop`, Pull Requests

**Jobs**:
- **Test Suite**: Runs `cargo test` on Ubuntu, Windows, and macOS
- **Rustfmt**: Checks code formatting
- **Clippy**: Lint checks with warnings treated as errors
- **Code Coverage**: Generates coverage report with tarpaulin and uploads to Codecov.io

**Requirements**: None (uses stable Rust)

### `release.yml` - Release Build & Distribution

**Triggers**: Push of version tags (e.g., `git push origin v0.2.0`)

**Jobs**:
1. **Create Release**: Creates a GitHub release
2. **Build Linux**: Compiles for Linux, creates `tar.gz` archive with binaries and docs
3. **Build Windows**: Compiles for Windows, creates `zip` archive with binaries and docs
4. **Build macOS**: Compiles for macOS, creates `tar.gz` archive with binaries and docs

**Release Assets Created**:
- `BetterDefaultCPMV-v{version}-linux.tar.gz`
- `BetterDefaultCPMV-v{version}-windows.zip`
- `BetterDefaultCPMV-v{version}-macos.tar.gz`

### `pages.yml` - GitHub Pages Deployment

**Triggers**: Push to `master`, Manual trigger via workflow_dispatch

**Jobs**:
- Automatically deploys contents of `docs/` folder to GitHub Pages

## How to Release

### 1. Create and Push Release Tag

```bash
# Create annotated tag
git tag -a v0.2.0 -m "Release v0.2.0"

# Push tag to GitHub
git push origin v0.2.0
```

### 2. GitHub Actions Will Automatically

- ✅ Create a GitHub release page
- ✅ Build binaries for Linux, Windows, and macOS
- ✅ Attach compiled binaries to the release
- ✅ Deploy GitHub Pages documentation

### 3. Manual Steps

After the workflow completes:

1. Go to GitHub release page
2. Add release notes from `GITHUB_RELEASE.md`
3. Review attached binaries
4. Publish the release

## Configuration Notes

- **Rust Toolchain**: Stable (updated via `dtolnay/rust-toolchain@stable`)
- **Caching**: Uses `Swatinem/rust-cache` for faster builds
- **Coverage**: Uploaded to Codecov.io (optional badge in README)
- **Pages**: Source is `docs` folder, auto-deployed on `master` push

## Environment Variables

Set these in GitHub repository settings if needed:

- `CARGO_TERM_COLOR`: `always` (default set in workflow)
- `RUST_BACKTRACE`: `1` (default set in workflow)

## Required Secrets

- **GITHUB_TOKEN**: Automatically provided by GitHub Actions

## Troubleshooting

### Release Not Triggering
- Ensure tag is in format `v*` (e.g., `v0.2.0`)
- Verify tag is pushed: `git push origin v0.2.0`

### Build Failures
- Check logs in "Actions" tab on GitHub
- Ensure all tests pass locally before pushing

### Pages Not Deploying
- Enable GitHub Pages in repository settings
- Source: `Deploy from a branch`
- Branch: `master`
- Folder: `/docs`
- Or let the workflow configure it automatically

## Future Enhancements

- [ ] Publish to crates.io automatically
- [ ] Create man pages during release
- [ ] Run benchmarks and post results
- [ ] Auto-generate shell completions
- [ ] Code signing for Windows binaries
