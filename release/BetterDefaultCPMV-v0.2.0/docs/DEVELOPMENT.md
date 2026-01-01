# Development Guide for BetterDefaultCPMV

## Getting Started

### Prerequisites

- Rust 1.70+ (install from https://rustup.rs/)
- Git
- A text editor or IDE (VS Code, JetBrains IDEs, vim, etc.)

### Setup Development Environment

```bash
# Clone repository
git clone <repository-url>
cd better-cp

# Verify Rust installation
rustc --version  # Should be 1.70+
cargo --version

# Install development tools
cargo install cargo-watch  # Auto-rebuild on changes
cargo install cargo-clippy # Linting
cargo install cargo-tarpaulin # Code coverage
```

## Building

### Development Build

```bash
cargo build
# Output: target/debug/better-cp (slower, full debug info)
```

### Release Build

```bash
cargo build --release
# Output: target/release/better-cp (optimized, stripped)
```

### Fast Iterative Development

```bash
# Auto-recompile on file changes
cargo watch -x build

# Or run tests continuously
cargo watch -x test

# With logging
RUST_LOG=debug cargo watch -x run
```

## Testing

### Run All Tests

```bash
cargo test
```

### Run Specific Test

```bash
cargo test test_checksum_computation
```

### Run with Output

```bash
cargo test -- --nocapture
```

### Run Integration Tests

```bash
cargo test --test '*'
```

### Code Coverage

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --out Html --output-dir coverage/
```

## Code Quality

### Linting

```bash
# Run clippy (Rust linter)
cargo clippy -- -D warnings

# In watch mode
cargo watch -x "clippy -- -D warnings"
```

### Formatting

```bash
# Format all code
cargo fmt

# Check formatting without changing
cargo fmt -- --check
```

### Audit Dependencies

```bash
# Install cargo-audit
cargo install cargo-audit

# Check for known vulnerabilities
cargo audit
```

## Common Development Tasks

### Add a New Dependency

```bash
# Add to Cargo.toml
cargo add serde_yaml

# Or manually edit Cargo.toml and run
cargo update
```

### Create a New Module

```bash
# 1. Create file
touch src/new_module.rs

# 2. Add to src/lib.rs
# pub mod new_module;

# 3. Export in lib.rs if needed
# pub use new_module::SomePublicType;
```

### Run with Arguments

```bash
# Pass arguments to the binary
cargo run -- --help
cargo run -- --dry-run source dest
cargo run -- --verbose source dest
```

### Debug a Specific Scenario

```bash
# With logging
RUST_LOG=debug cargo run -- source dest

# In GDB (Linux/macOS)
rust-gdb target/debug/better-cp
```

## Debugging Tips

### Enable Logging

Codebase uses `tracing` crate. Enable with:

```bash
RUST_LOG=debug cargo run -- source dest
RUST_LOG=trace cargo run -- source dest  # More verbose
```

### Print Debugging

Use standard Rust:

```rust
dbg!(variable);  // Prints and returns variable
println!("{:#?}", object);  // Pretty-print Debug
```

### Backtrace on Panic

```bash
RUST_BACKTRACE=1 cargo run -- source dest
RUST_BACKTRACE=full cargo run -- source dest  # Full output
```

## Project Structure

```
better-cp/
├── src/
│   ├── lib.rs           # Library root, exports all modules
│   ├── bin/
│   │   ├── better_cp.rs # Copy binary entry point
│   │   └── better_mv.rs # Move binary entry point
│   ├── cli.rs           # CLI argument parsing
│   ├── copy.rs          # Core copy logic
│   ├── error.rs         # Error types
│   ├── progress.rs      # Progress tracking
│   ├── resume.rs        # Resume state management
│   ├── verify.rs        # Checksum verification
│   └── config.rs        # Configuration loading
├── tests/               # Integration tests
├── Cargo.toml          # Manifest
├── Cargo.lock          # Lock file (commit this)
└── README.md           # User documentation
```

## Code Organization Principles

1. **Single Responsibility**: Each module has one clear purpose
2. **Error Propagation**: Use `?` operator, return `Result<T>`
3. **Ownership**: Avoid unnecessary cloning; use references where possible
4. **Testing**: Write tests alongside implementation
5. **Documentation**: Comment non-obvious logic; use doc comments for public APIs

## Common Code Patterns

### Error Handling

```rust
// Return error using ?
fn foo() -> Result<String> {
    let data = fs::read_to_string("file.txt")?;
    Ok(data)
}

// Custom error context
fn bar() -> Result<()> {
    something_that_fails()
        .map_err(|e| Error::Custom(format!("Failed to do thing: {}", e)))?;
    Ok(())
}
```

### Working with Paths

```rust
use std::path::{Path, PathBuf};

let p = PathBuf::from("/home/user/file.txt");
let parent = p.parent().unwrap();
let filename = p.file_name().unwrap();

// Always validate paths exist
if !p.exists() {
    return Err(Error::SourceNotFound(...));
}
```

### Async I/O

```rust
// Mark function as async
pub async fn copy(&self) -> Result<()> {
    // Can use .await on async operations
    self.perform_copy().await?;
    Ok(())
}

// Run async in tests
#[tokio::test]
async fn test_copy() {
    let result = some_async_fn().await;
    assert!(result.is_ok());
}
```

### Thread-Safe Shared State

```rust
use std::sync::{Arc, Mutex};

let shared = Arc::new(Mutex::new(value));
let cloned = Arc::clone(&shared);

// In another thread/context
{
    let mut guard = cloned.lock().unwrap();
    *guard = new_value;
}
```

## Git Workflow

### Branch Naming

- Features: `feature/description`
- Bugfixes: `fix/issue-description`
- Documentation: `docs/something`
- Refactoring: `refactor/target-area`

### Commit Messages

```
# Good
feat: Add resume support for interrupted transfers

- Implement ResumeState serialization
- Validate state on load
- Tests for state persistence

# Bad
update code
fix bug
changes
```

### Before Pushing

```bash
# Format code
cargo fmt

# Run linter
cargo clippy -- -D warnings

# Run tests
cargo test

# Then push
git push origin feature/my-feature
```

## Performance Optimization

### Profiling

```bash
# CPU flamegraph (Linux)
cargo install flamegraph
cargo flamegraph -- source dest

# Memory usage (all platforms)
/usr/bin/time -v target/release/better-cp source dest
```

### Benchmarking

```bash
# Criterion benchmarks (if added to Cargo.toml)
cargo bench
```

### Common Optimization Techniques

1. **Reduce Allocations**: Use `Vec::with_capacity()`, avoid cloning
2. **Batch Operations**: Group I/O operations
3. **Caching**: Cache repeated computations
4. **Parallelization**: Use tokio tasks or rayon for CPU-bound work

## Release Process

### Prepare Release

```bash
# Update version in Cargo.toml
# 0.1.0 -> 0.2.0

# Update CHANGELOG.md

# Commit
git commit -am "Release v0.2.0"
git tag -a v0.2.0 -m "Release version 0.2.0"
git push origin v0.2.0
```

### Build Binaries

```bash
# Release build
cargo build --release

# Binary locations
./target/release/better-cp    # Linux/macOS
./target/release/better-cp.exe  # Windows
```

### Publish to Crates.io

```bash
cargo publish
```

## Troubleshooting

### Build Issues

**Problem**: `error: linking with 'cc' failed`

**Solution**: Install build essentials
```bash
# Ubuntu/Debian
sudo apt-get install build-essential

# macOS
xcode-select --install

# Windows
Download Visual Studio Build Tools
```

**Problem**: `error[E0514]: type mismatch`

**Solution**: Update dependencies and rebuild
```bash
cargo update
cargo clean
cargo build
```

### Test Failures

**Problem**: Flaky tests

**Solution**: Check for race conditions, use synchronization primitives

**Problem**: Permission denied during tests

**Solution**: Run with appropriate permissions or skip those tests on that platform
```rust
#[cfg(not(target_os = "windows"))]
#[test]
fn test_permission_denied() { ... }
```

### Runtime Issues

**Problem**: "broken pipe" error

**Solution**: Handle disconnections gracefully, use `Result` types

**Problem**: Memory leak

**Solution**: Check for reference cycles, use `drop()` explicitly where needed

## Documentation

### Writing Doc Comments

```rust
/// Brief description of what this function does.
///
/// More detailed explanation here.
///
/// # Arguments
/// * `arg1` - Description of arg1
/// * `arg2` - Description of arg2
///
/// # Returns
/// Description of return value
///
/// # Errors
/// Returns error if [condition]
///
/// # Examples
/// ```
/// let result = example_function(5);
/// assert_eq!(result, 10);
/// ```
pub fn example_function(arg1: i32) -> Result<i32> {
    Ok(arg1 * 2)
}
```

### Generate and View Docs

```bash
cargo doc --open
```

## Additional Resources

- Rust Book: https://doc.rust-lang.org/book/
- Rust By Example: https://doc.rust-lang.org/rust-by-example/
- Tokio Tutorial: https://tokio.rs/tokio/tutorial
- Clippy Lint List: https://rust-lang.github.io/rust-clippy/
